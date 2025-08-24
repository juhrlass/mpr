# MPR - Mouse Position Reader

Ein Windows-Tray-Tool, das die aktuelle Mausposition in Echtzeit anzeigt.

## Was macht es?

MPR ist ein kleines, effizientes Tool, das im System-Tray läuft und kontinuierlich die aktuelle Mausposition überwacht. Das Tray-Icon zeigt die X- und Y-Koordinaten der Maus als grüne Zahlen auf schwarzem Hintergrund an.

## Features

- **Echtzeit-Überwachung**: Aktualisiert die Mausposition alle 100ms
- **Tray-Integration**: Läuft im Hintergrund ohne sichtbares Fenster
- **Visuelle Anzeige**: Zeigt Koordinaten direkt im Tray-Icon an
- **Rechtsklick-Menü**: Einfaches Beenden über das Kontextmenü
- **Ressourcenschonend**: Minimaler Speicherverbrauch und CPU-Last

## Verwendung

### Installation

1. Stellen Sie sicher, dass Rust installiert ist
2. Klonen Sie das Repository
3. Führen Sie `cargo build --release` aus
4. Die ausführbare Datei befindet sich in `target/release/mpr.exe`

### Bedienung

1. **Starten**: Führen Sie `mpr.exe` aus
2. **Tray-Icon**: Das Tool erscheint als kleines Icon im System-Tray
3. **Koordinaten ablesen**: Die obere Zeile zeigt X, die untere Y-Koordinate
4. **Beenden**: Rechtsklick auf das Tray-Icon → "Beenden"

### Koordinaten-Format

- **X-Koordinate**: Obere Zeile (0-9999)
- **Y-Koordinate**: Untere Zeile (0-9999)
- **Aktualisierung**: Alle 100 Millisekunden
- **Anzeige**: 4-stellige Zahlen in grüner Schrift auf schwarzem Grund

## Technische Details

- **Sprache**: Rust
- **Windows-API**: Native Win32-API über das `windows-rs` Crate
- **Icon-Größe**: 24x24 Pixel
- **Font**: Eigenes 5x7 Pixel Bitmap-Font
- **Nachrichtenverarbeitung**: Windows-Nachrichtenschleife mit Timer

## Systemanforderungen

- Windows 10 oder höher
- Keine zusätzlichen Abhängigkeiten
- Minimaler Speicherverbrauch (~1-2 MB)

## Entwicklung

Für Entwickler siehe [DEVELOPMENT.md](DEVELOPMENT.md) für Details zur Build-Konfiguration und dem Manifest-System.

## Lizenz

Dieses Projekt ist für den privaten Gebrauch bestimmt.

## Bekannte Einschränkungen

- Koordinaten werden auf 4 Stellen begrenzt (0-9999)
- Funktioniert nur unter Windows
- Keine Konfigurationsmöglichkeiten
- Keine Persistierung der Einstellungen

## Fehlerbehebung

**Tray-Icon erscheint nicht**: Überprüfen Sie, ob das Tool als Administrator läuft
**Programm startet nicht**: Stellen Sie sicher, dass alle Windows-Updates installiert sind
**Hohe CPU-Last**: Das Tool aktualisiert alle 100ms - dies ist normal

## Support

Bei Problemen oder Fragen erstellen Sie bitte ein Issue im Repository.
