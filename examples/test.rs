use futures::{SinkExt, StreamExt};
use hyper::{Body, Request, Response, Server};
use routerify::prelude::*;
use routerify::{Middleware, Router, RouterService};
use routerify_websocket::{upgrade_ws, WebSocket};
use serde::{Deserialize, Serialize};
use std::{convert::Infallible, net::SocketAddr};
use tokio_tungstenite::tungstenite::protocol::Message as ClientMessage;

#[derive(Serialize, Deserialize, Debug)]
struct User {
    name: String,
    roll: u64,
}

async fn ws_handler(ws: WebSocket) {
    println!("new websocket connection: {}", ws.remote_addr());

    let (_tx, mut rx) = ws.split();

    while let Some(msg) = rx.next().await {
        let msg = msg.unwrap();

        println!("{:?}", msg.close_reason());
        println!("{}", String::from_utf8(msg.into_bytes()).unwrap());
    }
}

async fn logger(req: Request<Body>) -> Result<Request<Body>, Infallible> {
    println!("{} {} {}", req.remote_addr(), req.method(), req.uri().path());
    Ok(req)
}

// A handler for "/about" page.
async fn about_handler(_: Request<Body>) -> Result<Response<Body>, Infallible> {
    println!("{:?}", std::thread::current().id());
    Ok(Response::new(Body::from("About page")))
}

fn router() -> Router<Body, Infallible> {
    Router::builder()
        .middleware(Middleware::pre(logger))
        .get("/about", about_handler)
        .any_method("/ws", upgrade_ws(ws_handler))
        .build()
        .unwrap()
}

#[tokio::main]
async fn main() {
    let router = router();

    let service = RouterService::new(router).unwrap();

    let addr = SocketAddr::from(([127, 0, 0, 1], 3001));

    let server = Server::bind(&addr).serve(service);

    tokio::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

        let (mut ws, resp) = tokio_tungstenite::connect_async("ws://127.0.0.1:3001/ws")
            .await
            .unwrap();

        println!("{:?}", resp.headers());

        let msg = ClientMessage::text("hey");
        ws.send(msg).await.unwrap();

        ws.close(None).await.unwrap();

        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
    });

    println!("App is running on: {}", addr);
    if let Err(err) = server.await {
        eprintln!("Server error: {}", err);
    }
}
