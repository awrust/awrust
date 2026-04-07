# awrust

Lightweight AWS service emulators behind a single port. One Docker image, one `docker-compose` service, all your local AWS needs.

## Quick start

```yaml
services:
  awrust:
    image: ghcr.io/awrust/awrust:latest
    ports:
      - "4566:4566"
```

```python
import boto3

s3 = boto3.client("s3", endpoint_url="http://localhost:4566",
    aws_access_key_id="test", aws_secret_access_key="test", region_name="us-east-1")

s3.create_bucket(Bucket="my-bucket")
s3.put_object(Bucket="my-bucket", Key="hello.txt", Body=b"hello")
```

## Services

| Service | Status |
|---------|--------|
| S3 | Available |
| SQS | Planned |

## How it works

awrust is a facade that spawns service emulators as child processes and routes incoming requests by inspecting AWS SigV4 headers. All services share a single port.

See [Architecture](docs/ARCHITECTURE.md) and [Usage](docs/USAGE.md) for details.

## Philosophy

- Small, focused services
- Docker-first distribution
- Fast, deterministic local development
- Not production workloads, not perfect AWS compatibility
- MIT licensed
