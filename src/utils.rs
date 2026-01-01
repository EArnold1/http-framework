use http_body_util::{BodyExt, Empty, Full, Limited, combinators::BoxBody};
use hyper::body::Bytes;

use crate::{api::ApiRequest, error::LibError};

/// Maximum allowed request body size (64 KB)
const MAX_BODY_SIZE: usize = 1024 * 64;

/// Reads and returns the full request body as bytes, enforcing a maximum size.
pub async fn get_req_body(mut req: ApiRequest) -> Result<Bytes, LibError> {
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
