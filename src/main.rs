use axum::{
    routing::get_service,
    Router,
};
use tokio::net::TcpListener;
use tower_http::services::{ServeDir, ServeFile};

#[tokio::main]
async fn main() {
    let app = Router::new()
        .nest_service("/cv", get_service(ServeFile::new("static/cv/cv.pdf")))
        .fallback_service(get_service(ServeDir::new("static")))
        .into_make_service();

    let listener = TcpListener::bind("0.0.0.0:6500").await.unwrap();

    axum::serve(listener, app).await.unwrap();
}
