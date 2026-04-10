# Architecture

awrust is a process manager and reverse proxy that bundles AWS service emulators behind a single port.

## Design

```
Client → :4566 → awrust (facade)
                   │ route by: SigV4 scope → default S3
                   ├── awrust-s3-server (127.0.0.1:<ephemeral>)
                   └── ...

       → :53/udp → DNS responder (opt-in)
                   │ *.base_domain → container IP
                   └── other       → upstream DNS
```

The facade has four responsibilities:

1. **Process management** — spawn service binaries as child processes on random ephemeral ports, health-check them, and shut them down cleanly on SIGTERM.

2. **Service routing** — inspect incoming HTTP requests to determine which service should handle them, without reading the request body.

3. **Reverse proxy** — forward the request to the target service and stream the response back, without buffering.

4. **DNS responder** (opt-in) — resolve `*.{base_domain}` to the container IP so virtual-hosted S3 URLs work without external DNS configuration. Non-matching queries are forwarded upstream.

## Routing algorithm

1. Parse `Authorization` header for SigV4 credential scope (`Credential=.../s3/aws4_request`). The service name is the 4th `/`-delimited segment.
2. Default to S3.

This covers all AWS SDK calls (which always sign requests) and unsigned requests (which default to S3, the most common case).

## Health check

`GET /health` without AWS headers returns aggregated facade health:

| Facade status | HTTP code | Condition |
|---------------|-----------|-----------|
| `ok` | 200 | All services healthy and init scripts complete |
| `initializing` | 503 | Init scripts still running |
| `degraded` | 503 | One or more services unreachable |

Each service reports `ok` or `unhealthy` independently.

```json
{"status": "ok", "services": {"s3": {"status": "ok"}}}
```

`GET /health` with `Authorization` or `x-amz-*` headers routes to the target service.

## Process lifecycle

1. For each enabled service, bind a `TcpListener` to `127.0.0.1:0` to obtain a free port, then drop it.
2. Spawn the service binary, passing the port via its `*_LISTEN_ADDR` environment variable.
3. Poll TCP connect every 100ms until the service is accepting connections (15s timeout).
4. On SIGTERM/SIGINT, kill all child processes and wait for them to exit.

Child stdout/stderr are inherited — all services use JSON structured logging, and Docker handles log multiplexing.

## Init scripts

After all services are healthy, the facade runs shell scripts from the init directory (`/etc/awrust/init/ready.d/` by default, configurable via `AWRUST_INIT_DIR`).

1. Read all files from the init directory (subdirectories are ignored).
2. Sort by filename and execute each via `/bin/sh` in order.
3. Each script receives `AWRUST_ENDPOINT` (e.g., `http://127.0.0.1:4566`) as an environment variable.
4. Failed scripts are logged but do not block subsequent scripts.
5. The health endpoint returns `"initializing"` / 503 until all scripts finish.

This is the intended mechanism for provisioning resources (buckets, queues) at container startup.

## Docker image

The Docker image bundles the facade binary and all service binaries in a single Alpine container. Service binaries are pulled from their published GHCR images using multi-stage builds:

```dockerfile
FROM ghcr.io/awrust/awrust-s3:latest AS s3-bin
COPY --from=s3-bin /usr/local/bin/awrust-s3-server /usr/local/bin/
```

Adding a new service requires one `FROM` stage and one `COPY` line.

The [`awr` CLI](https://github.com/awrust/awrust-cli) is built from source in a parallel build stage and included in the image. It reads `AWRUST_ENDPOINT` — the same environment variable the init system provides — so init scripts can use `awr s3 mb my-bucket` instead of raw HTTP calls.

The runtime stage creates an unprivileged `awrust` user and includes a `HEALTHCHECK` instruction that polls `/health`.

## Non-goals

- Full AWS parity
- SigV4 signature verification
- TLS termination
- Request buffering, caching, or retry
- Dynamic service discovery or plugin system
- Configuration files (environment variables only)
