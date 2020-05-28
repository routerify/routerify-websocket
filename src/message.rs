use crate::CloseCode;
#[cfg(feature = "json")]
use serde::{de::DeserializeOwned, ser::Serialize};
use std::borrow::Cow;
use std::fmt;
use tokio_tungstenite::tungstenite::protocol::{self, CloseFrame};

/// A WebSocket message.
#[derive(Eq, PartialEq, Clone)]
pub struct Message {
    pub(crate) inner: protocol::Message,
}

impl Message {
    /// Create a new `Text` WebSocket message from a stringable.
    pub fn text<S: Into<String>>(s: S) -> Message {
        Message {
            inner: protocol::Message::text(s),
        }
    }

    /// Constructs a `Text` WebSocket message with the json value.
    ///
    /// # Optional
    ///
    /// This requires the optional `json` feature to be enabled.
    #[cfg(feature = "json")]
    pub fn json<T: Serialize>(value: &T) -> crate::Result<Message> {
        Ok(Message::text(
            serde_json::to_string(&value).map_err(|err| crate::WebsocketError::EncodeJson(err.into()))?,
        ))
    }

    /// Create a new `Binary` WebSocket message.
    pub fn binary<V: Into<Vec<u8>>>(v: V) -> Message {
        Message {
            inner: protocol::Message::binary(v),
        }
    }

    /// Construct a new `Ping` WebSocket message.
    ///
    /// The payload here must have a length less than 125 bytes.
    pub fn ping<V: Into<Vec<u8>>>(v: V) -> Message {
        Message {
            inner: protocol::Message::Ping(v.into()),
        }
    }

    /// Construct a new `Pong` WebSocket message.
    ///
    /// The payload here must have a length less than 125 bytes.
    pub fn pong<V: Into<Vec<u8>>>(v: V) -> Message {
        Message {
            inner: protocol::Message::Pong(v.into()),
        }
    }

    /// Construct the default `Close` WebSocket message.
    pub fn close() -> Message {
        Message {
            inner: protocol::Message::Close(None),
        }
    }

    /// Construct a `Close` WebSocket message with a code and reason.
    pub fn close_with<R: Into<Cow<'static, str>>>(code: CloseCode, reason: R) -> Message {
        Message {
            inner: protocol::Message::Close(Some(CloseFrame {
                code,
                reason: reason.into(),
            })),
        }
    }

    /// Returns true if this message is a `Text` message.
    pub fn is_text(&self) -> bool {
        self.inner.is_text()
    }

    /// Returns true if this message is a `Binary` message.
    pub fn is_binary(&self) -> bool {
        self.inner.is_binary()
    }

    /// Returns true if this message a is a `Close` message.
    pub fn is_close(&self) -> bool {
        self.inner.is_close()
    }

    /// Returns true if this message is a `Ping` message.
    pub fn is_ping(&self) -> bool {
        self.inner.is_ping()
    }

    /// Returns true if this message is a `Pong` message.
    pub fn is_pong(&self) -> bool {
        self.inner.is_pong()
    }

    /// Get the length of the WebSocket message.
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Returns true if the WebSocket message has no content.
    /// For example, if the other side of the connection sent an empty string.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// The `Close` code if available.
    pub fn close_code(&self) -> Option<CloseCode> {
        match self.inner {
            protocol::Message::Close(Some(ref data)) => Some(data.code),
            _ => None,
        }
    }

    /// The `Close` reason if available.
    pub fn close_reason(&self) -> Option<&str> {
        match self.inner {
            protocol::Message::Close(Some(ref data)) => Some(&data.reason),
            _ => None,
        }
    }

    /// Attempts to convert the message data as text in `UTF8` format.
    pub fn as_text(&self) -> crate::Result<&str> {
        self.inner
            .to_text()
            .map_err(|err| crate::WebsocketError::DecodeText(err.into()))
    }

    /// Return the bytes of this message.
    pub fn as_bytes(&self) -> &[u8] {
        match self.inner {
            protocol::Message::Text(ref s) => s.as_bytes(),
            protocol::Message::Binary(ref v) => v,
            protocol::Message::Ping(ref v) => v,
            protocol::Message::Pong(ref v) => v,
            protocol::Message::Close(_) => &[],
        }
    }

    /// Consumes the message and returns its data as bytes.
    pub fn into_bytes(self) -> Vec<u8> {
        self.inner.into_data()
    }

    /// Consumes the WebSocket message and attempts to converts it to a `String`.
    pub fn into_text(self) -> crate::Result<String> {
        self.inner
            .into_text()
            .map_err(|err| crate::WebsocketError::DecodeText(err.into()))
    }

    /// Try to deserialize the message data as `JSON`.
    ///
    /// # Optional
    ///
    /// This requires the optional `json` feature to be enabled.
    #[cfg(feature = "json")]
    pub fn decode_json<T: DeserializeOwned>(self) -> crate::Result<T> {
        serde_json::from_slice(&self.into_bytes()).map_err(|err| crate::WebsocketError::DecodeJson(err.into()))
    }
}

impl fmt::Debug for Message {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&self.inner, f)
    }
}

impl Into<Vec<u8>> for Message {
    fn into(self) -> Vec<u8> {
        self.into_bytes()
    }
}
