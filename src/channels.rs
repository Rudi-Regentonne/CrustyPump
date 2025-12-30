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
