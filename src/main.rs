use std::sync::Arc;

use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    routing::{get, get_service, post},
    Json, Router,
};
use serde::Serialize;
use sqlx::postgres::PgPoolOptions;
use teloxide::Bot;
use tokio::net::TcpListener;
use tower_http::services::{ServeDir, ServeFile};

mod alerter;
mod bot;
mod config;
mod db;
mod listener;
mod power_monitor;
mod services;

use db::{Db, SensorData};

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
    let webhook_secret = std::env::var("WEBHOOK_SECRET").expect("WEBHOOK_SECRET must be set");
    let bot_secret = std::env::var("BOT_SECRET").expect("BOT_SECRET must be set");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    let client = reqwest::Client::builder()
        .local_address(IpAddr::from_str("0.0.0.0").unwrap()) // <--- THE FIX
        .build()
        .expect("Failed to build client");

    let db = Db::new(pool.clone());
    let bot = Arc::new(Bot::from_env());

    let bot_state = bot::BotState {
        db: db.clone(),
        bot_secret,
    };

    let bot_router = bot::init_bot(bot.clone(), webhook_secret, bot_state)
        .await
        .expect("Failed to init bot");

    let alerter = alerter::Alerter::new(bot, db.clone());

    listener::spawn_sensor_listener(pool, alerter.clone())
        .await
        .expect("Failed to spawn sensor listener");

    power_monitor::spawn_power_monitor(db.clone(), alerter);

    let state = AppState { db, api_key };

    let app = Router::new()
        .merge(bot_router)
        .nest("/api", api_routes(state))
        .route_service("/cv", get_service(ServeFile::new("static/cv/cv.pdf")))
        .route_service("/cv/", get_service(ServeFile::new("static/cv/cv.pdf")))
        .fallback_service(get_service(ServeDir::new("static")))
        .into_make_service();

    println!("Server running on http://0.0.0.0:6500");
    let listener = TcpListener::bind("0.0.0.0:6500").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

fn api_routes(state: AppState) -> Router {
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
    pump_duration: u16,
}

async fn get_tasks(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<TasksResponse>, StatusCode> {
    if !check_api_key(&headers, &state.api_key) {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let pump_duration = state
        .db
        .get_pending_pump_command()
        .await
        .ok()
        .flatten()
        .unwrap_or(0) as u16;

    Ok(Json(TasksResponse { pump_duration }))
}
