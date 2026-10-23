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
│ On-Demand Fehlerprotokoll (Typst PDF Export).      │
└────────────────────────────────────────────────────┘
```

## Home Assistant Integration

CrustyPump stellt unter `http://<HOST-IP>:8050/api/ha/status` (oder `/api/status`) alle Sensordaten sowie akute Störungen und Benachrichtigungen bereit.

### Konfiguration in Home Assistant (`configuration.yaml`):

```yaml
rest:
  - resource: http://<CRUSTYPUMP-HOST>:8050/api/ha/status
    scan_interval: 30
    sensor:
      - name: "Wärmepumpe Status"
        value_template: >
          {% if value_json.has_error %}
            Störung ({{ value_json.error_code }})
          {% elif value_json.technician_active %}
            Fachmann-Modus
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
      - name: "Wärmepumpe Störung"
        value_template: "{{ value_json.has_error }}"
        device_class: problem
```

### Handy-Push-Benachrichtigung bei Fehlern:

```yaml
automation:
  - alias: "Wärmepumpe Störung Alarm"
    trigger:
      - platform: state
        entity_id: binary_sensor.waermepumpe_stoerung
        to: "on"
    action:
      - service: notify.notify
        data:
          title: "⚠️ Wärmepumpen-Störung (Code {{ state_attr('sensor.waermepumpe_status', 'error_code') }})"
          message: "{{ state_attr('sensor.waermepumpe_status', 'error_name') }}: {{ state_attr('sensor.waermepumpe_status', 'error_description') }}"
```

## Fehlerprotokoll & Fachmann-Code Logging

- **Fachmann-Code Erkennung:** Sobald am Display oder im Webinterface der Fachmann-Code eingegeben wird (`userlevel >= 2`), sendet die Wärmepumpe das Feld `loghist` mit.
- **Automatische Archivierung:** CrustyPump erkennt dies minütlich automatisch, parst alle historischen Ereignisse und speichert sie dedupliziert in TimescaleDB (`heatpump_error_log`).
- **On-Demand Typst PDF Export:** Über den Button im Dashboard oder direkt per `http://<HOST-IP>:8050/api/error_log.pdf` wird das Fehlerprotokoll live aus der Datenbank als PDF kompiliert.

Hinweise:

- Der `crustypump` Container läuft mit `network_mode: host`, um die Wärmepumpe im LAN zu erreichen. Port 5432 muss auf dem Host frei sein.
- Cron hat sechs Felder, Sekunden zuerst (z. B. `0 */5 * * * *` für alle 5 Minuten).
