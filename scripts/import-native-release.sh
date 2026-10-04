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
for report in metadata.json BASE_IMAGE_DIGESTS BROWSER_SYNTHETIC.json BROWSER_RESOURCE_BUDGET.json RELEASE_IDENTITY.json JAR_IDENTITY.json; do
  cp "$directory/$report" "$dist/${arch}-${report}"
done
for report in IMAGE_IDENTITY.json BROWSER_SYNTHETIC.json BROWSER_RESOURCE_BUDGET.json RELEASE_IDENTITY.json JAR_IDENTITY.json; do
  cp "$evidence/$report" "$dist/${arch}-transfer-${report}"
done
# Free only the validated archive in the exact, resolved CI import directory.
# Never recurse, delete a directory, or remove any source/user data.
rm -- "$archive"
