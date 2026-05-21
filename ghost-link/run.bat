@echo off
REM Ghost-Link Zero-Click launcher (Windows)
setlocal
set "ROOT=%~dp0"
set "VAULT=%UTAH_VAULT%"
if "%VAULT%"=="" set "VAULT=%USERPROFILE%\.utah_browser"
set "GHOST_HOME=%GHOST_LINK_HOME%"
if "%GHOST_HOME%"=="" set "GHOST_HOME=%VAULT%\ghost-link"
set "VENV=%GHOST_HOME%\env"

if not exist "%GHOST_HOME%\logs" mkdir "%GHOST_HOME%\logs"
if not exist "%GHOST_HOME%\out" mkdir "%GHOST_HOME%\out"

if not exist "%VENV%\Scripts\python.exe" (
  echo [GHOST-LINK] Creating venv...
  python -m venv "%VENV%"
)
call "%VENV%\Scripts\activate.bat"
pip install -q -r "%ROOT%requirements.txt"

cd /d "%ROOT%"
if "%1"=="--foreground" goto foreground
start "" /B pythonw -m ghost_link
echo [GHOST-LINK] Background daemon started. Logs: %GHOST_HOME%\logs\telemetry.log
exit /b 0

:foreground
python -m ghost_link %*
endlocal
