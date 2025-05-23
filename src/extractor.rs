use http::request::Parts;
use http_body_util::BodyExt;
use hyper::{Request, body::Incoming};
use serde::Deserialize;

use crate::response::Json;

pub trait FromRequest<S>: Send + Sync {
    fn from_request(req: Request<Incoming>, state: &S) -> impl Future<Output = Self> + Send;
}

impl<S, T> FromRequest<S> for Json<T>
where
    T: for<'a> Deserialize<'a> + Send + Sync,
    S: Clone + Send + Sync,
{
    async fn from_request(req: Request<Incoming>, _state: &S) -> Self {
        let (_, body) = req.into_parts();
        let body: Vec<u8> = body.collect().await.unwrap().to_bytes().to_vec();
        let json: T = serde_json::from_slice(&body).unwrap();

        Json(json)
    }
}

impl<S, T1> FromRequest<S> for (T1,)
where
    T1: FromRequest<S>,
    S: Clone + Send + Sync,
{
    async fn from_request(req: Request<Incoming>, state: &S) -> Self {
        let t1 = T1::from_request(req, &state).await;

        (t1,)
    }
}

impl<S, T1, T2> FromRequest<S> for (T1, T2)
where
    T1: FromRequestParts<S>,
    T2: FromRequest<S>,
    S: Clone + Send + Sync,
{
    async fn from_request(req: Request<Incoming>, state: &S) -> Self {
        let (parts, body) = req.into_parts();
        let t1 = T1::from_request_parts(parts.clone(), &state).await;

        let req = Request::from_parts(parts, body);
        let t2 = T2::from_request(req, &state).await;

        (t1, t2)
    }
}

pub trait FromRequestParts<S>: Send + Sync {
    fn from_request_parts(
        req: http::request::Parts,
        state: &S,
    ) -> impl Future<Output = Self> + Send;
}

pub struct State<T>(pub T);

impl<S> FromRequestParts<S> for State<S>
where
    S: Clone + Send + Sync,
{
    async fn from_request_parts(_req: Parts, state: &S) -> Self {
        State(state.to_owned())
    }
}

impl<S> FromRequest<S> for State<S>
where
    S: Clone + Send + Sync,
{
    async fn from_request(_req: Request<Incoming>, state: &S) -> Self {
        State(state.to_owned())
    }
}
