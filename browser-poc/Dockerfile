# Build context: reader-pro-restored root. This image tests packaging only;
# it is not the Reader production image or a fingerprint-browser selection.
FROM eclipse-temurin:11-jdk-jammy AS build
WORKDIR /work
COPY gradlew ./
COPY gradle/wrapper/ ./gradle/wrapper/
COPY browser-poc/ ./browser-poc/
ENV PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD=1
RUN chmod +x ./gradlew && ./gradlew -p browser-poc installDist --no-daemon

FROM eclipse-temurin:11-jre-jammy AS jre11

# The matched browser binaries and OS libraries are already in this image.
# The official Java image uses a newer JDK, so copy JRE 11 for the restored app.
FROM mcr.microsoft.com/playwright/java:v1.63.0-jammy
COPY --from=jre11 /opt/java/openjdk /opt/java/openjdk
COPY --from=build /work/browser-poc/build/install/reader-browser-poc /opt/browser-poc
ENV JAVA_HOME=/opt/java/openjdk \
    PATH=/opt/java/openjdk/bin:$PATH \
    PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD=1
USER pwuser
WORKDIR /home/pwuser
ENTRYPOINT ["/opt/browser-poc/bin/reader-browser-poc", "bundled"]
CMD ["data:text/html,%3Chtml%3E%3Cbody%3Ebrowser-ok%3C%2Fbody%3E%3C%2Fhtml%3E"]
