use crate::structs::{self, ChannelTypes, ParsedChannels};
use chrono::{DateTime, Local, Utc};
use futures::stream;
use influxdb2::{Client, models::DataPoint};
use log::{debug, error};

pub async fn write_to_influx(
    cfg: &structs::Config,
    values: ParsedChannels,
    datetime: DateTime<Local>,
    measurement: &str,
) {
    let client = Client::new(
        cfg.influx_url.clone(),
        cfg.influx_org.clone(),
        cfg.influx_token.clone(),
    );
    let ts = datetime.with_timezone(&Utc).timestamp_nanos_opt().unwrap();

    let points: Vec<DataPoint> = values
        .iter()
        .filter_map(|(name, channel_type_value)| {
            let builder = match channel_type_value {
                ChannelTypes::Float(val) => DataPoint::builder(measurement).field(name, *val),
                ChannelTypes::Int(val) => DataPoint::builder(measurement).field(name, *val),
                ChannelTypes::Bool(val) => DataPoint::builder(measurement).field(name, *val),
                ChannelTypes::Str(val) => DataPoint::builder(measurement).field(name, val.clone()),
            };
            debug!("{}:{:?}", name, channel_type_value);

            match builder.timestamp(ts).build() {
                Ok(p) => Some(p),
                Err(e) => {
                    error!("failed to build datapoint {}: {}", name, e);
                    None
                }
            }
        })
        .collect();

    let s = stream::iter(points);

    match client.write(&cfg.influx_bucket, s).await {
        Ok(()) => {
            debug!("Influx write complete.");
        }
        Err(e) => {
            error!("Error while writing to Influx: {e}");
        }
    }
}
