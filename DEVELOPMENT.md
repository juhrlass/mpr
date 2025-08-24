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
- **Release**: `target/release/mpr.exe` (smaller, faster, optimized)

### Dependencies

The project requires:

1. **Rust Toolchain**: Rust 1.70+ with Cargo
2. **Windows SDK**: For Windows API headers
3. **Build Tools**: Visual Studio Build Tools (optional)

## Development Environment

### Recommended Tools

- **IDE**: Visual Studio Code with Rust extension
- **Debugger**: Windows Debugger or Visual Studio
- **Build Tools**: Cargo (comes with Rust)

### Debugging

```bash
# Debug version with logging
cargo run

# Test release version
cargo run --release

# Specific Cargo features
cargo run --features debug
```

### Common Build Issues

1. **Windows API not found**: Check the features in `Cargo.toml`
2. **Manifest error**: Make sure `build.rs` is correctly configured
3. **Linker error**: Check Windows SDK installation

## Extensions

### Adding New Features

1. **New Windows API**: Add feature to `Cargo.toml`
2. **Additional dependencies**: Add to `[dependencies]`
3. **Extend build script**: Adapt `build.rs` for new resources

### Example: New Feature

```toml
[dependencies]
serde = "1.0"                    # JSON serialization
serde_json = "1.0"              # JSON parsing
```

### Example: Extend Build Script

```rust
// build.rs
fn main() {
    // Embed manifest
    embed_manifest::embed_manifest_file("Contoso.Sample.manifest");
    
    // Additional build steps
    if cfg!(target_os = "windows") {
        println!("cargo:rustc-link-search=native=lib");
    }
}
```

## Deployment

### Standalone Executable

The release version is a standalone .exe file:

- **No external DLLs** required (except Windows system DLLs)
- **Manifest embedded** for better Windows integration
- **Minimal size** through release optimizations

### Distribution

1. **Create release build**: `cargo build --release`
2. **Copy executable** from `target/release/`
3. **No additional files** required
4. **Portable** - works on other Windows systems

## Performance Optimizations

### Release Build

- **LTO**: Link Time Optimization enabled
- **Code optimization**: Rust compiler optimizations
- **Stripping**: Debug symbols removed
- **Inlining**: Functions are inlined

### Runtime Optimizations

- **Timer interval**: 100ms update rate (balanced between performance and accuracy)
- **Memory management**: Efficient icon cleanup
- **Windows API**: Direct API calls without abstraction layers
