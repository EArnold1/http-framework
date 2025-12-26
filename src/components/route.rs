use hyper::Method;

use crate::api::HandlerFn;

pub struct Route {
    pub method: Method,
    pub route: String,
    pub handler: HandlerFn,
}

impl Route {
    pub fn new(method: Method, route: &str, handler: HandlerFn) -> Self {
        Self {
            method,
            route: route.into(),
            handler,
        }
    }
}
