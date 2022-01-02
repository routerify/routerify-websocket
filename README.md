[![Github Actions Status](https://github.com/routerify/routerify-websocket/workflows/Test/badge.svg)](https://github.com/routerify/routerify-websocket/actions)
[![crates.io](https://img.shields.io/crates/v/routerify-websocket.svg)](https://crates.io/crates/routerify-websocket)
[![Documentation](https://docs.rs/routerify-websocket/badge.svg)](https://docs.rs/routerify-websocket)
[![MIT](https://img.shields.io/crates/l/routerify-websocket.svg)](./LICENSE)

# routerify-websocket

The `WebSocket` support for the [Routerify](https://github.com/routerify/routerify) library.

[Docs](https://docs.rs/routerify-websocket)

## Install

Add this to your `Cargo.toml` file:

```toml
[dependencies]
routerify = "3"
routerify-websocket = "3"
```

## Example
 
```rust
// Import `SinkExt` and `StreamExt` to send and read websocket messages.
use futures::{SinkExt, StreamExt};
use hyper::{Body, Response, Server};
use routerify::{Router, RouterService};
// Import websocket types.
use routerify_websocket::{upgrade_ws, Message, WebSocket};
use std::{convert::Infallible, net::SocketAddr};

// A handler for websocket connections.
async fn ws_handler(ws: WebSocket) {
    println!("New websocket connection: {}", ws.remote_addr());

    // The `WebSocket` implements the `Sink` and `Stream` traits
    // to read and write messages.
    let (mut tx, mut rx) = ws.split();

    // Read messages.
    while let Some(msg) = rx.next().await {
        let msg = msg.unwrap();

        // Check message type and take appropriate actions.
        if msg.is_text() {
            println!("{}", msg.into_text().unwrap());
        } else if msg.is_binary() {
            println!("{:?}", msg.into_bytes());
        }

        // Send a text message.
        let send_msg = Message::text("Hello world");
        tx.send(send_msg).await.unwrap();
    }
}

fn router() -> Router<Body, Infallible> {
    // Create a router and specify the path and the handler for new websocket connections.
    Router::builder()
        // It will accept websocket connections at `/ws` path with any method type.
        .any_method("/ws", upgrade_ws(ws_handler))
        // It will accept http connections at `/` path.
        .get("/", |_req| async move {
            Ok(Response::new("I also serve http requests".into()))
        })
        .build()
        .unwrap()
}

#[tokio::main]
async fn main() {
    let router = router();

    // Create a Service from the router above to handle incoming requests.
    let service = RouterService::new(router).unwrap();

    // The address on which the server will be listening.
    let addr = SocketAddr::from(([127, 0, 0, 1], 3001));

    // Create a server by passing the created service to `.serve` method.
    let server = Server::bind(&addr).serve(service);

    println!("App is running on: {}", addr);
    if let Err(err) = server.await {
        eprintln!("Server error: {}", err);
    }
}
```

## Contributing

Your PRs and suggestions are always welcome.
