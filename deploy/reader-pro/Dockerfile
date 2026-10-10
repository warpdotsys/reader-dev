ARG TEMURIN_JRE_IMAGE
ARG PLAYWRIGHT_PYTHON_IMAGE
FROM ${TEMURIN_JRE_IMAGE} AS jre11

# The only distributed image includes the Camoufox runtime and its Linux dependencies.
# Pin the Python browser base by OCI digest. It already contains Python, pip,
# the Playwright browsers and their Ubuntu dependencies, so this build does
# not need a live apt repository to install the Camoufox worker runtime.
FROM ${PLAYWRIGHT_PYTHON_IMAGE}
COPY --from=jre11 /opt/java/openjdk /opt/java/openjdk

RUN groupadd --gid 10001 reader \
    && useradd --uid 10001 --gid 10001 --create-home --shell /usr/sbin/nologin reader \
    && mkdir -p /storage /logs \
    && chown reader:reader /storage /logs

COPY requirements-camoufox.lock /tmp/requirements-camoufox.lock
COPY --chown=10001:10001 camoufox-browser.lock install_camoufox_pinned.py /tmp/
COPY --chmod=0755 docker-entrypoint.sh /usr/local/bin/reader-entrypoint
ARG REQUIRE_WHEEL_HASHES=true
RUN if [ "$REQUIRE_WHEEL_HASHES" = true ]; then hash_arg='--require-hashes'; else hash_arg=''; fi \
    && python3 -m pip install --no-cache-dir $hash_arg --requirement /tmp/requirements-camoufox.lock \
    && chown -R reader:reader /home/reader \
    && rm -f /tmp/requirements-camoufox.lock

WORKDIR /app
ARG READER_JAR
ARG READER_VERSION
ARG READER_BUILD_REVISION
COPY ${READER_JAR} /app/reader.jar

ENV JAVA_HOME=/opt/java/openjdk \
    PATH=/opt/java/openjdk/bin:$PATH \
    PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD=1 \
    PYTHONUNBUFFERED=1 \
    READER_APP_WEBVIEWRENDERER=camoufox \
    READER_APP_WEBUI=vue3 \
    READER_APP_CAMOUFOXPYTHONEXECUTABLE=/usr/bin/python3 \
    READER_APP_CAMOUFOXBROWSERVERSION=152.0.4-beta.30 \
    READER_RELEASE_VERSION=${READER_VERSION} \
    READER_BUILD_REVISION=${READER_BUILD_REVISION} \
    HOME=/home/reader \
    TZ=Asia/Shanghai \
    LANG=C.UTF-8 \
    JAVA_TOOL_OPTIONS="-Xms256m -Xmx768m -Dfile.encoding=UTF-8"

# Vert.x 3.8.5 hard-codes the legacy relative staging directory.
# Preserve that path without making the application/JAR root writable.
RUN test -n "$READER_VERSION" \
    && test -n "$READER_BUILD_REVISION" \
    && test ! -e /app/file-uploads && test ! -L /app/file-uploads \
    && ln -s /tmp/reader-file-uploads /app/file-uploads

USER 10001:10001
RUN python /tmp/install_camoufox_pinned.py \
    && rm -f /tmp/install_camoufox_pinned.py /tmp/camoufox-browser.lock
EXPOSE 8080
ENTRYPOINT ["/usr/local/bin/reader-entrypoint"]
