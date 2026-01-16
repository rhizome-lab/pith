//! HTTP/1.1 protocol implementation.
//!
//! Provides parsing and serialization of HTTP/1.1 requests and responses.

use std::collections::HashMap;
use std::io::{BufRead, Write};

/// HTTP/1.1 errors.
#[derive(Debug)]
pub enum Error {
    InvalidRequestLine,
    InvalidStatusLine,
    InvalidHeader,
    InvalidMethod,
    InvalidContentLength,
    Io(std::io::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidRequestLine => write!(f, "invalid request line"),
            Self::InvalidStatusLine => write!(f, "invalid status line"),
            Self::InvalidHeader => write!(f, "invalid header"),
            Self::InvalidMethod => write!(f, "invalid method"),
            Self::InvalidContentLength => write!(f, "invalid content length"),
            Self::Io(e) => write!(f, "I/O error: {}", e),
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
    Connect,
    Trace,
}

impl Method {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Get => "GET",
            Self::Head => "HEAD",
            Self::Post => "POST",
            Self::Put => "PUT",
            Self::Delete => "DELETE",
            Self::Patch => "PATCH",
            Self::Options => "OPTIONS",
            Self::Connect => "CONNECT",
            Self::Trace => "TRACE",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, Error> {
        match s {
            "GET" => Ok(Self::Get),
            "HEAD" => Ok(Self::Head),
            "POST" => Ok(Self::Post),
            "PUT" => Ok(Self::Put),
            "DELETE" => Ok(Self::Delete),
            "PATCH" => Ok(Self::Patch),
            "OPTIONS" => Ok(Self::Options),
            "CONNECT" => Ok(Self::Connect),
            "TRACE" => Ok(Self::Trace),
            _ => Err(Error::InvalidMethod),
        }
    }
}

/// HTTP request.
#[derive(Debug, Clone)]
pub struct Request {
    pub method: Method,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

/// HTTP response.
#[derive(Debug, Clone)]
pub struct Response {
    pub status: u16,
    pub reason: String,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

impl Response {
    /// Create a new response with a status code.
    pub fn new(status: u16) -> Self {
        Self {
            status,
            reason: reason_phrase(status).to_string(),
            headers: HashMap::new(),
            body: Vec::new(),
        }
    }

    /// Set a header.
    pub fn header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(name.into(), value.into());
        self
    }

    /// Set the body.
    pub fn body(mut self, body: impl Into<Vec<u8>>) -> Self {
        self.body = body.into();
        self
    }
}

/// Parse an HTTP request from a buffered reader.
pub fn parse_request<R: BufRead>(reader: &mut R) -> Result<Request, Error> {
    let mut line = String::new();

    // Request line
    reader.read_line(&mut line)?;
    let parts: Vec<&str> = line.trim_end().split(' ').collect();
    if parts.len() < 2 {
        return Err(Error::InvalidRequestLine);
    }

    let method = Method::from_str(parts[0])?;
    let path = parts[1].to_string();

    // Headers
    let mut headers = HashMap::new();
    loop {
        line.clear();
        reader.read_line(&mut line)?;
        let trimmed = line.trim_end();
        if trimmed.is_empty() {
            break;
        }
        if let Some((name, value)) = trimmed.split_once(':') {
            headers.insert(name.trim().to_lowercase(), value.trim().to_string());
        }
    }

    // Body
    let body = if let Some(len) = headers.get("content-length") {
        let len: usize = len.parse().map_err(|_| Error::InvalidContentLength)?;
        let mut body = vec![0u8; len];
        reader.read_exact(&mut body)?;
        body
    } else {
        Vec::new()
    };

    Ok(Request {
        method,
        path,
        headers,
        body,
    })
}

/// Parse an HTTP response from a buffered reader.
pub fn parse_response<R: BufRead>(reader: &mut R) -> Result<Response, Error> {
    let mut line = String::new();

    // Status line
    reader.read_line(&mut line)?;
    let parts: Vec<&str> = line.trim_end().splitn(3, ' ').collect();
    if parts.len() < 2 {
        return Err(Error::InvalidStatusLine);
    }

    let status: u16 = parts[1].parse().map_err(|_| Error::InvalidStatusLine)?;
    let reason = parts.get(2).unwrap_or(&"").to_string();

    // Headers
    let mut headers = HashMap::new();
    loop {
        line.clear();
        reader.read_line(&mut line)?;
        let trimmed = line.trim_end();
        if trimmed.is_empty() {
            break;
        }
        if let Some((name, value)) = trimmed.split_once(':') {
            headers.insert(name.trim().to_lowercase(), value.trim().to_string());
        }
    }

    // Body
    let body = if let Some(len) = headers.get("content-length") {
        let len: usize = len.parse().map_err(|_| Error::InvalidContentLength)?;
        let mut body = vec![0u8; len];
        reader.read_exact(&mut body)?;
        body
    } else {
        Vec::new()
    };

    Ok(Response {
        status,
        reason,
        headers,
        body,
    })
}

/// Write an HTTP request to a writer.
pub fn write_request<W: Write>(writer: &mut W, request: &Request) -> Result<(), Error> {
    write!(writer, "{} {} HTTP/1.1\r\n", request.method.as_str(), request.path)?;

    for (name, value) in &request.headers {
        write!(writer, "{}: {}\r\n", name, value)?;
    }

    if !request.body.is_empty() && !request.headers.contains_key("content-length") {
        write!(writer, "content-length: {}\r\n", request.body.len())?;
    }

    write!(writer, "\r\n")?;
    writer.write_all(&request.body)?;
    writer.flush()?;

    Ok(())
}

/// Write an HTTP response to a writer.
pub fn write_response<W: Write>(writer: &mut W, response: &Response) -> Result<(), Error> {
    write!(writer, "HTTP/1.1 {} {}\r\n", response.status, response.reason)?;

    for (name, value) in &response.headers {
        write!(writer, "{}: {}\r\n", name, value)?;
    }

    if !response.body.is_empty() && !response.headers.contains_key("content-length") {
        write!(writer, "content-length: {}\r\n", response.body.len())?;
    }

    write!(writer, "\r\n")?;
    writer.write_all(&response.body)?;
    writer.flush()?;

    Ok(())
}

/// Get the standard reason phrase for a status code.
pub fn reason_phrase(status: u16) -> &'static str {
    match status {
        100 => "Continue",
        101 => "Switching Protocols",
        200 => "OK",
        201 => "Created",
        204 => "No Content",
        301 => "Moved Permanently",
        302 => "Found",
        304 => "Not Modified",
        400 => "Bad Request",
        401 => "Unauthorized",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        500 => "Internal Server Error",
        501 => "Not Implemented",
        502 => "Bad Gateway",
        503 => "Service Unavailable",
        _ => "Unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn parse_simple_request() {
        let data = b"GET /path HTTP/1.1\r\nHost: example.com\r\n\r\n";
        let mut cursor = Cursor::new(data.as_slice());
        let req = parse_request(&mut cursor).unwrap();

        assert_eq!(req.method, Method::Get);
        assert_eq!(req.path, "/path");
        assert_eq!(req.headers.get("host"), Some(&"example.com".to_string()));
    }

    #[test]
    fn parse_request_with_body() {
        let data = b"POST /submit HTTP/1.1\r\nContent-Length: 5\r\n\r\nhello";
        let mut cursor = Cursor::new(data.as_slice());
        let req = parse_request(&mut cursor).unwrap();

        assert_eq!(req.method, Method::Post);
        assert_eq!(req.body, b"hello");
    }

    #[test]
    fn parse_simple_response() {
        let data = b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nhi";
        let mut cursor = Cursor::new(data.as_slice());
        let res = parse_response(&mut cursor).unwrap();

        assert_eq!(res.status, 200);
        assert_eq!(res.body, b"hi");
    }

    #[test]
    fn roundtrip_request() {
        let req = Request {
            method: Method::Post,
            path: "/api".to_string(),
            headers: HashMap::from([("host".to_string(), "localhost".to_string())]),
            body: b"data".to_vec(),
        };

        let mut buf = Vec::new();
        write_request(&mut buf, &req).unwrap();

        let mut cursor = Cursor::new(buf.as_slice());
        let parsed = parse_request(&mut cursor).unwrap();

        assert_eq!(parsed.method, req.method);
        assert_eq!(parsed.path, req.path);
        assert_eq!(parsed.body, req.body);
    }

    #[test]
    fn roundtrip_response() {
        let res = Response::new(201)
            .header("x-custom", "value")
            .body(b"created".to_vec());

        let mut buf = Vec::new();
        write_response(&mut buf, &res).unwrap();

        let mut cursor = Cursor::new(buf.as_slice());
        let parsed = parse_response(&mut cursor).unwrap();

        assert_eq!(parsed.status, 201);
        assert_eq!(parsed.body, b"created");
    }
}
