use std::ffi::c_void;
use std::mem::size_of;
use windows::core::w;
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Shell::*;
use windows::Win32::UI::WindowsAndMessaging::*;

// Ein 5x7 Pixel Bitmap-Font für die Ziffern 0-9
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

// Konstante für Tray-Icon Nachrichten
const TRAY_MESSAGE: u32 = WM_USER + 1;

// Globale Variable für das aktuelle Icon, um es später freigeben zu können
static mut CURRENT_ICON: HICON = HICON(0 as *mut c_void);

fn main() {
    unsafe {
        let hinstance = GetModuleHandleW(None).unwrap();
        let class_name = w!("TrayMousePos");

        let wc = WNDCLASSW {
            lpfnWndProc: Some(wndproc),
            hInstance: hinstance.into(),
            lpszClassName: class_name,
            ..Default::default()
        };
        RegisterClassW(&wc);

        let hwnd = CreateWindowExW(
            Default::default(), class_name, w!(""), WS_OVERLAPPEDWINDOW,
            0, 0, 0, 0, None, None, hinstance, None,
        ).unwrap();

        let mut nid = NOTIFYICONDATAW {
            cbSize: size_of::<NOTIFYICONDATAW>() as u32,
            hWnd: hwnd, uID: 1,
            uFlags: NIF_MESSAGE | NIF_ICON | NIF_TIP,
            uCallbackMessage: TRAY_MESSAGE,
            ..Default::default()
        };

        CURRENT_ICON = create_icon_with_coords(0, 0);
        nid.hIcon = CURRENT_ICON;
        let tooltip_text: [u16; 13] = ['M', 'a', 'u', 's', 'p', 'o', 's', 'i', 't', 'i', 'o', 'n', '\0']
            .map(|c| c as u16);
        let tip_ptr = std::ptr::addr_of_mut!(nid.szTip) as *mut u16;
        std::ptr::copy_nonoverlapping(tooltip_text.as_ptr(), tip_ptr, tooltip_text.len());

        let _ = Shell_NotifyIconW(NIM_ADD, &mut nid);
        SetTimer(hwnd, 1, 100, None);

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).into() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        let _ = Shell_NotifyIconW(NIM_DELETE, &mut nid);
        if !CURRENT_ICON.is_invalid() {
            let _ = DestroyIcon(CURRENT_ICON);
        }
    }
}

unsafe fn create_icon_with_coords(x_pos: u32, y_pos: u32) -> HICON {
    let hdc = GetDC(None);
    let memdc = CreateCompatibleDC(hdc);
    let bmp = CreateCompatibleBitmap(hdc, 24, 24);
    let old_bmp = SelectObject(memdc, bmp);

    let _ = PatBlt(memdc, 0, 0, 24, 24, BLACKNESS);

    let numbers_to_draw = [x_pos % 10000, y_pos % 10000];
    let text_color = COLORREF(0x0000FF00);

    let y_positions = [4, 13];

    for (row_idx, number) in numbers_to_draw.iter().enumerate() {
        let digits = [
            (number / 1000) % 10,
            (number / 100) % 10,
            (number / 10) % 10,
            *number % 10,
        ];
        let start_y = y_positions[row_idx];

        for (i, &digit) in digits.iter().enumerate() {
            let glyph = FONT[digit as usize];
            let start_x = 1 + i as i32 * 6;

            for (y, row) in glyph.iter().enumerate() {
                for (x, &pixel) in row.iter().enumerate() {
                    if pixel == 1 {
                        SetPixel(memdc, start_x + x as i32, start_y + y as i32, text_color);
                    }
                }
            }
        }
    }

    let mut ii = ICONINFO { fIcon: true.into(), hbmMask: bmp, hbmColor: bmp, ..Default::default() };
    let hicon = CreateIconIndirect(&mut ii).unwrap();

    SelectObject(memdc, old_bmp);
    let _ = DeleteObject(bmp);
    let _ = DeleteDC(memdc);
    ReleaseDC(None, hdc);

    hicon
}

extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        match msg {
            WM_TIMER => {
                let mut pt = POINT::default();
                let _ = GetCursorPos(&mut pt);

                let new_icon = create_icon_with_coords(pt.x as u32, pt.y as u32);

                if !CURRENT_ICON.is_invalid() {
                    let _ = DestroyIcon(CURRENT_ICON);
                }
                CURRENT_ICON = new_icon;

                let mut nid = NOTIFYICONDATAW {
                    cbSize: size_of::<NOTIFYICONDATAW>() as u32,
                    hWnd: hwnd, uID: 1,
                    uFlags: NIF_ICON,
                    hIcon: CURRENT_ICON,
                    ..Default::default()
                };
                let _ = Shell_NotifyIconW(NIM_MODIFY, &mut nid);
            }
            WM_DESTROY => {
                PostQuitMessage(0);
            }
            TRAY_MESSAGE => {
                // Tray-Icon Nachricht
                match lparam.0 as u32 {
                    WM_RBUTTONUP => {
                        // Rechtsklick auf Tray-Icon - Kontextmenü anzeigen
                        let mut pt = POINT::default();
                        let _ = GetCursorPos(&mut pt);
                        
                        let hmenu = CreatePopupMenu().unwrap();
                        let _ = AppendMenuW(hmenu, MF_STRING, 1001, w!("Beenden"));
                        
                        let _ = SetForegroundWindow(hwnd);
                        let _ = TrackPopupMenu(hmenu, TPM_LEFTALIGN | TPM_RIGHTBUTTON, pt.x, pt.y, 0, hwnd, None);
                        let _ = DestroyMenu(hmenu);
                    }
                    WM_LBUTTONUP => {
                        // Linksklick auf Tray-Icon (optional: Fenster anzeigen/verstecken)
                    }
                    _ => {}
                }
            }
            WM_COMMAND => {
                match wparam.0 as u32 {
                    1001 => {
                        // "Beenden" wurde ausgewählt
                        PostQuitMessage(0);
                    }
                    _ => {}
                }
            }
            _ => return DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
    LRESULT(0)
}