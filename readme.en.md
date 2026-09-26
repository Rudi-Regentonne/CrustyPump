```
  ____                _         ____
 / ___|_ __ _   _ ___| |_ _   _|  _ \ _   _ _ __ ___  _ __
| |   | '__| | | / __| __| | | | |_) | | | | '_ ` _ \| '_ \
| |___| |  | |_| \__ \ |_| |_| |  __/| |_| | | | | | | |_) |
 \____|_|   \__,_|___/\__|\__, |_|    \__,_|_| |_| |_| .__/
                          |___/                      |_|

```

# CrustyPump – IDM Heat Pump Scraper

CrustyPump reads sensor data from your IDM heat pump's web interface and writes it to TimescaleDB. Two sources are used:

- Live graphs in the statistics (current channel values)
- Settings → inputs and outputs

## Quick Start

1. Create the secrets file:

```bash
cp env.example .env
```

Set `HEATPUMP_IP`, `HEATPUMP_PIN`, and `DB_PASSWORD`. `CRON_EXPRESSION` is optional. No spaces around `=`.

2. Start:

```bash
docker compose up -d --build
```

With Podman: `podman-compose up -d --build`.

Compose sets `DATABASE_URL`. The scraper creates the TimescaleDB schema (hypertable + index) on startup.

3. Open the dashboard at http://localhost:8050:

```
┌────────────────────────────────────────────────────┐
│ CrustyPump Dashboard                               │
│                                                    │
│ Select channels from the sidebar → interactive     │
│ Chart.js graphs with zoom and a time range.        │
│ Reorder tiles via drag-and-drop or ↑↓ buttons.     │
│ Save/Load layouts. Export current view as PDF.     │
└────────────────────────────────────────────────────┘
```

Notes:

- The `crustypump` container runs with `network_mode: host` to reach the heat pump on your LAN. Port 5432 must be free on the host.
- Cron has six fields, seconds first (e.g. `0 */5 * * * *` for every 5 minutes).
