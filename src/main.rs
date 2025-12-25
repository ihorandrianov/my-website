use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    routing::{get, get_service, post},
    Json, Router,
};
use serde::Serialize;
use sqlx::postgres::PgPoolOptions;
use tokio::net::TcpListener;
use tower_http::services::{ServeDir, ServeFile};

use crate::db::{Db, SensorData};

mod db;

#[derive(Clone)]
struct AppState {
    db: Db,
    api_key: String,
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let api_key = std::env::var("API_KEY").expect("API_KEY must be set");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    let state = AppState {
        db: Db::new(pool.clone()),
        api_key,
    };

    let app = Router::new()
        .nest("/api", iot_api(state.clone()))
        .route_service("/cv", get_service(ServeFile::new("static/cv/cv.pdf")))
        .route_service("/cv/", get_service(ServeFile::new("static/cv/cv.pdf")))
        .fallback_service(get_service(ServeDir::new("static")))
        .into_make_service();

    println!("Server running on http://0.0.0.0:6500");
    let listener = TcpListener::bind("0.0.0.0:6500").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

fn iot_api(state: AppState) -> Router {
    Router::new()
        .route("/tasks", get(get_tasks))
        .route("/sensor", post(post_sensor))
        .with_state(state)
}

fn check_api_key(headers: &HeaderMap, expected: &str) -> bool {
    headers
        .get("X-Api-Key")
        .and_then(|v| v.to_str().ok())
        .map(|v| v == expected)
        .unwrap_or(false)
}

// POST /api/sensor - receive sensor data from device
async fn post_sensor(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(data): Json<SensorData>,
) -> StatusCode {
    if !check_api_key(&headers, &state.api_key) {
        return StatusCode::UNAUTHORIZED;
    }

    println!(
        "Received sensor data: T={:.1}C H={:.1}% P={:.1}hPa",
        data.temperature, data.humidity, data.pressure
    );

    match state.db.write_sensor_data(data).await {
        Ok(_) => StatusCode::OK,
        Err(e) => {
            eprintln!("Failed to write sensor data: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

#[derive(Serialize)]
struct TasksResponse {
    tasks: Vec<Task>,
}

#[allow(dead_code)]
#[derive(Serialize)]
#[serde(tag = "type")]
enum Task {
    PumpOn { duration_secs: u32 },
    PumpOff,
    ReadNow,
}

async fn get_tasks(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<TasksResponse>, StatusCode> {
    if !check_api_key(&headers, &state.api_key) {
        return Err(StatusCode::UNAUTHORIZED);
    }

    // TODO: Fetch pending tasks from database idk
    // For now, return empty task list
    Ok(Json(TasksResponse { tasks: vec![] }))
}
