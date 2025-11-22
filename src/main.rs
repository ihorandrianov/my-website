use axum::{
    routing::{get, get_service},
    Router,
};
use tokio::net::TcpListener;
use tower_http::services::{ServeDir, ServeFile};

#[tokio::main]
async fn main() {
    let app = Router::new()
        .nest_service("/blog", get_service(ServeDir::new("static/blog")))
        .nest_service("/cv", get_service(ServeFile::new("static/cv/cv.pdf")))
        .nest_service("/", get_service(ServeDir::new("static/main_page")))
        .into_make_service();

    let listener = TcpListener::bind("0.0.0.0:6500").await.unwrap();

    axum::serve(listener, app).await.unwrap();
}
