//! HTTP interfaces.
//!
//! Based on WASI HTTP.

use std::collections::HashMap;
use std::future::Future;

/// HTTP errors.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("invalid URL")]
    InvalidUrl,
    #[error("connection failed")]
    ConnectionFailed,
    #[error("timeout")]
    Timeout,
    #[error("protocol error")]
    ProtocolError,
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Other(String),
}

/// HTTP method.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Method {
    Get,
    Head,
    Post,
    Put,
    Delete,
    Patch,
    Options,
}

/// An HTTP request.
#[derive(Debug, Clone)]
pub struct Request {
    pub method: Method,
    pub url: String,
    pub headers: HashMap<String, String>,
    pub body: Option<Vec<u8>>,
}

/// An HTTP response.
#[derive(Debug, Clone)]
pub struct Response {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

/// HTTP client for making outgoing requests.
pub trait HttpClient {
    /// Send an HTTP request.
    fn send(&self, request: Request) -> impl Future<Output = Result<Response, Error>>;
}

/// HTTP handler for incoming requests.
pub trait HttpHandler {
    /// Handle an incoming HTTP request.
    fn handle(&self, request: Request) -> impl Future<Output = Response>;
}
