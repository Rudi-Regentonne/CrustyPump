use std::{env, str::FromStr};

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
    pub cron_expression: String,
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
        let cron_expression = match env::var("CRON_EXPRESSION") {
            Ok(ref v) if !v.trim().is_empty() => v.clone(),
            _ => "0 * * * * *".to_string(),
        };
        Ok(Self {
            heatpump_ip,
            heatpump_pin,
            influx_org,
            influx_bucket,
            influx_token,
            influx_url,
            cron_expression,
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
impl FromStr for ChannelTypes {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();

        // 1) Bool (case-insensitive)
        match s.to_ascii_lowercase().as_str() {
            "true" => return Ok(ChannelTypes::Bool(true)),
            "false" => return Ok(ChannelTypes::Bool(false)),
            _ => {}
        }

        // 2) Integer
        if let Ok(i) = s.parse::<i64>() {
            return Ok(ChannelTypes::Int(i));
        }

        // 3) Float (f64)
        if let Ok(f) = s.parse::<f64>() {
            return Ok(ChannelTypes::Float(f));
        }

        // 4) Fallback: String
        Ok(ChannelTypes::Str(s.to_string()))
    }
}
pub type ParsedChannels = HashMap<String, ChannelTypes>;

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn config_from_env_integration() {
        unsafe { env::set_var("HEATPUMP_IP", "10.0.0.1") };
        unsafe { env::set_var("HEATPUMP_PIN", "0000") };
        unsafe { env::set_var("INFLUX_ORG", "docs") };
        unsafe { env::set_var("INFLUX_BUCKET", "test") };
        unsafe { env::set_var("INFLUX_URL", "http://127.0.0.1:8086") };
        unsafe { env::set_var("INFLUX_TOKEN", "token") };
        unsafe { env::set_var("CRON_EXPRESSION", "0 * * * * *") };

        let cfg = Config::from_env().expect("config read");
        assert_eq!(cfg.heatpump_ip, "10.0.0.1");
        assert_eq!(cfg.heatpump_pin, "0000");
        assert_eq!(cfg.influx_bucket, "test");
        assert_eq!(cfg.influx_org, "docs");
        assert_eq!(cfg.influx_token, "token");
        assert_eq!(cfg.influx_url, "http://127.0.0.1:8086");
        assert_eq!(cfg.cron_expression, "0 * * * * *");

        unsafe { env::remove_var("HEATPUMP_IP") };
        unsafe { env::remove_var("HEATPUMP_PIN") };
        unsafe { env::remove_var("INFLUX_ORG") };
        unsafe { env::remove_var("INFLUX_BUCKET") };
        unsafe { env::remove_var("INFLUX_URL") };
        unsafe { env::remove_var("INFLUX_TOKEN") };
        unsafe { env::remove_var("CRON_EXPRESSION") };
    }
}
