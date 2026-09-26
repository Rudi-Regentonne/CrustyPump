use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, postgres::PgPoolOptions};
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::services::ServeDir;
use log::{info, error};

#[derive(Clone)]
struct AppState {
    db: PgPool,
    config_path: String,
}

#[derive(Serialize, Deserialize, sqlx::FromRow)]
struct Channel {
    channel: String,
    measurement: String,
}

#[derive(Deserialize)]
struct DataQuery {
    channel: String,
    measurement: String,
    start: String,
    end: String,
    granularity: String,
}

#[derive(Serialize, sqlx::FromRow)]
struct DataResponse {
    bucket: Option<DateTime<Utc>>,
    val: Option<f64>,
}

#[tokio::main]
async fn main() {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://crustypump:password@localhost:5432/heatpump".to_string());
    let config_path = std::env::var("DASHBOARD_CONFIG")
        .unwrap_or_else(|_| "data/layout.json".to_string());

    let pool = loop {
        match PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
        {
            Ok(pool) => break pool,
            Err(e) => {
                error!("Database connection failed: {} — retrying in 2s", e);
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            }
        }
    };

    let state = AppState {
        db: pool,
        config_path,
    };

    let app = Router::new()
        .route("/api/channels", get(get_channels))
        .route("/api/data", get(get_data))
        .route("/api/config", get(get_config).post(save_config))
        .nest_service("/", ServeDir::new("assets"))
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8050));
    info!("Dashboard server listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn get_channels(State(state): State<AppState>) -> Result<Json<Vec<Channel>>, StatusCode> {
    let rows = sqlx::query_as::<_, Channel>(
        "SELECT DISTINCT channel, measurement FROM heatpump_data ORDER BY channel"
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        error!("DB error: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(rows))
}

async fn get_data(
    State(state): State<AppState>,
    Query(params): Query<DataQuery>,
) -> Result<Json<Vec<DataResponse>>, StatusCode> {
    let start_dt = chrono::DateTime::parse_from_rfc3339(&params.start)
        .or_else(|_| chrono::DateTime::parse_from_rfc3339(&format!("{}Z", params.start)))
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    let end_dt = chrono::DateTime::parse_from_rfc3339(&params.end)
        .or_else(|_| chrono::DateTime::parse_from_rfc3339(&format!("{}Z", params.end)))
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    // Handle granularity formatting slightly if needed, e.g. "1 hour", "15 minutes"
    let rows = sqlx::query_as::<_, DataResponse>(
        r#"
        SELECT time_bucket(CAST($5 AS interval), time) AS "bucket",
               AVG(COALESCE(val_float, val_int::float8, val_bool::int::float8)) AS "val"
        FROM heatpump_data
        WHERE channel = $1 AND measurement = $2 AND time >= $3 AND time <= $4
        GROUP BY bucket
        ORDER BY bucket
        "#
    )
    .bind(&params.channel)
    .bind(&params.measurement)
    .bind(start_dt.with_timezone(&Utc))
    .bind(end_dt.with_timezone(&Utc))
    .bind(&params.granularity)
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        error!("DB query error: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(rows))
}

async fn get_config(State(state): State<AppState>) -> Result<Json<serde_json::Value>, StatusCode> {
    match std::fs::read_to_string(&state.config_path) {
        Ok(content) => {
            let json: serde_json::Value = serde_json::from_str(&content).unwrap_or_else(|_| serde_json::json!({}));
            Ok(Json(json))
        }
        Err(_) => {
            // Default config
            let default = serde_json::json!({
                "time_range_preset": "1h",
                "granularity": "1 minute",
                "tiles": []
            });
            Ok(Json(default))
        }
    }
}

async fn save_config(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<StatusCode, StatusCode> {
    if let Some(parent) = std::path::Path::new(&state.config_path).parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    std::fs::write(&state.config_path, payload.to_string()).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(StatusCode::OK)
}
