# awrust — Project Instructions

## Build & Test

```bash
cargo fmt --all --check
cargo clippy -- -D warnings
cargo test --workspace
```

## Development workflow

### TDD is mandatory
1. Write failing tests first (BDD scenarios + Rust unit tests)
2. Run tests — confirm they fail (RED)
3. Implement the production code
4. Run tests — confirm they pass (GREEN)
5. Run `cargo fmt --all && cargo clippy -- -D warnings` before committing

## Code principles

- No comments; code is truth
- No backwards compatibility hacks; only move forward
- No storing what can be computed
- No duplication; extract when patterns emerge
- Expose what must be exposed, hide what must be hidden
