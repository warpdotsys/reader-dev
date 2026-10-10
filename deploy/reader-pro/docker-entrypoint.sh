#!/bin/sh
set -eu

# This is staging only, not storage/data or storage/assets. The image's fixed
# /app/file-uploads symlink keeps the original Vert.x relative path unchanged.
# Reject an externally substituted or non-private temporary directory; never
# chmod/chown the application root or an existing user's storage to fix uploads.
upload_tmp=/tmp/reader-file-uploads
test ! -L "$upload_tmp"
mkdir -p -m 0700 "$upload_tmp"
test -d "$upload_tmp"
test "$(stat -c '%u' -- "$upload_tmp")" = "$(id -u)"
test "$(stat -c '%a' -- "$upload_tmp")" = 700
test -w "$upload_tmp"

# This non-secret marker lets the public release probe verify that traffic
# reached the freshly deployed image. It is intentionally written below the
# existing assets mount so it survives the image's read-only application root.
mkdir -p /storage/assets
printf '{"version":"%s","buildRevision":"%s"}\n' \
  "${READER_RELEASE_VERSION:?missing READER_RELEASE_VERSION}" \
  "${READER_BUILD_REVISION:?missing READER_BUILD_REVISION}" \
  > /storage/assets/reader-release.json
chmod 0644 /storage/assets/reader-release.json

exec java -jar /app/reader.jar
