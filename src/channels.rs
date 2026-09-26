use std::{collections::HashMap, error::Error};

use crate::structs::{self, ChannelTypes, ParsedChannels};

pub async fn parse_channels(
    channels_str: &str,
    channel_values_str: &str,
) -> Result<ParsedChannels, Box<dyn Error>> {
    let raw_channels: structs::RawChannels = serde_json::from_str(&channels_str)?;

    let channel_values: structs::RawChannelValues = serde_json::from_str(&channel_values_str)?;
    let mut channels: ParsedChannels = HashMap::new();
    for (id, channel_value) in channel_values.iter() {
        if let Some(channel) = raw_channels.get(id) {
            let value = match channel.channel_type.as_str() {
                "binary" => {
                    let num_value = channel_value.parse::<i64>().map_err(|e| {
                        format!("Error while parsing binary data {}: {}", channel_value, e)
                    })?;
                    ChannelTypes::Bool(num_value != 0)
                }
                "line" => ChannelTypes::Float(channel_value.parse::<f64>()?),
                "step" => ChannelTypes::Int(channel_value.parse::<i64>()?),
                _ => ChannelTypes::Str(channel_value.clone()),
            };

            channels.insert(channel.name.clone(), value);
        };
    }

    Ok(channels)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_channels_json() -> &'static str {
        r##"{
            "1": {"defaultColor":"#FF0000","defaultWidth":2,"group":1,"legend":true,"name":"Aussentemperatur","type":"line"},
            "2": {"defaultColor":"#00FF00","defaultWidth":2,"group":1,"legend":true,"name":"Kompressor","type":"binary"},
            "3": {"defaultColor":"#0000FF","defaultWidth":2,"group":1,"legend":false,"name":"Schritte","type":"step"},
            "4": {"defaultColor":"#FFFF00","defaultWidth":2,"group":1,"legend":true,"name":"Status","type":"other"}
        }"##
    }

    fn sample_values_json() -> &'static str {
        r##"{"1":"23.5","2":"1","3":"42","4":"läuft"}"##
    }

    #[tokio::test]
    async fn parse_channels_line_type() {
        let result = parse_channels(sample_channels_json(), sample_values_json()).await.unwrap();
        match result.get("Aussentemperatur") {
            Some(ChannelTypes::Float(f)) => assert!((*f - 23.5).abs() < 1e-10),
            other => panic!("expected Float, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn parse_channels_binary_type() {
        let result = parse_channels(sample_channels_json(), sample_values_json()).await.unwrap();
        assert!(matches!(result.get("Kompressor"), Some(ChannelTypes::Bool(true))));
    }

    #[tokio::test]
    async fn parse_channels_step_type() {
        let result = parse_channels(sample_channels_json(), sample_values_json()).await.unwrap();
        assert!(matches!(result.get("Schritte"), Some(ChannelTypes::Int(42))));
    }

    #[tokio::test]
    async fn parse_channels_unknown_type() {
        let result = parse_channels(sample_channels_json(), sample_values_json()).await.unwrap();
        assert!(matches!(result.get("Status"), Some(ChannelTypes::Str(s)) if s == "läuft"));
    }

    #[tokio::test]
    async fn parse_channels_missing_channel_skipped() {
        let result = parse_channels(sample_channels_json(), r#"{"99":"nothing"}"#).await.unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn parse_channels_invalid_json_errors() {
        assert!(parse_channels("not json", "{}").await.is_err());
    }
}
