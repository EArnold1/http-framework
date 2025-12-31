use hyper::Method;

use crate::api::HandlerFn;

pub struct Route {
    pub method: Method,
    pub route: String,
    pub handler: HandlerFn,
}

impl Route {
    /// Create a new route with method, path, and handler.
    pub fn new(method: Method, route: &str, handler: HandlerFn) -> Self {
        Self {
            method,
            route: route.into(),
            handler,
        }
    }
}
