# Contributing to Emberforge Chain

Thanks for helping improve Emberforge Chain.

## Development workflow

1. Create a focused branch from `main`.
2. Make the smallest clear change that addresses the issue.
3. Run `cargo fmt -- --check` and `cargo test`.
4. Update the README when behavior or public APIs change.
5. Open a pull request describing the design and its security implications.

Blockchain changes need tests for both the valid path and the failure path. Never add real private keys, seed phrases, or production credentials to the repository.
