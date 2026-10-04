#!/usr/bin/env bash
# Execute generated fixtures only; never a registry write or production service.
set -euo pipefail
test "$#" = 4
arch="$1"
revision="$2"
seconds="$3"
output="$4"
[[ "$arch" = amd64 || "$arch" = arm64 ]]
[[ "$revision" =~ ^[0-9a-f]{40}$ ]]
[[ "$seconds" = 600 || "$seconds" = 1800 || "$seconds" = 3600 ]]
: "${RUNNER_TEMP:?Expected a fresh GitHub-hosted runner}"
test "${GITHUB_ACTIONS:-}" = true
test "$output" = "$RUNNER_TEMP/reader-soak-report"
for report in PRELOAD_IDENTITY.json LOADED_IDENTITY.json RUNNING_JAR_IDENTITY.json SOAK_REPORT.json SOAK_TRACE.jsonl; do
  test ! -e "$output/$report"
done
case "$arch:$RUNNER_ARCH:$(uname -m)" in
  amd64:X64:x86_64|arm64:ARM64:aarch64) ;;
  *) exit 1 ;;
esac
version=$(jq -er '.version' imported/metadata.json)
node scripts/release-native-artifacts.mjs check "$arch" "$version" "$revision" dist imported \
  | tee "$output/PRELOAD_IDENTITY.json"
docker load -i imported/reader-image.tar.gz
image="reader-pro:camoufox-smoke-$arch"
docker image inspect "$image" > imported/image-inspect.json
node scripts/release-native-artifacts.mjs loaded "$arch" "$version" "$revision" dist imported \
  | tee "$output/LOADED_IDENTITY.json"
container="reader-offline-soak-$arch"
storage="$RUNNER_TEMP/reader-offline-soak-$arch-storage"
if docker container inspect "$container" >/dev/null 2>&1 || test -e "$storage"; then
  echo 'Refusing to reuse an existing soak container or storage' >&2
  exit 1
fi
install -d "$storage"
sudo chown 10001:10001 "$storage"
# Preserve the runner's report ownership; grant only the container UID write access.
sudo setfacl -m u:10001:rwx "$output"
container_id=''
cleanup() {
  result=$?
  trap - EXIT
  if [[ -n "$container_id" ]]; then
    docker stats --no-stream --format 'name={{.Name}} memory={{.MemUsage}} pids={{.PIDs}}' "$container_id" || true
    docker inspect --format '{{json .State}}' "$container_id" > "$output/CONTAINER_STATE.json" || true
    if [[ "$result" != 0 ]]; then docker logs --tail=120 "$container_id" || true; fi
    docker rm -f "$container_id" || true
  fi
  exit "$result"
}
trap cleanup EXIT
container_id=$(docker run -d --init --name "$container" --network none --shm-size=1g \
  --memory=2g --memory-swap=2g --pids-limit=256 --cpus=2 \
  --security-opt no-new-privileges:true --cap-drop ALL \
  --tmpfs /tmp:size=256m,mode=1777 \
  -v "$storage:/storage" -v "$PWD/scripts:/verification-scripts:ro" \
  -v "$output:/verification-output" \
  -e READER_SERVER_PORT=18892 -e READER_SERVER_BINDADDRESS=127.0.0.1 \
  -e READER_APP_WORKDIR=/ -e READER_APP_SECURE=true \
  -e READER_APP_LICENSECHECKENABLED=false -e READER_APP_WEBVIEWRENDERER=camoufox \
  -e READER_APP_BROWSERTIMEOUTMS=5000 -e READER_BROWSER_ALLOW_PRIVATE_NETWORKS=true "$image")
test "$(docker inspect --format '{{.HostConfig.NetworkMode}}' "$container_id")" = none
ready=false
for attempt in $(seq 1 60); do
  if docker exec "$container_id" python -c '
import json, urllib.request
opener=urllib.request.build_opener(urllib.request.ProxyHandler({}))
with opener.open("http://127.0.0.1:18892/reader3/getSystemInfo", timeout=3) as r:
    assert r.status == 200 and json.load(r)["isSuccess"] is True
'; then ready=true; break; fi
  sleep 2
done
test "$ready" = true
expected_jar=$(jq -er '.jarSha256' imported/metadata.json)
actual_jar=$(docker exec "$container_id" sha256sum /app/reader.jar | awk '{print $1}')
test "$actual_jar" = "$expected_jar"
jq -n --arg revision "$revision" --arg jarSha256 "$actual_jar" \
  '{revision: $revision, jarSha256: $jarSha256, network: "none"}' > "$output/RUNNING_JAR_IDENTITY.json"
docker exec "$container_id" python /verification-scripts/soak-bundled-browser.py \
  --reader-base http://127.0.0.1:18892 --expected-revision "$revision" --seconds "$seconds"
jq -e --arg revision "$revision" --argjson seconds "$seconds" \
  '.passed == true and .expectedImageRevision == $revision and .continuousSeconds >= $seconds and
   .rounds >= 20 and (.faults | length) == 3 and .targetCookieOrShapeErrors == 0 and
   .resources.swapMaxBytes == 0 and .resources.memoryEvents.max == 0 and
   .resources.memoryEvents.oom == 0 and .resources.memoryEvents.oom_kill == 0 and
   .resources.pidsEvents.max == 0' "$output/SOAK_REPORT.json" >/dev/null
