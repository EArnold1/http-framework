use http_body_util::combinators::BoxBody;
use hyper::body::Bytes;
use hyper::header::{CONTENT_TYPE, HeaderName, HeaderValue};
use hyper::{HeaderMap, Request as BaseRequest, Response as BaseResponse, StatusCode};
use serde::Serialize;
use std::pin::Pin;

use crate::error::LibError;
use crate::utils::full;

pub type ApiResponse<T> = BaseResponse<T>;

pub type ApiRequest = BaseRequest<hyper::body::Incoming>;

type BodyResponse = ApiResponse<BoxBody<Bytes, LibError>>;

pub type HandlerResult = Result<BodyResponse, LibError>;

pub type HandlerFuture = Pin<Box<dyn Future<Output = Result<BodyResponse, LibError>> + Send>>;

/// Using function pointer here
/// This can easily be a `trait object` with Fn, With trait objects the `route_handlers` can be async functions or closures
pub type ApiHandlerFn = fn(ApiRequest) -> HandlerFuture;

/// Builder for creating HTTP responses.
pub struct HttpResponse {}

impl HttpResponse {
    /// Returns a new `HttpResponseBuilder` for constructing a response.
    pub fn builder() -> HttpResponseBuilder {
        HttpResponseBuilder::default()
    }
}

/// Builder for customizing HTTP response status and headers.
pub struct HttpResponseBuilder {
    /// The HTTP status code for the response.
    status: StatusCode,
    /// The headers to include in the response.
    headers: HeaderMap,
}

impl Default for HttpResponseBuilder {
    /// Creates a default `HttpResponseBuilder` with status 200 OK and empty headers.
    fn default() -> Self {
        Self {
            status: StatusCode::OK,
            headers: HeaderMap::new(),
        }
    }
}

impl HttpResponseBuilder {
    /// Sets the HTTP status code for the response.
    ///
    /// # Arguments
    /// * `status` - The desired HTTP status code.
    pub fn status_code(mut self, status: StatusCode) -> Self {
        self.status = status;

        self
    }

    /// Adds a header to the response.
    ///
    /// # Arguments
    /// * `(header_name, header_value)` - The header name and value to add.
    pub fn header(
        mut self,
        (header_name, header_value): (HeaderName, impl Into<HeaderValue>),
    ) -> Self {
        let value = HeaderValue::try_from(header_value).expect("Invalid header value");

        self.headers.insert(header_name, value);

        self
    }

    /// Creates a JSON response body from a serializable object and sets the `CONTENT_TYPE`.
    ///
    /// # Arguments
    /// * `body` - The object to serialize as JSON.
    ///
    /// # Errors
    /// Returns `LibError` if serialization fails.
    pub fn json<T>(mut self, body: &T) -> Result<BodyResponse, LibError>
    where
        T: ?Sized + Serialize,
    {
        let json_string = serde_json::to_string(body)?;

        let mut response = ApiResponse::new(full(json_string));

        // setting the CONTENT_TYPE to avoid wrong content type for JSON if it was previously set
        self.headers
            .insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        *response.status_mut() = self.status;
        *response.headers_mut() = self.headers;

        Ok(response)
    }

    /// Creates a plain text response body.
    ///
    /// # Arguments
    /// * `body` - The body content to include in the response.
    ///
    /// # Errors
    /// Returns `LibError` if an error occurs.
    pub fn body<T>(self, body: T) -> Result<BodyResponse, LibError>
    where
        T: Into<Bytes>,
    {
        let mut response = ApiResponse::new(full(body));

        *response.status_mut() = self.status;
        *response.headers_mut() = self.headers;

        Ok(response)
    }
}
