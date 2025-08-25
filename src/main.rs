#![windows_subsystem = "windows"]

// Import of necessary standard libraries
use std::ffi::c_void;
use std::mem::size_of;
use std::ptr::null_mut;

// Import of Windows-specific functions and structures
use windows::core::w; // KORREKTUR: PCWSTR entfernt, da ungenutzt
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Shell::*;
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::Win32::UI::Controls::Dialogs::*;

// KORREKTUR 1: Vollständige Font-Daten wiederhergestellt
const FONT: [[[u8; 5]; 7]; 10] = [
    // 0
    [[0,1,1,1,0], [1,0,0,0,1], [1,0,0,0,1], [1,0,0,0,1], [1,0,0,0,1], [1,0,0,0,1], [0,1,1,1,0]],
    // 1
    [[0,0,1,0,0], [0,1,1,0,0], [0,0,1,0,0], [0,0,1,0,0], [0,0,1,0,0], [0,0,1,0,0], [0,1,1,1,0]],
    // 2
    [[0,1,1,1,0], [1,0,0,0,1], [0,0,0,0,1], [0,0,1,1,0], [0,1,0,0,0], [1,0,0,0,0], [1,1,1,1,1]],
    // 3
    [[1,1,1,1,0], [0,0,0,0,1], [0,0,0,1,0], [0,0,1,1,0], [0,0,0,0,1], [1,0,0,0,1], [0,1,1,1,0]],
    // 4
    [[0,0,0,1,0], [0,0,1,1,0], [0,1,0,1,0], [1,0,0,1,0], [1,1,1,1,1], [0,0,0,1,0], [0,0,0,1,0]],
    // 5
    [[1,1,1,1,1], [1,0,0,0,0], [1,1,1,1,0], [0,0,0,0,1], [0,0,0,0,1], [1,0,0,0,1], [0,1,1,1,0]],
    // 6
    [[0,0,1,1,0], [0,1,0,0,0], [1,0,0,0,0], [1,1,1,1,0], [1,0,0,0,1], [1,0,0,0,1], [0,1,1,1,0]],
    // 7
    [[1,1,1,1,1], [0,0,0,0,1], [0,0,0,1,0], [0,0,1,0,0], [0,0,1,0,0], [0,0,1,0,0], [0,0,1,0,0]],
    // 8
    [[0,1,1,1,0], [1,0,0,0,1], [1,0,0,0,1], [0,1,1,1,0], [1,0,0,0,1], [1,0,0,0,1], [0,1,1,1,0]],
    // 9
    [[0,1,1,1,0], [1,0,0,0,1], [1,0,0,0,1], [0,1,1,1,1], [0,0,0,0,1], [0,0,1,0,0], [0,1,1,0,0]],
];

/// Custom message ID for tray icon events
const TRAY_MESSAGE: u32 = WM_USER + 1;

/// Menu item IDs for context menu
const MENU_ID_EXIT: u32 = 1001;
const MENU_ID_SETTINGS: u32 = 1002;
const ID_COLOR_BUTTON: isize = 2001;

/// Global variables
static mut CURRENT_ICON: HICON = HICON(null_mut());
static mut SETTINGS_HWND: HWND = HWND(null_mut());
static mut CURRENT_TEXT_COLOR: COLORREF = COLORREF(0x00E6D8AD);
static mut COLOR_BUTTON_BRUSH: HBRUSH = HBRUSH(null_mut());
static mut COLOR_BUTTON_HWND: HWND = HWND(null_mut());

/// Helper functions (unverändert)
#[inline] unsafe fn get_current_icon() -> HICON { CURRENT_ICON }
#[inline] unsafe fn set_current_icon(icon: HICON) { CURRENT_ICON = icon; }
#[inline] unsafe fn get_current_text_color() -> COLORREF { CURRENT_TEXT_COLOR }
#[inline] unsafe fn set_current_text_color(color: COLORREF) { CURRENT_TEXT_COLOR = color; }

/// Creates a settings window
unsafe fn create_settings_window(hinstance: HINSTANCE) -> Result<HWND, windows::core::Error> {
    let class_name = w!("MPR_Settings");

    let wc = WNDCLASSW {
        lpfnWndProc: Some(settings_wndproc),
        hInstance: hinstance.into(),
        lpszClassName: class_name,
        hbrBackground: HBRUSH(GetStockObject(WHITE_BRUSH).0),
        ..Default::default()
    };

    if RegisterClassW(&wc) == 0 { return Err(windows::core::Error::from_win32()); }

    let hwnd = CreateWindowExW(
        Default::default(), class_name, w!("Settings"),
        WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_MINIMIZEBOX,
        CW_USEDEFAULT, CW_USEDEFAULT, 400, 300,
        None, None, hinstance, None,
    )?;

    let _label_hwnd = CreateWindowExW(
        Default::default(), w!("STATIC"), w!("Text Color:"),
        WS_CHILD | WS_VISIBLE, 20, 30, 100, 20,
        hwnd, None, hinstance, None,
    );

    let color_button_hwnd = CreateWindowExW(
        Default::default(), w!("BUTTON"), w!(""),
        WS_CHILD | WS_VISIBLE, 130, 25, 50, 30,
        hwnd,
        HMENU(ID_COLOR_BUTTON as *mut c_void),
        hinstance, None,
    )?;

    COLOR_BUTTON_HWND = color_button_hwnd;

    Ok(hwnd)
}

/// Settings window procedure
extern "system" fn settings_wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        match msg {
            WM_CLOSE => {
                if !COLOR_BUTTON_BRUSH.is_invalid() {
                    DeleteObject(COLOR_BUTTON_BRUSH);
                    COLOR_BUTTON_BRUSH = HBRUSH(null_mut());
                }
                DestroyWindow(hwnd);
                SETTINGS_HWND = HWND(null_mut());
                LRESULT(0)
            }

            WM_DESTROY => {
                SETTINGS_HWND = HWND(null_mut());
                LRESULT(0)
            }

            WM_COMMAND => {
                let control_id = (wparam.0 & 0xFFFF) as isize;
                if control_id == ID_COLOR_BUTTON {
                    static mut CUSTOM_COLORS: [COLORREF; 16] = [COLORREF(0); 16];
                    let mut cc = CHOOSECOLORW {
                        lStructSize: size_of::<CHOOSECOLORW>() as u32,
                        hwndOwner: hwnd,
                        rgbResult: get_current_text_color(),
                        lpCustColors: CUSTOM_COLORS.as_mut_ptr(),
                        Flags: CC_FULLOPEN | CC_RGBINIT,
                        ..Default::default()
                    };
                    if ChooseColorW(&mut cc).as_bool() {
                        set_current_text_color(cc.rgbResult);
                        InvalidateRect(hwnd, None, true);
                    }
                }
                LRESULT(0)
            }

            WM_CTLCOLORBTN => {
                if lparam.0 as isize == COLOR_BUTTON_HWND.0 as isize {
                    if !COLOR_BUTTON_BRUSH.is_invalid() {
                        DeleteObject(COLOR_BUTTON_BRUSH);
                    }
                    COLOR_BUTTON_BRUSH = CreateSolidBrush(get_current_text_color());
                    // KORREKTUR 2: Caste den Zeiger zu isize für LRESULT
                    return LRESULT(COLOR_BUTTON_BRUSH.0 as isize);
                }
                DefWindowProcW(hwnd, msg, wparam, lparam)
            }

            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}

/// Creates a 24x24 pixel icon with the specified coordinates
unsafe fn create_icon_with_cursor_position(x_pos: u32, y_pos: u32) -> Result<HICON, windows::core::Error> {
    let hdc = GetDC(None);
    if hdc.is_invalid() { return Err(windows::core::Error::from_win32()); }

    let memdc = CreateCompatibleDC(hdc);
    if memdc.is_invalid() {
        ReleaseDC(None, hdc);
        return Err(windows::core::Error::from_win32());
    }

    let bmp = CreateCompatibleBitmap(hdc, 24, 24);
    if bmp.is_invalid() {
        let _ = DeleteDC(memdc);
        let _ = ReleaseDC(None, hdc);
        return Err(windows::core::Error::from_win32());
    }

    let old_bmp = SelectObject(memdc, bmp);
    let _ = PatBlt(memdc, 0, 0, 24, 24, BLACKNESS);

    let numbers_to_draw = [x_pos % 10000, y_pos % 10000];
    let text_color = get_current_text_color();
    let y_positions = [3, 14];

    for (row_idx, &number) in numbers_to_draw.iter().enumerate() {
        let start_y = y_positions[row_idx];
        for i in 0..4 {
            let digit_value = (number / 10_u32.pow(3 - i as u32)) % 10;
            let glyph = FONT[digit_value as usize];
            let start_x = 1 + i as i32 * 6;
            for (y, row) in glyph.iter().enumerate() {
                for (x, &pixel) in row.iter().enumerate() {
                    if pixel == 1 {
                        let _ = SetPixel(memdc, start_x + x as i32, start_y + y as i32, text_color);
                    }
                }
            }
        }
    }

    let mut ii = ICONINFO {
        fIcon: true.into(),
        hbmMask: bmp,
        hbmColor: bmp,
        ..Default::default()
    };
    let hicon = CreateIconIndirect(&mut ii)?;

    SelectObject(memdc, old_bmp);
    let _ = DeleteObject(bmp);
    let _ = DeleteDC(memdc);
    let _ = ReleaseDC(None, hdc);

    Ok(hicon)
}

/// Window procedure - processes all messages for the window
extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        match msg {
            WM_TIMER => {
                let mut pt = POINT::default();
                if GetCursorPos(&mut pt).is_ok() {
                    if let Ok(new_icon) = create_icon_with_cursor_position(pt.x as u32, pt.y as u32) {
                        let current_icon = get_current_icon();
                        if !current_icon.is_invalid() {
                            let _ = DestroyIcon(current_icon);
                        }
                        set_current_icon(new_icon);

                        let mut nid = NOTIFYICONDATAW {
                            cbSize: size_of::<NOTIFYICONDATAW>() as u32,
                            hWnd: hwnd, uID: 1,
                            uFlags: NIF_ICON,
                            hIcon: get_current_icon(),
                            ..Default::default()
                        };
                        let _ = Shell_NotifyIconW(NIM_MODIFY, &mut nid);
                    }
                }
                LRESULT(0)
            }

            WM_DESTROY => {
                PostQuitMessage(0);
                LRESULT(0)
            }

            TRAY_MESSAGE => {
                match lparam.0 as u32 {
                    WM_RBUTTONUP => {
                        let mut pt = POINT::default();
                        if GetCursorPos(&mut pt).is_ok() {
                            if let Ok(hmenu) = CreatePopupMenu() {
                                let _ = AppendMenuW(hmenu, MF_STRING, MENU_ID_SETTINGS as usize, w!("Settings..."));
                                let _ = AppendMenuW(hmenu, MF_STRING, MENU_ID_EXIT as usize, w!("Exit"));
                                let _ = SetForegroundWindow(hwnd);
                                let _ = TrackPopupMenu(hmenu, TPM_LEFTALIGN | TPM_RIGHTBUTTON, pt.x, pt.y, 0, hwnd, None);
                                let _ = DestroyMenu(hmenu);
                            }
                        }
                        LRESULT(0)
                    }
                    _ => LRESULT(0),
                }
            }

            WM_COMMAND => {
                match wparam.0 as u32 {
                    MENU_ID_EXIT => {
                        PostQuitMessage(0);
                        LRESULT(0)
                    }
                    MENU_ID_SETTINGS => {
                        if let Ok(hinstance) = GetModuleHandleW(None) {
                            if let Ok(hwnd) = create_settings_window(hinstance.into()) {
                                let _ = ShowWindow(hwnd, SW_SHOW);
                                let _ = SetForegroundWindow(hwnd);
                                SETTINGS_HWND = hwnd;
                            }
                        }
                        LRESULT(0)
                    }
                    _ => LRESULT(0),
                }
            }

            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}

/// Main function of the program
fn main() -> Result<(), Box<dyn std::error::Error>> {
    unsafe {
        let hinstance = GetModuleHandleW(None)?;
        let class_name = w!("MPR");

        let wc = WNDCLASSW {
            lpfnWndProc: Some(wndproc),
            hInstance: hinstance.into(),
            lpszClassName: class_name,
            ..Default::default()
        };

        if RegisterClassW(&wc) == 0 {
            return Err("Failed to register window class".into());
        }

        let hwnd = CreateWindowExW(
            Default::default(), class_name, w!(""), WS_OVERLAPPEDWINDOW,
            0, 0, 0, 0, None, None, hinstance, None,
        )?;

        let mut nid = NOTIFYICONDATAW {
            cbSize: size_of::<NOTIFYICONDATAW>() as u32,
            hWnd: hwnd,
            uID: 1,
            uFlags: NIF_MESSAGE | NIF_ICON | NIF_TIP,
            uCallbackMessage: TRAY_MESSAGE,
            ..Default::default()
        };

        let initial_icon = create_icon_with_cursor_position(0, 0)?;
        set_current_icon(initial_icon);
        nid.hIcon = get_current_icon();

        let tooltip_text = "Mouse Position";
        let utf16_chars: Vec<u16> = tooltip_text.encode_utf16().collect();
        nid.szTip[..utf16_chars.len()].copy_from_slice(&utf16_chars);

        if !Shell_NotifyIconW(NIM_ADD, &mut nid).as_bool() {
            return Err("Failed to add tray icon".into());
        }

        if SetTimer(hwnd, 1, 100, None) == 0 {
            return Err("Failed to set timer".into());
        }

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).into() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        let _ = Shell_NotifyIconW(NIM_DELETE, &mut nid);
        let current_icon = get_current_icon();
        if !current_icon.is_invalid() {
            let _ = DestroyIcon(current_icon);
        }

        Ok(())
    }
}