#![windows_subsystem = "windows"]

// Import of necessary standard libraries
use std::ffi::c_void;
use std::mem::size_of;

// Import of Windows-specific functions and structures
use windows::core::{w, PCWSTR}; // Macro for wide string literals and PCWSTR type
use windows::Win32::Foundation::*; // Basic Windows types
use windows::Win32::Graphics::Gdi::*; // Graphics Device Interface for icon creation
use windows::Win32::System::LibraryLoader::GetModuleHandleW; // Get module handle
use windows::Win32::UI::Shell::*; // Shell functions for tray icons
use windows::Win32::UI::WindowsAndMessaging::*; // Window and message handling
use windows::Win32::UI::Controls::Dialogs::*; // Common dialogs including ChooseColor

/// A 5x7 pixel bitmap font for digits 0-9
/// Each digit is defined as a 2D array of pixels (0 = empty, 1 = filled)
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

/// Custom message ID for tray icon events
/// WM_USER + 1 is used as a custom message for tray icon events
const TRAY_MESSAGE: u32 = WM_USER + 1;

/// Menu item IDs for context menu
const MENU_ID_EXIT: u32 = 1001;
const MENU_ID_SETTINGS: u32 = 1002;

/// Global variable for the current icon to release it later
/// Needed to avoid memory leaks when creating new icons
static mut CURRENT_ICON: HICON = HICON(0 as *mut c_void);

/// Settings window handle
static mut SETTINGS_HWND: HWND = HWND(std::ptr::null_mut());

/// Global variable for the current text color
static mut CURRENT_TEXT_COLOR: COLORREF = COLORREF(0x00E6D8AD);

/// Helper function for safe access to CURRENT_ICON
#[inline]
unsafe fn get_current_icon() -> HICON {
    CURRENT_ICON
}

/// Helper function for safe setting of CURRENT_ICON
#[inline]
unsafe fn set_current_icon(icon: HICON) {
    CURRENT_ICON = icon;
}

/// Helper function for safe access to CURRENT_TEXT_COLOR
#[inline]
unsafe fn get_current_text_color() -> COLORREF {
    CURRENT_TEXT_COLOR
}

/// Helper function for safe setting of CURRENT_TEXT_COLOR
#[inline]
unsafe fn set_current_text_color(color: COLORREF) {
    CURRENT_TEXT_COLOR = color;
}

/// Helper function to get button rectangle
unsafe fn get_button_rect(hwnd: HWND) -> RECT {
    let mut rect = RECT::default();
    GetClientRect(hwnd, &mut rect);
    rect
}

/// Creates a settings window
///
/// # Arguments
/// * `hinstance` - Instance handle for the window
///
/// # Returns
/// * `HWND` - Handle to the created settings window
///
/// # Safety
/// This function is unsafe because it calls Windows API functions
unsafe fn create_settings_window(hinstance: HINSTANCE) -> Result<HWND, windows::core::Error> {
    let class_name = w!("MPR_Settings");

    // Register settings window class
    let wc = WNDCLASSW {
        lpfnWndProc: Some(settings_wndproc),
        hInstance: hinstance.into(),
        lpszClassName: class_name,
        hbrBackground: HBRUSH(GetStockObject(WHITE_BRUSH).0),
        ..Default::default()
    };

    if RegisterClassW(&wc) == 0 {
        return Err(windows::core::Error::from_win32());
    }

    // Create settings window
    let hwnd = CreateWindowExW(
        Default::default(),
        class_name,
        w!("Settings"),
        WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_MINIMIZEBOX,
        CW_USEDEFAULT,
        CW_USEDEFAULT,
        400,
        300,
        None,
        None,
        hinstance,
        None,
    )?;

    // Create "Text Color" label
    let _label_hwnd = CreateWindowExW(
        Default::default(),
        w!("STATIC"),
        w!("Text Color:"),
        WS_CHILD | WS_VISIBLE,
        20,
        30,
        100,
        20,
        hwnd,
        None,
        hinstance,
        None,
    );

    // Create color preview button (clickable area)
    let color_button_hwnd = CreateWindowExW(
        Default::default(),
        w!("BUTTON"),
        w!(""),
        WS_CHILD | WS_VISIBLE,
        130,
        25,
        50,
        30,
        hwnd,
        None,
        hinstance,
        None,
    )?;

    // Store the color button handle for later use
    if !color_button_hwnd.is_invalid() {
        SetWindowLongPtrW(
            color_button_hwnd,
            GWLP_USERDATA,
            color_button_hwnd.0 as isize,
        );
    }

    Ok(hwnd)
}

/// Settings window procedure
///
/// This function processes all messages for the settings window
///
/// # Arguments
/// * `hwnd` - Handle to the settings window
/// * `msg` - Message ID
/// * `wparam` - Additional parameter 1
/// * `lparam` - Additional parameter 2
///
/// # Returns
/// * `LRESULT` - Result of message processing
extern "system" fn settings_wndproc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    unsafe {
        match msg {
            WM_CLOSE => {
                // Close button clicked - destroy window
                let _ = DestroyWindow(hwnd);
                SETTINGS_HWND = HWND(std::ptr::null_mut());
                LRESULT(0)
            }

            WM_DESTROY => {
                // Window is destroyed
                SETTINGS_HWND = HWND(std::ptr::null_mut());
                LRESULT(0)
            }

            WM_LBUTTONDOWN => {
                // Handle left mouse button clicks
                let x = lparam.0 as i32 & 0xFFFF;
                let y = (lparam.0 as i32 >> 16) & 0xFFFF;

                // Check if click is on the color button area (130, 25, 50, 30)
                if x >= 130 && x <= 180 && y >= 25 && y <= 55 {
                    // Color button clicked - show ChooseColor dialog
                    // Statischer Puffer für benutzerdefinierte Farben
                    static mut CUSTOM_COLORS: [COLORREF; 16] = [COLORREF(0); 16];

                    let mut cc = CHOOSECOLORW {
                        lStructSize: size_of::<CHOOSECOLORW>() as u32,
                        hwndOwner: hwnd,
                        hInstance: HWND(std::ptr::null_mut()), // KORREKTUR 1
                        rgbResult: get_current_text_color(),
                        lpCustColors: CUSTOM_COLORS.as_mut_ptr(),
                        Flags: CC_FULLOPEN | CC_RGBINIT,
                        lCustData: LPARAM(0),
                        lpfnHook: None,
                        lpTemplateName: PCWSTR::null(), // KORREKTUR 2 (benötigt Import)
                    };

                    if ChooseColorW(&mut cc).as_bool() {
                        // Color selected - update global color
                        set_current_text_color(cc.rgbResult);

                        // Redraw the window to show new color
                        InvalidateRect(hwnd, None, true);
                    }
                }
                LRESULT(0)
            }

            _ => {
                // Forward all other messages to the standard window procedure
                DefWindowProcW(hwnd, msg, wparam, lparam)
            }
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
/// * `Result<HICON, windows::core::Error>` - Handle to the created icon or error
///
/// # Safety
/// This function is unsafe because it calls Windows API functions
unsafe fn create_icon_with_cursor_position(
    x_pos: u32,
    y_pos: u32,
) -> Result<HICON, windows::core::Error> {
    // Get device context for the screen
    let hdc = GetDC(None);
    if hdc.is_invalid() {
        return Err(windows::core::Error::from_win32());
    }

    // Create compatible memory device context
    let memdc = CreateCompatibleDC(hdc);
    if memdc.is_invalid() {
        ReleaseDC(None, hdc);
        return Err(windows::core::Error::from_win32());
    }

    // Create 24x24 bitmap
    let bmp = CreateCompatibleBitmap(hdc, 24, 24);
    if bmp.is_invalid() {
        let _ = DeleteDC(memdc);
        let _ = ReleaseDC(None, hdc);
        return Err(windows::core::Error::from_win32());
    }

    let old_bmp = SelectObject(memdc, bmp);

    // Fill background black
    let _ = PatBlt(memdc, 0, 0, 24, 24, BLACKNESS);

    // Limit coordinates to 4 digits (0-9999)
    let numbers_to_draw = [x_pos % 10000, y_pos % 10000];

    // Text color: Use current global text color
    let text_color = get_current_text_color();

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
                        let _ = SetPixel(
                            memdc,
                            start_x + x as i32,
                            start_y + y as i32,
                            text_color,
                        );
                    }
                }
            }
        }
    }

    // Create icon from bitmap
    let mut ii = ICONINFO {
        fIcon: true.into(), // It's an icon (not a cursor)
        hbmMask: bmp,       // Mask bitmap
        hbmColor: bmp,      // Color bitmap
        ..Default::default()
    };

    let hicon = CreateIconIndirect(&mut ii)?;

    // Clean up resources
    SelectObject(memdc, old_bmp);
    let _ = DeleteObject(bmp);
    let _ = DeleteDC(memdc);
    let _ = ReleaseDC(None, hdc);

    Ok(hicon)
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
extern "system" fn wndproc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    unsafe {
        match msg {
            WM_TIMER => {
                // Timer message every 100ms - poll mouse position and update icon
                let mut pt = POINT::default();
                if GetCursorPos(&mut pt).is_ok() {
                    // Create new icon with current coordinates
                    if let Ok(new_icon) =
                        create_icon_with_cursor_position(pt.x as u32, pt.y as u32)
                    {
                        // Release old icon to avoid memory leaks
                        let current_icon = get_current_icon();
                        if !current_icon.is_invalid() {
                            let _ = DestroyIcon(current_icon);
                        }
                        set_current_icon(new_icon);

                        // Update tray icon with the new icon
                        let mut nid = NOTIFYICONDATAW {
                            cbSize: size_of::<NOTIFYICONDATAW>() as u32,
                            hWnd: hwnd,
                            uID: 1,
                            uFlags: NIF_ICON, // Only update icon
                            hIcon: get_current_icon(),
                            ..Default::default()
                        };
                        let _ = Shell_NotifyIconW(NIM_MODIFY, &mut nid);
                    }
                }
                LRESULT(0)
            }

            WM_DESTROY => {
                // Window is destroyed - terminate program
                PostQuitMessage(0);
                LRESULT(0)
            }

            TRAY_MESSAGE => {
                // Custom message from tray icon
                match lparam.0 as u32 {
                    WM_RBUTTONUP => {
                        // Right-click on tray icon - show context menu
                        let mut pt = POINT::default();
                        if GetCursorPos(&mut pt).is_ok() {
                            // Create popup menu
                            if let Ok(hmenu) = CreatePopupMenu() {
                                let _ = AppendMenuW(
                                    hmenu,
                                    MF_STRING,
                                    MENU_ID_SETTINGS as usize,
                                    w!("Settings..."),
                                );
                                let _ = AppendMenuW(
                                    hmenu,
                                    MF_STRING,
                                    MENU_ID_EXIT as usize,
                                    w!("Exit"),
                                );

                                // Bring window to foreground (important for menu display)
                                let _ = SetForegroundWindow(hwnd);

                                // Display menu at current mouse position
                                let _ = TrackPopupMenu(
                                    hmenu,
                                    TPM_LEFTALIGN | TPM_RIGHTBUTTON,
                                    pt.x,
                                    pt.y,
                                    0,
                                    hwnd,
                                    None,
                                );

                                // Clean up menu
                                let _ = DestroyMenu(hmenu);
                            }
                        }
                        LRESULT(0)
                    }

                    WM_LBUTTONUP => {
                        // Left-click on tray icon (optional: show/hide window)
                        // Currently not implemented
                        LRESULT(0)
                    }

                    _ => {
                        // Ignore other mouse events
                        LRESULT(0)
                    }
                }
            }

            WM_COMMAND => {
                // Process menu selection
                match wparam.0 as u32 {
                    MENU_ID_EXIT => {
                        // "Exit" selected - terminate program
                        PostQuitMessage(0);
                        LRESULT(0)
                    }
                    MENU_ID_SETTINGS => {
                        // "Settings..." selected - open settings window
                        if let Ok(hinstance) = GetModuleHandleW(None) {
                            if let Ok(hwnd) = create_settings_window(hinstance.into()) {
                                let _ = ShowWindow(hwnd, SW_SHOW);
                                let _ = SetForegroundWindow(hwnd);
                                SETTINGS_HWND = hwnd;
                            }
                        }
                        LRESULT(0)
                    }
                    _ => {
                        // Ignore other menu items
                        LRESULT(0)
                    }
                }
            }

            _ => {
                // Forward all other messages to the standard window procedure
                DefWindowProcW(hwnd, msg, wparam, lparam)
            }
        }
    }
}

/// Main function of the program
/// Creates a hidden window and a tray icon that displays the current mouse position
fn main() -> Result<(), Box<dyn std::error::Error>> {
    unsafe {
        // Get module handle of the current application
        let hinstance = GetModuleHandleW(None)?;

        // Unique class name for the window
        let class_name = w!("MPR");

        // Register window class - defines the behavior of the window
        let wc = WNDCLASSW {
            lpfnWndProc: Some(wndproc),  // Pointer to window procedure
            hInstance: hinstance.into(), // Instance handle
            lpszClassName: class_name,   // Class name
            ..Default::default()         // Set all other fields to default values
        };

        if RegisterClassW(&wc) == 0 {
            return Err("Failed to register window class".into());
        }

        // Create hidden window (0x0 size, not displayed)
        // The window is necessary to receive Windows messages
        let hwnd = CreateWindowExW(
            Default::default(),
            class_name,
            w!(""),
            WS_OVERLAPPEDWINDOW,
            0,
            0,
            0,
            0,
            None,
            None,
            hinstance,
            None,
        )?;

        // Prepare tray icon data structure
        let mut nid = NOTIFYICONDATAW {
            cbSize: size_of::<NOTIFYICONDATAW>() as u32, // Size of the structure
            hWnd: hwnd,                                 // Window handle for messages
            uID: 1,                                     // Unique ID for the icon
            uFlags: NIF_MESSAGE | NIF_ICON | NIF_TIP, // Which fields are used
            uCallbackMessage: TRAY_MESSAGE,             // Custom message
            ..Default::default()
        };

        // Create initial icon with coordinates (0,0)
        let initial_icon = create_icon_with_cursor_position(0, 0)?;
        set_current_icon(initial_icon);
        nid.hIcon = get_current_icon();

        // Set tooltip text for the tray icon
        let tooltip_text = "Mouse Position";
        let utf16_chars: Vec<u16> = tooltip_text.encode_utf16().collect();
        nid.szTip[..utf16_chars.len()].copy_from_slice(&utf16_chars);

        // Add tray icon to system tray
        if !Shell_NotifyIconW(NIM_ADD, &mut nid).as_bool() {
            return Err("Failed to add tray icon".into());
        }

        // Start timer to poll mouse position every 100ms
        if SetTimer(hwnd, 1, 100, None) == 0 {
            return Err("Failed to set timer".into());
        }

        // Main message loop - processes all Windows messages
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).into() {
            let _ = TranslateMessage(&msg); // Translate keyboard messages
            DispatchMessageW(&msg); // Forward message to window procedure
        }

        // Clean up: Remove tray icon and release icon
        let _ = Shell_NotifyIconW(NIM_DELETE, &mut nid);
        let current_icon = get_current_icon();
        if !current_icon.is_invalid() {
            let _ = DestroyIcon(current_icon);
        }

        Ok(())
    }
}
