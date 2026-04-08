# Usage

## Quick start

```yaml
services:
  awrust:
    image: ghcr.io/awrust/awrust:latest
    ports:
      - "4566:4566"
    environment:
      - AWRUST_SERVICES=s3
```

```python
import boto3

s3 = boto3.client(
    "s3",
    endpoint_url="http://localhost:4566",
    aws_access_key_id="test",
    aws_secret_access_key="test",
    region_name="us-east-1",
)

s3.create_bucket(Bucket="my-bucket")
s3.put_object(Bucket="my-bucket", Key="hello.txt", Body=b"hello")
```

## Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `AWRUST_LISTEN_ADDR` | `[::]:4566` | Facade listen address (dual-stack IPv4+IPv6) |
| `AWRUST_SERVICES` | `s3` | Comma-separated list of services to enable |
| `AWRUST_BASE_DOMAIN` | `localhost` | Base domain for virtual-host routing (cascades to all services) |
| `AWRUST_LOG` | `info` | Log level filter (tracing syntax) |

Service-specific variables are passed through to child processes.
Each service inherits `AWRUST_BASE_DOMAIN` unless its own `*_BASE_DOMAIN` is set.

| Variable | Default | Description |
|----------|---------|-------------|
| `AWRUST_S3_STORE` | `memory` | S3 storage backend (`memory` or `fs`) |
| `AWRUST_S3_DATA_DIR` | `/data` | Filesystem storage directory |
| `AWRUST_S3_BASE_DOMAIN` | `AWRUST_BASE_DOMAIN` | S3 virtual-host domain override |

DNS variables (opt-in, resolves `*.{AWRUST_BASE_DOMAIN}` for all services):

| Variable | Default | Description |
|----------|---------|-------------|
| `AWRUST_DNS` | `false` | Enable built-in DNS responder (`true` or `1`) |
| `AWRUST_DNS_ADDR` | `[::]:53` | DNS listen address (dual-stack IPv4+IPv6) |
| `AWRUST_DNS_RESOLVE_IP` | auto-detect | IP returned for matching queries |
| `AWRUST_DNS_UPSTREAM` | from `/etc/resolv.conf` | Upstream DNS for non-matching queries |

## Virtual-hosted style

To use virtual-hosted-style URLs (e.g., `http://mybucket.awrust:4566/key`), enable the DNS responder and point client containers at it:

```yaml
services:
  awrust:
    image: ghcr.io/awrust/awrust:latest
    ports:
      - "4566:4566"
    environment:
      - AWRUST_SERVICES=s3
      - AWRUST_BASE_DOMAIN=awrust
      - AWRUST_DNS=true
    networks:
      default:
        ipv4_address: 172.20.0.10

  app:
    build: .
    dns: 172.20.0.10
    environment:
      - AWS_ENDPOINT_URL=http://awrust:4566

networks:
  default:
    ipam:
      config:
        - subnet: 172.20.0.0/16
```

The DNS responder resolves `*.{AWRUST_BASE_DOMAIN}` to the container IP, forwarding all other queries upstream. This works for any service that uses virtual-hosted routing — set `AWRUST_BASE_DOMAIN` once and every service inherits it.

## Health check

```bash
curl http://localhost:4566/health
```

```json
{"status": "ok", "services": {"s3": {"status": "ok"}}}
```

## AWS CLI

```bash
aws --endpoint-url http://localhost:4566 s3 mb s3://my-bucket
aws --endpoint-url http://localhost:4566 s3 cp file.txt s3://my-bucket/
aws --endpoint-url http://localhost:4566 s3 ls s3://my-bucket/
```

## Supported services

| Service | Status |
|---------|--------|
| S3 | Available |
| SQS | Planned |
