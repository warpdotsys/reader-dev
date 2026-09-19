FROM eclipse-temurin:17-jre
WORKDIR /app
COPY build/libs/reader-license-server-*-all.jar /app/license-server.jar
ENV LICENSE_BIND=0.0.0.0 LICENSE_PORT=8080 LICENSE_DATA_DIR=/var/lib/reader-license
RUN mkdir -p /var/lib/reader-license && chown -R 10001:10001 /var/lib/reader-license
VOLUME ["/var/lib/reader-license"]
EXPOSE 8080
USER 10001
ENTRYPOINT ["java","-jar","/app/license-server.jar"]
