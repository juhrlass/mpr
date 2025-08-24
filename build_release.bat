@echo off
setlocal

:: Setze den Namen deiner Executable hier.
:: Cargo verwendet standardmäßig den Paketnamen.
set "EXECUTABLE_NAME=mpr.exe"
set "ICON_NAME=logo.ico"

echo [1/3] Cleaning previous build artifacts...
cargo clean

echo [2/3] Building the release binary...

:: Führe den normalen Cargo-Build aus.
cargo build --release

:: Überprüfe, ob der Build erfolgreich war.
:: %errorlevel% ist der Exit-Code des letzten Befehls. 0 bedeutet Erfolg.
if %errorlevel% neq 0 (
    echo.
    echo [ERROR] Cargo build failed. Aborting script.
    exit /b 1
)

echo [3/3] Attaching the icon with rcedit...

:: Der Pfad zur fertigen EXE-Datei
set "EXE_PATH=target\release\%EXECUTABLE_NAME%"

:: Überprüfe, ob die EXE-Datei existiert
if not exist "%EXE_PATH%" (
    echo.
    echo [ERROR] Could not find the executable at "%EXE_PATH%".
    exit /b 1
)

:: Führe rcedit auf die gerade erstellte EXE aus.
rcedit "%EXE_PATH%" --set-icon "%ICON_NAME%"

if %errorlevel% neq 0 (
    echo.
    echo [ERROR] rcedit failed to attach the icon.
    echo Please ensure 'rcedit.exe' is in your system's PATH.
    exit /b 1
)

echo.
echo [SUCCESS] Build complete. Icon was attached to %EXECUTABLE_NAME%.