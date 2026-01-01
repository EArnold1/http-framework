use std::{collections::HashMap, sync::Arc};

use http_body_util::BodyExt;
use hyper::{Method, StatusCode};

use crate::{
    request::ApiRequest,
    response::{HttpResponse, empty},
    route::{ApiHandlerFn, HandlerResult, Route},
};

#[derive(Clone, Default)]
pub struct Router {
    routes: Arc<HashMap<(Method, String), ApiHandlerFn>>,
}

impl Router {
    /// Add a route to the router.
    pub fn route(&mut self, route: Route) {
        // We need to get mutable access to the HashMap inside Arc.
        // NOTE: since this is single-threaded we can do this
        Arc::get_mut(&mut self.routes)
            .expect("Cannot add routes after router is shared")
            .insert((route.method, route.route), route.handler);
    }

    /// Handle a request by matching it to a route handler.
    pub async fn make_service(&self, req: ApiRequest) -> HandlerResult {
        let key = (req.method().clone(), req.uri().path().to_owned());

        match self.routes.get(&key) {
            Some(handler) => (handler)(req).await,
            None => {
                let body = empty().collect().await?.to_bytes();
                HttpResponse::builder()
                    .status_code(StatusCode::NOT_FOUND)
                    .body(body)
            }
        }
    }
}
