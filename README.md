<p align="center">
  <img src="logo-wb.svg" alt="AWRust" width="200">
</p>

<h1 align="center">AWRust</h1>

<p align="center">
  Run AWS services locally with a single Docker container.
</p>

## What is AWRust?

AWRust lets you develop and test against AWS services without an AWS account or internet connection. Point your application at `localhost:4566` and use the same AWS SDKs and CLI tools you already know — no code changes required.

All services run behind a single port, managed by a lightweight facade that automatically routes requests to the right service emulator.

## Quick start

```bash
docker compose up
```

Your local AWS endpoint is now available at `http://localhost:4566`.

## Services

| Service | Status |
|---------|--------|
| S3      | Available |
| SQS     | Planned |

## Init scripts

Drop shell scripts into `/etc/awrust/init/ready.d/` (or bind-mount your own directory) to provision resources at startup — create buckets, seed data, anything you need before your app connects. See [Usage](docs/USAGE.md#init-scripts) for details.

## Documentation

- [Architecture](docs/ARCHITECTURE.md)
- [Usage](docs/USAGE.md)
