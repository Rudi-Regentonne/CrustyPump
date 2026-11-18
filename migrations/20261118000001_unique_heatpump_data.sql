DELETE FROM heatpump_data a
USING (
    SELECT ctid,
           ROW_NUMBER() OVER (PARTITION BY time, measurement, channel ORDER BY ctid) AS rn
    FROM heatpump_data
) b
WHERE a.ctid = b.ctid AND b.rn > 1;

CREATE UNIQUE INDEX IF NOT EXISTS idx_heatpump_data_unique
    ON heatpump_data (time, measurement, channel);
