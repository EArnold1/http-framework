use http_body_util::combinators::BoxBody;
use hyper::body::Bytes;
use hyper::header::{CONTENT_TYPE, HeaderName, HeaderValue};
use hyper::{HeaderMap, Request as BaseRequest, Response as BaseResponse, StatusCode};
use serde::Serialize;
use std::pin::Pin;

use crate::error::LibError;
use crate::utils::full;

pub type Response<T> = BaseResponse<T>;

pub type Request = BaseRequest<hyper::body::Incoming>;

type PayloadData = Response<BoxBody<Bytes, LibError>>;

pub type HandlerReturn = Result<PayloadData, LibError>;

pub type HandlerFuture = Pin<Box<dyn Future<Output = Result<PayloadData, LibError>> + Send>>;

/// Using function pointer here
/// This can easily be a `trait object` with Fn, With trait objects the `route_handlers` can be async functions or closures
pub type HandlerFn = fn(Request) -> HandlerFuture;

/// Creates a http response with a builder
pub struct HttpResponse {}

impl HttpResponse {
    pub fn builder() -> HttpResponseBuilder {
        HttpResponseBuilder::default()
    }
}

pub struct HttpResponseBuilder {
    status: StatusCode,
    headers: HeaderMap,
}

impl Default for HttpResponseBuilder {
    fn default() -> Self {
        Self {
            status: StatusCode::OK,
            headers: HeaderMap::new(),
        }
    }
}

impl HttpResponseBuilder {
    /// Set status code
    pub fn status_code(mut self, status: StatusCode) -> Self {
        self.status = status;

        self
    }

    /// Add header
    pub fn header(mut self, (header_name, header_value): (HeaderName, &'static str)) -> Self {
        self.headers
            .insert(header_name, HeaderValue::from_static(header_value));

        self
    }

    /// Create a JSON response body
    pub fn json<T>(&mut self, body: &T) -> Result<PayloadData, LibError>
    where
        T: ?Sized + Serialize,
    {
        let json_string = serde_json::to_string(body)?;

        let mut response = Response::new(full(json_string));

        self.headers
            .insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        let headers = self.headers.clone();

        *response.status_mut() = self.status;

        *response.headers_mut() = headers;

        Ok(response)
    }

    /// Create a response body
    pub fn body<T>(&mut self, body: T) -> Result<PayloadData, LibError>
    where
        T: Into<Bytes>,
    {
        let mut response = Response::new(full(body));

        self.headers
            .insert(CONTENT_TYPE, HeaderValue::from_static("text/plain"));

        let headers = self.headers.clone();

        *response.status_mut() = self.status;

        *response.headers_mut() = headers;

        Ok(response)
    }
}
