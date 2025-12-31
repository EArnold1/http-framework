use http_body_util::{BodyExt, Empty, Full, Limited, combinators::BoxBody};
use hyper::{
    StatusCode,
    body::Bytes,
    header::{CONTENT_TYPE, HeaderValue},
};
use serde::Serialize;

use crate::{
    api::{Request, Response},
    error::LibError,
};

/// Maximum allowed request body size (64 KB)
const MAX_BODY_SIZE: usize = 1024 * 64;

/// Reads and returns the full request body as bytes, enforcing a maximum size.
pub async fn get_req_body(mut req: Request) -> Result<Bytes, LibError> {
    let body = Limited::new(req.body_mut(), MAX_BODY_SIZE)
        .collect() // Possible because of the `BodyExt` trait
        .await
        .map_err(|_| LibError::RequestBodyTooLarge)?
        .to_bytes();

    Ok(body)
}

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

/// Creates a response body from the given data.
pub fn create_response_body<T>(body: T) -> Response<BoxBody<Bytes, LibError>>
where
    T: http_body_util::BodyExt<Data = Bytes> + Send + Sync + 'static,
    T::Error: Into<LibError>, // Tells the compiler that it can convert `T::Error` to `LibError`
{
    Response::new(body.map_err(Into::into).boxed())
}

// TODO: create a JSON response builder

/// Create a JSON response body
pub fn json_response<T>(body: T) -> Result<Response<BoxBody<Bytes, LibError>>, LibError>
where
    T: Serialize,
{
    let json_string = serde_json::to_string(&body)?;

    Response::builder()
        .status(StatusCode::OK)
        .header(
            CONTENT_TYPE,
            "application/json"
                .parse::<HeaderValue>()
                .expect("Failed to parse content type"),
        )
        .body(full(json_string))
        .map_err(LibError::Http)
}
