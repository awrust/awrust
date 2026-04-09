FROM ghcr.io/awrust/awrust-s3:latest AS s3-bin

FROM rust:1.93-alpine AS builder

RUN apk add --no-cache musl-dev

WORKDIR /build
COPY . .

RUN cargo build --release -p awrust

FROM alpine:3

RUN addgroup -S awrust && adduser -S awrust -G awrust
RUN mkdir -p /data /etc/awrust/init/ready.d && chown -R awrust:awrust /data /etc/awrust

COPY --from=builder /build/target/release/awrust /usr/local/bin/awrust
COPY --from=s3-bin /usr/local/bin/awrust-s3-server /usr/local/bin/awrust-s3-server

USER awrust

EXPOSE 4566 53/udp

HEALTHCHECK --interval=5s --timeout=3s --start-period=10s --retries=3 \
  CMD wget -qO- http://localhost:4566/health || exit 1

ENTRYPOINT ["awrust"]
