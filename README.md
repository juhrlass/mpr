# MPR - Mouse Position Reader

A Windows tray tool that displays the current mouse position in real-time.

## What does it do?

MPR is a small, efficient tool that runs in the system tray and continuously monitors the current mouse position. The tray icon displays the X and Y coordinates of the mouse as green numbers on a black background.

## Features

- **Real-time monitoring**: Updates mouse position every 100ms
- **Tray integration**: Runs in the background without visible window
- **Visual display**: Shows coordinates directly in the tray icon
- **Right-click menu**: Simple exit via context menu
- **Resource efficient**: Minimal memory usage and CPU load

## Usage

### Installation

1. Make sure Rust is installed
2. Clone the repository
3. Run `cargo build --release`
4. The executable is located in `target/release/mpr.exe`

### Operation

1. **Start**: Run `mpr.exe`
2. **Tray icon**: The tool appears as a small icon in the system tray
3. **Read coordinates**: The top line shows X, the bottom line Y coordinate
4. **Exit**: Right-click on the tray icon → "Exit"

### Coordinate Format

- **X-coordinate**: Top line (0-9999)
- **Y-coordinate**: Bottom line (0-9999)
- **Update**: Every 100 milliseconds
- **Display**: 4-digit numbers in green text on black background

## Technical Details

- **Language**: Rust
- **Windows API**: Native Win32 API via the `windows-rs` crate
- **Icon size**: 24x24 pixels
- **Font**: Custom 5x7 pixel bitmap font
- **Message processing**: Windows message loop with timer

## System Requirements

- Windows 10 or higher
- No additional dependencies
- Minimal memory usage (~1-2 MB)

## Development

For developers, see [DEVELOPMENT.md](DEVELOPMENT.md) for details on build configuration and the manifest system.

## License

This project is intended for private use.

## Known Limitations

- Coordinates are limited to 4 digits (0-9999)
- Only works under Windows
- No configuration options
- No persistence of settings

## Troubleshooting

**Tray icon doesn't appear**: Check if the tool is running as administrator
**Program won't start**: Make sure all Windows updates are installed
**High CPU load**: The tool updates every 100ms - this is normal

## Support

For problems or questions, please create an issue in the repository.
