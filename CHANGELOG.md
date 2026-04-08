# Changelog

## 0.1.0

Initial release with S3 support.

### Facade

- Single-port gateway that spawns and supervises service emulators
- SigV4 header-based request routing
- Reverse proxy with URI rewriting to child processes
- Health endpoint (`/health`) reporting per-service status
- Ephemeral port allocation for child services
- Graceful shutdown on SIGTERM/SIGINT
- JSON-structured logging via `tracing`

### S3

- CreateBucket / HeadBucket / ListBuckets
- PutObject / GetObject / HeadObject
- CopyObject
- DeleteObjects (batch)
- CreateMultipartUpload / UploadPart / CompleteMultipartUpload

### Configuration

- `AWRUST_LISTEN_ADDR` — facade listen address (default `0.0.0.0:4566`)
- `AWRUST_SERVICES` — comma-separated service list (default `s3`)
- `AWRUST_LOG` — tracing filter directive (default `info`)
- `AWRUST_S3_LISTEN_ADDR` — S3 service listen address
- `AWRUST_S3_STORE` — storage backend (`memory`)
- `AWRUST_S3_DATA_DIR` — data directory for persistent storage
- `AWRUST_S3_BASE_DOMAIN` — base domain for virtual-hosted buckets

### Docker

- Multi-arch image (linux/amd64, linux/arm64)
- Alpine-based, non-root runtime
- Published to `ghcr.io/awrust/awrust`
