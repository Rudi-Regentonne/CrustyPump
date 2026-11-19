use axum::{
    extract::{Query, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, postgres::PgPoolOptions};
use std::net::SocketAddr;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tower_http::services::ServeDir;
use log::{info, error};

const CHANNELS_CACHE_TTL: Duration = Duration::from_secs(600);

#[derive(Clone)]
struct AppState {
    db: Option<PgPool>,
    config_path: String,
    channels_cache: Arc<RwLock<Option<(Instant, Vec<Channel>)>>>,
    refreshing_channels: Arc<AtomicBool>,
}

#[derive(Clone, Serialize, Deserialize, sqlx::FromRow)]
struct Channel {
    channel: String,
    measurement: String,
}

#[derive(Deserialize)]
struct DataQuery {
    channel: String,
    #[allow(dead_code)]
    measurement: Option<String>,
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
    let mock_db = matches!(
        std::env::var("MOCK_DB").as_deref(),
        Ok("1") | Ok("true") | Ok("yes")
    );

    let pool = if mock_db {
        info!("MOCK_DB enabled — skipping PostgreSQL connection and serving synthetic data");
        None
    } else {
        Some(loop {
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
        })
    };

    let state = AppState {
        db: pool,
        config_path,
        channels_cache: Arc::new(RwLock::new(None)),
        refreshing_channels: Arc::new(AtomicBool::new(false)),
    };

    // Warm the channel list cache at startup so the first dropdown load is instant.
    if state.db.is_some() {
        let warmup_state = state.clone();
        tokio::spawn(async move {
            if let Err(e) = refresh_channels_cache(&warmup_state).await {
                error!("Initial channel cache warmup failed: {}", e);
            }
        });
    }

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

async fn fetch_channels(db: &PgPool) -> Result<Vec<Channel>, sqlx::Error> {
    let recent = sqlx::query_as::<_, Channel>(
        "SELECT DISTINCT ON (channel) channel, measurement FROM heatpump_data WHERE time >= NOW() - INTERVAL '7 days' ORDER BY channel, time DESC",
    )
    .fetch_all(db)
    .await?;

    if !recent.is_empty() {
        return Ok(recent);
    }

    sqlx::query_as::<_, Channel>(
        "SELECT DISTINCT ON (channel) channel, measurement FROM heatpump_data ORDER BY channel, time DESC",
    )
    .fetch_all(db)
    .await
}

async fn refresh_channels_cache(state: &AppState) -> Result<Vec<Channel>, sqlx::Error> {
    let rows = match &state.db {
        Some(db) => fetch_channels(db).await?,
        None => mock_channels(),
    };
    let mut guard = state.channels_cache.write().await;
    *guard = Some((Instant::now(), rows.clone()));
    Ok(rows)
}

fn mock_channels() -> Vec<Channel> {
    vec![
        Channel { channel: "Platinentemperatur".into(), measurement: "Fühlereingänge".into() },
        Channel { channel: "Außentemperatur".into(), measurement: "Fühlereingänge".into() },
        Channel { channel: "Vorlauftemperatur".into(), measurement: "Fühlereingänge".into() },
        Channel { channel: "Rücklauftemperatur".into(), measurement: "Fühlereingänge".into() },
        Channel { channel: "Heizen".into(), measurement: "live_data".into() },
    ]
}

fn parse_granularity_secs(granularity: &str) -> i64 {
    match granularity {
        "1 minute" => 60,
        "15 minutes" => 900,
        "1 hour" => 3600,
        "1 day" => 86400,
        _ => 60,
    }
}

fn mock_data(
    channel: &str,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    granularity: &str,
) -> Vec<DataResponse> {
    let step_secs = parse_granularity_secs(granularity).max(1);
    let total_secs = (end - start).num_seconds().max(step_secs);
    let max_points = 120;
    let effective_step = (total_secs / max_points).max(step_secs);

    let base = match channel {
        "Heizen" => 1.0,
        "Außentemperatur" => 8.5,
        "Vorlauftemperatur" => 34.0,
        "Rücklauftemperatur" => 28.5,
        _ => 24.0,
    };

    let mut rows = Vec::new();
    let mut ts = start;
    let mut idx = 0_i64;
    while ts <= end {
        let wave = ((idx as f64) * 0.35).sin() * 2.5;
        let val = if channel == "Heizen" {
            if idx % 6 < 4 { 1.0 } else { 0.0 }
        } else {
            ((base + wave) * 10.0).round() / 10.0
        };
        rows.push(DataResponse {
            bucket: Some(ts),
            val: Some(val),
        });
        ts += chrono::Duration::seconds(effective_step);
        idx += 1;
    }
    rows
}

async fn get_channels(State(state): State<AppState>) -> Result<Json<Vec<Channel>>, StatusCode> {
    {
        let guard = state.channels_cache.read().await;
        if let Some((cached_at, ref rows)) = *guard {
            if cached_at.elapsed() >= CHANNELS_CACHE_TTL
                && !state.refreshing_channels.swap(true, Ordering::AcqRel)
            {
                let bg_state = state.clone();
                tokio::spawn(async move {
                    if let Err(e) = refresh_channels_cache(&bg_state).await {
                        error!("Background channel cache refresh failed: {}", e);
                    }
                    bg_state.refreshing_channels.store(false, Ordering::Release);
                });
            }
            return Ok(Json(rows.clone()));
        }
    }

    let rows = refresh_channels_cache(&state).await.map_err(|e| {
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

    let Some(db) = &state.db else {
        return Ok(Json(mock_data(
            &params.channel,
            start_dt.with_timezone(&Utc),
            end_dt.with_timezone(&Utc),
            &params.granularity,
        )));
    };

    // Handle granularity formatting slightly if needed, e.g. "1 hour", "15 minutes"
    let rows = sqlx::query_as::<_, DataResponse>(
        r#"
        SELECT time_bucket(CAST($4 AS interval), time) AS "bucket",
               AVG(COALESCE(val_float, val_int::float8, val_bool::int::float8)) AS "val"
        FROM heatpump_data
        WHERE channel = $1 AND time >= $2 AND time <= $3
        GROUP BY bucket
        ORDER BY bucket
        "#
    )
    .bind(&params.channel)
    .bind(start_dt.with_timezone(&Utc))
    .bind(end_dt.with_timezone(&Utc))
    .bind(&params.granularity)
    .fetch_all(db)
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
