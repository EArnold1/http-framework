use std::{collections::HashMap, sync::Arc};

use hyper::{Method, Response, StatusCode};

use crate::{
    api::{HandlerFn, HandlerReturn, Request},
    components::route::Route,
    utils::empty,
};

#[derive(Clone, Default)]
pub struct Router {
    routes: Arc<HashMap<(Method, String), HandlerFn>>,
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
    pub async fn make_service(&self, req: Request) -> HandlerReturn {
        let key = (req.method().clone(), req.uri().path().to_owned());

        match self.routes.get(&key) {
            Some(handler) => (handler)(req).await,
            None => {
                let mut not_found = Response::new(empty());
                *not_found.status_mut() = StatusCode::NOT_FOUND;
                Ok(not_found)
            }
        }
    }
}
