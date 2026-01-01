# hyper_api

A minimal, modular HTTP web framework built on Hyper, providing routing, and async utilities for building fast Rust APIs.

## Features

- **Routing**: Define routes with method and path matching.
- **Async**: Built on top of `tokio` and `hyper` for high performance and scalability.
- **Error Handling**: Custom error types for robust API responses.
- **Extensible**: Easily add new components and utilities.

## Example

```rust
use hyper_api::request::ApiRequest;
use hyper_api::response::HttpResponse;
use hyper_api::route::{HandlerFuture, Route};
use hyper_api::router::Router;
use hyper_api::server::Server;
use hyper_api::error::LibError;
use hyper::Method;

 fn index_route(_req: ApiRequest) -> HttpResponse {
    Box::pin(async { HttpResponse::builder().body("Hello from the server") })
}

#[tokio::main]
async fn main() -> Result<(), LibError> {
    let mut router = Router::new();
    router.add(Route::new(Method::GET, "/", index_route));

    Server::init(([127, 0, 0, 1], 3000))
        .run(move |req| {
            let router = router.clone();
            async move { router.make_service(req).await }
        })
        .await?;

    Ok(())
}
```

## Getting Started

1. Clone or copy this repository to your local machine:
   ```sh
   git clone https://github.com/EArnold1/http-framework.git
   cd http-framework
   ```
2. Open the project in your preferred Rust development environment.
3. Run the project:
   ```sh
   cargo run
   ```

## Requirements

- Rust 1.75+ (2024 edition)

## License

MIT

## Author

[Arnold Emmanuel](https://github.com/EArnold1)
