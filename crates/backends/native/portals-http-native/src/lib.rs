//! Native implementation of portals-http using reqwest.

use portals_http::{Error, HttpClient, Method, Request, Response};

/// HTTP client using reqwest.
#[derive(Debug, Clone)]
pub struct ReqwestClient {
    inner: reqwest::Client,
}

impl Default for ReqwestClient {
    fn default() -> Self {
        Self::new()
    }
}

impl ReqwestClient {
    pub fn new() -> Self {
        Self {
            inner: reqwest::Client::new(),
        }
    }

    pub fn with_client(client: reqwest::Client) -> Self {
        Self { inner: client }
    }
}

impl HttpClient for ReqwestClient {
    async fn send(&self, request: Request) -> Result<Response, Error> {
        let method = match request.method {
            Method::Get => reqwest::Method::GET,
            Method::Head => reqwest::Method::HEAD,
            Method::Post => reqwest::Method::POST,
            Method::Put => reqwest::Method::PUT,
            Method::Delete => reqwest::Method::DELETE,
            Method::Patch => reqwest::Method::PATCH,
            Method::Options => reqwest::Method::OPTIONS,
        };

        let mut req = self.inner.request(method, &request.url);

        for (key, value) in &request.headers {
            req = req.header(key, value);
        }

        if let Some(body) = request.body {
            req = req.body(body);
        }

        let resp = req.send().await.map_err(|e| {
            if e.is_connect() {
                Error::ConnectionFailed
            } else if e.is_timeout() {
                Error::Timeout
            } else {
                Error::ProtocolError
            }
        })?;

        let status = resp.status().as_u16();
        let headers = resp
            .headers()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
            .collect();
        let body = resp.bytes().await.map_err(|_| Error::ProtocolError)?.to_vec();

        Ok(Response {
            status,
            headers,
            body,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests require network access
    // In a real test suite, you'd use a mock server

    #[tokio::test]
    #[ignore] // Requires network
    async fn get_request_works() {
        let client = ReqwestClient::new();
        let request = Request {
            method: Method::Get,
            url: "https://httpbin.org/get".to_string(),
            headers: Default::default(),
            body: None,
        };

        let response = client.send(request).await.unwrap();
        assert_eq!(response.status, 200);
    }
}
