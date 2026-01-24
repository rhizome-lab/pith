//! HTTP interfaces.
//!
//! Based on WASI HTTP.

use std::collections::HashMap;
use std::future::Future;

/// HTTP errors.
#[derive(Debug)]
pub enum Error {
    InvalidUrl,
    ConnectionFailed,
    Timeout,
    ProtocolError,
    Io(std::io::Error),
    Other(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidUrl => write!(f, "invalid URL"),
            Self::ConnectionFailed => write!(f, "connection failed"),
            Self::Timeout => write!(f, "timeout"),
            Self::ProtocolError => write!(f, "protocol error"),
            Self::Io(e) => write!(f, "I/O error: {}", e),
            Self::Other(s) => write!(f, "{}", s),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
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
