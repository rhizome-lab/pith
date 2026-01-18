//! WASM implementation of pith-http.
//!
//! Uses the Fetch API via `gloo-net`.

use gloo_net::http::RequestBuilder;
use rhizome_pith_http::{Error, HttpClient, Method, Request, Response};
use std::collections::HashMap;

/// HTTP client using the Fetch API.
#[derive(Debug, Default, Clone, Copy)]
pub struct FetchClient;

impl HttpClient for FetchClient {
    async fn send(&self, request: Request) -> Result<Response, Error> {
        use gloo_net::http::Method as GlooMethod;

        let gloo_method = match request.method {
            Method::Get => GlooMethod::GET,
            Method::Head => GlooMethod::HEAD,
            Method::Post => GlooMethod::POST,
            Method::Put => GlooMethod::PUT,
            Method::Delete => GlooMethod::DELETE,
            Method::Patch => GlooMethod::PATCH,
            Method::Options => GlooMethod::OPTIONS,
        };

        let mut builder = RequestBuilder::new(&request.url).method(gloo_method);

        for (key, value) in &request.headers {
            builder = builder.header(key, value);
        }

        let gloo_response = if let Some(body) = request.body {
            builder
                .body(body)
                .map_err(|e| Error::Other(e.to_string()))?
                .send()
                .await
        } else {
            builder.send().await
        }
        .map_err(|e| Error::Other(e.to_string()))?;

        let status = gloo_response.status();

        let headers = HashMap::new();
        // gloo-net doesn't expose headers iterator directly
        // For full header access, we'd need to use web-sys directly

        let body = gloo_response
            .binary()
            .await
            .map_err(|e| Error::Other(e.to_string()))?;

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
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    // Note: These tests require a running server or will fail due to CORS
    // They're included as examples but are skipped in normal test runs

    #[wasm_bindgen_test]
    fn client_can_be_created() {
        let _client = FetchClient;
    }
}
