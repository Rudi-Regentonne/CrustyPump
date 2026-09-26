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

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_html_table() -> &'static str {
        r#"<table>
            <tr><td>1</td><td>Aussentemperatur</td><td>23.5</td><td>°C</td></tr>
            <tr><td>2</td><td>Kompressor</td><td>1</td><td></td></tr>
            <tr><td>3</td><td>Status</td><td>Standby</td><td></td></tr>
        </table>"#
    }

    #[test]
    fn parse_sensor_float_value() {
        let result = parse_sensor(sample_html_table()).unwrap();
        match result.get("Aussentemperatur") {
            Some(ChannelTypes::Float(f)) => assert!((*f - 23.5).abs() < 1e-10),
            other => panic!("expected Float, got {:?}", other),
        }
    }

    #[test]
    fn parse_sensor_int_value() {
        let result = parse_sensor(sample_html_table()).unwrap();
        assert!(matches!(result.get("Kompressor"), Some(ChannelTypes::Int(1))));
    }

    #[test]
    fn parse_sensor_string_value() {
        let result = parse_sensor(sample_html_table()).unwrap();
        assert!(matches!(result.get("Status"), Some(ChannelTypes::Str(s)) if s == "Standby"));
    }

    #[test]
    fn parse_sensor_empty_table() {
        let result = parse_sensor("<table></table>").unwrap();
        assert!(result.is_empty());
    }

    fn sample_settings_json() -> String {
        r#"[
            {
                "edesc": "_INFORMATIONS",
                "name": "Informationen",
                "value": null,
                "items": [
                    {
                        "edesc": "_INPUTS_OUTPUTS_INFO",
                        "name": "Ein- und Ausgänge",
                        "value": null,
                        "items": [
                            {
                                "edesc": null,
                                "name": "Sensorwerte",
                                "value": "<table><tr><td>1</td><td>Temperatur</td><td>25.0</td><td>°C</td></tr></table>",
                                "items": null
                            }
                        ]
                    }
                ]
            }
        ]"#.to_string()
    }

    #[test]
    fn get_channels_parses_settings() {
        let result = get_channels(&sample_settings_json()).unwrap();
        let sensor = result.get("Sensorwerte").unwrap();
        match sensor.get("Temperatur") {
            Some(ChannelTypes::Float(f)) => assert!((*f - 25.0).abs() < 1e-10),
            other => panic!("expected Float, got {:?}", other),
        }
    }

    #[test]
    fn get_channels_invalid_json_errors() {
        assert!(get_channels("not json").is_err());
    }
}
