use thiserror::Error;

#[derive(Error, Debug)]
pub enum LibError {
    #[error("Request body exceeded the allowed maximum size")]
    RequestBodyTooLarge,

    #[error("An error occurred in the HTTP runtime layer: {0}")]
    Hyper(#[from] hyper::Error),

    #[error("An error occurred in the HTTP layer: {0}")]
    Http(#[from] hyper::http::Error),

    #[error("An error occurred in the standard library: {0}")]
    Std(#[from] std::io::Error),

    #[error("Failed to parse JSON payload: {0}")]
    JsonParseError(#[from] serde_json::Error),
}
