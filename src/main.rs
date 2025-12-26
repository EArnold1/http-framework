use http_body_util::BodyExt;
use hyper::Method;
use hyper::body::{Bytes, Frame};
use hyper_api::api::{HandlerFuture, Request, Response};
use hyper_api::components::route::Route;
use hyper_api::components::router::Router;
use hyper_api::components::service::Service;
use hyper_api::utils::{full, get_req_body};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct Player {
    name: String,
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

fn create_player(req: Request) -> HandlerFuture {
    Box::pin(async move {
        match get_req_body(req).await? {
            Some(body) => {
                let player: Player = serde_json::from_slice(&body).unwrap();

                println!("{player:?}");

                Ok(Response::new(full("Player added")))
            }
            None => {
                let mut resp = Response::new(full("Body too big"));
                *resp.status_mut() = hyper::StatusCode::PAYLOAD_TOO_LARGE;
                Ok(resp)
            }
        }
    })
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut router = Router::default();
    router.route(Route::new(Method::GET, "/", index_route));
    router.route(Route::new(Method::POST, "/echo", echo));
    router.route(Route::new(Method::POST, "/echo/uppercase", echo_uppercase));
    router.route(Route::new(Method::POST, "/echo/reversed", echo_reversed));
    router.route(Route::new(Method::POST, "/player", create_player));

    Service::init(([127, 0, 0, 1], 3000))
        .run(move |req| {
            let router = router.clone();
            async move { router.make_service(req).await }
        })
        .await?;

    Ok(())
}
