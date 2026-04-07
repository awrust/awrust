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
| `AWRUST_LISTEN_ADDR` | `0.0.0.0:4566` | Facade listen address |
| `AWRUST_SERVICES` | `s3` | Comma-separated list of services to enable |
| `AWRUST_LOG` | `info` | Log level filter (tracing syntax) |

Service-specific variables are passed through to child processes:

| Variable | Default | Description |
|----------|---------|-------------|
| `AWRUST_S3_STORE` | `memory` | S3 storage backend (`memory` or `fs`) |
| `AWRUST_S3_DATA_DIR` | `/data` | Filesystem storage directory |
| `AWRUST_S3_BASE_DOMAIN` | `localhost` | Base domain for virtual-host routing |

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
