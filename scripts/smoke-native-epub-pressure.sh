#!/usr/bin/env bash
# Actual default packaged UI, generated data only, sequential and fail closed.
set -euo pipefail
test "$#" = 5
arch="$1"
version="$2"
revision="$3"
stage="$4"
report="$5"
[[ "$arch" = amd64 || "$arch" = arm64 ]]
[[ "$version" =~ ^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$ ]]
[[ "$revision" =~ ^[0-9a-f]{40}$ ]]
[[ "$stage" = built || "$stage" = transferred ]]
test "${GITHUB_ACTIONS:-}" = true
: "${RUNNER_TEMP:?Expected an isolated hosted runner temporary directory}"
test ! -e "$report"
test ! -L "$report"
test -s "dist/reader-pro-v${version}.jar"
case "$arch:$RUNNER_ARCH:$(uname -m)" in
  amd64:X64:x86_64|arm64:ARM64:aarch64) ;;
  *) exit 1 ;;
esac
image="reader-pro:camoufox-smoke-$arch"
test "$(docker image inspect --format '{{.Architecture}}' "$image")" = "$arch"
jar_sha=$(sha256sum "dist/reader-pro-v${version}.jar" | awk '{print $1}')
[[ "$jar_sha" =~ ^[0-9a-f]{64}$ ]]
mkdir -p "$(dirname "$report")"
# systemd creates the aggregate budget before the controller, generator, JRE
# and packaged browser start. The dispatcher itself only sets up/collects it.
sudo -n -- "$(command -v python3)" scripts/epub-pressure/runner.py \
  --image "$image" --architecture "$arch" --revision "$revision" \
  --jar-sha "$jar_sha" --work-root "$RUNNER_TEMP" --report "$report"
python3 scripts/epub-pressure/verify.py "$report" \
  --architecture "$arch" --revision "$revision" --jar-sha "$jar_sha"
