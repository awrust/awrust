FROM ghcr.io/awrust/awrust-s3:latest AS s3-bin

FROM rust:1.93-alpine AS facade-builder

RUN apk add --no-cache musl-dev

WORKDIR /build
COPY . .

RUN cargo build --release -p awrust

FROM --platform=$BUILDPLATFORM rust:1.93 AS cli-builder

ARG TARGETARCH
ARG AWRUST_CLI_VERSION=v0.1.0

RUN apt-get update && apt-get install -y --no-install-recommends git musl-tools \
    && if [ "$TARGETARCH" = "arm64" ]; then \
         apt-get install -y --no-install-recommends gcc-aarch64-linux-gnu; \
         rustup target add aarch64-unknown-linux-musl; \
       else \
         rustup target add x86_64-unknown-linux-musl; \
       fi \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build
RUN git clone --depth 1 --branch ${AWRUST_CLI_VERSION} https://github.com/awrust/awrust-cli.git .

RUN if [ "$TARGETARCH" = "arm64" ]; then \
      CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_LINKER=aarch64-linux-gnu-gcc \
        cargo build --release --target aarch64-unknown-linux-musl; \
      cp target/aarch64-unknown-linux-musl/release/awr /usr/local/bin/awr; \
    else \
      cargo build --release --target x86_64-unknown-linux-musl; \
      cp target/x86_64-unknown-linux-musl/release/awr /usr/local/bin/awr; \
    fi

FROM alpine:3

RUN addgroup -S awrust && adduser -S awrust -G awrust
RUN mkdir -p /data /etc/awrust/init/ready.d && chown -R awrust:awrust /data /etc/awrust

COPY --from=facade-builder /build/target/release/awrust /usr/local/bin/awrust
COPY --from=cli-builder /usr/local/bin/awr /usr/local/bin/awr
COPY --from=s3-bin /usr/local/bin/awrust-s3-server /usr/local/bin/awrust-s3-server

USER awrust

EXPOSE 4566 53/udp

HEALTHCHECK --interval=5s --timeout=3s --start-period=10s --retries=3 \
  CMD wget -qO- http://localhost:4566/health || exit 1

ENTRYPOINT ["awrust"]
