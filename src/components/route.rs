use hyper::Method;

use crate::api::ApiHandlerFn;

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
