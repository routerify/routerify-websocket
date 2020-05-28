use crate::{CloseCode, Message, WebSocketConfig};
use futures::{ready, FutureExt, Sink, Stream};
use std::borrow::Cow;
use std::fmt;
use std::net::SocketAddr;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio_tungstenite::{
    tungstenite::protocol::{CloseFrame, Role},
    WebSocketStream,
};

/// The WebSocket input-output stream.
///
/// It implements the [`Stream`](https://docs.rs/futures/0.3.5/futures/stream/trait.Stream.html) and [`Sink`](https://docs.rs/futures/0.3.5/futures/sink/trait.Sink.html)
/// traits, so the socket is just a stream of messages coming in and going out.
pub struct WebSocket {
    inner: WebSocketStream<hyper::upgrade::Upgraded>,
    remote_addr: SocketAddr,
}

impl WebSocket {
    pub(crate) async fn from_raw_socket(
        upgraded: hyper::upgrade::Upgraded,
        remote_addr: SocketAddr,
        config: WebSocketConfig,
    ) -> Self {
        WebSocketStream::from_raw_socket(upgraded, Role::Server, Some(config))
            .map(|inner| WebSocket { inner, remote_addr })
            .await
    }

    /// Get the peer's remote address.
    pub fn remote_addr(&self) -> SocketAddr {
        self.remote_addr
    }

    /// Consumes the websocket connection and gracefully closes it.
    pub async fn close(self) -> crate::Result<()> {
        let mut this = self;
        this.inner
            .close(None)
            .await
            .map_err(|err| crate::WebsocketError::WebSocketClose(err.into()))
    }

    /// Consumes the websocket connection and gracefully closes it with a code and reason.
    pub async fn close_with<R: Into<Cow<'static, str>>>(self, code: CloseCode, reason: R) -> crate::Result<()> {
        let mut this = self;
        this.inner
            .close(Some(CloseFrame {
                code,
                reason: reason.into(),
            }))
            .await
            .map_err(|err| crate::WebsocketError::WebSocketClose(err.into()))
    }
}

impl Stream for WebSocket {
    type Item = Result<Message, crate::WebsocketError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        match ready!(Pin::new(&mut self.inner).poll_next(cx)) {
            Some(Ok(item)) => Poll::Ready(Some(Ok(Message { inner: item }))),
            Some(Err(err)) => Poll::Ready(Some(Err(crate::WebsocketError::MessageReceive(err.into())))),
            None => Poll::Ready(None),
        }
    }
}

impl Sink<Message> for WebSocket {
    type Error = crate::WebsocketError;

    fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        match ready!(Pin::new(&mut self.inner).poll_ready(cx)) {
            Ok(()) => Poll::Ready(Ok(())),
            Err(err) => Poll::Ready(Err(crate::WebsocketError::ReadyStatus(err.into()))),
        }
    }

    fn start_send(mut self: Pin<&mut Self>, item: Message) -> Result<(), Self::Error> {
        match Pin::new(&mut self.inner).start_send(item.inner) {
            Ok(()) => Ok(()),
            Err(err) => Err(crate::WebsocketError::MessageSend(err.into())),
        }
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        match ready!(Pin::new(&mut self.inner).poll_flush(cx)) {
            Ok(()) => Poll::Ready(Ok(())),
            Err(err) => Poll::Ready(Err(crate::WebsocketError::MessageFlush(err.into()))),
        }
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        match ready!(Pin::new(&mut self.inner).poll_close(cx)) {
            Ok(()) => Poll::Ready(Ok(())),
            Err(err) => Poll::Ready(Err(crate::WebsocketError::WebSocketClose(err.into()))),
        }
    }
}

impl fmt::Debug for WebSocket {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("WebSocket").finish()
    }
}
