@echo off
echo 关闭占用 5173 端口的进程...

for /f "tokens=5" %%a in ('netstat -ano ^| findstr :5173 ^| findstr LISTENING') do (
    echo 找到进程 PID: %%a
    taskkill //F //PID %%a
)

for /f "tokens=5" %%a in ('netstat -ano ^| findstr :5174 ^| findstr LISTENING') do (
    echo 找到进程 PID: %%a
    taskkill //F //PID %%a
)

echo 完成
pause
