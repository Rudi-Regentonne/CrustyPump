```
  ____                _         ____
 / ___|_ __ _   _ ___| |_ _   _|  _ \ _   _ _ __ ___  _ __
| |   | '__| | | / __| __| | | | |_) | | | | '_ ` _ \| '_ \
| |___| |  | |_| \__ \ |_| |_| |  __/| |_| | | | | | | |_) |
 \____|_|   \__,_|___/\__|\__, |_|    \__,_|_| |_| |_| .__/
                          |___/                      |_|

```

# CrustyPump – IDM Heat Pump Scraper

CrustyPump liest Sensordaten aus der Weboberfläche deiner IDM‑Wärmepumpe und schreibt sie in InfluxDB. Dabei werden zwei Quellen genutzt:

- Live‑Graphen der Statistiken (aktuelle Kanalwerte)
- Einstellungen → Ein und Ausgänge

## Quick Start (Docker)

1. `.env` anlegen:

```dotenv
# IDM Heat Pump
HEATPUMP_IP=IP:80
#Der Pin vom Webinterface (keine Sorge das Web interface ist viel beschissener als mein code)
HEATPUMP_PIN=1234

# InfluxDB
INFLUX_URL=http://localhost:8086
INFLUX_TOKEN=your_token
INFLUX_ORG=home
INFLUX_BUCKET=heatpump

# Zeitplan (Cron), z. B. jede Minute (Optional)
CRON_EXPRESSION="0 * * * * *"
```

2. Starten:

```bash
docker compose up -d
```

Hinweise:

- InfluxDB muss vom Container aus erreichbar sein (z. B. http://<host-ip>:8086).
- Cron‑Syntax korrekt setzen (z. B. `0 */5 * * * *` für alle 5 Minuten).
