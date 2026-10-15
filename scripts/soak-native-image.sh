#!/usr/bin/env bash
# Execute generated fixtures only; never a registry write or production service.
set -euo pipefail
test "$#" = 4
arch="$1"
revision="$2"
seconds="$3"
output="$4"
memory_policy="${READER_SOAK_MEMORY_POLICY:-unchanged}"
case "$memory_policy" in unchanged|high-1536m) ;; *) exit 1 ;; esac
source scripts/reader-browser-budget-slice.sh
[[ "$arch" = amd64 || "$arch" = arm64 ]]
[[ "$revision" =~ ^[0-9a-f]{40}$ ]]
[[ "$seconds" = 600 || "$seconds" = 1800 || "$seconds" = 3600 ]]
: "${RUNNER_TEMP:?Expected a fresh GitHub-hosted runner}"
test "${GITHUB_ACTIONS:-}" = true
test "$output" = "$RUNNER_TEMP/reader-soak-report"
for report in PRELOAD_IDENTITY.json LOADED_IDENTITY.json RUNNING_JAR_IDENTITY.json SOAK_REPORT.json SOAK_TRACE.jsonl VERIFIED_SOAK.json CONTAINER_REMOVAL.json POST_REMOVAL_VERIFIED_SOAK.json MEMORY_HIGH_PRESTART.json MEMORY_HIGH_MEMBERSHIP.json MEMORY_HIGH_AFTER_SOAK.json MEMORY_HIGH_FINAL.json MEMORY_HIGH_CLEANUP.json; do
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
# UID 10001 may create its own generated observations, but may not unlink or
# replace host-owned independent identity/resource receipts in this directory.
test -O "$output"
test "$(stat -c %u "$output")" != 10001
sudo chmod +t "$output"
test -k "$output"
container_id=''
cleanup() {
  result=$?
  trap - EXIT
  if ! observe_browser_budget "$output"; then result=1; fi
  if [[ -n "$container_id" ]]; then
    docker stats --no-stream --format 'name={{.Name}} memory={{.MemUsage}} pids={{.PIDs}}' "$container_id" || true
    docker inspect --format '{{json .State}}' "$container_id" > "$output/CONTAINER_STATE.json" || true
    if [[ "$result" != 0 ]]; then docker logs --tail=120 "$container_id" || true; fi
    removed=false
    inventory_observed=false
    remaining_count=null
    if docker rm -f "$container_id"; then removed=true; fi
    # A successful host inventory query distinguishes removal from a dead or
    # inaccessible Docker daemon. The full ID belongs only to this fresh run.
    if remaining=$(docker ps -aq --no-trunc --filter "id=$container_id"); then
      inventory_observed=true
      remaining_count=0
      if [[ -n "$remaining" ]]; then remaining_count=1; fi
    fi
    jq -n --arg containerId "$container_id" --argjson removed "$removed" \
      --argjson inventoryObserved "$inventory_observed" --argjson remaining "$remaining_count" \
      '{containerId: $containerId, removalSucceeded: $removed,
        postRemovalInventoryObserved: $inventoryObserved, ownedContainersRemaining: $remaining}' \
      > "$output/CONTAINER_REMOVAL.json"
    if [[ "$removed" != true || "$inventory_observed" != true || "$remaining_count" != 0 ]]; then
      result=1
    fi
  fi
  if ! remove_browser_budget "$output"; then result=1; fi
  if [[ "$result" = 0 ]]; then
    if ! verify_observations --require-container-state --require-removal \
      > "$output/POST_REMOVAL_VERIFIED_SOAK.json"; then result=1; fi
  fi
  exit "$result"
}
trap cleanup EXIT
prepare_browser_budget "$memory_policy" "$output"
container_id=$(docker run -d --init --name "$container" --network none --shm-size=1g \
  "${budget_args[@]}" \
  --memory=2g --memory-swap=2g --pids-limit=256 --cpus=2 \
  --security-opt no-new-privileges:true --cap-drop ALL \
  --tmpfs /tmp:size=256m,mode=1777 \
  -v "$storage:/storage" -v "$PWD/scripts:/verification-scripts:ro" \
  -v "$output:/verification-output" \
  -e READER_SERVER_PORT=18892 -e READER_SERVER_BINDADDRESS=127.0.0.1 \
  -e READER_APP_WORKDIR=/ -e READER_APP_SECURE=true \
  -e READER_APP_LICENSECHECKENABLED=false -e READER_APP_WEBVIEWRENDERER=camoufox \
  -e READER_APP_BROWSERTIMEOUTMS=5000 -e READER_BROWSER_ALLOW_PRIVATE_NETWORKS=true "$image")
verify_browser_budget_membership "$container_id" "$output"
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
jq -n --arg revision "$revision" --arg jarSha256 "$actual_jar" --arg containerId "$container_id" \
  --arg memoryPolicy "$memory_policy" \
  '{revision: $revision, jarSha256: $jarSha256, network: "none", containerId: $containerId, memoryPolicy: $memoryPolicy}' \
  > "$output/RUNNING_JAR_IDENTITY.json"
docker exec "$container_id" python /verification-scripts/soak-bundled-browser.py \
  --reader-base http://127.0.0.1:18892 --expected-revision "$revision" --seconds "$seconds"
jq -e --arg revision "$revision" --argjson seconds "$seconds" \
  '.passed == true and .expectedImageRevision == $revision and .continuousSeconds >= $seconds and
   .rounds >= 20 and (.faults | length) == 3 and .targetCookieOrShapeErrors == 0 and
   .resources.swapMaxBytes == 0 and .resources.memoryEvents.max == 0 and
   .resources.memoryEvents.oom == 0 and .resources.memoryEvents.oom_kill == 0 and
   .resources.pidsEvents.max == 0' "$output/SOAK_REPORT.json" >/dev/null
observe_browser_budget "$output" MEMORY_HIGH_AFTER_SOAK.json
# Cross-check every observed round and each identity; the summary alone cannot
# prove continuous requests, fault recovery, or browser-process reclamation.
verify_observations() {
  python3 scripts/verify-bundled-browser-soak.py "$output" \
    --architecture "$arch" --expected-revision "$revision" \
    --expected-source-revision "$(jq -er '.head_sha' "$output/SOURCE_RUN.json")" \
    --expected-native-run "$(jq -er '.id' "$output/SOURCE_RUN.json")" \
    --expected-jar "$expected_jar" --expected-image "$(jq -er '.imageId' imported/metadata.json)" \
    --seconds "$seconds" --memory-policy "$memory_policy" "$@"
}
verify_observations | tee "$output/VERIFIED_SOAK.json"
