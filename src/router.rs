use std::any::Any;
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::{Arc, RwLock};

use futures::FutureExt;
use futures::future::BoxFuture;
use http_body_util::Full;
use hyper::body::{Bytes, Incoming};
// use hyper::service::Service;
use hyper::{Request, Response, StatusCode};
use tower::Service;
use tower::util::{BoxCloneService, BoxCloneSyncService};
use tower::{Layer, ServiceBuilder};

use crate::endpoint::{HandlerService, IntoHandler, IntoHandlerStruct};
use crate::response::IntoMiniResponse;

#[derive(Clone, Default)]
pub struct Router<S = ()> {
    pub inner: Arc<RwLock<HashMap<String, DynService>>>,
    state: S,
}

impl<S> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    pub fn with_state(state: S) -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
            state,
        }
    }

    pub fn route<T, E>(self, route: &str, endpoint: E) -> Self
    where
        T: 'static + Sync + Send,
        E: IntoHandler<T, S> + Clone + Send + Sync + 'static,
        IntoHandlerStruct<E, T, S>: tower::Service<
                Request<Incoming>,
                Response = Response<Full<Bytes>>,
                Error = hyper::Error,
                Future = Pin<
                    Box<dyn Future<Output = Result<Response<Full<Bytes>>, hyper::Error>> + Send>,
                >,
            > + 'static,
    {
        let endpoint = endpoint.into_handler(self.state.clone());

        self.inner
            .write()
            .unwrap()
            .insert(route.to_string(), BoxCloneSyncService::new(endpoint));

        self
    }

    pub fn layer<L>(mut self, layer: L) -> Self
    where
        L: Layer<DynService> + Clone + Send + Sync + 'static,
        L::Service: Service<
                Request<Incoming>,
                Response = Response<Full<Bytes>>,
                Error = hyper::Error,
                Future = BoxFuture<'static, Result<Response<Full<Bytes>>, hyper::Error>>,
            > + Clone
            + Send
            + Sync
            + 'static,
        <L::Service as Service<Request<Incoming>>>::Future: Send + 'static,
    {
        let res: HashMap<String, DynService> = self
            .inner
            .write()
            .unwrap()
            .clone()
            .into_iter()
            .map(|(k, v)| {
                let service = ServiceBuilder::new().layer(layer.clone()).service(v);

                (k, BoxCloneSyncService::new(service))
            })
            .collect();

        self.inner = Arc::new(RwLock::new(res));

        self
    }
}
type DynService = BoxCloneSyncService<Request<Incoming>, Response<Full<Bytes>>, hyper::Error>;

impl Router<()> {
    pub fn stateless() -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
            state: (),
        }
    }
}

impl<S> hyper::service::Service<Request<Incoming>> for Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    type Response = Response<Full<Bytes>>;
    type Error = hyper::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&self, req: Request<Incoming>) -> Self::Future {
        let rdr = self.inner.read().unwrap();
        let path = req.uri().path();

        println!("Path: {path}");
        if let Some(func) = rdr.get(req.uri().path()) {
            let mut func = func.clone();
            Box::pin(async move { func.call(req).await })
        } else {
            Box::pin(async move {
                Ok((StatusCode::NOT_FOUND, "Not found")
                    .into_response()
                    .hyper_response())
            })
        }
    }
}
