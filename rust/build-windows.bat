@echo off
REM Windows 本地构建脚本

echo Building HeartIO for Windows...

REM 构建 Release 版本
cargo build --release

echo Windows build completed!
echo Executable: target\release\heartio-rust.exe

REM 创建打包文件夹
set PACKAGE_NAME=HeartIO-Windows
set PACKAGE_DIR=target\package\%PACKAGE_NAME%
mkdir "%PACKAGE_DIR%" 2>nul

REM 复制可执行文件
copy target\release\heartio-rust.exe "%PACKAGE_DIR%\"

REM 创建示例配置文件
(
echo {
echo   "heart_rate": {
echo     "thresholds": {
echo       "very_low": 50,
echo       "low": 60,
echo       "normal": 100,
echo       "high": 140,
echo       "very_high": 180
echo     },
echo     "labels": {
echo       "very_low": "很低",
echo       "low": "低",
echo       "normal": "正常",
echo       "high": "高",
echo       "very_high": "很高"
echo     }
echo   },
echo   "osc": {
echo     "host": "127.0.0.1",
echo     "port": 9000,
echo     "address": "/avatar/parameters/HeartRate",
echo     "address_zone": "/avatar/parameters/HeartRateZone"
echo   },
echo   "bluetooth": {
echo     "device_name": "",
echo     "auto_connect": true,
echo     "connection_timeout": 10000
echo   },
echo   "server": {
echo     "port": 3000,
echo     "apple_watch_support": true
echo   },
echo   "system": {
echo     "prevent_sleep": true,
echo     "gui_enabled": true
echo   }
echo }
) > "%PACKAGE_DIR%\heartio.config.json"

REM 创建 README
(
echo # HeartIO for Windows
echo.
echo ## 使用方法
echo.
echo 1. 将 `heartio-rust.exe` 可执行文件放在任意目录
echo 2. 在同一目录下创建或编辑 `heartio.config.json` 配置文件
echo 3. 双击运行 `heartio-rust.exe` 或在命令行中运行
echo.
echo ## 配置说明
echo.
echo - `heart_rate`: 心率阈值和标签配置
echo - `osc`: OSC 消息发送配置
echo - `bluetooth`: 蓝牙设备连接配置
echo - `server`: Apple Watch 支持的 HTTP 服务器配置
echo - `system`: 系统设置（防止休眠、GUI界面）
echo.
echo ## 依赖要求
echo.
echo - Windows 10 版本 1703 或更新版本（支持蓝牙 LE）
echo - 支持蓝牙 4.0+ 的 Windows 设备
echo.
echo 如果第一次运行需要授权蓝牙访问权限，请在 Windows 设置中允许。
) > "%PACKAGE_DIR%\README.md"

echo Package created: %PACKAGE_DIR%
echo Contents:
dir "%PACKAGE_DIR%"
