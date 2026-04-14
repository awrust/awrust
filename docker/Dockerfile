FROM ghcr.io/awrust/awrust-s3:0.3.3 AS s3-bin

FROM rust:1.93-alpine AS facade-builder

RUN apk add --no-cache musl-dev

WORKDIR /build
COPY . .

RUN cargo build --release -p awrust

FROM --platform=$BUILDPLATFORM alpine:3 AS cli-downloader

ARG TARGETARCH
ARG AWRUST_CLI_VERSION=v0.1.2

RUN apk add --no-cache curl
RUN case "$TARGETARCH" in \
      amd64) ARCH=x86_64 ;; \
      arm64) ARCH=aarch64 ;; \
    esac \
    && curl -sL "https://github.com/awrust/awrust-cli/releases/download/${AWRUST_CLI_VERSION}/awrust-cli-${ARCH}-unknown-linux-musl.tar.xz" \
       | tar xJ --strip-components=1 -C /usr/local/bin awrust-cli-${ARCH}-unknown-linux-musl/awr

FROM alpine:3

RUN addgroup -S awrust && adduser -S awrust -G awrust
RUN mkdir -p /data /etc/awrust/init/ready.d && chown -R awrust:awrust /data /etc/awrust

COPY --from=facade-builder /build/target/release/awrust /usr/local/bin/awrust
COPY --from=cli-downloader /usr/local/bin/awr /usr/local/bin/awr
COPY --from=s3-bin /usr/local/bin/awrust-s3-server /usr/local/bin/awrust-s3-server

USER awrust

EXPOSE 4566 53/udp

HEALTHCHECK --interval=5s --timeout=3s --start-period=10s --retries=3 \
  CMD wget -qO- http://localhost:4566/health || exit 1

ENTRYPOINT ["awrust"]
