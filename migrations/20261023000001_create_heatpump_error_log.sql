-- Error history log (from loghist when Fachmann code is active)
CREATE TABLE IF NOT EXISTS heatpump_error_log (
    time TIMESTAMPTZ NOT NULL,
    code TEXT NOT NULL,
    message TEXT NOT NULL,
    is_ack BOOLEAN NOT NULL DEFAULT FALSE,
    PRIMARY KEY (time, code, message)
);

CREATE INDEX IF NOT EXISTS idx_heatpump_error_log_time
    ON heatpump_error_log (time DESC);

-- Latest live status and active notifications (for Home Assistant & dashboard banner)
CREATE TABLE IF NOT EXISTS heatpump_status (
    id INT PRIMARY KEY DEFAULT 1,
    updated_at TIMESTAMPTZ NOT NULL,
    userlevel INT NOT NULL DEFAULT 0,
    has_error BOOLEAN NOT NULL DEFAULT FALSE,
    error_code INT,
    error_name TEXT,
    error_description TEXT,
    error_date TEXT,
    notifications JSONB,
    CONSTRAINT single_row CHECK (id = 1)
);
