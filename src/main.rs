use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::{get, get_service},
    Router,
};
use tokio::net::TcpListener;
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() {
    let api_router = Router::<()>::new().route("/hello", get(hello));

    let app = Router::new()
        .nest("/api", api_router)
        .fallback_service(get_service(ServeDir::new("static")))
        .into_make_service();

    let listener = TcpListener::bind("0.0.0.0:6500").await.unwrap();

    axum::serve(listener, app).await.unwrap();
}

async fn hello() -> impl IntoResponse {
    return (StatusCode::OK, "Hello!");
}
