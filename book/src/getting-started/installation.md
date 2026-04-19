# Installation

Add `jotun` to your `Cargo.toml`:

```toml
[dependencies]
jotun = "0.1"
```

Or, if you want the pure engine with no runtime assumptions:

```toml
[dependencies]
jotun-core = "0.1"
```

## Requirements

- Rust 1.85+ (edition 2024)
- `protoc` installed on your build machine (used by `prost-build` to compile the wire format)

On Ubuntu/Debian:

```bash
sudo apt install protobuf-compiler
```

On macOS:

```bash
brew install protobuf
```
