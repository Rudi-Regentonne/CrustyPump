use std::env;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
#[derive(Debug, Clone)]
pub struct Config {
    pub heatpump_ip: String,
    pub heatpump_pin: String,
    pub influx_url: String,
    pub influx_org: String,
    pub influx_bucket: String,
    pub influx_token: String,
}
impl Config {
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        let heatpump_ip =
            env::var("HEATPUMP_IP").map_err(|_| "HEATPUMP_IP environment variable missing")?;
        let heatpump_pin =
            env::var("HEATPUMP_PIN").map_err(|_| "HEATPUMP_PIN environment variable missing")?;
        let influx_org =
            env::var("INFLUX_ORG").map_err(|_| "INFLUX_ORG environment variable missing")?;
        let influx_bucket =
            env::var("INFLUX_BUCKET").map_err(|_| "INFLUX_BUCKET environment variable missing")?;
        let influx_token =
            env::var("INFLUX_TOKEN").map_err(|_| "INFLUX_TOKEN environment variable missing")?;
        let influx_url =
            env::var("INFLUX_URL").map_err(|_| "INFLUX_URL environment variable missing")?;

        Ok(Self {
            heatpump_ip,
            heatpump_pin,
            influx_org,
            influx_bucket,
            influx_token,
            influx_url,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawChannel {
    #[serde(rename = "defaultColor")]
    pub default_color: String,
    #[serde(rename = "defaultWidth")]
    pub default_width: u32,
    pub group: u32,
    pub legend: bool,
    pub name: String,
    #[serde(rename = "type")]
    pub channel_type: String,
}
pub type RawChannels = HashMap<String, RawChannel>;
pub type RawChannelValues = HashMap<String, String>;
#[derive(Debug)]
pub enum ChannelTypes {
    Str(String),
    Int(i64),
    Float(f64),
    Bool(bool),
}
pub type ParsedChannels = HashMap<String, ChannelTypes>;
