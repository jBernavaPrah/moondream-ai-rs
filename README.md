# Moondream

An async Rust client for the [Moondream API](https://moondream.ai/).

This library wraps the HTTP endpoints exposed by the service, providing a simple interface for
point detection, object detection, captioning and visual question answering.

## Usage

Add the crate to your `Cargo.toml`:

```toml
moondream = "0.1"
```

### Instantiate the client

For a local deployment:

```rust
use moondream::MoonDream;

let md = MoonDream::local("http://localhost:8000");
```

For the hosted service:

```rust
let md = MoonDream::remote("YOUR_TOKEN");
```

### Examples

The `examples` directory contains runnable samples. Execute one with:

```bash
cargo run -p moondream --example points
```

## Features

- `/point` - detect objects and return centre points
- `/detect` - bounding box detection
- `/caption` - generate captions for images
- `/query` - visual question answering

## Testing

```bash
cargo test -p moondream
```