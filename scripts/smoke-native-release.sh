#!/usr/bin/env bash
# Generated fixtures only. This never connects to production or uses real books.
set -euo pipefail
test "$#" = 5
arch="$1"
version="$2"
revision="$3"
stage="$4"
output="$5"
[[ "$arch" = amd64 || "$arch" = arm64 ]]
[[ "$version" =~ ^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$ ]]
[[ "$revision" =~ ^[0-9a-f]{40}$ ]]
[[ "$stage" = built || "$stage" = transferred ]]
: "${RUNNER_TEMP:?Expected an isolated GitHub runner temporary directory}"
image="reader-pro:camoufox-smoke-$arch"
container="reader-native-$stage-$arch"
storage="$RUNNER_TEMP/reader-native-smoke-$stage-$arch"
port=18890
if [[ "$stage" = transferred ]]; then port=18891; fi
test "$(docker image inspect --format '{{.Architecture}}' "$image")" = "$arch"
if docker container inspect "$container" >/dev/null 2>&1 || test -e "$storage"; then
  echo 'Refusing to reuse an existing smoke container or storage directory' >&2
  exit 1
fi
mkdir -p "$output"
install -d "$storage"
sudo chown 10001:10001 "$storage"
container_id=''
cleanup() {
  result=$?
  trap - EXIT
  if [[ -n "$container_id" ]]; then
    if [[ "$result" != 0 ]]; then docker logs --tail=120 "$container_id" || true; fi
    docker stats --no-stream --format 'name={{.Name}} memory={{.MemUsage}} pids={{.PIDs}}' "$container_id" || true
    docker rm -f "$container_id" || true
  fi
  exit "$result"
}
trap cleanup EXIT
container_id=$(docker run -d --init --name "$container" --network host --shm-size=1g \
  --memory=2g --memory-swap=3g --pids-limit=256 --cpus=2 \
  --security-opt no-new-privileges:true --cap-drop ALL \
  --tmpfs /tmp:size=256m,mode=1777 \
  -v "$storage:/storage" \
  -e "READER_SERVER_PORT=$port" -e READER_SERVER_BINDADDRESS=127.0.0.1 \
  -e READER_APP_WORKDIR=/ -e READER_APP_SECURE=true \
  -e READER_APP_LICENSECHECKENABLED=false -e READER_APP_WEBVIEWRENDERER=camoufox \
  -e READER_BROWSER_ALLOW_PRIVATE_NETWORKS=true "$image")
ready=false
for attempt in $(seq 1 60); do
  if curl -fsS --max-time 5 "http://127.0.0.1:$port/reader3/getSystemInfo" \
      | jq -e '.isSuccess == true' >/dev/null; then
    ready=true
    break
  fi
  sleep 2
done
test "$ready" = true
expected_jar=$(sha256sum "dist/reader-pro-v${version}.jar" | awk '{print $1}')
actual_jar=$(docker exec "$container_id" sha256sum /app/reader.jar | awk '{print $1}')
test "$actual_jar" = "$expected_jar"
jq -n --arg arch "$arch" --arg revision "$revision" --arg jarSha256 "$actual_jar" \
  '{architecture: $arch, revision: $revision, jarSha256: $jarSha256}' > "$output/JAR_IDENTITY.json"
ss -lntH | grep ":$port" | tee "$output/reader-release-listener.txt"
grep -Eq "(^|[[:space:]])(127\\.0\\.0\\.1|\\[::ffff:127\\.0\\.0\\.1\\]):$port([[:space:]]|$)" \
  "$output/reader-release-listener.txt"
curl -fsS --max-time 10 "http://127.0.0.1:$port/assets/reader-release.json" \
  | jq -e --arg version "$version" --arg revision "$revision" \
    '.version == $version and .buildRevision == $revision' > "$output/RELEASE_IDENTITY.json"
docker exec "$container_id" python -m camoufox version
docker exec "$container_id" sh -ec '
  family=$(fc-list :lang=zh family | sed -n "1p")
  test -n "$family"
  printf "CJK font family: %s\n" "$family"
'
python3 scripts/smoke-local-webview.py --reader-base "http://127.0.0.1:$port" \
  | tee "$output/BROWSER_SYNTHETIC.json"
docker exec -i "$container_id" python - < scripts/report-browser-cgroup.py \
  | tee "$output/BROWSER_RESOURCE_BUDGET.json"
