use http_body_util::{BodyExt, Empty, Full, Limited, combinators::BoxBody};
use hyper::body::Bytes;

use crate::{
    api::{Request, Response},
    error::LibError,
};

const MAX_BODY_SIZE: u64 = 1024 * 64; // 64kb

pub async fn get_req_body(mut req: Request) -> Result<Bytes, LibError> {
    let body = Limited::new(req.body_mut(), MAX_BODY_SIZE as usize)
        .collect() // Possible because of the `BodyExt` trait
        .await
        .map_err(|_| LibError::RequestBodyTooLarge)?
        .to_bytes();

    Ok(body)
}

// From hyper documentation
pub fn empty() -> BoxBody<Bytes, LibError> {
    Empty::<Bytes>::new()
        .map_err(|never| match never {})
        .boxed()
}
pub fn full<T: Into<Bytes>>(chunk: T) -> BoxBody<Bytes, LibError> {
    Full::new(chunk.into())
        .map_err(|never| match never {})
        .boxed()
}

pub fn create_response_body<T>(body: T) -> Response<BoxBody<Bytes, LibError>>
where
    T: http_body_util::BodyExt<Data = Bytes> + Send + Sync + 'static,
    T::Error: Into<LibError>, // Tells the compiler that it can convert `T::Error` to `LibError`
{
    Response::new(body.map_err(Into::into).boxed())
}
