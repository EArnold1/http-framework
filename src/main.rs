use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;

use http_body_util::{BodyExt, Empty, Full, combinators::BoxBody};
use hyper::body::{Body, Bytes, Frame};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Method, StatusCode};
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;

#[derive(Serialize, Deserialize, Debug)]
struct Player {
    name: String,
}

type BoxFutureResponse = Pin<
    Box<dyn Future<Output = Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error>> + Send>,
>;

type HandlerFn = Box<dyn Fn(Vec<u8>) -> BoxFutureResponse + Send + Sync>;

struct Route {
    method: Method,
    route: String,
    handler: HandlerFn,
}

impl Route {
    fn new<H, Fut>(method: Method, route: &str, handler: H) -> Self
    where
        H: Fn(Vec<u8>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error>>
            + Send
            + 'static,
    {
        Self {
            method,
            route: route.into(),
            handler: Box::new(move |body| Box::pin(handler(body))),
        }
    }
}

struct Service {
    routes: Vec<Route>,
}

impl Service {
    fn new() -> Self {
        Self { routes: Vec::new() }
    }

    fn route(&mut self, route: Route) {
        self.routes.push(route);
    }

    async fn make_service(
        &self,
        req: Request<hyper::body::Incoming>,
    ) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
        let upper = req.body().size_hint().upper().unwrap_or(u64::MAX);
        // max body size 64kb
        if upper > 1024 * 64 {
            let mut resp = Response::new(full("Body too big"));
            *resp.status_mut() = hyper::StatusCode::PAYLOAD_TOO_LARGE;
            return Ok(resp);
        }

        let route = self.routes.iter().find(|Route { method, route, .. }| {
            method == *req.method() && *route == req.uri().path()
        });

        // Await the whole body to be collected into a single `Bytes`...
        let body = req.collect().await?.to_bytes().to_vec();

        if let Some(route) = route {
            return (route.handler)(body).await;
        }

        let mut not_found = Response::new(empty());
        *not_found.status_mut() = StatusCode::NOT_FOUND;
        Ok(not_found)
    }
}

async fn handler(
    req: Request<hyper::body::Incoming>,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    let mut router = Service::new();
    router.route(Route::new(Method::GET, "/", index_route));
    // router.route(Method::POST, "/echo");
    // router.route(Method::POST, "/echo/uppercase");
    // router.route(Method::POST, "/echo/reversed");
    //
    router.make_service(req).await
}

async fn index_route(_: Vec<u8>) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    Ok(Response::new(full("Try POSTing data to /echo")))
}

async fn echo(
    req: Request<hyper::body::Incoming>,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/") => Ok(Response::new(full("Try POSTing data to /echo"))),
        (&Method::POST, "/echo") => Ok(Response::new(req.into_body().boxed())),
        (&Method::POST, "/echo/uppercase") => {
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

            Ok(Response::new(frame_stream.boxed()))
        }
        (&Method::POST, "/echo/reversed") => {
            // Protect our server from massive bodies.
            let upper = req.body().size_hint().upper().unwrap_or(u64::MAX);
            // max body size 64kb
            if upper > 1024 * 64 {
                let mut resp = Response::new(full("Body too big"));
                *resp.status_mut() = hyper::StatusCode::PAYLOAD_TOO_LARGE;
                return Ok(resp);
            }

            // Await the whole body to be collected into a single `Bytes`...
            let whole_body = req.collect().await?.to_bytes().to_vec();

            let deserialized_struct: Player = serde_json::from_slice(&whole_body).unwrap();

            println!("{deserialized_struct:?}");

            // Iterate the whole body in reverse order and collect into a new Vec.
            let reversed_body = whole_body.iter().rev().cloned().collect::<Vec<u8>>();

            Ok(Response::new(full(reversed_body)))
        }

        // Return 404 Not Found for other routes.
        _ => {
            let mut not_found = Response::new(empty());
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
        }
    }
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
