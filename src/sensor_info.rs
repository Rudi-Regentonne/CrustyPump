use std::{collections::HashMap, error::Error};

use log::debug;

use crate::{
    info_structs::{self, MenuItem},
    structs::{ChannelTypes, ParsedChannels},
};

pub fn get_channels(json_str: &str) -> Result<HashMap<String, ParsedChannels>, Box<dyn Error>> {
    let mut channels: HashMap<String, ParsedChannels> = HashMap::new();
    let settings_items: Vec<MenuItem> = serde_json::from_str(&json_str)?;
    let sensor_infos_opt = info_structs::find_inputs_outputs_info_root(&settings_items);
    if let Some(sensor_infos) = sensor_infos_opt {
        let inputs_outputs_info_opt =
            info_structs::MenuItem::find_by_edesc(sensor_infos, "_INPUTS_OUTPUTS_INFO");

        if let Some(inputs_outputs_info) = inputs_outputs_info_opt {
            if let Some(info_channels) = &inputs_outputs_info.items {
                for channel in info_channels {
                    if let Some(channel_values) = &channel.value {
                        channels.insert(
                            channel.name.clone(),
                            parse_sensor(&channel_values.to_string()).unwrap(),
                        );
                    }
                }
            }
        }
    }
    Ok(channels)
}
fn parse_sensor(sensors: &str) -> Result<ParsedChannels, Box<dyn Error>> {
    let mut channels: ParsedChannels = HashMap::new();

    let rows = info_structs::parse_table(&sensors);
    for row in rows {
        debug!("{}: {} {}", row.name, row.value, row.unit);
        channels.insert(row.name, row.value.parse::<ChannelTypes>().unwrap());
    }

    Ok(channels)
}
