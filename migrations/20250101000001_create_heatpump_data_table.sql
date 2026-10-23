CREATE TABLE IF NOT EXISTS heatpump_data (
    time         TIMESTAMPTZ       NOT NULL,
    measurement  TEXT              NOT NULL,
    channel      TEXT              NOT NULL,
    channel_type TEXT              NOT NULL,
    val_float    DOUBLE PRECISION,
    val_int      BIGINT,
    val_bool     BOOLEAN,
    val_string   TEXT
);

SELECT create_hypertable('heatpump_data', 'time', if_not_exists => true);

CREATE INDEX IF NOT EXISTS idx_heatpump_data_lookup
    ON heatpump_data (measurement, channel, time DESC);
