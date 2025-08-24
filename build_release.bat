@echo off
setlocal

:: Set the name of your executable here.
:: Cargo uses the package name by default.
set "EXECUTABLE_NAME=mpr.exe"
set "ICON_NAME=logo.ico"

echo [1/3] Cleaning previous build artifacts...
cargo clean

echo [2/3] Building the release binary...

:: Execute the normal Cargo build.
cargo build --release

:: Check if the build was successful.
:: %errorlevel% is the exit code of the last command. 0 means success.
if %errorlevel% neq 0 (
    echo.
    echo [ERROR] Cargo build failed. Aborting script.
    exit /b 1
)

echo [3/3] Attaching the icon with rcedit...

:: The path to the finished EXE file
set "EXE_PATH=target\release\%EXECUTABLE_NAME%"

:: Check if the EXE file exists
if not exist "%EXE_PATH%" (
    echo.
    echo [ERROR] Could not find the executable at "%EXE_PATH%".
    exit /b 1
)

:: Execute rcedit on the just created EXE.
rcedit "%EXE_PATH%" --set-icon "%ICON_NAME%"

if %errorlevel% neq 0 (
    echo.
    echo [ERROR] rcedit failed to attach the icon.
    echo Please ensure 'rcedit.exe' is in your system's PATH.
    exit /b 1
)

echo.
echo [SUCCESS] Build complete. Icon was attached to %EXECUTABLE_NAME%.