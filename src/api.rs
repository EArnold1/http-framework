use http_body_util::combinators::BoxBody;
use hyper::body::Bytes;
use hyper::{Request as BaseRequest, Response as BaseResponse};
use std::pin::Pin;

use crate::error::LibError;

pub type Response<T> = BaseResponse<T>;

pub type Request = BaseRequest<hyper::body::Incoming>;

type PayloadData = Response<BoxBody<Bytes, LibError>>;

pub type HandlerReturn = Result<PayloadData, LibError>;

pub type HandlerFuture = Pin<Box<dyn Future<Output = Result<PayloadData, LibError>> + Send>>;

/// Using function pointer here
/// This can easily be a `trait object` with Fn, With trait objects the `route_handlers` can be async functions or closures
pub type HandlerFn = fn(Request) -> HandlerFuture;
