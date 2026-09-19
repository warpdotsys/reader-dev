FROM eclipse-temurin:11-jre-jammy

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl fontconfig fonts-noto-cjk tini \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd --gid 10001 reader \
    && useradd --uid 10001 --gid 10001 --no-create-home --shell /usr/sbin/nologin reader

WORKDIR /app
COPY reader-4.0.7.jar /app/reader.jar

ENV TZ=Asia/Shanghai \
    LANG=C.UTF-8 \
    JAVA_TOOL_OPTIONS="-Xms256m -Xmx512m -Dfile.encoding=UTF-8"

USER 10001:10001
EXPOSE 8080
ENTRYPOINT ["/usr/bin/tini", "--", "java", "-jar", "/app/reader.jar"]

