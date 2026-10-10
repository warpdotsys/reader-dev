#!/usr/bin/env bash
# Shared by the credential-free rehearsal and the formal publisher.
# Consume, never rebuild, the exact native images and fresh-runner evidence.
set -euo pipefail
test "$#" = 4
arch="$1"
version="$2"
revision="$3"
dist="$4"
[[ "$arch" = amd64 || "$arch" = arm64 ]]
: "${RUNNER_TEMP:?Expected an isolated GitHub runner temporary directory}"
test "${GITHUB_ACTIONS:-}" = true
temporary=$(realpath "$RUNNER_TEMP")
directory=$(realpath "$RUNNER_TEMP/reader-native-import/$arch")
test "$directory" = "$temporary/reader-native-import/$arch"
archive="$directory/reader-image.tar.gz"
test -f "$archive"
test ! -L "$archive"
evidence="$RUNNER_TEMP/reader-native-transfers/reader-native-transfer-$arch-$revision"
(cd "$dist" && sha256sum -c SHA256SUMS)
node scripts/release-native-artifacts.mjs check "$arch" "$version" "$revision" "$dist" "$directory"
docker load --input "$archive"
docker image inspect "reader-pro:camoufox-smoke-$arch" > "$directory/image-inspect.json"
node scripts/release-native-artifacts.mjs loaded "$arch" "$version" "$revision" "$dist" "$directory"
cmp "$evidence/metadata.json" "$directory/metadata.json"
for report_directory in "$directory" "$evidence"; do
  python3 scripts/verify-reader-async-smoke.py "$report_directory/BROWSER_SYNTHETIC.json"
  python3 scripts/verify-reader-metadata-smoke.py "$report_directory/BROWSER_SYNTHETIC.json"
  # Retain and require the actual identity object, not only a boolean assertion.
  jq -e --arg version "$version" --arg revision "$revision" \
    'type == "object" and .version == $version and .buildRevision == $revision' \
    "$report_directory/RELEASE_IDENTITY.json" >/dev/null
  jar_sha=$(jq -er '.jarSha256' "$directory/metadata.json")
  worker_sha=$(sha256sum src/main/resources/camoufox/worker.py | awk '{print $1}')
  python3 scripts/verify-camoufox-tls.py "$report_directory/BROWSER_TLS.json" \
    --jar-sha "$jar_sha" --worker-sha "$worker_sha" --revision "$revision" --architecture "$arch"
  python3 scripts/verify-reader-default-ui.py check "$report_directory/DEFAULT_UI.json" \
    --expected-jar-sha "$jar_sha"
  jq -e --arg arch "$arch" --arg revision "$revision" --arg jar_sha "$jar_sha" \
    'type == "object" and .architecture == $arch and .revision == $revision and .jarSha256 == $jar_sha' \
    "$report_directory/JAR_IDENTITY.json" >/dev/null
done
for report in metadata.json BASE_IMAGE_DIGESTS BROWSER_SYNTHETIC.json BROWSER_RESOURCE_BUDGET.json BROWSER_TLS.json RELEASE_IDENTITY.json JAR_IDENTITY.json DEFAULT_UI.json; do
  cp "$directory/$report" "$dist/${arch}-${report}"
done
for report in IMAGE_IDENTITY.json BROWSER_SYNTHETIC.json BROWSER_RESOURCE_BUDGET.json BROWSER_TLS.json RELEASE_IDENTITY.json JAR_IDENTITY.json DEFAULT_UI.json; do
  cp "$evidence/$report" "$dist/${arch}-transfer-${report}"
done
# Free only the validated archive in the exact, resolved CI import directory.
# Never recurse, delete a directory, or remove any source/user data.
rm -- "$archive"
