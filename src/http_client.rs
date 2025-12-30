use log::{debug, info};
use regex::Regex;
use reqwest::header::{HeaderMap, HeaderValue};

use std::time::Duration;

use crate::{get_token, structs};

pub async fn get_new_token(
    client: &reqwest::Client,
    cfg: &structs::Config,
) -> Result<String, Box<dyn std::error::Error>> {
    let params = [("pin", &cfg.heatpump_pin)];
    info!("Requesting token from {}", cfg.heatpump_ip);

    let resp = client
        .post(format!("http://{}/index.php", cfg.heatpump_ip))
        .form(&params)
        .timeout(Duration::from_secs(5))
        .send()
        .await?;
    let body = resp.text().await?;

    let re = Regex::new(r#"csrf_token="([a-zA-Z0-9]+)""#)?;
    let token = re
        .captures(&body)
        .and_then(|cap| cap.get(1))
        .map(|m| m.as_str().to_owned())
        .ok_or("No CSRF-Token found!")?;

    info!("Token: {}", token);
    Ok(token)
}

pub async fn get_data(
    client: &reqwest::Client,
    cfg: &structs::Config,
    file: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut headers = HeaderMap::new();
    let token = match get_token(&client, cfg).await {
        Ok(it) => it,
        Err(err) => return Err(err),
    };

    headers.insert("CSRF-Token", HeaderValue::from_str(&token)?);
    let response = client
        .get(format!("http://{}{}", cfg.heatpump_ip, file))
        .headers(headers)
        .send()
        .await?;

    let text = response.text().await?;
    debug!("Response from {}: {}", file, text);

    Ok(text)
}
