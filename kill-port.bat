@echo off
setlocal enabledelayedexpansion

set PORT=5173

for /f "tokens=5" %%a in ('netstat -ano ^| findstr :%PORT% ^| findstr LISTENING') do (
    echo Killing process on port %PORT%: PID %%a
    taskkill //F //PID %%a >nul 2>&1
)

for /f "tokens=5" %%a in ('netstat -ano ^| findstr :5174 ^| findstr LISTENING') do (
    echo Killing process on port 5174: PID %%a
    taskkill //F //PID %%a >nul 2>&1
)

echo Port cleanup done.
