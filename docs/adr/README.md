# Architecture Decision Records (ADR)

This directory contains the architectural decisions for awrust.

ADRs document:
- Why decisions were made
- What alternatives were considered
- The consequences of those decisions

They exist to preserve context and prevent repeated debates.

---

## Status Legend

- **Accepted** — Currently in use
- **Superseded** — Replaced by a newer ADR
- **Proposed** — Under discussion
- **Deprecated** — No longer relevant

---

## Index

| ADR | Title | Status |
|-----|-------|--------|
| 0001 | Include curl in the Docker image | Superseded by 0007 |
| 0002 | Tokio as async runtime | Accepted |
| 0003 | Hyper as HTTP layer instead of a framework | Accepted |
| 0004 | socket2 for dual-stack networking | Accepted |
| 0005 | tracing for structured observability | Accepted |
| 0006 | Hyper for the facade, Axum for services — no forced standardization | Accepted |
| 0007 | Ship awrust CLI in the Docker image, remove curl | Accepted |
