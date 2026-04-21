# Installation

Add `jotun` to your `Cargo.toml`:

```toml
[dependencies]
jotun = "0.1"
```

Or, for the engine without the runtime:

```toml
[dependencies]
jotun-core = "0.1"
```

## Requirements

- Rust 1.85+ (edition 2024)
- `protoc` installed (used by `prost-build` to compile the wire format)

On Debian/Ubuntu:

```bash
sudo apt install protobuf-compiler
```

On macOS:

```bash
brew install protobuf
```
