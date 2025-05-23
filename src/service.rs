use std::pin::Pin;

use tokio::net::TcpListener;

use hyper::server::conn::http1;
use hyper_util::rt::TokioIo;

use crate::router::Router;

pub struct Service<S> {
    tcp: TcpListener,
    router: Router<S>,
}

impl<S> Service<S>
where
    S: Clone + Send + Sync + 'static,
{
    pub fn new(tcp: TcpListener, router: Router<S>) -> Self {
        Self { tcp, router }
    }

    async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            let (stream, _) = self.tcp.accept().await?;
            let io = TokioIo::new(stream);

            let rtr = self.router.clone();
            tokio::task::spawn(async move {
                if let Err(err) = http1::Builder::new().serve_connection(io, rtr).await {
                    eprintln!("Error serving connection: {:?}", err);
                }
            });
        }
    }
}

impl<S> IntoFuture for Service<S>
where
    S: Clone + Send + Sync + 'static,
{
    type Output = Result<(), Box<dyn std::error::Error>>;
    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + Send>>;
    fn into_future(self) -> Self::IntoFuture {
        Box::pin(self.run())
    }
}
