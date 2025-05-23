use serde::{Deserialize, Serialize};

use http_body_util::Full;
use hyper::body::Bytes;
use hyper::{Response, StatusCode};

pub trait IntoMiniResponse {
    fn into_response(self) -> MiniResponse;
}

pub struct MiniResponse {
    code: StatusCode,
    content_type: String,
    bytes: Bytes,
}

impl MiniResponse {
    fn new(code: StatusCode, content_type: &str, bytes: Bytes) -> Self {
        Self {
            code,
            content_type: content_type.to_string(),
            bytes,
        }
    }

    pub fn hyper_response(self) -> Response<Full<Bytes>> {
        Response::builder()
            .status(self.code)
            .header("Content-Type", self.content_type)
            .body(Full::new(self.bytes))
            .unwrap()
    }
}

#[derive(Deserialize, Serialize)]
pub struct Json<T>(pub T);

impl<T> IntoMiniResponse for (StatusCode, Json<T>)
where
    T: Serialize,
{
    fn into_response(self) -> MiniResponse {
        let (code, bytes) = self;
        let bytes = Bytes::from(serde_json::to_vec(&bytes.0).unwrap());

        MiniResponse::new(code, "application/json", bytes)
    }
}

impl<T> IntoMiniResponse for Json<T>
where
    T: Serialize,
{
    fn into_response(self) -> MiniResponse {
        let bytes = Bytes::from(serde_json::to_vec(&self.0).unwrap());

        MiniResponse::new(StatusCode::OK, "application/json", bytes)
    }
}

impl IntoMiniResponse for (StatusCode, &'static str) {
    fn into_response(self) -> MiniResponse {
        let (code, bytes) = self;
        let bytes = Bytes::from(serde_json::to_vec(&bytes).unwrap());

        MiniResponse::new(code, "text/plain", bytes)
    }
}

impl IntoMiniResponse for &'static str {
    fn into_response(self) -> MiniResponse {
        let bytes = Bytes::from(serde_json::to_vec(&self).unwrap());

        MiniResponse::new(StatusCode::OK, "text/plain", bytes)
    }
}

impl<T, E> IntoMiniResponse for Result<T, E>
where
    T: IntoMiniResponse,
    E: IntoMiniResponse,
{
    fn into_response(self) -> MiniResponse {
        match self {
            Ok(res) => res.into_response(),
            Err(err) => err.into_response(),
        }
    }
}
