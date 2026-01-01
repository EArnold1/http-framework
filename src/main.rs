use http_body_util::BodyExt;
use hyper::body::{Bytes, Frame};
use hyper::{Method, StatusCode};
use hyper_api::api::{ApiRequest, HandlerFuture, HttpResponse};
use hyper_api::components::route::Route;
use hyper_api::components::router::Router;
use hyper_api::components::service::Service;
use hyper_api::error::LibError;
use hyper_api::utils::get_req_body;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct Player {
    name: String,
}

fn index_route(_: ApiRequest) -> HandlerFuture {
    Box::pin(async { HttpResponse::builder().body("Try POSTing data to /echo") })
}

fn echo(req: ApiRequest) -> HandlerFuture {
    Box::pin(async {
        let body = req.into_body().collect().await?.to_bytes();

        HttpResponse::builder().body(body)
    })
}

fn echo_uppercase(req: ApiRequest) -> HandlerFuture {
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

    Box::pin(async {
        let data = frame_stream.collect().await?.to_bytes();

        HttpResponse::builder().body(data)
    })
}

fn echo_reversed(req: ApiRequest) -> HandlerFuture {
    Box::pin(async move {
        let body = get_req_body(req).await?;
        let reversed_body = body.iter().rev().cloned().collect::<Vec<u8>>();

        HttpResponse::builder().body(reversed_body)
    })
}

fn create_player(req: ApiRequest) -> HandlerFuture {
    Box::pin(async move {
        let body = get_req_body(req).await?;

        let player: Player = match serde_json::from_slice(&body) {
            Ok(data) => data,
            Err(e) => return Err(LibError::JsonParseError(e)),
        };

        println!("{player:?}");

        HttpResponse::builder()
            .status_code(StatusCode::CREATED)
            .json(&player)
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
