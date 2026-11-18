CREATE INDEX IF NOT EXISTS idx_heatpump_data_channel_time
    ON heatpump_data (channel, time DESC);
