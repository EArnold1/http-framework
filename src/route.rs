use std::pin::Pin;

use hyper::Method;

use crate::{error::LibError, request::ApiRequest, response::BodyResponse};

pub type HandlerResult = Result<BodyResponse, LibError>;

pub type HandlerFuture = Pin<Box<dyn Future<Output = Result<BodyResponse, LibError>> + Send>>;

/// Using function pointer here
/// This can easily be a `trait object` with Fn, With trait objects the `route_handlers` can be async functions or closures
pub type ApiHandlerFn = fn(ApiRequest) -> HandlerFuture;

pub struct Route {
    pub method: Method,
    pub route: String,
    pub handler: ApiHandlerFn,
}

impl Route {
    /// Create a new route with method, path, and handler.
    pub fn new(method: Method, route: &str, handler: ApiHandlerFn) -> Self {
        Self {
            method,
            route: route.into(),
            handler,
        }
    }
}
