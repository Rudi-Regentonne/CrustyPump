use log::{debug, info};
use regex::Regex;
use reqwest::header::{HeaderMap, HeaderValue};

use std::{thread::sleep, time::Duration};

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

    debug!("Token: {}", token);
    Ok(token)
}

pub async fn get_data(
    client: &reqwest::Client,
    cfg: &structs::Config,
    file: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut attempts = 0;
    let retries = 3;
    let mut text;
    loop {
        let mut headers = HeaderMap::new();

        let token = match get_token(&client, cfg, attempts != 0).await {
            Ok(it) => it,
            Err(err) => return Err(err),
        };

        headers.insert("CSRF-Token", HeaderValue::from_str(&token)?);
        let response = client
            .get(format!("http://{}{}", cfg.heatpump_ip, file))
            .headers(headers)
            .send()
            .await?;

        text = response.text().await?;
        debug!("Response from {}: {}", file, text);
        if text.contains("invalid csrf token") {
            attempts += 1;
            info!("CSRF-Token invalid, retrying (attempt {})...", attempts);
            if attempts >= retries {
                return Err("CSRF-Token invalid. Stopped trying".into());
            }
            sleep(Duration::from_secs(1));
            continue;
        }
        return Ok(text);
    }
}

#[cfg(test)]
mod tests {
    use httpmock::{Method::POST, MockServer};

    use crate::{http_client, structs};

    #[tokio::test]
    async fn get_new_token_works_against_mock() {
        // Start mock server
        let server = MockServer::start_async().await;

        // Stub the login endpoint
        let _m = server
            .mock_async(|when, then| {
                when.method(POST).path("/index.php");
                then.status(200).body(r#"csrf_token="TOKEN123""#);
            })
            .await;

        // Build a Config that points to mock server's host:port (match your function's expectation)
        let cfg = structs::Config {
            heatpump_ip: server.address().to_string(),
            heatpump_pin: "123".into(),
            influx_org: "".into(),
            influx_bucket: "".into(),
            influx_token: "".into(),
            influx_url: "".into(),
            cron_expression: "".into(),
        };

        let client = reqwest::Client::builder()
            .cookie_store(true)
            .build()
            .unwrap();
        let token = http_client::get_new_token(&client, &cfg).await.unwrap();
        assert_eq!(token, "TOKEN123");
    }
}
