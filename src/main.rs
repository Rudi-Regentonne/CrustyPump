use std::error::Error;

use chrono::Local;
use dotenv::dotenv;
use log::debug;
use reqwest::Client as HttpClient;
use tokio::sync::OnceCell;

use crate::structs::Config;

mod channels;
mod http_client;
mod influx;
mod info_structs;
mod sensor_info;
mod structs;
static AUTH_TOKEN: OnceCell<String> = OnceCell::const_new();

async fn get_token(
    client: &reqwest::Client,
    cfg: &structs::Config,
) -> Result<String, Box<dyn Error>> {
    if let Some(t) = AUTH_TOKEN.get() {
        return Ok(t.clone());
    }

    let token = match http_client::get_new_token(client, cfg).await {
        Ok(it) => it,
        Err(err) => return Err(err),
    };

    let _ = AUTH_TOKEN.set(token.clone());
    Ok(token)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let cfg = structs::Config::from_env().expect("missing configuration");
    env_logger::init();
    let client = HttpClient::builder().cookie_store(true).build()?;
    _ = scrape(client, cfg);
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
