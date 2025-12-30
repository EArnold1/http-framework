use http_body_util::BodyExt;
use hyper::Method;
use hyper::body::{Bytes, Frame};
use hyper_api::api::{HandlerFuture, Request};
use hyper_api::components::route::Route;
use hyper_api::components::router::Router;
use hyper_api::components::service::Service;
use hyper_api::error::LibError;
use hyper_api::utils::{create_response_body, full, get_req_body};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct Player {
    name: String,
}

fn index_route(_: Request) -> HandlerFuture {
    Box::pin(async { Ok(create_response_body(full("Try POSTing data to /echo"))) })
}

fn echo(req: Request) -> HandlerFuture {
    let body = req.into_body();
    Box::pin(async { Ok(create_response_body(body)) })
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

    let body = create_response_body(frame_stream);

    Box::pin(async { Ok(body) })
}

fn echo_reversed(req: Request) -> HandlerFuture {
    Box::pin(async move {
        let body = get_req_body(req).await?;
        let reversed_body = body.iter().rev().cloned().collect::<Vec<u8>>();
        Ok(create_response_body(full(reversed_body)))
    })
}

fn create_player(req: Request) -> HandlerFuture {
    Box::pin(async move {
        let body = get_req_body(req).await?;

        let player: Player = match serde_json::from_slice(&body) {
            Ok(data) => data,
            Err(e) => return Err(LibError::JsonParseError(e)),
        };

        println!("{player:?}");

        Ok(create_response_body(full("Player added")))
    })
}

#[tokio::main]
async fn main() -> Result<(), LibError> {
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
