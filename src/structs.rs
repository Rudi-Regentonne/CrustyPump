use std::{env, str::FromStr};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
#[derive(Debug, Clone)]
pub struct Config {
    pub heatpump_ip: String,
    pub heatpump_pin: String,
    pub database_url: String,
    pub cron_expression: String,
}
impl Config {
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        let heatpump_ip =
            env::var("HEATPUMP_IP").map_err(|_| "HEATPUMP_IP environment variable missing")?;
        let heatpump_pin =
            env::var("HEATPUMP_PIN").map_err(|_| "HEATPUMP_PIN environment variable missing")?;
        let database_url =
            env::var("DATABASE_URL").map_err(|_| "DATABASE_URL environment variable missing")?;
        let cron_expression = match env::var("CRON_EXPRESSION") {
            Ok(ref v) if !v.trim().is_empty() => v.clone(),
            _ => "0 * * * * *".to_string(),
        };
        Ok(Self {
            heatpump_ip,
            heatpump_pin,
            database_url,
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
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn config_from_env_integration() {
        let _guard = ENV_LOCK.lock().unwrap();
        unsafe { env::set_var("HEATPUMP_IP", "10.0.0.1") };
        unsafe { env::set_var("HEATPUMP_PIN", "0000") };
        unsafe { env::set_var("DATABASE_URL", "postgres://user:pass@localhost:5432/heatpump") };
        unsafe { env::set_var("CRON_EXPRESSION", "0 * * * * *") };

        let cfg = Config::from_env().expect("config read");
        assert_eq!(cfg.heatpump_ip, "10.0.0.1");
        assert_eq!(cfg.heatpump_pin, "0000");
        assert_eq!(cfg.database_url, "postgres://user:pass@localhost:5432/heatpump");
        assert_eq!(cfg.cron_expression, "0 * * * * *");

        unsafe { env::remove_var("HEATPUMP_IP") };
        unsafe { env::remove_var("HEATPUMP_PIN") };
        unsafe { env::remove_var("DATABASE_URL") };
        unsafe { env::remove_var("CRON_EXPRESSION") };
    }

    #[test]
    fn config_default_cron() {
        let _guard = ENV_LOCK.lock().unwrap();
        unsafe { env::set_var("HEATPUMP_IP", "10.0.0.1") };
        unsafe { env::set_var("HEATPUMP_PIN", "0000") };
        unsafe { env::set_var("DATABASE_URL", "postgres://user:pass@localhost:5432/heatpump") };

        let cfg = Config::from_env().expect("config read");
        assert_eq!(cfg.cron_expression, "0 * * * * *");

        unsafe { env::remove_var("HEATPUMP_IP") };
        unsafe { env::remove_var("HEATPUMP_PIN") };
        unsafe { env::remove_var("DATABASE_URL") };
    }

    #[test]
    fn config_empty_cron_defaults() {
        let _guard = ENV_LOCK.lock().unwrap();
        unsafe { env::set_var("HEATPUMP_IP", "10.0.0.1") };
        unsafe { env::set_var("HEATPUMP_PIN", "0000") };
        unsafe { env::set_var("DATABASE_URL", "postgres://user:pass@localhost:5432/heatpump") };
        unsafe { env::set_var("CRON_EXPRESSION", "") };

        let cfg = Config::from_env().expect("config read");
        assert_eq!(cfg.cron_expression, "0 * * * * *");

        unsafe { env::remove_var("HEATPUMP_IP") };
        unsafe { env::remove_var("HEATPUMP_PIN") };
        unsafe { env::remove_var("DATABASE_URL") };
        unsafe { env::remove_var("CRON_EXPRESSION") };
    }

    #[test]
    fn channel_type_bool_true() {
        assert!(matches!("true".parse::<ChannelTypes>().unwrap(), ChannelTypes::Bool(true)));
        assert!(matches!("True".parse::<ChannelTypes>().unwrap(), ChannelTypes::Bool(true)));
        assert!(matches!("TRUE".parse::<ChannelTypes>().unwrap(), ChannelTypes::Bool(true)));
    }

    #[test]
    fn channel_type_bool_false() {
        assert!(matches!("false".parse::<ChannelTypes>().unwrap(), ChannelTypes::Bool(false)));
        assert!(matches!("False".parse::<ChannelTypes>().unwrap(), ChannelTypes::Bool(false)));
        assert!(matches!("FALSE".parse::<ChannelTypes>().unwrap(), ChannelTypes::Bool(false)));
    }

    #[test]
    fn channel_type_int() {
        assert!(matches!("42".parse::<ChannelTypes>().unwrap(), ChannelTypes::Int(42)));
        assert!(matches!("-5".parse::<ChannelTypes>().unwrap(), ChannelTypes::Int(-5)));
        assert!(matches!("0".parse::<ChannelTypes>().unwrap(), ChannelTypes::Int(0)));
    }

    #[test]
    fn channel_type_float() {
        assert!(matches!("23.5".parse::<ChannelTypes>().unwrap(), ChannelTypes::Float(f) if (f - 23.5).abs() < 1e-10));
        assert!(matches!("-1.5".parse::<ChannelTypes>().unwrap(), ChannelTypes::Float(f) if (f + 1.5).abs() < 1e-10));
    }

    #[test]
    fn channel_type_string_fallback() {
        assert!(matches!("hello".parse::<ChannelTypes>().unwrap(), ChannelTypes::Str(ref s) if s == "hello"));
        assert!(matches!("".parse::<ChannelTypes>().unwrap(), ChannelTypes::Str(ref s) if s.is_empty()));
        assert!(matches!("  spaced  ".parse::<ChannelTypes>().unwrap(), ChannelTypes::Str(ref s) if s == "spaced"));
    }

    #[test]
    fn channel_type_int_precedence_over_float() {
        assert!(matches!("42".parse::<ChannelTypes>().unwrap(), ChannelTypes::Int(42)));
    }
}
