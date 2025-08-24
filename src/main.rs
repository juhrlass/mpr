use windows::core::w;
use std::mem::size_of;

use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Shell::*;
use windows::Win32::UI::WindowsAndMessaging::*;

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
        )
        .unwrap();

        let mut nid = NOTIFYICONDATAW {
            cbSize: size_of::<NOTIFYICONDATAW>() as u32,
            hWnd: hwnd,
            uID: 1,
            uFlags: NIF_MESSAGE | NIF_ICON | NIF_TIP,
            uCallbackMessage: WM_USER + 1,
            ..Default::default()
        };

        nid.hIcon = create_icon_with_text("0,0");

        // KORREKTUR HIER:
        let tooltip_text: [u16; 9] = [
            'M' as u16, 'o' as u16, 'u' as u16, 's' as u16, 'e' as u16, 0, 0, 0, 0,
        ];
        // 1. Adresse als rohen Pointer holen, ohne Referenz zu erstellen
        let tip_ptr = std::ptr::addr_of_mut!(nid.szTip);
        // 2. Den Pointer in den richtigen Typ für die Kopierfunktion umwandeln
        let tip_ptr_u16 = tip_ptr as *mut u16;
        // 3. Sicher kopieren
        std::ptr::copy_nonoverlapping(tooltip_text.as_ptr(), tip_ptr_u16, tooltip_text.len());

        Shell_NotifyIconW(NIM_ADD, &mut nid);
        SetTimer(hwnd, 1, 500, None);

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).into() {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        Shell_NotifyIconW(NIM_DELETE, &mut nid);
    }
}

unsafe fn create_icon_with_text(text: &str) -> HICON {
    let hdc = GetDC(None);
    let memdc = CreateCompatibleDC(hdc);
    let bmp = CreateCompatibleBitmap(hdc, 16, 16);
    let old = SelectObject(memdc, bmp);

    PatBlt(memdc, 0, 0, 16, 16, WHITENESS);

    let wide: Vec<u16> = text.encode_utf16().chain(Some(0)).collect();
    // KORREKTUR 2: 'as i32' entfernt
    SetBkMode(memdc, TRANSPARENT);
    TextOutW(memdc, 0, 0, &wide);

    let mut ii = ICONINFO {
        fIcon: true.into(),
        xHotspot: 0,
        yHotspot: 0,
        hbmMask: bmp,
        hbmColor: bmp,
    };
    // KORREKTUR 3: .unwrap() hinzugefügt
    let hicon = CreateIconIndirect(&mut ii).unwrap();

    SelectObject(memdc, old);
    DeleteDC(memdc);
    ReleaseDC(None, hdc);

    hicon
}

// wndproc bleibt unverändert
extern "system" fn wndproc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    unsafe {
        match msg {
            WM_TIMER => {
                let mut pt = POINT { x: 0, y: 0 };
                GetCursorPos(&mut pt);

                let text = format!("{},{}", pt.x % 100, pt.y % 100);

                let mut nid = NOTIFYICONDATAW {
                    cbSize: size_of::<NOTIFYICONDATAW>() as u32,
                    hWnd: hwnd,
                    uID: 1,
                    uFlags: NIF_ICON | NIF_TIP,
                    ..Default::default()
                };
                nid.hIcon = create_icon_with_text(&text);

                Shell_NotifyIconW(NIM_MODIFY, &mut nid);
            }
            WM_DESTROY => {
                PostQuitMessage(0);
            }
            _ => return DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
    LRESULT(0)
}