ARG TEMURIN_JRE_IMAGE
ARG PLAYWRIGHT_JAVA_IMAGE
FROM ${TEMURIN_JRE_IMAGE} AS jre11

# The only distributed image includes the Camoufox runtime and its Linux dependencies.
# Keep this Playwright version equal to the Java dependency in both Gradle files.
FROM ${PLAYWRIGHT_JAVA_IMAGE}
COPY --from=jre11 /opt/java/openjdk /opt/java/openjdk

COPY apt-sources.list /etc/apt/sources.list

RUN rm -rf /etc/apt/sources.list.d \
    && mkdir -p /etc/apt/sources.list.d \
    && apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl fontconfig fonts-noto-cjk tini python3 python3-venv \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd --gid 10001 reader \
    && useradd --uid 10001 --gid 10001 --create-home --shell /usr/sbin/nologin reader \
    && mkdir -p /storage /logs \
    && chown reader:reader /storage /logs

COPY requirements-camoufox.lock /tmp/requirements-camoufox.lock
COPY camoufox-browser.lock install_camoufox_pinned.py /tmp/
COPY --chmod=0755 docker-entrypoint.sh /usr/local/bin/reader-entrypoint
ARG REQUIRE_WHEEL_HASHES=true
RUN python3 -m venv /opt/reader-camoufox \
    && if [ "$REQUIRE_WHEEL_HASHES" = true ]; then hash_arg='--require-hashes'; else hash_arg=''; fi \
    && /opt/reader-camoufox/bin/pip install --no-cache-dir $hash_arg --requirement /tmp/requirements-camoufox.lock \
    && /opt/reader-camoufox/bin/python -m playwright install-deps firefox \
    && chown -R reader:reader /opt/reader-camoufox /home/reader \
    && rm -f /tmp/requirements-camoufox.lock \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
ARG READER_JAR
ARG READER_VERSION
ARG READER_BUILD_REVISION
COPY ${READER_JAR} /app/reader.jar

ENV JAVA_HOME=/opt/java/openjdk \
    PATH=/opt/reader-camoufox/bin:/opt/java/openjdk/bin:$PATH \
    PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD=1 \
    PYTHONUNBUFFERED=1 \
    READER_APP_WEBVIEWRENDERER=camoufox \
    READER_APP_CAMOUFOXPYTHONEXECUTABLE=/opt/reader-camoufox/bin/python \
    READER_APP_CAMOUFOXBROWSERVERSION=152.0.4-beta.30 \
    READER_RELEASE_VERSION=${READER_VERSION} \
    READER_BUILD_REVISION=${READER_BUILD_REVISION} \
    HOME=/home/reader \
    TZ=Asia/Shanghai \
    LANG=C.UTF-8 \
    JAVA_TOOL_OPTIONS="-Xms256m -Xmx768m -Dfile.encoding=UTF-8"

RUN test -n "$READER_VERSION" \
    && test -n "$READER_BUILD_REVISION"

USER 10001:10001
RUN python /tmp/install_camoufox_pinned.py \
    && rm -f /tmp/install_camoufox_pinned.py /tmp/camoufox-browser.lock
EXPOSE 8080
ENTRYPOINT ["/usr/local/bin/reader-entrypoint"]
