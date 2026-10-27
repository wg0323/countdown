@echo off
setlocal
cd /d "%~dp0"

set "EXE=src-tauri\target\release\countdown.exe"
set "NSIS_DIR=src-tauri\target\release\bundle\nsis"

echo ==========================================
echo  Countdown - build script
echo    build.bat      build installer (falls back to exe)
echo    build.bat exe  build portable exe only
echo ==========================================
echo.

where node >nul 2>nul
if errorlevel 1 (
    echo [ERROR] Node.js not found in PATH.
    goto :fail
)

if exist "node_modules" (
    echo [1/3] dependencies already installed
) else (
    echo [1/3] installing dependencies ...
    call npm install --no-audit --no-fund
    if errorlevel 1 (
        echo [ERROR] npm install failed.
        goto :fail
    )
)

tasklist /fi "IMAGENAME eq countdown.exe" 2>nul | find /i "countdown.exe" >nul
if not errorlevel 1 echo [WARN] countdown.exe is running; close it if the build fails with "file in use".

if /i "%~1"=="exe" goto :build_exe

echo [2/3] building installer ...
call npm run build
if errorlevel 1 (
    echo.
    echo [WARN] installer build failed - usually the NSIS download timed out.
    echo        falling back to portable exe ...
    goto :build_exe
)
echo [3/3] done.
echo.
echo installer dir : %NSIS_DIR%
for %%f in ("%NSIS_DIR%\*.exe") do echo   - %%~nxf
echo executable    : %EXE%
goto :done

:build_exe
echo [2/3] building portable exe ...
call npm run build:exe
if errorlevel 1 (
    echo [ERROR] build failed, see cargo output above.
    goto :fail
)
echo [3/3] done.
echo.
echo executable : %EXE%
for %%f in ("%EXE%") do echo size       : %%~zf bytes
goto :done

:fail
set "RC=1"

:done
echo.
pause
exit /b %RC%
