```
  ____                _         ____
 / ___|_ __ _   _ ___| |_ _   _|  _ \ _   _ _ __ ___  _ __
| |   | '__| | | / __| __| | | | |_) | | | | '_ ` _ \| '_ \
| |___| |  | |_| \__ \ |_| |_| |  __/| |_| | | | | | | |_) |
 \____|_|   \__,_|___/\__|\__, |_|    \__,_|_| | |_| |_| .__/
                          |___/                      |_|

```

# CrustyPump – IDM Heat Pump Scraper

CrustyPump liest Sensordaten aus der Weboberfläche deiner IDM‑Wärmepumpe und schreibt sie in TimescaleDB. Dabei werden zwei Quellen genutzt:

- Live‑Graphen der Statistiken (aktuelle Kanalwerte)
- Einstellungen → Ein und Ausgänge

## Quick Start

1. Secrets anlegen:

```bash
cp env.example .env
```

In `.env` nur diese Werte setzen: `HEATPUMP_IP`, `HEATPUMP_PIN`, `DB_PASSWORD`. `CRON_EXPRESSION` ist optional. Keine Leerzeichen um `=`.

2. Starten:

```bash
docker compose up -d --build
```

Mit Podman: `podman-compose up -d --build`.

Compose setzt `DATABASE_URL` selbst. Das TimescaleDB-Schema (Hypertable + Index) legt der Scraper beim Start an.

3. Dashboard unter http://localhost:8050 öffnen:

```
┌────────────────────────────────────────────────────┐
│ CrustyPump Dashboard                               │
│                                                    │
│ Kanäle aus der Seitenleiste wählen → interaktive   │
│ Chart.js-Graphen mit Zoom und Zeitbereich.         │
│ Kacheln per Drag&Drop oder ↑↓ sortieren.           │
│ Layouts speichern/laden. Aktuelle Ansicht als PDF. │
└────────────────────────────────────────────────────┘
```

Hinweise:

- Der `crustypump` Container läuft mit `network_mode: host`, um die Wärmepumpe im LAN zu erreichen. Port 5432 muss auf dem Host frei sein.
- Cron hat sechs Felder, Sekunden zuerst (z. B. `0 */5 * * * *` für alle 5 Minuten).
