# Entwicklung - MPR

Diese Datei erklärt die technischen Aspekte der Entwicklung und Build-Konfiguration von MPR.

## Projektstruktur

```
mpr/
├── src/
│   └── main.rs          # Hauptquellcode
├── build.rs             # Build-Skript für Manifest-Einbettung
├── Cargo.toml           # Rust-Projekt-Konfiguration
├── Cargo.lock           # Abhängigkeits-Versionssperre
└── Contoso.Sample.manifest  # Windows-Manifest-Datei
```

## Build-System

### Cargo.toml

Die `Cargo.toml` definiert das Rust-Projekt und seine Abhängigkeiten:

```toml
[dependencies]
windows = { version = "0.58", features = [
    "Win32_UI_WindowsAndMessaging",  # Fenster- und Nachrichtenbehandlung
    "Win32_UI_Shell",                # Shell-Funktionen (Tray-Icons)
    "Win32_Graphics_Gdi",            # Graphics Device Interface
    "Win32_Foundation",              # Grundlegende Windows-Typen
    "Win32_System_LibraryLoader",    # Modul-Handling
    "Win32_UI_Controls"              # Steuerelemente (Menüs)
] }

[build-dependencies]
embed-manifest = "1.4"               # Manifest-Einbettung
```

### Windows-API Features

Das Projekt verwendet spezifische Windows-API-Module:

- **Foundation**: Grundlegende Typen wie `HICON`, `HWND`, `POINT`
- **Graphics_Gdi**: Icon-Erstellung und Bitmap-Manipulation
- **UI_Shell**: Tray-Icon-Verwaltung (`Shell_NotifyIconW`)
- **UI_WindowsAndMessaging**: Fenster-Nachrichten und Timer
- **UI_Controls**: Popup-Menüs und Steuerelemente

## Build-Skript (build.rs)

### Zweck

Die `build.rs` Datei wird während des Build-Prozesses ausgeführt und dient zur:

1. **Manifest-Einbettung**: Windows-Manifest in die ausführbare Datei einbetten
2. **Build-Zeit-Konfiguration**: Plattformspezifische Einstellungen
3. **Ressourcen-Verwaltung**: Einbettung von Windows-spezifischen Ressourcen

### Funktionsweise

```rust
fn main() {
    // Manifest in die ausführbare Datei einbetten
    embed_manifest::embed_manifest_file("mpr.manifest");
    
    // Build-Informationen an Cargo weitergeben
    println!("cargo:rerun-if-changed=Contoso.Sample.manifest");
}
```

### Vorteile der Manifest-Einbettung

- **UAC-Kompatibilität**: Richtige Benutzerkontensteuerung
- **Windows-Integration**: Bessere System-Integration
- **Vista+-Kompatibilität**: Unterstützung moderner Windows-Versionen
- **Keine externen Dateien**: Manifest ist in der .exe eingebettet

## Windows-Manifest (Contoso.Sample.manifest)

### Was ist ein Manifest?

Ein Windows-Manifest ist eine XML-Datei, die Metadaten über eine Anwendung enthält:

- **Compatibility**: Windows-Version-Kompatibilität
- **UAC**: Benutzerkontensteuerung-Einstellungen
- **Dependencies**: Benötigte System-Bibliotheken
- **Visual Styles**: Erscheinungsbild und Themes

### Manifest-Inhalt

```xml
<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
  <assemblyIdentity version="1.0.0.0" processorArchitecture="*" name="Contoso.Sample" type="win32"/>
  <description>Sample Application</description>
  <dependency>
    <dependentAssembly>
      <assemblyIdentity type="win32" name="Microsoft.Windows.Common-Controls" version="6.0.0.0" processorArchitecture="*" publicKeyToken="6595b64144ccf1df" language="*"/>
    </dependentAssembly>
  </dependency>
</assembly>
```

### Warum wird es benötigt?

1. **Common Controls**: Moderne Steuerelemente (Menüs, Buttons)
2. **Visual Styles**: Windows 10/11 Erscheinungsbild
3. **DPI-Awareness**: Korrekte Darstellung auf hochauflösenden Bildschirmen
4. **UAC**: Richtige Berechtigungsanfragen

## Build-Prozess

### Kompilierung

```bash
# Debug-Build
cargo build

# Release-Build (optimiert)
cargo build --release

# Nur kompilieren ohne Ausführung
cargo check
```

### Build-Ausgabe

- **Debug**: `target/debug/mpr.exe` (größer, langsamer, Debug-Informationen)
- **Release**: `target/release/mpr.exe` (kleiner, schneller, optimiert)

### Abhängigkeiten

Das Projekt benötigt:

1. **Rust Toolchain**: Rust 1.70+ mit Cargo
2. **Windows SDK**: Für Windows-API-Header
3. **Build-Tools**: Visual Studio Build Tools (optional)

## Entwicklungsumgebung

### Empfohlene Tools

- **IDE**: Visual Studio Code mit Rust-Erweiterung
- **Debugger**: Windows Debugger oder Visual Studio
- **Build-Tools**: Cargo (kommt mit Rust)

### Debugging

```bash
# Debug-Version mit Logging
cargo run

# Release-Version testen
cargo run --release

# Spezifische Cargo-Features
cargo run --features debug
```

### Häufige Build-Probleme

1. **Windows-API nicht gefunden**: Überprüfen Sie die Features in `Cargo.toml`
2. **Manifest-Fehler**: Stellen Sie sicher, dass `build.rs` korrekt konfiguriert ist
3. **Linker-Fehler**: Überprüfen Sie die Windows SDK-Installation

## Erweiterungen

### Neue Features hinzufügen

1. **Neue Windows-API**: Feature in `Cargo.toml` hinzufügen
2. **Zusätzliche Abhängigkeiten**: In `[dependencies]` eintragen
3. **Build-Skript erweitern**: `build.rs` für neue Ressourcen anpassen

### Beispiel: Neues Feature

```toml
[dependencies]
serde = "1.0"                    # JSON-Serialisierung
serde_json = "1.0"              # JSON-Parsing
```

### Beispiel: Build-Skript erweitern

```rust
// build.rs
fn main() {
    // Manifest einbetten
    embed_manifest::embed_manifest_file("Contoso.Sample.manifest");
    
    // Zusätzliche Build-Schritte
    if cfg!(target_os = "windows") {
        println!("cargo:rustc-link-search=native=lib");
    }
}
```

## Deployment

### Standalone-Executable

Die Release-Version ist eine eigenständige .exe-Datei:

- **Keine externen DLLs** erforderlich (außer Windows-System-DLLs)
- **Manifest eingebettet** für bessere Windows-Integration
- **Minimale Größe** durch Release-Optimierungen

### Verteilung

1. **Release-Build** erstellen: `cargo build --release`
2. **Executable kopieren** aus `target/release/`
3. **Keine zusätzlichen Dateien** erforderlich
4. **Portabel** - funktioniert auf anderen Windows-Systemen

## Performance-Optimierungen

### Release-Build

- **LTO**: Link Time Optimization aktiviert
- **Code-Optimierung**: Rust-Compiler-Optimierungen
- **Stripping**: Debug-Symbole entfernt
- **Inlining**: Funktionen werden inline eingefügt

### Runtime-Optimierungen

- **Timer-Intervall**: 100ms Update-Rate (ausgewogen zwischen Performance und Aktualität)
- **Memory-Management**: Effiziente Icon-Freigabe
- **Windows-API**: Direkte API-Aufrufe ohne Abstraktionsschichten
