//! Native WebSocket implementation using tungstenite.

use futures_util::{SinkExt, StreamExt};
use rhizome_pith_websocket::{Error, Message, WebSocketClient};
use tokio::net::TcpStream;
use tokio_tungstenite::{
    connect_async, tungstenite::protocol::Message as TungMessage, MaybeTlsStream, WebSocketStream,
};

/// Native WebSocket connection.
pub struct NativeWebSocket {
    inner: WebSocketStream<MaybeTlsStream<TcpStream>>,
}

impl NativeWebSocket {
    /// Connect to a WebSocket server.
    pub async fn connect(url: &str) -> Result<Self, Error> {
        let (ws, _) = connect_async(url)
            .await
            .map_err(|e| Error::ConnectionFailed(e.to_string()))?;
        Ok(NativeWebSocket { inner: ws })
    }
}

impl WebSocketClient for NativeWebSocket {
    async fn send(&mut self, msg: Message) -> Result<(), Error> {
        let tung_msg = match msg {
            Message::Text(s) => TungMessage::Text(s.into()),
            Message::Binary(b) => TungMessage::Binary(b.into()),
            Message::Ping(b) => TungMessage::Ping(b.into()),
            Message::Pong(b) => TungMessage::Pong(b.into()),
            Message::Close => TungMessage::Close(None),
        };
        self.inner.send(tung_msg).await.map_err(|_| Error::SendFailed)
    }

    async fn recv(&mut self) -> Result<Message, Error> {
        match self.inner.next().await {
            Some(Ok(msg)) => {
                let msg = match msg {
                    TungMessage::Text(s) => Message::Text(s.to_string()),
                    TungMessage::Binary(b) => Message::Binary(b.to_vec()),
                    TungMessage::Ping(b) => Message::Ping(b.to_vec()),
                    TungMessage::Pong(b) => Message::Pong(b.to_vec()),
                    TungMessage::Close(_) => Message::Close,
                    TungMessage::Frame(_) => Message::Close,
                };
                Ok(msg)
            }
            Some(Err(e)) => Err(Error::Protocol(e.to_string())),
            None => Err(Error::Closed),
        }
    }

    async fn close(&mut self) -> Result<(), Error> {
        self.inner
            .close(None)
            .await
            .map_err(|e| Error::Protocol(e.to_string()))
    }
}
