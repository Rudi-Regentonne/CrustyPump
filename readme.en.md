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
│ On-Demand Error Log Report (Typst PDF Export).     │
└────────────────────────────────────────────────────┘
```

## Home Assistant Integration

CrustyPump provides all sensor values, acute error states, and notifications via `http://<HOST-IP>:8050/api/ha/status` (or `/api/status`).

### Configuration in Home Assistant (`configuration.yaml`):

```yaml
rest:
  - resource: http://<CRUSTYPUMP-HOST>:8050/api/ha/status
    scan_interval: 30
    sensor:
      - name: "Heat Pump Status"
        value_template: >
          {% if value_json.has_error %}
            Error ({{ value_json.error_code }})
          {% elif value_json.technician_active %}
            Technician Mode
          {% else %}
            Normal
          {% endif %}
        json_attributes:
          - has_error
          - error_code
          - error_name
          - error_description
          - error_date
          - userlevel
          - technician_active
    binary_sensor:
      - name: "Heat Pump Problem"
        value_template: "{{ value_json.has_error }}"
        device_class: problem
```

### Mobile Push Notifications on Errors:

```yaml
automation:
  - alias: "Heat Pump Error Alert"
    trigger:
      - platform: state
        entity_id: binary_sensor.heat_pump_problem
        to: "on"
    action:
      - service: notify.notify
        data:
          title: "⚠️ Heat Pump Fault (Code {{ state_attr('sensor.heat_pump_status', 'error_code') }})"
          message: "{{ state_attr('sensor.heat_pump_status', 'error_name') }}: {{ state_attr('sensor.heat_pump_status', 'error_description') }}"
```

## Error Log & Technician Code Logging

- **Technician Code Detection:** When the technician code is entered (`userlevel >= 2`), the heat pump transmits the `loghist` field.
- **Automatic Archival:** CrustyPump detects this every minute, parses historical event rows, and writes them deduplicated into TimescaleDB (`heatpump_error_log`).
- **On-Demand Typst PDF Export:** Click "Fehlerprotokoll" in the dashboard or open `http://<HOST-IP>:8050/api/error_log.pdf` to compile a PDF report on the fly from the database.

Notes:

- The `crustypump` container runs with `network_mode: host` to reach the heat pump on your LAN. Port 5432 must be free on the host.
- Cron has six fields, seconds first (e.g. `0 */5 * * * *` for every 5 minutes).
