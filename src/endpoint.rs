use std::any::Any;
use std::marker::PhantomData;
use std::sync::Arc;

use bytes::Bytes;
use futures::TryFutureExt;
use futures::future::BoxFuture;
use http_body_util::Full;
use hyper::body::Incoming;
// use hyper::service::Service;
use hyper::{Request, Response};
use tower::{Layer, Service, ServiceBuilder};

use crate::extractor::{FromRequest, FromRequestParts};
use crate::response::IntoMiniResponse;

pub trait IntoHandler<T, S>: Sized + Clone {
    fn into_handler(self, state: S) -> IntoHandlerStruct<Self, T, S>;
}

impl<F, Fut, I, S> IntoHandler<(), S> for F
where
    F: Fn() -> Fut + Clone + Send + Sync + 'static,
    Fut: Future<Output = I> + Send + 'static,
    I: IntoMiniResponse,
{
    fn into_handler(self, state: S) -> IntoHandlerStruct<Self, (), S> {
        IntoHandlerStruct {
            inner: self,
            state,
            _tytypes: PhantomData,
        }
    }
}

impl<F, Fut, I, S, T1> IntoHandler<(T1,), S> for F
where
    F: Fn(T1) -> Fut + Clone + Send + Sync + 'static,
    Fut: Future<Output = I> + Send + 'static,
    I: IntoMiniResponse,
    T1: FromRequest<S> + Send + 'static,
{
    fn into_handler(self, state: S) -> IntoHandlerStruct<Self, (T1,), S> {
        IntoHandlerStruct {
            inner: self,
            state,
            _tytypes: PhantomData,
        }
    }
}

impl<F, Fut, I, S, T1, T2> IntoHandler<(T1, T2), S> for F
where
    F: Fn(T1, T2) -> Fut + Clone + Send + Sync + 'static,
    Fut: Future<Output = I> + Send + 'static,
    I: IntoMiniResponse,
    T1: FromRequestParts<S> + Send + 'static,
    T2: FromRequest<S> + Send + 'static,
{
    fn into_handler(self, state: S) -> IntoHandlerStruct<Self, (T1, T2), S> {
        IntoHandlerStruct {
            inner: self,
            state,
            _tytypes: PhantomData,
        }
    }
}

pub struct IntoHandlerStruct<H, T, S> {
    inner: H,
    state: S,
    _tytypes: PhantomData<T>,
}

impl<H, T, S> Clone for IntoHandlerStruct<H, T, S>
where
    H: Clone,
    S: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            state: self.state.clone(),
            _tytypes: PhantomData,
        }
    }
}

impl<H, T, S> IntoHandlerStruct<H, T, S> {
    pub fn into_service(self, state: S) -> HandlerService<H, T, S> {
        HandlerService {
            inner: self.inner,
            state,
            _tytypes: self._tytypes,
        }
    }
}

pub struct HandlerService<H, T, S> {
    inner: H,
    state: S,
    _tytypes: PhantomData<T>,
}

impl<H, T, S> Clone for HandlerService<H, T, S>
where
    H: Clone,
    S: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            state: self.state.clone(),
            _tytypes: PhantomData,
        }
    }
}

impl<H, Fut, S, I> tower::Service<Request<Incoming>> for IntoHandlerStruct<H, (), S>
where
    H: Fn() -> Fut + Clone + Send + Sync + 'static,
    Fut: Future<Output = I> + Send + 'static,
    I: IntoMiniResponse,
    S: Clone + Send + Sync + 'static,
{
    type Error = hyper::Error;
    type Response = Response<Full<Bytes>>;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;
    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }
    fn call(&mut self, _req: Request<Incoming>) -> Self::Future {
        let thing = self.inner.clone();

        Box::pin(async move { Ok((thing)().await.into_response().hyper_response()) })
    }
}

impl<H, Fut, S, I, T1> tower::Service<Request<Incoming>> for IntoHandlerStruct<H, (T1,), S>
where
    H: Fn(T1) -> Fut + Clone + Send + Sync + 'static,
    Fut: Future<Output = I> + Send + 'static,
    I: IntoMiniResponse,
    T1: FromRequest<S> + Send + 'static,
    S: Send + Clone + Sync + 'static,
{
    type Error = hyper::Error;
    type Response = Response<Full<Bytes>>;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;
    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }
    fn call(&mut self, req: Request<Incoming>) -> Self::Future {
        let thing = self.inner.clone();
        let state = self.state.clone();

        Box::pin(async move {
            let (t1) = T1::from_request(req, &state).await;
            Ok((thing)(t1).await.into_response().hyper_response())
        })
    }
}

impl<H, Fut, S, I, T1, T2> tower::Service<Request<Incoming>> for IntoHandlerStruct<H, (T1, T2), S>
where
    H: Fn(T1, T2) -> Fut + Clone + Send + Sync + 'static,
    Fut: Future<Output = I> + Send + 'static,
    I: IntoMiniResponse,
    T2: FromRequest<S> + Send + 'static,
    T1: FromRequestParts<S> + Send + 'static,
    S: Send + Clone + Sync + 'static,
{
    type Error = hyper::Error;
    type Response = Response<Full<Bytes>>;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;
    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }
    fn call(&mut self, req: Request<Incoming>) -> Self::Future {
        let thing = self.inner.clone();
        let state = self.state.clone();

        Box::pin(async move {
            let (t1, t2) = <(T1, T2)>::from_request(req, &state).await;
            Ok((thing)(t1, t2).await.into_response().hyper_response())
        })
    }
}

// impl<H, T, S> tower::Service<Request<Incoming>> for HandlerService<H, T, S>
// where
//     S: Clone + Send + Sync + 'static,
//     H: Endpoint<S> + Clone + Send + Sync + 'static,
// {
//     type Error = hyper::Error;
//     type Response = Response<Full<Bytes>>;
//     type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;
//     fn poll_ready(
//         &mut self,
//         _cx: &mut std::task::Context<'_>,
//     ) -> std::task::Poll<Result<(), Self::Error>> {
//         std::task::Poll::Ready(Ok(()))
//     }

//     fn call(&mut self, req: Request<Incoming>) -> Self::Future {
//         let service = self.inner.clone();
//         let state = self.state.clone();
//         Box::pin(async move { service.call_handler(req, state).await })
//     }
// }

// impl<H, T, S> Endpoint<S> for HandlerService<H, T, S>
// where
//     S: Clone + Send + Sync + 'static,
//     // H: Endpoint<S> + Clone + Send + Sync + 'static,
//     T: Sync + Send + 'static,
// {
//     fn call_handler(
//         &self,
//         req: Request<Incoming>,
//         state: S,
//     ) -> BoxFuture<'static, Result<Response<Full<Bytes>>, hyper::Error>> {
//         let service = self.inner.clone();
//         Box::pin(async move { service.call_handler(req, state).await })
//     }
// }

// pub trait Endpoint<S>: Any + Send + Sync {
//     fn call_handler(
//         &self,
//         req: Request<Incoming>,
//         state: S,
//     ) -> BoxFuture<'static, Result<Response<Full<Bytes>>, hyper::Error>>;
// }

// impl<H, Fut, I, S> Endpoint<S> for IntoHandlerStruct<H, (), S>
// where
//     H: Fn() -> Fut + Clone + Send + Sync + 'static,
//     Fut: Future<Output = I> + Send,
//     I: IntoMiniResponse,
//     S: Clone + Send + Sync + 'static,
// {
//     fn call_handler(
//         &self,
//         _req: Request<Incoming>,
//         _state: S,
//     ) -> BoxFuture<'static, Result<Response<Full<Bytes>>, hyper::Error>> {
//         let inner = self.inner.clone();
//         Box::pin(async move { Ok((inner)().await.into_response().hyper_response()) })
//     }
// }

// impl<H, Fut, I, T1, S> Endpoint<S> for IntoHandlerStruct<H, (T1,), S>
// where
//     H: Fn(T1) -> Fut + Clone + Send + Sync + 'static,
//     Fut: Future<Output = I> + Send,
//     I: IntoMiniResponse,
//     S: Clone + Send + Sync + 'static,
//     T1: FromRequest<S> + Send + Sync + 'static,
// {
//     fn call_handler(
//         &self,
//         req: Request<Incoming>,
//         state: S,
//     ) -> BoxFuture<'static, Result<Response<Full<Bytes>>, hyper::Error>> {
//         let inner = self.inner.clone();
//         Box::pin(async move {
//             let res = T1::from_request(req, &state).await;

//             Ok((inner)(res).await.into_response().hyper_response())
//         })
//     }
// }

// impl<H, Fut, I, T1, T2, S> Endpoint<S> for IntoHandlerStruct<H, (T1, T2), S>
// where
//     H: Fn(T1, T2) -> Fut + Clone + Send + Sync + 'static,
//     Fut: Future<Output = I> + Send,
//     I: IntoMiniResponse,
//     S: Clone + Send + Sync + 'static,
//     T1: FromRequestParts<S> + Send + Sync + 'static,
//     T2: FromRequest<S> + Send + Sync + 'static,
// {
//     fn call_handler(
//         &self,
//         req: Request<Incoming>,
//         state: S,
//     ) -> BoxFuture<'static, Result<Response<Full<Bytes>>, hyper::Error>> {
//         let inner = self.inner.clone();
//         Box::pin(async move {
//             let (t1, t2) = <(T1, T2)>::from_request(req, &state).await;

//             Ok((inner)(t1, t2).await.into_response().hyper_response())
//         })
//     }
// }
