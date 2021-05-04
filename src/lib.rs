

pub use self::error::WebsocketError;
pub use message::Message;
pub use tokio_tungstenite::tungstenite::protocol::{frame::coding::CloseCode, WebSocketConfig};
pub use upgrade::{upgrade_ws, upgrade_ws_with_config};
pub use websocket::WebSocket;

mod error;
mod message;
mod upgrade;
mod websocket;

/// A Result type often returned while handing websocket connection.
pub type Result<T> = std::result::Result<T, WebsocketError>;
