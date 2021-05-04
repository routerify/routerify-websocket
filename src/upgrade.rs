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
            match hyper::upgrade::on(req).await {
                Ok(upgraded) => {
                    handler(WebSocket::from_raw_socket(upgraded, remote_addr, config).await).await;
                }
                Err(err) => log::error!("{:?}", crate::WebsocketError::Upgrade(err.into())),
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
