use http_body_util::combinators::BoxBody;
use hyper::body::Bytes;
use hyper::{Request as BaseRequest, Response as BaseResponse};
use std::pin::Pin;

pub type Response<T> = BaseResponse<T>;

pub type HyperError = hyper::Error;

pub type Request = BaseRequest<hyper::body::Incoming>;

type PayloadData = Response<BoxBody<Bytes, HyperError>>;

pub type HandlerReturn = Result<PayloadData, HyperError>;

pub type HandlerFuture = Pin<Box<dyn Future<Output = Result<PayloadData, HyperError>> + Send>>;

/// Using function pointer here
/// This can easily be a `trait object` with Fn, With trait objects the `route_handlers` can be async functions or closures
pub type HandlerFn = fn(Request) -> HandlerFuture;
