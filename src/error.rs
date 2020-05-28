use derive_more::Display;
use std::fmt::{self, Debug, Display, Formatter};

type BoxError = Box<dyn std::error::Error + Send + Sync>;

/// A set of errors that can occur during handling the websocket connections and in other operations.
#[derive(Display)]
#[display(fmt = "routerify-websocket: {}")]
pub enum WebsocketError {
    /// Websocket upgrade error.
    #[display(fmt = "Websocket upgrade error: {}", _0)]
    Upgrade(BoxError),

    /// Failed to receive a message from the websocket connection.
    #[display(fmt = "Failed to receive a message from the websocket connection: {}", _0)]
    MessageReceive(BoxError),

    /// Failed to check websocket's ready status to send messages.
    #[display(fmt = "Failed to check websocket's ready status to send messages: {}", _0)]
    ReadyStatus(BoxError),

    /// Failed to send a message to the websocket connection.
    #[display(fmt = "Failed to send a message to the websocket connection: {}", _0)]
    MessageSend(BoxError),

    /// Failed to flush messages to the websocket connection.
    #[display(fmt = "Failed to flush messages to the websocket connection: {}", _0)]
    MessageFlush(BoxError),

    /// Failed to decode message data as text.
    #[display(fmt = "Failed to decode message data as text: {}", _0)]
    DecodeText(BoxError),

    /// Failed to decode the message data as `JSON` in [`message.decode_json()`](./struct.Message.html#method.decode_json) method.
    #[cfg(feature = "json")]
    #[display(fmt = "Failed to decode the message data as JSON: {}", _0)]
    DecodeJson(BoxError),

    /// Failed to convert a struct to `JSON` in [`message.json()`](./struct.Message.html#method.json) method.
    #[cfg(feature = "json")]
    #[display(fmt = "Failed to convert a struct to JSON: {}", _0)]
    EncodeJson(BoxError),

    /// Failed to close the websocket connection.
    #[display(fmt = "Failed to close the websocket connection: {}", _0)]
    WebSocketClose(BoxError),

    #[doc(hidden)]
    __Nonexhaustive,
}

impl Debug for WebsocketError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(self, f)
    }
}

impl std::error::Error for WebsocketError {}

impl PartialEq for WebsocketError {
    fn eq(&self, other: &Self) -> bool {
        self.to_string().eq(&other.to_string())
    }
}

impl Eq for WebsocketError {}
