use chrono::{DateTime, Local, Utc};
use log::error;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

use crate::info_structs::{ErrorLogEntry, InfoResponse};
use crate::structs::{ChannelTypes, ParsedChannels};

pub async fn init_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await
}

pub async fn run_migrations(pool: &PgPool) -> Result<(), sqlx::migrate::MigrateError> {
    sqlx::migrate!("./migrations").run(pool).await
}

pub async fn write_data_points(
    pool: &PgPool,
    values: &ParsedChannels,
    datetime: DateTime<Local>,
    measurement: &str,
) -> Result<(), sqlx::Error> {
    let ts = datetime.with_timezone(&Utc);
    let mut first_err = None;

    for (name, channel_type) in values.iter() {
        let (channel_type_str, val_float, val_int, val_bool, val_string) = match channel_type {
            ChannelTypes::Float(f) => ("float", Some(*f), None::<i64>, None::<bool>, None::<String>),
            ChannelTypes::Int(i) => ("int", None::<f64>, Some(*i), None::<bool>, None::<String>),
            ChannelTypes::Bool(b) => ("bool", None::<f64>, None::<i64>, Some(*b), None::<String>),
            ChannelTypes::Str(s) => {
                ("string", None::<f64>, None::<i64>, None::<bool>, Some(s.clone()))
            }
        };

        let result = sqlx::query(
            "INSERT INTO heatpump_data (time, measurement, channel, channel_type, val_float, val_int, val_bool, val_string) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
        )
        .bind(ts)
        .bind(measurement)
        .bind(name)
        .bind(channel_type_str)
        .bind(val_float)
        .bind(val_int)
        .bind(val_bool)
        .bind(val_string)
        .execute(pool)
        .await;

        if let Err(e) = result {
            error!("Error writing data point {}: {}", name, e);
            if first_err.is_none() {
                first_err = Some(e);
            }
        }
    }
    match first_err {
        Some(e) => Err(e),
        None => Ok(()),
    }
}

pub async fn insert_error_logs(
    pool: &PgPool,
    entries: &[ErrorLogEntry],
) -> Result<usize, sqlx::Error> {
    let mut inserted = 0;
    for entry in entries {
        let res = sqlx::query(
            "INSERT INTO heatpump_error_log (time, code, message, is_ack) VALUES ($1, $2, $3, $4) ON CONFLICT (time, code, message) DO NOTHING",
        )
        .bind(entry.datetime)
        .bind(&entry.code)
        .bind(&entry.message)
        .bind(entry.is_ack)
        .execute(pool)
        .await?;
        if res.rows_affected() > 0 {
            inserted += 1;
        }
    }
    Ok(inserted)
}

pub async fn update_status(pool: &PgPool, info: &InfoResponse) -> Result<(), sqlx::Error> {
    let has_error = info.notiftype.as_deref() == Some("error")
        || info
            .notifications
            .as_ref()
            .map(|n| !n.is_empty())
            .unwrap_or(false);
    let first_notif = info.notifications.as_ref().and_then(|n| n.first());
    let error_code = first_notif.and_then(|n| n.code).map(|c| c as i32);
    let error_name = first_notif.and_then(|n| n.name.clone());
    let error_description = first_notif.and_then(|n| n.description.clone());
    let error_date = first_notif.and_then(|n| n.date.clone());
    let notifications_json =
        serde_json::to_value(&info.notifications).unwrap_or(serde_json::Value::Null);
    let userlevel = info.userlevel.unwrap_or(0) as i32;

    sqlx::query(
        r#"
        INSERT INTO heatpump_status (id, updated_at, userlevel, has_error, error_code, error_name, error_description, error_date, notifications)
        VALUES (1, NOW(), $1, $2, $3, $4, $5, $6, $7)
        ON CONFLICT (id) DO UPDATE SET
            updated_at = EXCLUDED.updated_at,
            userlevel = EXCLUDED.userlevel,
            has_error = EXCLUDED.has_error,
            error_code = EXCLUDED.error_code,
            error_name = EXCLUDED.error_name,
            error_description = EXCLUDED.error_description,
            error_date = EXCLUDED.error_date,
            notifications = EXCLUDED.notifications
        "#,
    )
    .bind(userlevel)
    .bind(has_error)
    .bind(error_code)
    .bind(error_name)
    .bind(error_description)
    .bind(error_date)
    .bind(notifications_json)
    .execute(pool)
    .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn channel_type_to_column_mapping_float() {
        let val = ChannelTypes::Float(23.5);
        match val {
            ChannelTypes::Float(f) => {
                assert_eq!(("float", Some(f), None::<i64>, None::<bool>, None::<String>), ("float", Some(23.5), None::<i64>, None::<bool>, None::<String>));
            }
            _ => panic!("expected Float"),
        }
    }

    #[test]
    fn channel_type_to_column_mapping_int() {
        let val = ChannelTypes::Int(42);
        match val {
            ChannelTypes::Int(i) => {
                assert_eq!(("int", None::<f64>, Some(i), None::<bool>, None::<String>), ("int", None::<f64>, Some(42), None::<bool>, None::<String>));
            }
            _ => panic!("expected Int"),
        }
    }

    #[test]
    fn channel_type_to_column_mapping_bool() {
        let val = ChannelTypes::Bool(true);
        match val {
            ChannelTypes::Bool(b) => {
                assert_eq!(("bool", None::<f64>, None::<i64>, Some(b), None::<String>), ("bool", None::<f64>, None::<i64>, Some(true), None::<String>));
            }
            _ => panic!("expected Bool"),
        }
    }

    #[test]
    fn channel_type_to_column_mapping_string() {
        let val = ChannelTypes::Str("hello".to_string());
        match val {
            ChannelTypes::Str(ref s) => {
                assert_eq!(("string", None::<f64>, None::<i64>, None::<bool>, Some(s.clone())), ("string", None::<f64>, None::<i64>, None::<bool>, Some("hello".to_string())));
            }
            _ => panic!("expected Str"),
        }
    }
}
