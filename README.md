[![Github Actions Status](https://github.com/routerify/routerify-websocket/workflows/Test/badge.svg)](https://github.com/routerify/routerify-websocket/actions)
[![crates.io](https://img.shields.io/crates/v/routerify-websocket.svg)](https://crates.io/crates/routerify-websocket)
[![Documentation](https://docs.rs/routerify-websocket/badge.svg)](https://docs.rs/routerify-websocket)
[![MIT](https://img.shields.io/crates/l/routerify-websocket.svg)](./LICENSE)

# routerify-websocket

An websocket extension for Routerify.

[Docs](https://docs.rs/routerify-websocket)

## Install

Add this to your `Cargo.toml` file:

```toml
[dependencies]
routerify = "1.1"
routerify-websocket = "0.1.0"
```

## Example

```rust
use routerify_websocket;

fn main() {
  println!("{}", routerify_websocket::add(2, 3));
}
```

## Contributing

Your PRs and suggestions are always welcome.
