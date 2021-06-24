use futures::StreamExt;
use hyper::{Response, Server};
use routerify::{Router, RouterService};
use routerify_websocket::{upgrade_ws, WebSocket};
use std::{convert::Infallible, net::SocketAddr};
use stream_body::StreamBody;
use tokio;

async fn ws_handler(ws: WebSocket) {
    println!("new websocket connection: {}", ws.remote_addr());

    let (_tx, mut rx) = ws.split();

    while let Some(msg) = rx.next().await {
        let msg = msg.unwrap();

        println!("{:?}", msg.close_reason());
        println!("{}", String::from_utf8(msg.into_bytes()).unwrap());
    }
}

fn router() -> Router<StreamBody, Infallible> {
    Router::builder()
        .any_method("/ws", upgrade_ws(ws_handler))
        // Add options handler.
        .options(
            "/*",
            |_req| async move { Ok(Response::new(StreamBody::from("Options"))) },
        )
        // Add 404 page handler.
        .any(|_req| async move { Ok(Response::new(StreamBody::from("Not Found"))) })
        // Add an error handler.
        .err_handler(|err| async move { Response::new(StreamBody::from(format!("Error: {}", err))) })
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
