#!/usr/bin/env bash
# Sequential, generated-only HTTPS gate. No host/browser session or Reader data.
set -euo pipefail
test "$#" = 5
image="$1"
arch="$2"
revision="$3"
expected_jar="$4"
report="$5"
[[ "$arch" = amd64 || "$arch" = arm64 ]]
[[ "$revision" =~ ^[0-9a-f]{40}$ ]]
[[ "$expected_jar" =~ ^[0-9a-f]{64}$ ]]
test "${GITHUB_ACTIONS:-}" = true
: "${RUNNER_TEMP:?Expected a private GitHub runner temporary directory}"
test ! -e "$report"
test ! -L "$report"
test "$(docker image inspect --format '{{.Architecture}}' "$image")" = "$arch"
expected_worker=$(sha256sum src/main/resources/camoufox/worker.py | awk '{print $1}')
temporary=$(realpath "$RUNNER_TEMP")
directory=$(mktemp -d "$temporary/reader-native-tls.XXXXXXXX")
[[ "$directory" = "$temporary/reader-native-tls."* ]]
[[ "$directory" != *','* && "$directory" != *':'* ]]
name="reader-native-tls-$(basename "$directory")"
container_id=''
cleanup() {
  result=$?
  trap - EXIT
  if [[ -n "$container_id" ]]; then
    docker logs --tail=40 "$container_id" >&2 || true
    docker rm -f "$container_id" >/dev/null || true
  fi
  # Keep the tiny generated inputs/logs for diagnosis. No recursive deletion.
  exit "$result"
}
trap cleanup EXIT
for script in smoke-camoufox-tls.py compare-webview-cookie.py reader_header_security_differential.py report-browser-cgroup.py; do
  cp "scripts/$script" "$directory/$script"
done
# Discover the installed browser path/policy from the exact immutable image.
# This short read-only query cannot import CA files or reach an outside network.
docker run --rm --network none --read-only --user 10001:10001 --cap-drop ALL \
  --security-opt no-new-privileges:true --cpus=2 --memory=256m --memory-swap=256m --pids-limit=32 \
  -e OPENBLAS_NUM_THREADS=1 -e OMP_NUM_THREADS=1 -e MKL_NUM_THREADS=1 -e NUMEXPR_NUM_THREADS=1 \
  --entrypoint python "$image" -c '
import hashlib,json,os
from pathlib import Path
from camoufox.multiversion import get_active_path
p=Path(get_active_path()).resolve()/"distribution"
raw=(p/"policies.json").read_bytes()
print(json.dumps({"distribution":str(p),"policy":json.loads(raw),"policySha256":hashlib.sha256(raw).hexdigest(),"version":os.environ["READER_APP_CAMOUFOXBROWSERVERSION"]}))
' > "$directory/browser.json"
test "$(stat -c %s "$directory/browser.json")" -le 65536
distribution=$(jq -er '.distribution | select(type == "string" and test("^/home/reader/\\.cache/camoufox/browsers/official/[A-Za-z0-9._-]+/distribution$"))' "$directory/browser.json")
jq -e '.policy.policies | type == "object" and (has("Certificates") | not)' "$directory/browser.json" >/dev/null
chmod 755 "$directory"
chmod 444 "$directory"/*
container_id=$(docker run -d --init --name "$name" --network none --read-only \
  --user 10001:10001 --cap-drop ALL --security-opt no-new-privileges:true \
  --cpus=2 --memory=2g --memory-swap=2g --pids-limit=256 --shm-size=512m \
  --tmpfs /tmp:size=256m,mode=1777 \
  --tmpfs /home/reader/.camoufox:size=16m,mode=700,uid=10001,gid=10001 \
  --tmpfs "$distribution:size=1m,mode=700,uid=10001,gid=10001" \
  --mount "type=bind,source=$directory,target=/verification,readonly" \
  --entrypoint python "$image" /verification/smoke-camoufox-tls.py \
  --jar /app/reader.jar --expected-jar-sha "$expected_jar" --expected-worker-sha "$expected_worker" \
  --revision "$revision" --architecture "$arch" --distribution "$distribution" \
  --seed-policy /verification/browser.json)
docker inspect "$container_id" > "$directory/container-inspect.json"
jq -e --arg distribution "$distribution" '
  length == 1 and .[0].Config.User == "10001:10001" and
  .[0].HostConfig.ReadonlyRootfs == true and .[0].HostConfig.NetworkMode == "none" and
  .[0].HostConfig.Memory == 2147483648 and .[0].HostConfig.MemorySwap == 2147483648 and
  .[0].HostConfig.NanoCpus == 2000000000 and .[0].HostConfig.PidsLimit == 256 and
  .[0].HostConfig.Privileged == false and
  (.[0].HostConfig.CapDrop | index("ALL")) != null and
  (.[0].HostConfig.SecurityOpt | index("no-new-privileges:true")) != null and
  .[0].HostConfig.Tmpfs[$distribution] == "size=1m,mode=700,uid=10001,gid=10001"
' "$directory/container-inspect.json" >/dev/null
timeout --signal=TERM 300 docker wait "$container_id" > "$directory/exit-code"
test "$(cat "$directory/exit-code")" = 0
docker logs "$container_id" > "$directory/result.json" 2> "$directory/diagnostic.log"
python3 scripts/verify-camoufox-tls.py "$directory/result.json" --jar-sha "$expected_jar" \
  --worker-sha "$expected_worker" --revision "$revision" --architecture "$arch"
test "$(docker inspect --format '{{.State.Running}} {{.State.Pid}}' "$container_id")" = 'false 0'
docker rm "$container_id" >/dev/null
if docker inspect "$container_id" >/dev/null 2>&1; then
  echo 'Owned TLS container was not removed' >&2
  exit 1
fi
container_id=''
# A fresh, verified report only. The publisher will re-check all actual fields.
cp --no-clobber "$directory/result.json" "$report"
test -f "$report"
cmp "$directory/result.json" "$report"
