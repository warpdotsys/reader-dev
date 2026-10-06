#!/usr/bin/env bash
# Reuse the exact tested artifact. No registry write, rebuild or production data.
set -euo pipefail
test "$#" = 3 || test "$#" = 4
arch="$1"
revision="$2"
output="$3"
capture_mode="${4:-bounded-dom}"
[[ "$arch" = amd64 || "$arch" = arm64 ]]
[[ "$revision" =~ ^[0-9a-f]{40}$ ]]
case "$capture_mode" in
  bounded-dom|snapshot-only) ;;
  *) exit 1 ;;
esac
: "${RUNNER_TEMP:?Expected a fresh GitHub-hosted runner}"
test "${GITHUB_ACTIONS:-}" = true
test "$output" = "$RUNNER_TEMP/reader-public-metadata-report"
case "$arch:$RUNNER_ARCH:$(uname -m)" in
  amd64:X64:x86_64|arm64:ARM64:aarch64) ;;
  *) exit 1 ;;
esac
for report in PRELOAD_IDENTITY.json LOADED_IDENTITY.json RUNNING_JAR_IDENTITY.json PUBLIC_METADATA_REPORT.json; do
  test ! -e "$output/$report"
done
version=$(jq -er '.version' imported/metadata.json)
node scripts/release-native-artifacts.mjs check "$arch" "$version" "$revision" dist imported \
  | tee "$output/PRELOAD_IDENTITY.json"
docker load -i imported/reader-image.tar.gz
image="reader-pro:camoufox-smoke-$arch"
docker image inspect "$image" > imported/image-inspect.json
node scripts/release-native-artifacts.mjs loaded "$arch" "$version" "$revision" dist imported \
  | tee "$output/LOADED_IDENTITY.json"
container="reader-public-metadata-$arch"
storage="$RUNNER_TEMP/reader-public-metadata-$arch-storage"
if docker container inspect "$container" >/dev/null 2>&1 || test -e "$storage"; then
  echo 'Refusing an existing public-probe container or storage' >&2
  exit 1
fi
install -d "$storage"
sudo chown 10001:10001 "$storage"
sudo setfacl -m u:10001:rwx "$output"
container_id=''
cleanup() {
  result=$?
  trap - EXIT
  if [[ -n "$container_id" ]]; then
    # Never dump Reader logs: source responses may contain anonymous WAF tokens.
    docker inspect --format '{{json .State}}' "$container_id" \
      | jq '{Running, OOMKilled, ExitCode}' > "$output/CONTAINER_STATE.json" || true
    removed=false
    if docker rm -f "$container_id" >/dev/null; then removed=true; fi
    jq -n --argjson removed "$removed" --argjson probeExit "$result" \
      '{ownedContainerRemoved: $removed, probeExit: $probeExit}' > "$output/CLEANUP.json"
    if [[ "$removed" != true ]]; then result=1; fi
  fi
  exit "$result"
}
trap cleanup EXIT
container_id=$(docker run -d --init --name "$container" --network bridge --shm-size=1g \
  --memory=2g --memory-swap=2g --pids-limit=256 --cpus=2 \
  --security-opt no-new-privileges:true --cap-drop ALL \
  --tmpfs /tmp:size=256m,mode=1777 \
  -v "$storage:/storage" -v "$PWD/scripts:/verification-scripts:ro" \
  -v "$output:/verification-output" \
  -e READER_SERVER_PORT=18893 -e READER_SERVER_BINDADDRESS=127.0.0.1 \
  -e READER_APP_WORKDIR=/ -e READER_APP_SECURE=true \
  -e READER_APP_LICENSECHECKENABLED=false -e READER_APP_WEBVIEWRENDERER=camoufox \
  -e READER_APP_BROWSERTIMEOUTMS=18000 -e READER_BROWSER_ALLOW_PRIVATE_NETWORKS=false \
  -e READER_APP_DEBUG=false -e READER_APP_DEBUGLOG=false \
  -e READER_APP_CACHECHAPTERCONTENT=false -e READER_APP_AUTOBACKUPUSERDATA=false \
  -e LOGGING_LEVEL_ROOT=OFF "$image")
test "$(docker inspect --format '{{.HostConfig.NetworkMode}}' "$container_id")" = bridge
docker inspect "$container_id" | jq -e '((.[0].HostConfig.PortBindings // {}) | length) == 0' >/dev/null
ready=false
for attempt in $(seq 1 60); do
  if docker exec "$container_id" python -c '
import json, urllib.request
opener=urllib.request.build_opener(urllib.request.ProxyHandler({}))
try:
    with opener.open("http://127.0.0.1:18893/reader3/getSystemInfo", timeout=2) as r:
        good=r.status == 200 and json.load(r).get("isSuccess") is True
except Exception:
    good=False
raise SystemExit(0 if good else 1)
'; then ready=true; break; fi
  sleep 2
done
test "$ready" = true
expected_jar=$(jq -er '.jarSha256' imported/metadata.json)
actual_jar=$(docker exec "$container_id" sha256sum /app/reader.jar | awk '{print $1}')
test "$actual_jar" = "$expected_jar"
jq -n --arg revision "$revision" --arg jarSha256 "$actual_jar" \
  '{revision: $revision, jarSha256: $jarSha256, network: "bridge", publishedPorts: 0}' \
  > "$output/RUNNING_JAR_IDENTITY.json"
probe_args=(--expected-revision "$revision")
if [[ "$capture_mode" = bounded-dom ]]; then probe_args+=(--wait-dom); fi
docker exec "$container_id" python /verification-scripts/probe-public-metadata.py "${probe_args[@]}"
