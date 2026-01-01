use crate::error::LibError;
use http_body_util::{BodyExt, Empty, Full, combinators::BoxBody};
use hyper::{
    HeaderMap, Response as BaseResponse, StatusCode,
    body::Bytes,
    header::{CONTENT_TYPE, HeaderName, HeaderValue},
};
use serde::Serialize;

pub type ApiResponse<T> = BaseResponse<T>;

pub type BodyResponse = ApiResponse<BoxBody<Bytes, LibError>>;

/// Returns an empty response body.
pub fn empty() -> BoxBody<Bytes, LibError> {
    Empty::<Bytes>::new()
        .map_err(|never| match never {})
        .boxed()
}

/// A function to create a body with data
pub fn full<T: Into<Bytes>>(chunk: T) -> BoxBody<Bytes, LibError> {
    Full::new(chunk.into())
        .map_err(|never| match never {})
        .boxed()
}

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
