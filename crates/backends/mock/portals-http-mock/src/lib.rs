//! Mock implementation of portals-http for testing.
//!
//! Provides a mock HTTP client that returns canned responses and records requests.

use portals_http::{Error, HttpClient, Method, Request, Response};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

/// A mock HTTP client for testing.
///
/// Queues responses to return and records all requests made.
#[derive(Debug, Clone, Default)]
pub struct MockHttpClient {
    inner: Arc<Mutex<MockState>>,
}

#[derive(Debug, Default)]
struct MockState {
    responses: VecDeque<MockResponse>,
    requests: Vec<Request>,
    default_response: Option<Response>,
}

#[derive(Debug)]
enum MockResponse {
    Success(Response),
    Error(ErrorKind),
}

#[derive(Debug, Clone, Copy)]
enum ErrorKind {
    InvalidUrl,
    ConnectionFailed,
    Timeout,
    ProtocolError,
}

impl MockHttpClient {
    /// Create a new mock HTTP client.
    pub fn new() -> Self {
        Self::default()
    }

    /// Queue a response to be returned for the next request.
    pub fn queue_response(&self, response: Response) {
        let mut state = self.inner.lock().unwrap();
        state.responses.push_back(MockResponse::Success(response));
    }

    /// Queue an error to be returned for the next request.
    pub fn queue_error(&self, error: &str) {
        let kind = match error {
            "invalid_url" => ErrorKind::InvalidUrl,
            "connection_failed" => ErrorKind::ConnectionFailed,
            "timeout" => ErrorKind::Timeout,
            "protocol_error" => ErrorKind::ProtocolError,
            _ => ErrorKind::ConnectionFailed,
        };
        let mut state = self.inner.lock().unwrap();
        state.responses.push_back(MockResponse::Error(kind));
    }

    /// Set a default response to return when the queue is empty.
    pub fn set_default_response(&self, response: Response) {
        let mut state = self.inner.lock().unwrap();
        state.default_response = Some(response);
    }

    /// Get all requests that have been made.
    pub fn requests(&self) -> Vec<Request> {
        let state = self.inner.lock().unwrap();
        state.requests.clone()
    }

    /// Get the number of requests made.
    pub fn request_count(&self) -> usize {
        let state = self.inner.lock().unwrap();
        state.requests.len()
    }

    /// Clear all recorded requests.
    pub fn clear_requests(&self) {
        let mut state = self.inner.lock().unwrap();
        state.requests.clear();
    }

    /// Clear all queued responses.
    pub fn clear_responses(&self) {
        let mut state = self.inner.lock().unwrap();
        state.responses.clear();
    }

    /// Assert that a request was made to the given URL.
    pub fn assert_requested(&self, url: &str) {
        let state = self.inner.lock().unwrap();
        assert!(
            state.requests.iter().any(|r| r.url == url),
            "expected request to {} but none was made",
            url
        );
    }

    /// Assert that a request was made with the given method and URL.
    pub fn assert_requested_with(&self, method: Method, url: &str) {
        let state = self.inner.lock().unwrap();
        assert!(
            state
                .requests
                .iter()
                .any(|r| r.method == method && r.url == url),
            "expected {:?} request to {} but none was made",
            method,
            url
        );
    }
}

impl HttpClient for MockHttpClient {
    async fn send(&self, request: Request) -> Result<Response, Error> {
        let mut state = self.inner.lock().unwrap();
        state.requests.push(request);

        match state.responses.pop_front() {
            Some(MockResponse::Success(response)) => Ok(response),
            Some(MockResponse::Error(kind)) => Err(match kind {
                ErrorKind::InvalidUrl => Error::InvalidUrl,
                ErrorKind::ConnectionFailed => Error::ConnectionFailed,
                ErrorKind::Timeout => Error::Timeout,
                ErrorKind::ProtocolError => Error::ProtocolError,
            }),
            None => {
                if let Some(ref default) = state.default_response {
                    Ok(default.clone())
                } else {
                    // Return a 200 OK with empty body as fallback
                    Ok(Response {
                        status: 200,
                        headers: Default::default(),
                        body: Vec::new(),
                    })
                }
            }
        }
    }
}

/// Builder for creating Response objects easily.
pub struct ResponseBuilder {
    status: u16,
    headers: std::collections::HashMap<String, String>,
    body: Vec<u8>,
}

impl ResponseBuilder {
    /// Create a new response builder with the given status code.
    pub fn new(status: u16) -> Self {
        Self {
            status,
            headers: Default::default(),
            body: Vec::new(),
        }
    }

    /// Create a 200 OK response builder.
    pub fn ok() -> Self {
        Self::new(200)
    }

    /// Create a 404 Not Found response builder.
    pub fn not_found() -> Self {
        Self::new(404)
    }

    /// Create a 500 Internal Server Error response builder.
    pub fn server_error() -> Self {
        Self::new(500)
    }

    /// Set a header.
    pub fn header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(name.into(), value.into());
        self
    }

    /// Set the body from bytes.
    pub fn body(mut self, body: impl Into<Vec<u8>>) -> Self {
        self.body = body.into();
        self
    }

    /// Set the body from a string.
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.body = text.into().into_bytes();
        self.headers
            .insert("content-type".to_string(), "text/plain".to_string());
        self
    }

    /// Set the body from JSON (as a string).
    pub fn json(mut self, json: impl Into<String>) -> Self {
        self.body = json.into().into_bytes();
        self.headers
            .insert("content-type".to_string(), "application/json".to_string());
        self
    }

    /// Build the response.
    pub fn build(self) -> Response {
        Response {
            status: self.status,
            headers: self.headers,
            body: self.body,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn make_request(method: Method, url: &str) -> Request {
        Request {
            method,
            url: url.to_string(),
            headers: HashMap::new(),
            body: None,
        }
    }

    #[tokio::test]
    async fn returns_queued_response() {
        let client = MockHttpClient::new();
        client.queue_response(ResponseBuilder::ok().json(r#"{"ok":true}"#).build());

        let response = client
            .send(make_request(Method::Get, "https://example.com"))
            .await
            .unwrap();

        assert_eq!(response.status, 200);
        assert_eq!(response.body, br#"{"ok":true}"#);
    }

    #[tokio::test]
    async fn returns_queued_error() {
        let client = MockHttpClient::new();
        client.queue_error("timeout");

        let result = client
            .send(make_request(Method::Get, "https://example.com"))
            .await;

        assert!(matches!(result, Err(Error::Timeout)));
    }

    #[tokio::test]
    async fn records_requests() {
        let client = MockHttpClient::new();

        client
            .send(make_request(Method::Get, "https://example.com/a"))
            .await
            .unwrap();
        client
            .send(make_request(Method::Post, "https://example.com/b"))
            .await
            .unwrap();

        assert_eq!(client.request_count(), 2);
        client.assert_requested("https://example.com/a");
        client.assert_requested_with(Method::Post, "https://example.com/b");
    }

    #[tokio::test]
    async fn uses_default_when_queue_empty() {
        let client = MockHttpClient::new();
        client.set_default_response(ResponseBuilder::not_found().build());

        let response = client
            .send(make_request(Method::Get, "https://example.com"))
            .await
            .unwrap();

        assert_eq!(response.status, 404);
    }

    #[tokio::test]
    async fn response_builder_works() {
        let response = ResponseBuilder::ok()
            .header("x-custom", "value")
            .json(r#"{"status":"ok"}"#)
            .build();

        assert_eq!(response.status, 200);
        assert_eq!(response.headers.get("x-custom"), Some(&"value".to_string()));
        assert_eq!(
            response.headers.get("content-type"),
            Some(&"application/json".to_string())
        );
    }
}
