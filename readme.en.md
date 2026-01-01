```
  ____                _         ____
 / ___|_ __ _   _ ___| |_ _   _|  _ \ _   _ _ __ ___  _ __
| |   | '__| | | / __| __| | | | |_) | | | | '_ ` _ \| '_ \
| |___| |  | |_| \__ \ |_| |_| |  __/| |_| | | | | | | |_) |
 \____|_|   \__,_|___/\__|\__, |_|    \__,_|_| |_| |_| .__/
                          |___/                      |_|

```

# CrustyPump – IDM Heat Pump Scraper

CrustyPump reads sensor data from your IDM heat pump’s web interface and writes it to InfluxDB. Two sources are used:

- Live graphs in the statistics (current channel values)
- Settings → inputs and outputs

## Quick Start (Docker)

1. Create a `.env`:

```dotenv
# IDM Heat Pump
HEATPUMP_IP=192.168.178.xx
HEATPUMP_PIN=1234

# InfluxDB
INFLUX_URL=http://localhost:8086
INFLUX_TOKEN=your_token
INFLUX_ORG=home
INFLUX_BUCKET=heatpump

# Schedule (Cron), e.g., every minute (optional)
CRON_EXPRESSION="0 * * * * *"
```

2. Run:

```bash
docker compose up -d
```

Notes:

- InfluxDB must be reachable from the container (e.g., http://<host-ip>:8086).
- Ensure valid cron syntax (e.g., `0 */5 * * * *` for every 5 minutes).
