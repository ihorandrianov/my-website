use axum::{
    routing::{get, get_service},
    Router,
};
use tokio::net::TcpListener;
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .nest_service("/", get_service(ServeDir::new("static/main_page")))
        .nest_service("/blog/", get_service(ServeDir::new("static/blog")))
        .into_make_service();

    let listener = TcpListener::bind("0.0.0.0:6500").await.unwrap();

    axum::serve(listener, app).await.unwrap();
}
