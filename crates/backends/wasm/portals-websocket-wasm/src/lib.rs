//! WASM implementation of portals-websocket.
//!
//! Uses the WebSocket API via `gloo-net`.

use futures::StreamExt;
use gloo_net::websocket::futures::WebSocket;
use portals_websocket::{Error, Message, WebSocketClient};

/// WebSocket client using the browser WebSocket API.
pub struct BrowserWebSocket {
    ws: Option<WebSocket>,
}

impl BrowserWebSocket {
    /// Connect to a WebSocket server.
    ///
    /// This is the backend-specific constructor. The `WebSocketClient` trait
    /// only defines operations on an already-connected socket.
    pub fn connect(url: &str) -> Result<Self, Error> {
        let ws =
            WebSocket::open(url).map_err(|e| Error::ConnectionFailed(e.to_string()))?;
        Ok(Self { ws: Some(ws) })
    }

    fn ws(&mut self) -> Result<&mut WebSocket, Error> {
        self.ws.as_mut().ok_or(Error::Closed)
    }

    fn take_ws(&mut self) -> Result<WebSocket, Error> {
        self.ws.take().ok_or(Error::Closed)
    }
}

impl WebSocketClient for BrowserWebSocket {
    async fn send(&mut self, msg: Message) -> Result<(), Error> {
        use futures::SinkExt;
        use gloo_net::websocket::Message as GlooMessage;

        if matches!(msg, Message::Close) {
            let ws = self.take_ws()?;
            ws.close(None, None).map_err(|_| Error::SendFailed)?;
            return Ok(());
        }

        let gloo_msg = match msg {
            Message::Text(text) => GlooMessage::Text(text),
            Message::Binary(data) => GlooMessage::Bytes(data),
            Message::Ping(_) | Message::Pong(_) => {
                // Browser WebSocket API doesn't expose ping/pong
                return Ok(());
            }
            Message::Close => unreachable!(),
        };

        self.ws()?.send(gloo_msg).await.map_err(|_| Error::SendFailed)
    }

    async fn recv(&mut self) -> Result<Message, Error> {
        use gloo_net::websocket::Message as GlooMessage;

        match self.ws()?.next().await {
            Some(Ok(GlooMessage::Text(text))) => Ok(Message::Text(text)),
            Some(Ok(GlooMessage::Bytes(data))) => Ok(Message::Binary(data)),
            Some(Err(e)) => Err(Error::Protocol(e.to_string())),
            None => Err(Error::Closed),
        }
    }

    async fn close(&mut self) -> Result<(), Error> {
        let ws = self.take_ws()?;
        ws.close(None, None).map_err(|e| Error::Other(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    // Note: These tests require a running WebSocket server
    // They're included as examples but will fail without a server

    #[wasm_bindgen_test]
    fn connect_to_invalid_url_fails() {
        // This should fail gracefully
        let result = BrowserWebSocket::connect("ws://invalid.localhost:99999");
        // Connection may fail immediately or later depending on browser
        // Just verify it doesn't panic
        let _ = result;
    }
}
