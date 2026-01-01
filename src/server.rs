use std::net::{IpAddr, SocketAddr};

use hyper::{server::conn::http1, service::service_fn};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;

use crate::{error::LibError, request::ApiRequest, route::HandlerResult};

#[derive(Debug, Clone)]
pub struct Server {
    addr: SocketAddr,
}

impl Server {
    pub fn init<I>(addr: (I, u16)) -> Self
    where
        I: Into<IpAddr>,
    {
        Self {
            addr: SocketAddr::from(addr),
        }
    }

    /// Run the HTTP server and handle incoming connections.
    pub async fn run<H, Fut>(&self, handler: H) -> Result<(), LibError>
    where
        H: Fn(ApiRequest) -> Fut + Clone + Send + Sync + 'static,
        Fut: Future<Output = HandlerResult> + Send + 'static,
    {
        let listener = TcpListener::bind(self.addr).await?;

        loop {
            let (stream, _) = listener.accept().await?;

            let io = TokioIo::new(stream);

            let handler = handler.clone();

            // Spawn a tokio task to serve multiple connections concurrently
            tokio::task::spawn(async move {
                // bind the incoming connection to our service
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
}
