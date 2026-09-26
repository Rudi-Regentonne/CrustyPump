use chrono::Local;
use dotenv::dotenv;
use log::{error, info};
use reqwest::Client as HttpClient;
use sqlx::PgPool;
use std::error::Error;
use std::sync::Mutex;
use tokio_cron_scheduler::{Job, JobScheduler};

#[cfg(unix)]
use tokio::signal::unix::{SignalKind, signal};

use crate::structs::Config;

mod channels;
mod database;
mod http_client;
mod info_structs;
mod sensor_info;
mod structs;

static AUTH_TOKEN: Mutex<Option<String>> = Mutex::new(None);

async fn get_token(
    client: &reqwest::Client,
    cfg: &structs::Config,
    force_refresh: bool,
) -> Result<String, Box<dyn Error>> {
    {
        let guard = AUTH_TOKEN.lock().unwrap();
        if !force_refresh {
            if let Some(ref token) = *guard {
                return Ok(token.clone());
            }
        }
    }

    let token = http_client::get_new_token(client, cfg).await?;
    {
        let mut guard = AUTH_TOKEN.lock().unwrap();
        *guard = Some(token.clone());
    }
    Ok(token)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let cfg = match structs::Config::from_env() {
        Ok(c) => c,
        Err(e) => {
            error!("Config Error! Details: {}", e);
            // Keep container alive to allow inspection
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(60)).await;
            }
        }
    };

    let pool = loop {
        match database::init_pool(&cfg.database_url).await {
            Ok(pool) => match database::run_migrations(&pool).await {
                Ok(()) => break pool,
                Err(e) => error!("Migration failed: {} — retrying in 2s", e),
            },
            Err(e) => error!("Database connection failed: {} — retrying in 2s", e),
        }
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    };

    info!("Started sceduler: {}", cfg.cron_expression);
    let client = HttpClient::builder().cookie_store(true).build()?;

    let mut sched = match JobScheduler::new().await {
        Ok(s) => s,
        Err(e) => {
            error!("Scheduler init failed: {}", e);
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(60)).await;
            }
        }
    };

    let cron_str = cfg.cron_expression.clone();

    let job = Job::new_async(cron_str.as_str(), move |_uuid, _l| {
        let client_clone = client.clone();
        let cfg_clone = cfg.clone();
        let pool_clone = pool.clone();
        Box::pin(async move {
            if let Err(e) = scrape(client_clone, cfg_clone, pool_clone).await {
                error!("Error: {}", e);
            }
        })
    })
    .expect("Bad Cron-Syntax!");

    if let Err(e) = sched.add(job).await {
        error!("Job add failed: {}", e);
    }
    if let Err(e) = sched.start().await {
        error!("Scheduler start failed: {}", e);
    }

    // wait for SIGTERM in containers; avoid ctrl_c which may fire immediately as PID 1
    #[cfg(unix)]
    {
        let mut sigterm = signal(SignalKind::terminate())?;
        sigterm.recv().await;
        info!("SIGTERM recieved");
    }
    #[cfg(not(unix))]
    {
        tokio::signal::ctrl_c().await?;
    }

    info!("stopping system...");
    sched.shutdown().await.unwrap();
    info!("stopped sceduler.");

    Ok(())
}
async fn scrape(client: reqwest::Client, cfg: Config, pool: PgPool) -> Result<(), Box<dyn std::error::Error>> {
    let values = http_client::get_data(&client, &cfg, "/data/channellive.php").await?;
    let channels = http_client::get_data(&client, &cfg, "/data/channels.php").await?;
    let channel_data = channels::parse_channels(&channels, &values).await?;
    let live_result = database::write_data_points(&pool, &channel_data, Local::now(), "live_data").await;

    let info_sensors = http_client::get_data(&client, &cfg, "/data/settings.php").await?;
    let info_channels = sensor_info::get_channels(&info_sensors)?;
    let mut first_err = live_result.err();
    for (field, channel) in info_channels {
        if let Err(e) = database::write_data_points(&pool, &channel, Local::now(), &field).await {
            if first_err.is_none() {
                first_err = Some(e);
            }
        }
    }
    if let Some(e) = first_err {
        return Err(e.into());
    }
    Ok(())
}
