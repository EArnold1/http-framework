use http_body_util::{BodyExt, Empty, Full, combinators::BoxBody};
use hyper::body::{Body, Bytes};

use crate::api::Request;

// TODO: Create lib error and return `big body size` error
pub async fn get_req_body(req: Request) -> Result<Option<Bytes>, hyper::Error> {
    let upper = req.body().size_hint().upper().unwrap_or(u64::MAX);
    // max body size 64kb
    if upper > 1024 * 64 {
        return Ok(None);
    }

    let body = req.collect().await?.to_bytes();
    Ok(Some(body))
}

// From hyper documentation
pub fn empty() -> BoxBody<Bytes, hyper::Error> {
    Empty::<Bytes>::new()
        .map_err(|never| match never {})
        .boxed()
}
pub fn full<T: Into<Bytes>>(chunk: T) -> BoxBody<Bytes, hyper::Error> {
    Full::new(chunk.into())
        .map_err(|never| match never {})
        .boxed()
}
