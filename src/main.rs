use std::collections::HashMap;
use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;

use http_body_util::{BodyExt, Empty, Full, combinators::BoxBody};
use hyper::body::{Body, Bytes, Frame};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Method, StatusCode};
use hyper::{Request as BaseRequest, Response};
use hyper_util::rt::TokioIo;
// use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;

// #[derive(Serialize, Deserialize, Debug)]
// struct Player {
//     name: String,
// }

type Request = BaseRequest<hyper::body::Incoming>;

type HandlerFuture = Pin<
    Box<dyn Future<Output = Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error>> + Send>,
>;

/// Using function pointer here
/// This can easily be a `trait object` with Fn, With trait objects the `route_handlers` can be async functions or closures
type HandlerFn = fn(Request) -> HandlerFuture;

struct Route {
    method: Method,
    route: String,
    handler: HandlerFn,
}

impl Route {
    fn new(method: Method, route: &str, handler: HandlerFn) -> Self {
        Self {
            method,
            route: route.into(),
            handler,
        }
    }
}

struct Service {
    routes: HashMap<(Method, String), HandlerFn>,
}

impl Service {
    fn new() -> Self {
        Self {
            routes: HashMap::new(),
        }
    }

    fn route(&mut self, route: Route) {
        self.routes
            .insert((route.method, route.route), route.handler);
    }

    async fn make_service(
        &self,
        req: Request,
    ) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
        let method = req.method().clone();
        let key = (method, req.uri().path().to_owned());

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

async fn get_req_body(req: Request) -> Result<Option<Bytes>, hyper::Error> {
    let upper = req.body().size_hint().upper().unwrap_or(u64::MAX);
    // max body size 64kb
    if upper > 1024 * 64 {
        return Ok(None);
    }

    let body = req.collect().await?.to_bytes();
    Ok(Some(body))
}

async fn handler(req: Request) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    let mut router = Service::new();
    router.route(Route::new(Method::GET, "/", index_route));
    router.route(Route::new(Method::POST, "/echo", echo));
    router.route(Route::new(Method::POST, "/echo/uppercase", echo_uppercase));
    router.route(Route::new(Method::POST, "/echo/reversed", echo_reversed));
    //
    router.make_service(req).await
}

fn index_route(_: Request) -> HandlerFuture {
    Box::pin(async { Ok(Response::new(full("Try POSTing data to /echo"))) })
}

fn echo(req: Request) -> HandlerFuture {
    let body = req.into_body().boxed();
    Box::pin(async { Ok(Response::new(body)) })
}

fn echo_uppercase(req: Request) -> HandlerFuture {
    // Map this body's frame to a different type
    let frame_stream = req.into_body().map_frame(|frame| {
        let frame = if let Ok(data) = frame.into_data() {
            // Convert every byte in every Data frame to uppercase
            data.iter()
                .map(|byte| byte.to_ascii_uppercase())
                .collect::<Bytes>()
        } else {
            Bytes::new()
        };

        Frame::data(frame)
    });

    Box::pin(async { Ok(Response::new(frame_stream.boxed())) })
}

fn echo_reversed(req: Request) -> HandlerFuture {
    Box::pin(async move {
        match get_req_body(req).await? {
            Some(body) => {
                let reversed_body = body.iter().rev().cloned().collect::<Vec<u8>>();
                Ok(Response::new(full(reversed_body)))
            }
            None => {
                let mut resp = Response::new(full("Body too big"));
                *resp.status_mut() = hyper::StatusCode::PAYLOAD_TOO_LARGE;
                Ok(resp)
            }
        }
    })
}

// We create some utility functions to make Empty and Full bodies
// fit our broadened Response body type.
fn empty() -> BoxBody<Bytes, hyper::Error> {
    Empty::<Bytes>::new()
        .map_err(|never| match never {})
        .boxed()
}
fn full<T: Into<Bytes>>(chunk: T) -> BoxBody<Bytes, hyper::Error> {
    Full::new(chunk.into())
        .map_err(|never| match never {})
        .boxed()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    // We create a TcpListener and bind it to 127.0.0.1:3000
    let listener = TcpListener::bind(addr).await?;

    // We start a loop to continuously accept incoming connections
    loop {
        let (stream, _) = listener.accept().await?;

        // Use an adapter to access something implementing `tokio::io` traits as if they implement
        // `hyper::rt` IO traits.
        let io = TokioIo::new(stream);

        // Spawn a tokio task to serve multiple connections concurrently
        tokio::task::spawn(async move {
            // Finally, we bind the incoming connection to our `hello` service
            if let Err(err) = http1::Builder::new()
                // `service_fn` converts our function in a `Service`
                .serve_connection(io, service_fn(handler))
                .await
            {
                eprintln!("Error serving connection: {:?}", err);
            }
        });
    }
}
