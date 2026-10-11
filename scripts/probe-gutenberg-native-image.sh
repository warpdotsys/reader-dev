#!/usr/bin/env bash
# Fixed public-domain book, exact tested image; no rebuild, registry or production.
set -euo pipefail
test "$#" = 3
arch="$1"
revision="$2"
output="$3"
[[ "$arch" = amd64 || "$arch" = arm64 ]]
[[ "$revision" =~ ^[0-9a-f]{40}$ ]]
: "${RUNNER_TEMP:?Expected a fresh GitHub-hosted runner}"
test "${GITHUB_ACTIONS:-}" = true
test "$output" = "$RUNNER_TEMP/reader-public-metadata-report"
case "$arch:$RUNNER_ARCH:$(uname -m)" in
  amd64:X64:x86_64|arm64:ARM64:aarch64) ;;
  *) exit 1 ;;
esac
for report in PRELOAD_IDENTITY.json LOADED_IDENTITY.json RUNNING_JAR_IDENTITY.json GUTENBERG_REPORT.json WORKER_DIAGNOSTICS.json GUTENBERG_UI_REPORT.json UI_BROWSER_RESOURCE_BUDGET.json UI_SERVER_RESOURCE_BUDGET.json UI_CLIENT_CLEANUP.json; do
  test ! -e "$output/$report"
done
test "$(jq -er '.jarSha256' imported/metadata.json)" = f5c9da58803d361849d3cf4e946a3b952081a0a770baf536ec1b6a18e0d6262b
version=$(jq -er '.version' imported/metadata.json)
node scripts/release-native-artifacts.mjs check "$arch" "$version" "$revision" dist imported \
  | tee "$output/PRELOAD_IDENTITY.json"
docker load -i imported/reader-image.tar.gz
image="reader-pro:camoufox-smoke-$arch"
docker image inspect "$image" > imported/image-inspect.json
node scripts/release-native-artifacts.mjs loaded "$arch" "$version" "$revision" dist imported \
  | tee "$output/LOADED_IDENTITY.json"
container="reader-gutenberg-$arch"
storage="$RUNNER_TEMP/reader-gutenberg-$arch-storage"
if docker container inspect "$container" >/dev/null 2>&1 || test -e "$storage"; then
  echo 'Refusing an existing probe container or storage' >&2
  exit 1
fi
install -d "$storage"
sudo chown 10001:10001 "$storage"
sudo setfacl -m u:10001:rwx "$output"
container_id=''
client_id=''
cleanup() {
  result=$?
  trap - EXIT
  if [[ -n "$client_id" ]]; then
    client_removed=false
    if docker rm -f "$client_id" >/dev/null; then client_removed=true; fi
    jq -n --argjson removed "$client_removed" \
      '{ownedTestClientRemoved: $removed, testClientNotDeploymentDependency: true, clientOutsideReaderServerBudget: true}' \
      > "$output/UI_CLIENT_CLEANUP.json"
    if [[ "$client_removed" != true ]]; then result=1; fi
  fi
  if [[ -n "$container_id" ]]; then
    if [[ -n "$client_id" ]]; then
      docker exec "$container_id" python /verification-scripts/report-browser-cgroup.py --require-no-swap \
        | jq '.scope = "same Reader server lifetime: API then actual default UI; test client excluded"' \
        > "$output/UI_SERVER_RESOURCE_BUDGET.json" || result=1
    fi
    if docker logs --tail 200 "$container_id" 2>&1 |
        python3 scripts/collect-browser-failure.py "$output/WORKER_DIAGNOSTICS.json"; then
      :
    else
      echo 'Finite diagnostic collection incomplete' >&2
      result=1
    fi
    docker inspect --format '{{json .State}}' "$container_id" \
      | jq '{Running, OOMKilled, ExitCode}' > "$output/CONTAINER_STATE.json" || result=1
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
  -e READER_APP_SHELFUPDATEINTEVAL=0 -e READER_APP_REMOTEBOOKSOURCEUPDATEINTERVAL=0 \
  -e READER_APP_AUTOCLEARINACTIVEUSER=0 -e LOGGING_LEVEL_ROOT=OFF "$image")
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
docker exec "$container_id" python /verification-scripts/probe-gutenberg-reader.py --expected-revision "$revision"

# A normal external UI client, using the SAME pinned packaged browser bytes.
# Not a WebView service, not part of deployment, not a combined 2-GiB claim.
client="reader-gutenberg-ui-$arch"
if docker container inspect "$client" >/dev/null 2>&1; then exit 1; fi
client_id=$(docker create --init --name "$client" --network "container:$container_id" --shm-size=1g \
  --memory=2g --memory-swap=2g --pids-limit=256 --cpus=2 \
  --security-opt no-new-privileges:true --cap-drop ALL \
  --tmpfs /tmp:size=256m,mode=1777 \
  -v "$PWD/scripts:/verification-scripts:ro" -v "$output:/verification-output" \
  --entrypoint python "$image" /verification-scripts/probe-gutenberg-ui.py --expected-revision "$revision")
test "$(docker inspect --format '{{.Config.User}}' "$client_id")" = 10001:10001
docker inspect "$client_id" | jq -e '((.[0].HostConfig.PortBindings // {}) | length) == 0' >/dev/null
# No raw browser logs are uploaded. The Python probe emits only finite facts.
timeout --signal=TERM 210s docker start -a "$client_id" >/dev/null 2>&1
