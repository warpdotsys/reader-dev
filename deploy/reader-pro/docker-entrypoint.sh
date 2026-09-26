#!/bin/sh
set -eu

# This non-secret marker lets the public release probe verify that traffic
# reached the freshly deployed image. It is intentionally written below the
# existing assets mount so it survives the image's read-only application root.
mkdir -p /storage/assets
printf '{"version":"%s","buildRevision":"%s"}\n' \
  "${READER_RELEASE_VERSION:?missing READER_RELEASE_VERSION}" \
  "${READER_BUILD_REVISION:?missing READER_BUILD_REVISION}" \
  > /storage/assets/reader-release.json
chmod 0644 /storage/assets/reader-release.json

exec /usr/bin/tini -- java -jar /app/reader.jar
