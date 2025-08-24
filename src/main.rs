// Import der notwendigen Standard-Bibliotheken
use std::ffi::c_void;
use std::mem::size_of;

// Import der Windows-spezifischen Funktionen und Strukturen
use windows::core::w;  // Makro für Wide-String-Literale
use windows::Win32::Foundation::*;  // Grundlegende Windows-Typen
use windows::Win32::Graphics::Gdi::*;  // Graphics Device Interface für Icon-Erstellung
use windows::Win32::System::LibraryLoader::GetModuleHandleW;  // Modul-Handle abrufen
use windows::Win32::UI::Shell::*;  // Shell-Funktionen für Tray-Icons
use windows::Win32::UI::WindowsAndMessaging::*;  // Fenster- und Nachrichtenbehandlung

// Ein 5x7 Pixel Bitmap-Font für die Ziffern 0-9
// Jede Ziffer wird als 2D-Array von Pixeln definiert (0 = leer, 1 = gefüllt)
const FONT: [[[u8; 5]; 7]; 10] = [
    // 0 - Oben und unten offen, Seiten geschlossen
    [[0,1,1,1,0], [1,0,0,0,1], [1,0,0,0,1], [1,0,0,0,1], [1,0,0,0,1], [1,0,0,0,1], [0,1,1,1,0]],
    // 1 - Einfache vertikale Linie mit kleinem Haken oben
    [[0,0,1,0,0], [0,1,1,0,0], [0,0,1,0,0], [0,0,1,0,0], [0,0,1,0,0], [0,0,1,0,0], [0,1,1,1,0]],
    // 2 - Oben geschlossen, unten offen, mit Kurve in der Mitte
    [[0,1,1,1,0], [1,0,0,0,1], [0,0,0,0,1], [0,0,1,1,0], [0,1,0,0,0], [1,0,0,0,0], [1,1,1,1,1]],
    // 3 - Oben und unten geschlossen, mit Kurve in der Mitte
    [[1,1,1,1,0], [0,0,0,0,1], [0,0,0,1,0], [0,0,1,1,0], [0,0,0,0,1], [1,0,0,0,1], [0,1,1,1,0]],
    // 4 - Links offen, rechts geschlossen, mit horizontaler Linie in der Mitte
    [[0,0,0,1,0], [0,0,1,1,0], [0,1,0,1,0], [1,0,0,1,0], [1,1,1,1,1], [0,0,0,1,0], [0,0,0,1,0]],
    // 5 - Oben offen, unten geschlossen, mit Kurve in der Mitte
    [[1,1,1,1,1], [1,0,0,0,0], [1,1,1,1,0], [0,0,0,0,1], [0,0,0,0,1], [1,0,0,0,1], [0,1,1,1,0]],
    // 6 - Oben offen, unten geschlossen, mit Kurve in der Mitte
    [[0,0,1,1,0], [0,1,0,0,0], [1,0,0,0,0], [1,1,1,1,0], [1,0,0,0,1], [1,0,0,0,1], [0,1,1,1,0]],
    // 7 - Oben geschlossen, unten offen, mit diagonalem Strich
    [[1,1,1,1,1], [0,0,0,0,1], [0,0,0,1,0], [0,0,1,0,0], [0,0,1,0,0], [0,0,1,0,0], [0,0,1,0,0]],
    // 8 - Oben und unten geschlossen, mit Kurve in der Mitte
    [[0,1,1,1,0], [1,0,0,0,1], [1,0,0,0,1], [0,1,1,1,0], [1,0,0,0,1], [1,0,0,0,1], [0,1,1,1,0]],
    // 9 - Oben geschlossen, unten offen, mit Kurve in der Mitte
    [[0,1,1,1,0], [1,0,0,0,1], [1,0,0,0,1], [0,1,1,1,1], [0,0,0,0,1], [0,0,1,0,0], [0,1,1,0,0]],
];

// Konstante für Tray-Icon Nachrichten
// WM_USER + 1 wird als benutzerdefinierte Nachricht für Tray-Icon-Events verwendet
const TRAY_MESSAGE: u32 = WM_USER + 1;

// Globale Variable für das aktuelle Icon, um es später freigeben zu können
// Wird benötigt, um Memory-Leaks zu vermeiden, wenn neue Icons erstellt werden
static mut CURRENT_ICON: HICON = HICON(0 as *mut c_void);

// Hilfsfunktion zum sicheren Zugriff auf CURRENT_ICON
unsafe fn get_current_icon() -> HICON {
    CURRENT_ICON
}

// Hilfsfunktion zum sicheren Setzen von CURRENT_ICON
unsafe fn set_current_icon(icon: HICON) {
    CURRENT_ICON = icon;
}

/// Hauptfunktion des Programms
/// Erstellt ein verstecktes Fenster und ein Tray-Icon, das die aktuelle Mausposition anzeigt
fn main() {
    unsafe {
        // Modul-Handle der aktuellen Anwendung abrufen
        let hinstance = GetModuleHandleW(None).unwrap();
        
        // Eindeutiger Klassenname für das Fenster
        let class_name = w!("MPR");

        // Fensterklasse registrieren - definiert das Verhalten des Fensters
        let wc = WNDCLASSW {
            lpfnWndProc: Some(wndproc),  // Zeiger auf die Fensterprozedur
            hInstance: hinstance.into(),  // Instanz-Handle
            lpszClassName: class_name,   // Klassenname
            ..Default::default()          // Alle anderen Felder auf Standardwerte setzen
        };
        RegisterClassW(&wc);

        // Verstecktes Fenster erstellen (0x0 Größe, wird nicht angezeigt)
        // Das Fenster ist notwendig, um Windows-Nachrichten zu empfangen
        let hwnd = CreateWindowExW(
            Default::default(), class_name, w!(""), WS_OVERLAPPEDWINDOW,
            0, 0, 0, 0, None, None, hinstance, None,
        ).unwrap();

        // Tray-Icon-Datenstruktur vorbereiten
        let mut nid = NOTIFYICONDATAW {
            cbSize: size_of::<NOTIFYICONDATAW>() as u32,  // Größe der Struktur
            hWnd: hwnd,                                    // Fenster-Handle für Nachrichten
            uID: 1,                                        // Eindeutige ID für das Icon
            uFlags: NIF_MESSAGE | NIF_ICON | NIF_TIP,     // Welche Felder verwendet werden
            uCallbackMessage: TRAY_MESSAGE,                // Benutzerdefinierte Nachricht
            ..Default::default()
        };

        // Initiales Icon mit Koordinaten (0,0) erstellen
        set_current_icon(create_icon_with_cursor_position(0, 0));
        nid.hIcon = get_current_icon();
        
        // Tooltip-Text für das Tray-Icon setzen
        let tooltip_text = "Mausposition";
        let utf16_chars: Vec<u16> = tooltip_text.encode_utf16().collect();
        nid.szTip[..utf16_chars.len()].copy_from_slice(&utf16_chars);

        // Tray-Icon zum System-Tray hinzufügen
        let _ = Shell_NotifyIconW(NIM_ADD, &mut nid);
        
        // Timer starten, der alle 100ms die Mausposition abfragt
        SetTimer(hwnd, 1, 100, None);

        // Hauptnachrichtenschleife - verarbeitet alle Windows-Nachrichten
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).into() {
            let _ = TranslateMessage(&msg);    // Tastatur-Nachrichten übersetzen
            DispatchMessageW(&msg);            // Nachricht an die Fensterprozedur weiterleiten
        }

        // Aufräumen: Tray-Icon entfernen und Icon freigeben
        let _ = Shell_NotifyIconW(NIM_DELETE, &mut nid);
        let current_icon = get_current_icon();
        if !current_icon.is_invalid() {
            let _ = DestroyIcon(current_icon);
        }
    }
}

/// Erstellt ein 24x24 Pixel Icon mit den angegebenen Koordinaten
/// Die Koordinaten werden als 4-stellige Zahlen im Icon angezeigt
/// 
/// # Arguments
/// * `x_pos` - X-Koordinate der Maus
/// * `y_pos` - Y-Koordinate der Maus
/// 
/// # Returns
/// * `HICON` - Handle auf das erstellte Icon
unsafe fn create_icon_with_cursor_position(x_pos: u32, y_pos: u32) -> HICON {
    // Device Context für den Bildschirm abrufen
    let hdc = GetDC(None);
    
    // Kompatiblen Memory Device Context erstellen
    let memdc = CreateCompatibleDC(hdc);
    
    // 24x24 Bitmap erstellen
    let bmp = CreateCompatibleBitmap(hdc, 24, 24);
    let old_bmp = SelectObject(memdc, bmp);

    // Hintergrund schwarz färben
    let _ = PatBlt(memdc, 0, 0, 24, 24, BLACKNESS);

    // Koordinaten auf 4 Stellen begrenzen (0-9999)
    let numbers_to_draw = [x_pos % 10000, y_pos % 10000];
    
    // Textfarbe: Eisblau (RGB: 173, 216, 230)
    let text_color = COLORREF(0x00E6D8AD);

    // Y-Positionen für die beiden Zahlenzeilen
    let y_positions = [3, 14];

    // Beide Zahlen zeichnen (X und Y Koordinate)
    for (row_idx, &number) in numbers_to_draw.iter().enumerate() {
        let start_y = y_positions[row_idx];
        
        // Jede Ziffer einzeln zeichnen
        for i in 0..4 {
            let digit_value = (number / 10_u32.pow(3 - i as u32)) % 10;
            let glyph = FONT[digit_value as usize];
            let start_x = 1 + i as i32 * 6;

            // Jeden Pixel der Ziffer zeichnen
            for (y, row) in glyph.iter().enumerate() {
                for (x, &pixel) in row.iter().enumerate() {
                    if pixel == 1 {
                        SetPixel(memdc, start_x + x as i32, start_y + y as i32, text_color);
                    }
                }
            }
        }
    }

    // Icon aus dem Bitmap erstellen
    let mut ii = ICONINFO { 
        fIcon: true.into(),     // Es ist ein Icon (nicht ein Cursor)
        hbmMask: bmp,           // Mask-Bitmap
        hbmColor: bmp,          // Farb-Bitmap
        ..Default::default() 
    };
    let hicon = CreateIconIndirect(&mut ii).unwrap();

    // Ressourcen aufräumen
    SelectObject(memdc, old_bmp);
    let _ = DeleteObject(bmp);
    let _ = DeleteDC(memdc);
    ReleaseDC(None, hdc);

    hicon
}

/// Fensterprozedur - verarbeitet alle Nachrichten für das Fenster
/// Diese Funktion wird von Windows aufgerufen, wenn Nachrichten ankommen
/// 
/// # Arguments
/// * `hwnd` - Handle auf das Fenster
/// * `msg` - Nachrichten-ID
/// * `wparam` - Zusätzlicher Parameter 1
/// * `lparam` - Zusätzlicher Parameter 2
/// 
/// # Returns
/// * `LRESULT` - Ergebnis der Nachrichtenverarbeitung
extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        match msg {
            WM_TIMER => {
                // Timer-Nachricht alle 100ms - Mausposition abfragen und Icon aktualisieren
                let mut pt = POINT::default();
                let _ = GetCursorPos(&mut pt);

                // Neues Icon mit aktuellen Koordinaten erstellen
                let new_icon = create_icon_with_cursor_position(pt.x as u32, pt.y as u32);

                // Altes Icon freigeben, um Memory-Leaks zu vermeiden
                let current_icon = get_current_icon();
                if !current_icon.is_invalid() {
                    let _ = DestroyIcon(current_icon);
                }
                set_current_icon(new_icon);

                // Tray-Icon mit dem neuen Icon aktualisieren
                let mut nid = NOTIFYICONDATAW {
                    cbSize: size_of::<NOTIFYICONDATAW>() as u32,
                    hWnd: hwnd, uID: 1,
                    uFlags: NIF_ICON,  // Nur das Icon aktualisieren
                    hIcon: get_current_icon(),
                    ..Default::default()
                };
                let _ = Shell_NotifyIconW(NIM_MODIFY, &mut nid);
            }
            
            WM_DESTROY => {
                // Fenster wird zerstört - Programm beenden
                PostQuitMessage(0);
            }
            
            TRAY_MESSAGE => {
                // Benutzerdefinierte Nachricht vom Tray-Icon
                match lparam.0 as u32 {
                    WM_RBUTTONUP => {
                        // Rechtsklick auf Tray-Icon - Kontextmenü anzeigen
                        let mut pt = POINT::default();
                        let _ = GetCursorPos(&mut pt);
                        
                        // Popup-Menü erstellen
                        let hmenu = CreatePopupMenu().unwrap();
                        let _ = AppendMenuW(hmenu, MF_STRING, 1001, w!("Beenden"));
                        
                        // Fenster in den Vordergrund bringen (wichtig für Menü-Anzeige)
                        let _ = SetForegroundWindow(hwnd);
                        
                        // Menü an der aktuellen Mausposition anzeigen
                        let _ = TrackPopupMenu(hmenu, TPM_LEFTALIGN | TPM_RIGHTBUTTON, pt.x, pt.y, 0, hwnd, None);
                        
                        // Menü aufräumen
                        let _ = DestroyMenu(hmenu);
                    }
                    
                    WM_LBUTTONUP => {
                        // Linksklick auf Tray-Icon (optional: Fenster anzeigen/verstecken)
                        // Aktuell nicht implementiert
                    }
                    
                    _ => {
                        // Andere Maus-Events ignorieren
                    }
                }
            }
            
            WM_COMMAND => {
                // Menü-Auswahl verarbeiten
                match wparam.0 as u32 {
                    1001 => {
                        // "Beenden" wurde ausgewählt - Programm beenden
                        PostQuitMessage(0);
                    }
                    _ => {
                        // Andere Menüpunkte ignorieren
                    }
                }
            }
            
            _ => {
                // Alle anderen Nachrichten an die Standard-Fensterprozedur weiterleiten
                return DefWindowProcW(hwnd, msg, wparam, lparam);
            }
        }
    }
    LRESULT(0)  // Erfolgreich verarbeitet
}