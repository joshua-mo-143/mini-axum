use http::Request;
use hyper::body::Incoming;
use tower::Service;

#[derive(Clone)]
pub struct LogLayer;

impl<S> tower::Layer<S> for LogLayer {
    type Service = LogService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        LogService { inner }
    }
}

#[derive(Clone)]
pub struct LogService<S> {
    inner: S,
}

impl<S> tower::Service<Request<Incoming>> for LogService<S>
where
    S: Service<Request<Incoming>> + Clone,
{
    type Error = S::Error;
    type Response = S::Response;
    type Future = S::Future;

    fn call(&mut self, req: Request<Incoming>) -> Self::Future {
        let (parts, body) = req.into_parts();
        let req = Request::from_parts(parts.clone(), body);

        let mut service = self.inner.clone();

        service.call(req)
    }

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }
}
