FROM eclipse-temurin:11-jre-jammy AS jre11

# The only distributed image includes Chromium and its Linux dependencies.
# Keep this Playwright version equal to the Java dependency in both Gradle files.
FROM mcr.microsoft.com/playwright/java:v1.63.0-jammy
COPY --from=jre11 /opt/java/openjdk /opt/java/openjdk

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl fontconfig fonts-noto-cjk tini \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd --gid 10001 reader \
    && useradd --uid 10001 --gid 10001 --create-home --shell /usr/sbin/nologin reader \
    && mkdir -p /storage /logs \
    && chown reader:reader /storage /logs

WORKDIR /app
COPY reader-4.0.7.jar /app/reader.jar

ENV JAVA_HOME=/opt/java/openjdk \
    PATH=/opt/java/openjdk/bin:$PATH \
    PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD=1 \
    READER_APP_WEBVIEWRENDERER=local \
    HOME=/home/reader \
    TZ=Asia/Shanghai \
    LANG=C.UTF-8 \
    JAVA_TOOL_OPTIONS="-Xms256m -Xmx768m -Dfile.encoding=UTF-8"

USER 10001:10001
EXPOSE 8080
ENTRYPOINT ["/usr/bin/tini", "--", "java", "-jar", "/app/reader.jar"]
