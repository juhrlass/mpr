#![windows_subsystem = "windows"]

// Import of necessary standard libraries
use std::ffi::c_void;
use std::mem::size_of;

// Import of Windows-specific functions and structures
use windows::core::w;  // Macro for wide string literals
use windows::Win32::Foundation::*;  // Basic Windows types
use windows::Win32::Graphics::Gdi::*;  // Graphics Device Interface for icon creation
use windows::Win32::System::LibraryLoader::GetModuleHandleW;  // Get module handle
use windows::Win32::UI::Shell::*;  // Shell functions for tray icons
use windows::Win32::UI::WindowsAndMessaging::*;  // Window and message handling

// A 5x7 pixel bitmap font for digits 0-9
// Each digit is defined as a 2D array of pixels (0 = empty, 1 = filled)
const FONT: [[[u8; 5]; 7]; 10] = [
    // 0 - Open at top and bottom, closed at sides
    [[0,1,1,1,0], [1,0,0,0,1], [1,0,0,0,1], [1,0,0,0,1], [1,0,0,0,1], [1,0,0,0,1], [0,1,1,1,0]],
    // 1 - Simple vertical line with small hook at top
    [[0,0,1,0,0], [0,1,1,0,0], [0,0,1,0,0], [0,0,1,0,0], [0,0,1,0,0], [0,0,1,0,0], [0,1,1,1,0]],
    // 2 - Closed at top, open at bottom, with curve in middle
    [[0,1,1,1,0], [1,0,0,0,1], [0,0,0,0,1], [0,0,1,1,0], [0,1,0,0,0], [1,0,0,0,0], [1,1,1,1,1]],
    // 3 - Closed at top and bottom, with curve in middle
    [[1,1,1,1,0], [0,0,0,0,1], [0,0,0,1,0], [0,0,1,1,0], [0,0,0,0,1], [1,0,0,0,1], [0,1,1,1,0]],
    // 4 - Open at left, closed at right, with horizontal line in middle
    [[0,0,0,1,0], [0,0,1,1,0], [0,1,0,1,0], [1,0,0,1,0], [1,1,1,1,1], [0,0,0,1,0], [0,0,0,1,0]],
    // 5 - Open at top, closed at bottom, with curve in middle
    [[1,1,1,1,1], [1,0,0,0,0], [1,1,1,1,0], [0,0,0,0,1], [0,0,0,0,1], [1,0,0,0,1], [0,1,1,1,0]],
    // 6 - Open at top, closed at bottom, with curve in middle
    [[0,0,1,1,0], [0,1,0,0,0], [1,0,0,0,0], [1,1,1,1,0], [1,0,0,0,1], [1,0,0,0,1], [0,1,1,1,0]],
    // 7 - Closed at top, open at bottom, with diagonal stroke
    [[1,1,1,1,1], [0,0,0,0,1], [0,0,0,1,0], [0,0,1,0,0], [0,0,1,0,0], [0,0,1,0,0], [0,0,1,0,0]],
    // 8 - Closed at top and bottom, with curve in middle
    [[0,1,1,1,0], [1,0,0,0,1], [1,0,0,0,1], [0,1,1,1,0], [1,0,0,0,1], [1,0,0,0,1], [0,1,1,1,0]],
    // 9 - Closed at top, open at bottom, with curve in middle
    [[0,1,1,1,0], [1,0,0,0,1], [1,0,0,0,1], [0,1,1,1,1], [0,0,0,0,1], [0,0,1,0,0], [0,1,1,0,0]],
];

// Constant for tray icon messages
// WM_USER + 1 is used as a custom message for tray icon events
const TRAY_MESSAGE: u32 = WM_USER + 1;

// Global variable for the current icon to release it later
// Needed to avoid memory leaks when creating new icons
static mut CURRENT_ICON: HICON = HICON(0 as *mut c_void);

// Helper function for safe access to CURRENT_ICON
unsafe fn get_current_icon() -> HICON {
    CURRENT_ICON
}

// Helper function for safe setting of CURRENT_ICON
unsafe fn set_current_icon(icon: HICON) {
    CURRENT_ICON = icon;
}

/// Main function of the program
/// Creates a hidden window and a tray icon that displays the current mouse position
fn main() {
    unsafe {
        // Get module handle of the current application
        let hinstance = GetModuleHandleW(None).unwrap();
        
        // Unique class name for the window
        let class_name = w!("MPR");

        // Register window class - defines the behavior of the window
        let wc = WNDCLASSW {
            lpfnWndProc: Some(wndproc),  // Pointer to window procedure
            hInstance: hinstance.into(),  // Instance handle
            lpszClassName: class_name,   // Class name
            ..Default::default()          // Set all other fields to default values
        };
        RegisterClassW(&wc);

        // Create hidden window (0x0 size, not displayed)
        // The window is necessary to receive Windows messages
        let hwnd = CreateWindowExW(
            Default::default(), class_name, w!(""), WS_OVERLAPPEDWINDOW,
            0, 0, 0, 0, None, None, hinstance, None,
        ).unwrap();

        // Prepare tray icon data structure
        let mut nid = NOTIFYICONDATAW {
            cbSize: size_of::<NOTIFYICONDATAW>() as u32,  // Size of the structure
            hWnd: hwnd,                                    // Window handle for messages
            uID: 1,                                        // Unique ID for the icon
            uFlags: NIF_MESSAGE | NIF_ICON | NIF_TIP,     // Which fields are used
            uCallbackMessage: TRAY_MESSAGE,                // Custom message
            ..Default::default()
        };

        // Create initial icon with coordinates (0,0)
        set_current_icon(create_icon_with_cursor_position(0, 0));
        nid.hIcon = get_current_icon();
        
        // Set tooltip text for the tray icon
        let tooltip_text = "Mouse Position";
        let utf16_chars: Vec<u16> = tooltip_text.encode_utf16().collect();
        nid.szTip[..utf16_chars.len()].copy_from_slice(&utf16_chars);

        // Add tray icon to system tray
        let _ = Shell_NotifyIconW(NIM_ADD, &mut nid);
        
        // Start timer to poll mouse position every 100ms
        SetTimer(hwnd, 1, 100, None);

        // Main message loop - processes all Windows messages
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).into() {
            let _ = TranslateMessage(&msg);    // Translate keyboard messages
            DispatchMessageW(&msg);            // Forward message to window procedure
        }

        // Clean up: Remove tray icon and release icon
        let _ = Shell_NotifyIconW(NIM_DELETE, &mut nid);
        let current_icon = get_current_icon();
        if !current_icon.is_invalid() {
            let _ = DestroyIcon(current_icon);
        }
    }
}

/// Creates a 24x24 pixel icon with the specified coordinates
/// The coordinates are displayed as 4-digit numbers in the icon
/// 
/// # Arguments
/// * `x_pos` - X-coordinate of the mouse
/// * `y_pos` - Y-coordinate of the mouse
/// 
/// # Returns
/// * `HICON` - Handle to the created icon
unsafe fn create_icon_with_cursor_position(x_pos: u32, y_pos: u32) -> HICON {
    // Get device context for the screen
    let hdc = GetDC(None);
    
    // Create compatible memory device context
    let memdc = CreateCompatibleDC(hdc);
    
    // Create 24x24 bitmap
    let bmp = CreateCompatibleBitmap(hdc, 24, 24);
    let old_bmp = SelectObject(memdc, bmp);

    // Fill background black
    let _ = PatBlt(memdc, 0, 0, 24, 24, BLACKNESS);

    // Limit coordinates to 4 digits (0-9999)
    let numbers_to_draw = [x_pos % 10000, y_pos % 10000];
    
    // Text color: Light Blue (RGB: 173, 216, 230)
    let text_color = COLORREF(0x00E6D8AD);

    // Y-positions for the two number lines
    let y_positions = [3, 14];

    // Draw both numbers (X and Y coordinate)
    for (row_idx, &number) in numbers_to_draw.iter().enumerate() {
        let start_y = y_positions[row_idx];
        
        // Draw each digit individually
        for i in 0..4 {
            let digit_value = (number / 10_u32.pow(3 - i as u32)) % 10;
            let glyph = FONT[digit_value as usize];
            let start_x = 1 + i as i32 * 6;

            // Draw each pixel of the digit
            for (y, row) in glyph.iter().enumerate() {
                for (x, &pixel) in row.iter().enumerate() {
                    if pixel == 1 {
                        SetPixel(memdc, start_x + x as i32, start_y + y as i32, text_color);
                    }
                }
            }
        }
    }

    // Create icon from bitmap
    let mut ii = ICONINFO { 
        fIcon: true.into(),     // It's an icon (not a cursor)
        hbmMask: bmp,           // Mask bitmap
        hbmColor: bmp,          // Color bitmap
        ..Default::default() 
    };
    let hicon = CreateIconIndirect(&mut ii).unwrap();

    // Clean up resources
    SelectObject(memdc, old_bmp);
    let _ = DeleteObject(bmp);
    let _ = DeleteDC(memdc);
    ReleaseDC(None, hdc);

    hicon
}

/// Window procedure - processes all messages for the window
/// This function is called by Windows when messages arrive
/// 
/// # Arguments
/// * `hwnd` - Handle to the window
/// * `msg` - Message ID
/// * `wparam` - Additional parameter 1
/// * `lparam` - Additional parameter 2
/// 
/// # Returns
/// * `LRESULT` - Result of message processing
extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        match msg {
            WM_TIMER => {
                // Timer message every 100ms - poll mouse position and update icon
                let mut pt = POINT::default();
                let _ = GetCursorPos(&mut pt);

                // Create new icon with current coordinates
                let new_icon = create_icon_with_cursor_position(pt.x as u32, pt.y as u32);

                // Release old icon to avoid memory leaks
                let current_icon = get_current_icon();
                if !current_icon.is_invalid() {
                    let _ = DestroyIcon(current_icon);
                }
                set_current_icon(new_icon);

                // Update tray icon with the new icon
                let mut nid = NOTIFYICONDATAW {
                    cbSize: size_of::<NOTIFYICONDATAW>() as u32,
                    hWnd: hwnd, uID: 1,
                    uFlags: NIF_ICON,  // Only update icon
                    hIcon: get_current_icon(),
                    ..Default::default()
                };
                let _ = Shell_NotifyIconW(NIM_MODIFY, &mut nid);
            }
            
            WM_DESTROY => {
                // Window is destroyed - terminate program
                PostQuitMessage(0);
            }
            
            TRAY_MESSAGE => {
                // Custom message from tray icon
                match lparam.0 as u32 {
                    WM_RBUTTONUP => {
                        // Right-click on tray icon - show context menu
                        let mut pt = POINT::default();
                        let _ = GetCursorPos(&mut pt);
                        
                        // Create popup menu
                        let hmenu = CreatePopupMenu().unwrap();
                        let _ = AppendMenuW(hmenu, MF_STRING, 1002, w!("Settings..."));
                        let _ = AppendMenuW(hmenu, MF_STRING, 1001, w!("Exit"));
                        
                        // Bring window to foreground (important for menu display)
                        let _ = SetForegroundWindow(hwnd);
                        
                        // Display menu at current mouse position
                        let _ = TrackPopupMenu(hmenu, TPM_LEFTALIGN | TPM_RIGHTBUTTON, pt.x, pt.y, 0, hwnd, None);
                        
                        // Clean up menu
                        let _ = DestroyMenu(hmenu);
                    }
                    
                    WM_LBUTTONUP => {
                        // Left-click on tray icon (optional: show/hide window)
                        // Currently not implemented
                    }
                    
                    _ => {
                        // Ignore other mouse events
                    }
                }
            }
            
            WM_COMMAND => {
                // Process menu selection
                match wparam.0 as u32 {
                    1001 => {
                        // "Exit" selected - terminate program
                        PostQuitMessage(0);
                    }
                    1002 => {
                        // "Settings..." selected - not implemented yet
                        // TODO: Implement settings window
                    }
                    _ => {
                        // Ignore other menu items
                    }
                }
            }
            
            _ => {
                // Forward all other messages to the standard window procedure
                return DefWindowProcW(hwnd, msg, wparam, lparam);
            }
        }
    }
    LRESULT(0)  // Successfully processed
}