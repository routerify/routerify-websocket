use crate::{WebSocket, WebSocketConfig};
use futures::future::{ok, Ready};
use headers::{Connection, Header, SecWebsocketAccept, SecWebsocketKey, Upgrade};
use hyper::{
    body::HttpBody,
    header::{self, HeaderValue},
    Request, Response, StatusCode,
};
use routerify::ext::RequestExt;
use std::future::Future;

/// Upgrades the http requests to websocket with the provided [config](./struct.WebSocketConfig.html).
///
/// # Examples
///
/// ```no_run
/// # use hyper::{Body, Response, Server};
/// # use routerify::{Router, RouterService};
/// # // Import websocket types.
/// use routerify_websocket::{upgrade_ws_with_config, WebSocket, WebSocketConfig};
/// # use std::{convert::Infallible, net::SocketAddr};
///
/// # // A handler for websocket connections.
/// async fn ws_handler(ws: WebSocket) {
///     println!("New websocket connection: {}", ws.remote_addr());
///     // Handle websocket connection.
/// }
///
/// fn router() -> Router<Body, Infallible> {
///     // Create a router and specify the path and the handler for new websocket connections.
///     Router::builder()
///         // Upgrade the http requests at `/ws` path to websocket with the following config.
///         .any_method("/ws", upgrade_ws_with_config(ws_handler, WebSocketConfig::default()))
///         .build()
///         .unwrap()
/// }
///
/// # #[tokio::main]
/// # async fn main() {
/// #     let router = router();
/// #
/// #     // Create a Service from the router above to handle incoming requests.
/// #     let service = RouterService::new(router).unwrap();
/// #
/// #     // The address on which the server will be listening.
/// #     let addr = SocketAddr::from(([127, 0, 0, 1], 3001));
/// #
/// #     // Create a server by passing the created service to `.serve` method.
/// #      let server = Server::bind(&addr).serve(service);
/// #
/// #     println!("App is running on: {}", addr);
/// #     if let Err(err) = server.await {
/// #         eprintln!("Server error: {}", err);
/// #     }
/// # }
/// ```
pub fn upgrade_ws_with_config<H, R, B, E>(
    handler: H,
    config: WebSocketConfig,
) -> impl FnMut(Request<hyper::Body>) -> Ready<Result<Response<B>, E>> + Send + Sync + 'static
where
    H: Fn(WebSocket) -> R + Copy + Send + Sync + 'static,
    R: Future<Output = ()> + Send + 'static,
    B: From<&'static str> + HttpBody + Send + 'static,
    E: std::error::Error + Send + 'static,
{
    return move |req: Request<hyper::Body>| {
        let sec_key = extract_upgradable_key(&req);
        let remote_addr = req.remote_addr();

        if sec_key.is_none() {
            return ok(Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body("BAD REQUEST: The request is not websocket".into())
                .unwrap());
        }

        tokio::spawn(async move {
            match req.into_body().on_upgrade().await {
                Ok(upgraded) => {
                    handler(WebSocket::from_raw_socket(upgraded, remote_addr, config).await).await;
                }
                Err(err) => log::error!("{}", crate::WebsocketError::Upgrade(err.into())),
            }
        });

        let resp = Response::builder()
            .status(StatusCode::SWITCHING_PROTOCOLS)
            .header(header::CONNECTION, encode_header(Connection::upgrade()))
            .header(header::UPGRADE, encode_header(Upgrade::websocket()))
            .header(
                header::SEC_WEBSOCKET_ACCEPT,
                encode_header(SecWebsocketAccept::from(sec_key.unwrap())),
            )
            .body("".into())
            .unwrap();

        ok(resp)
    };
}

/// Upgrades the http requests to websocket.
///
/// # Examples
///
/// ```no_run
/// # use hyper::{Body, Response, Server};
/// # use routerify::{Router, RouterService};
/// # // Import websocket types.
/// use routerify_websocket::{upgrade_ws, WebSocket};
/// # use std::{convert::Infallible, net::SocketAddr};
///
/// # // A handler for websocket connections.
/// async fn ws_handler(ws: WebSocket) {
///     println!("New websocket connection: {}", ws.remote_addr());
///     // Handle websocket connection.
/// }
///
/// fn router() -> Router<Body, Infallible> {
///     // Create a router and specify the path and the handler for new websocket connections.
///     Router::builder()
///         // Upgrade the http requests at `/ws` path to websocket.
///         .any_method("/ws", upgrade_ws(ws_handler))
///         .build()
///         .unwrap()
/// }
///
/// # #[tokio::main]
/// # async fn main() {
/// #     let router = router();
/// #
/// #     // Create a Service from the router above to handle incoming requests.
/// #     let service = RouterService::new(router).unwrap();
/// #
/// #     // The address on which the server will be listening.
/// #     let addr = SocketAddr::from(([127, 0, 0, 1], 3001));
/// #
/// #     // Create a server by passing the created service to `.serve` method.
/// #      let server = Server::bind(&addr).serve(service);
/// #
/// #     println!("App is running on: {}", addr);
/// #     if let Err(err) = server.await {
/// #         eprintln!("Server error: {}", err);
/// #     }
/// # }
/// ```
pub fn upgrade_ws<H, R, B, E>(
    handler: H,
) -> impl FnMut(Request<hyper::Body>) -> Ready<Result<Response<B>, E>> + Send + Sync + 'static
where
    H: Fn(WebSocket) -> R + Copy + Send + Sync + 'static,
    R: Future<Output = ()> + Send + 'static,
    B: From<&'static str> + HttpBody + Send + 'static,
    E: std::error::Error + Send + 'static,
{
    return upgrade_ws_with_config(handler, WebSocketConfig::default());
}

fn extract_upgradable_key(req: &Request<hyper::Body>) -> Option<SecWebsocketKey> {
    let hdrs = req.headers();

    hdrs.get(header::CONNECTION)
        .and_then(|val| decode_header::<Connection>(val))
        .and_then(|conn| some(conn.contains("upgrade")))
        .and_then(|_| hdrs.get(header::UPGRADE))
        .and_then(|val| val.to_str().ok())
        .and_then(|val| some(val == "websocket"))
        .and_then(|_| hdrs.get(header::SEC_WEBSOCKET_VERSION))
        .and_then(|val| val.to_str().ok())
        .and_then(|val| some(val == "13"))
        .and_then(|_| hdrs.get(header::SEC_WEBSOCKET_KEY))
        .and_then(|val| decode_header::<SecWebsocketKey>(val))
}

fn decode_header<T: Header>(val: &HeaderValue) -> Option<T> {
    let values = [val];
    let mut iter = (&values).into_iter().copied();
    T::decode(&mut iter).ok()
}

fn encode_header<T: Header>(h: T) -> HeaderValue {
    let mut val = Vec::with_capacity(1);
    h.encode(&mut val);
    val.into_iter().nth(0).unwrap()
}

fn some(cond: bool) -> Option<()> {
    if cond {
        Some(())
    } else {
        None
    }
}
