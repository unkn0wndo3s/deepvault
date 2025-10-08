@echo off
echo Building DeepVault GUI...

REM Install Node.js dependencies
echo Installing Node.js dependencies...
npm install

REM Build the frontend
echo Building frontend...
npm run build

REM Build Tauri app
echo Building Tauri application...
cd src-tauri
cargo build --release
cd ..

echo Build complete!
echo Executable location: src-tauri\target\release\deepvault-gui.exe
