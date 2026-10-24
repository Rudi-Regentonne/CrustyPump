use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use chrono::{DateTime, Utc};
use log::{error, info};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::net::SocketAddr;
use tower_http::services::ServeDir;

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

#[derive(Serialize, sqlx::FromRow)]
struct DbStatus {
    updated_at: DateTime<Utc>,
    userlevel: i32,
    has_error: bool,
    error_code: Option<i32>,
    error_name: Option<String>,
    error_description: Option<String>,
    error_date: Option<String>,
    notifications: Option<serde_json::Value>,
}

#[derive(Serialize, sqlx::FromRow)]
struct SensorValRow {
    channel: String,
    val: Option<f64>,
}

#[derive(Serialize)]
struct StatusResponse {
    online: bool,
    updated_at: Option<DateTime<Utc>>,
    userlevel: i32,
    technician_active: bool,
    has_error: bool,
    error_code: Option<i32>,
    error_name: Option<String>,
    error_description: Option<String>,
    error_date: Option<String>,
    notifications: serde_json::Value,
    sensors: std::collections::HashMap<String, Option<f64>>,
}

#[derive(Serialize, sqlx::FromRow)]
struct DbErrorLog {
    time: DateTime<Utc>,
    code: String,
    message: String,
    is_ack: bool,
}

#[derive(Serialize)]
struct CategoryCount {
    category: String,
    count: usize,
}

#[derive(Serialize)]
struct ErrorLogHistoryResponse {
    total_count: usize,
    summary: Vec<CategoryCount>,
    history: Vec<ErrorLogEntryView>,
}

#[derive(Serialize)]
struct ErrorLogEntryView {
    datetime: String,
    code: String,
    message: String,
    is_ack: bool,
}

fn categorize_error(code: &str) -> String {
    if code == "999" {
        "Quittierung (Lokale Störungen)".to_string()
    } else if code == "20999" {
        "Quittierung (myiDM Cloud-Störungen)".to_string()
    } else if code == "997" {
        "System Neustart (Power ON)".to_string()
    } else if code == "044" {
        "Durchflusswächter 1 angesprochen".to_string()
    } else if code == "20017" || code == "20013" || code == "20016" {
        "myiDM Cloud-Verbindungsfehler".to_string()
    } else if code == "615" {
        "Fühler Differenztemperaturregelung Unterbrechung".to_string()
    } else if code == "036" {
        "E-Heizstab Überhitzung (Manueller Reset)".to_string()
    } else if code == "101" {
        "Außentemperatur Unterbrechung".to_string()
    } else if code == "516" {
        "Modbus Kommunikationsfehler".to_string()
    } else if code == "620" {
        "Systemtrennung nicht konfiguriert".to_string()
    } else {
        format!("Sonstiges: Code {}", code)
    }
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
        .route("/api/status", get(get_status))
        .route("/api/ha/status", get(get_status))
        .route("/api/error_log", get(get_error_log))
        .route("/api/error_log.pdf", get(get_error_log_pdf))
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

async fn get_status(State(state): State<AppState>) -> Result<Json<StatusResponse>, StatusCode> {
    let status_row = sqlx::query_as::<_, DbStatus>(
        "SELECT updated_at, userlevel, has_error, error_code, error_name, error_description, error_date, notifications FROM heatpump_status WHERE id = 1"
    )
    .fetch_optional(&state.db)
    .await
    .unwrap_or(None);

    let sensor_rows = sqlx::query_as::<_, SensorValRow>(
        r#"
        SELECT DISTINCT ON (channel) channel,
               AVG(COALESCE(val_float, val_int::float8, val_bool::int::float8)) OVER (PARTITION BY channel) AS val
        FROM heatpump_data
        WHERE time > NOW() - INTERVAL '2 hours'
        ORDER BY channel, time DESC
        "#
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    let mut sensors = std::collections::HashMap::new();
    for row in sensor_rows {
        sensors.insert(row.channel, row.val);
    }

    if let Some(s) = status_row {
        let ulevel = s.userlevel;
        Ok(Json(StatusResponse {
            online: true,
            updated_at: Some(s.updated_at),
            userlevel: ulevel,
            technician_active: ulevel >= 2,
            has_error: s.has_error,
            error_code: s.error_code,
            error_name: s.error_name,
            error_description: s.error_description,
            error_date: s.error_date,
            notifications: s.notifications.unwrap_or(serde_json::Value::Null),
            sensors,
        }))
    } else {
        Ok(Json(StatusResponse {
            online: false,
            updated_at: None,
            userlevel: 0,
            technician_active: false,
            has_error: false,
            error_code: None,
            error_name: None,
            error_description: None,
            error_date: None,
            notifications: serde_json::Value::Null,
            sensors,
        }))
    }
}

async fn get_error_log(State(state): State<AppState>) -> Result<Json<ErrorLogHistoryResponse>, StatusCode> {
    let rows = sqlx::query_as::<_, DbErrorLog>(
        "SELECT time, code, message, is_ack FROM heatpump_error_log ORDER BY time DESC"
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        error!("DB error: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    use std::collections::BTreeMap;
    let mut cat_counts = BTreeMap::new();
    let mut history = Vec::new();

    for r in &rows {
        let cat = categorize_error(&r.code);
        *cat_counts.entry(cat).or_insert(0) += 1;

        history.push(ErrorLogEntryView {
            datetime: r.time.format("%Y-%m-%d %H:%M:%S").to_string(),
            code: r.code.clone(),
            message: r.message.clone(),
            is_ack: r.is_ack,
        });
    }

    let mut summary: Vec<CategoryCount> = cat_counts
        .into_iter()
        .map(|(category, count)| CategoryCount { category, count })
        .collect();
    summary.sort_by(|a, b| b.count.cmp(&a.count));

    let total_count = history.len();
    Ok(Json(ErrorLogHistoryResponse {
        total_count,
        summary,
        history,
    }))
}

async fn get_error_log_pdf(State(state): State<AppState>) -> Result<impl IntoResponse, (StatusCode, String)> {
    let rows = sqlx::query_as::<_, DbErrorLog>(
        "SELECT time, code, message, is_ack FROM heatpump_error_log ORDER BY time DESC"
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))?;

    use std::collections::BTreeMap;
    let mut cat_counts = BTreeMap::new();
    let mut history = Vec::new();

    for r in &rows {
        let cat = categorize_error(&r.code);
        *cat_counts.entry(cat).or_insert(0) += 1;

        history.push(ErrorLogEntryView {
            datetime: r.time.format("%Y-%m-%d %H:%M:%S").to_string(),
            code: r.code.clone(),
            message: r.message.clone(),
            is_ack: r.is_ack,
        });
    }

    let mut summary: Vec<CategoryCount> = cat_counts
        .into_iter()
        .map(|(category, count)| CategoryCount { category, count })
        .collect();
    summary.sort_by(|a, b| b.count.cmp(&a.count));

    let total_count = history.len();
    let last_updated = chrono::Local::now().format("%d.%m.%Y %H:%M Uhr").to_string();

    let json_data = serde_json::json!({
        "last_updated": last_updated,
        "total_count": total_count,
        "summary": summary,
        "history": history
    });

    let pid = std::process::id();
    let now_ts = chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0);
    let tmp_dir = std::env::temp_dir().join(format!("crustypump_pdf_{}_{}", pid, now_ts));
    std::fs::create_dir_all(&tmp_dir).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let json_path = tmp_dir.join("error_history.json");
    std::fs::write(&json_path, serde_json::to_string_pretty(&json_data).unwrap())
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let template_source = if std::path::Path::new("assets/error_log.typ").exists() {
        "assets/error_log.typ"
    } else if std::path::Path::new("dashboard/assets/error_log.typ").exists() {
        "dashboard/assets/error_log.typ"
    } else {
        "error_log.typ"
    };

    let template_dest = tmp_dir.join("error_log.typ");
    let _ = std::fs::copy(template_source, &template_dest);

    let output_pdf = tmp_dir.join("error_log.pdf");

    let compile_res = tokio::process::Command::new("typst")
        .args(["compile", template_dest.to_str().unwrap(), output_pdf.to_str().unwrap()])
        .current_dir(&tmp_dir)
        .output()
        .await;

    match compile_res {
        Ok(out) if out.status.success() => {
            let pdf_bytes = std::fs::read(&output_pdf)
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
            let _ = std::fs::remove_dir_all(&tmp_dir);

            let mut headers = axum::http::HeaderMap::new();
            headers.insert(
                axum::http::header::CONTENT_TYPE,
                axum::http::HeaderValue::from_static("application/pdf"),
            );
            headers.insert(
                axum::http::header::CONTENT_DISPOSITION,
                axum::http::HeaderValue::from_static("inline; filename=\"error_log.pdf\""),
            );

            Ok((headers, pdf_bytes))
        }
        Ok(out) => {
            let err_msg = String::from_utf8_lossy(&out.stderr).to_string();
            let _ = std::fs::remove_dir_all(&tmp_dir);
            Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Typst compile failed: {}", err_msg)))
        }
        Err(e) => {
            let _ = std::fs::remove_dir_all(&tmp_dir);
            Err((StatusCode::SERVICE_UNAVAILABLE, format!("Typst compiler not found: {}", e)))
        }
    }
}

async fn get_config(State(state): State<AppState>) -> Result<Json<serde_json::Value>, StatusCode> {
    match std::fs::read_to_string(&state.config_path) {
        Ok(content) => {
            let json: serde_json::Value = serde_json::from_str(&content).unwrap_or_else(|_| serde_json::json!({}));
            Ok(Json(json))
        }
        Err(_) => {
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
