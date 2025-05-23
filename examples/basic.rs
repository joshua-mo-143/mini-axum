use hyper::StatusCode;
use mini_axum::{
    Router, Service,
    extractor::State,
    middleware::LogLayer,
    response::{IntoMiniResponse, Json},
};
use serde_json::{Value, json};
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;

#[tokio::main]
async fn main() {
    let rtr = Router::with_state("Hello world!")
        .route("/", hello_world)
        .route("/echo", echo_message)
        .layer(LogLayer);

    let tcp = TcpListener::bind("127.0.0.1:9999")
        .await
        .expect("to be able to bind to localhost port 9999");

    let svc = Service::new(tcp, rtr);

    svc.await.unwrap();
}

pub async fn hello_world() -> impl IntoMiniResponse {
    let json = json!({"message": "Hello world!"});

    (StatusCode::OK, Json(json))
}

pub async fn echo_message(Json(json): Json<Value>) -> Result<impl IntoMiniResponse, &'static str> {
    Ok(Json(json))
}
