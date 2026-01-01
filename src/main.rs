use chrono::Local;
use dotenv::dotenv;
use log::{debug, error, info};
use reqwest::Client as HttpClient;
use std::error::Error;
use std::sync::Mutex;
use tokio_cron_scheduler::{Job, JobScheduler};

#[cfg(unix)]
use tokio::signal::unix::{SignalKind, signal};

use crate::structs::Config;

mod channels;
mod http_client;
mod influx;
mod info_structs;
mod sensor_info;
mod structs;
//static AUTH_TOKEN: OnceCell<String> = OnceCell::const_new();

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
        Box::pin(async move {
            if let Err(e) = scrape(client_clone, cfg_clone).await {
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
async fn scrape(client: reqwest::Client, cfg: Config) -> Result<(), Box<dyn std::error::Error>> {
    let channels: String = http_client::get_data(&client, &cfg, "/data/channels.php").await?;
    debug!("{}", channels);
    let values = http_client::get_data(&client, &cfg, "/data/channellive.php").await?;
    let channel_data = channels::parse_channels(&channels, &values).await?;
    influx::write_to_influx(&cfg, channel_data, Local::now(), "live_data").await;

    let info_sensors = http_client::get_data(&client, &cfg, "/data/settings.php").await?;
    let info_channels = sensor_info::get_channels(&info_sensors)?;
    for (field, channel) in info_channels {
        influx::write_to_influx(&cfg, channel, Local::now(), &field).await;
    }
    Ok(())
}
