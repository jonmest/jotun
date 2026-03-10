# Contributing

Thanks for your interest. This document covers how to get set up and what to expect when contributing.

## Setup

You need Rust (stable) and `just`. The toolchain is pinned in `rust-toolchain.toml`, so rustup will install the right version automatically.

For the full local CI loop you also need a few cargo tools:

```
cargo install cargo-deny cargo-audit cargo-machete typos-cli
```

If you want to run the GitHub Actions workflow locally, install `act` and Docker.

## Build and test

```
just build
just test
```

Before opening a pull request, run the full local check:

```
just ci
```

This runs formatting, clippy, tests, license and advisory checks, unused-dependency detection, and typos. It's the same set of checks that GitHub Actions runs.

## Pull requests

- Keep changes focused. Separate refactors from behavior changes.
- Add tests for new behavior. For anything in the `transport/mapping` layer, add both a round-trip property test and a negative test.
- Clippy warnings are errors in CI. Fix them or explain why they are wrong with a scoped `#[allow]` and a comment.
- Don't check in commented-out code. If you want to preserve something, put it in the PR description.

## Commit messages

Write imperative, specific commit messages. "Fix panic in append-entries decoder" is good. "Updates" is not.

## License

By contributing, you agree that your contributions are licensed under the Business Source License 1.1, the same as the rest of the project.
