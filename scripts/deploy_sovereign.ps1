# scripts/deploy_sovereign.ps1
# Enterprise Packaging Script for Utah Browser (Windows)

Write-Host "[SYSTEM] Initiating Sovereign Deployment Matrix..." -ForegroundColor Cyan

$BUILD_DIR = "build\release_candidate"
$ASSETS_DIR = "assets\ui"
$FLUX_DIR = "flux"

# Step 1: Clean previous builds
Write-Host "[1/4] Cleansing build environment..." -ForegroundColor Yellow
if (Test-Path $BUILD_DIR) { Remove-Item -Recurse -Force $BUILD_DIR }
New-Item -ItemType Directory -Path "$BUILD_DIR\daemons" -Force | Out-Null
New-Item -ItemType Directory -Path "$BUILD_DIR\ui" -Force | Out-Null

# Step 2: Compile Python Daemons to Standalone Binaries
Write-Host "[2/4] Freezing Python Daemons (PyInstaller)..." -ForegroundColor Yellow
# Ensure PyInstaller is installed
# pip install pyinstaller
& pyinstaller --noconfirm --onedir --windowed --distpath "$BUILD_DIR\daemons" "$FLUX_DIR\email_nexus.py"
& pyinstaller --noconfirm --onedir --windowed --distpath "$BUILD_DIR\daemons" "$FLUX_DIR\career_forge.py"

# Step 3: Compile Rust Kernel
Write-Host "[3/4] Compiling Rust Core Engine..." -ForegroundColor Yellow
cargo build --release
Copy-Item "target\release\utah-browser.exe" "$BUILD_DIR\"

# Step 4: Assemble the Payload
Write-Host "[4/4] Assembling final portable structure..." -ForegroundColor Yellow
Copy-Item -Recurse "$ASSETS_DIR\*" "$BUILD_DIR\ui\"
Copy-Item "README.md" "$BUILD_DIR\READ_ME_FIRST.md"

# Package into a zip for distribution
$ZIP_FILE = "utah_browser_enterprise_v1.zip"
if (Test-Path $ZIP_FILE) { Remove-Item $ZIP_FILE }
Compress-Archive -Path "$BUILD_DIR\*" -DestinationPath $ZIP_FILE

Write-Host "[SUCCESS] Golden Master package generated: $ZIP_FILE" -ForegroundColor Green
Write-Host "[DEPLOYMENT] Ready for client distribution. Zero dependencies required." -ForegroundColor Green
