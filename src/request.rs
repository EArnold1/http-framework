use http_body_util::{BodyExt, Limited};
use hyper::{
    Request as BaseRequest,
    body::{Bytes, Incoming},
};

use crate::error::LibError;

pub type ApiRequest = BaseRequest<Incoming>;

/// Maximum allowed request body size (64 KB)
const MAX_BODY_SIZE: usize = 1024 * 64;

/// Reads and returns the full request body as bytes, enforcing a maximum size.
pub async fn get_req_body(incoming: &mut ApiRequest) -> Result<Bytes, LibError> {
    let body = Limited::new(incoming.body_mut(), MAX_BODY_SIZE)
        .collect() // Possible because of the `BodyExt` trait
        .await
        .map_err(|_| LibError::RequestBodyTooLarge)?
        .to_bytes();

    Ok(body)
}
