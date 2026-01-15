//! WebSocket interfaces.
//!
//! Based on RFC 6455.

use std::future::Future;

/// WebSocket errors.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("connection failed: {0}")]
    ConnectionFailed(String),
    #[error("send failed")]
    SendFailed,
    #[error("connection closed")]
    Closed,
    #[error("protocol error: {0}")]
    Protocol(String),
}

/// A WebSocket message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Message {
    /// Text message.
    Text(String),
    /// Binary message.
    Binary(Vec<u8>),
    /// Ping message.
    Ping(Vec<u8>),
    /// Pong message.
    Pong(Vec<u8>),
    /// Close message.
    Close,
}

/// A WebSocket client connection.
pub trait WebSocketClient {
    /// Send a message.
    fn send(&mut self, msg: Message) -> impl Future<Output = Result<(), Error>>;

    /// Receive the next message.
    fn recv(&mut self) -> impl Future<Output = Result<Message, Error>>;

    /// Close the connection.
    fn close(&mut self) -> impl Future<Output = Result<(), Error>>;
}

/// A WebSocket client connector.
pub trait WebSocketConnector {
    type Client: WebSocketClient;

    /// Connect to a WebSocket server.
    fn connect(&self, url: &str) -> impl Future<Output = Result<Self::Client, Error>>;
}
