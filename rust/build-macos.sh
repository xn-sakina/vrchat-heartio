#!/bin/bash

# macOS 本地构建脚本
# 创建 macOS 单一可执行文件

echo "Building HeartIO for macOS..."

# 构建 Release 版本
cargo build --release

echo "macOS build completed!"
echo "Executable: target/release/heartio-rust"

# 创建打包文件夹
PACKAGE_NAME="HeartIO-macOS"
PACKAGE_DIR="target/package/$PACKAGE_NAME"
mkdir -p "$PACKAGE_DIR"

# 复制可执行文件
cp target/release/heartio-rust "$PACKAGE_DIR/"

# 创建示例配置文件
cat > "$PACKAGE_DIR/heartio.config.json" << 'EOF'
{
  "heart_rate": {
    "thresholds": {
      "very_low": 50,
      "low": 60,
      "normal": 100,
      "high": 140,
      "very_high": 180
    },
    "labels": {
      "very_low": "很低",
      "low": "低",
      "normal": "正常",
      "high": "高",
      "very_high": "很高"
    }
  },
  "osc": {
    "host": "127.0.0.1",
    "port": 9000,
    "address": "/avatar/parameters/HeartRate",
    "address_zone": "/avatar/parameters/HeartRateZone"
  },
  "bluetooth": {
    "device_name": "",
    "auto_connect": true,
    "connection_timeout": 10000
  },
  "server": {
    "port": 3000,
    "apple_watch_support": true
  },
  "system": {
    "prevent_sleep": true,
    "gui_enabled": true
  }
}
EOF

# 创建 README
cat > "$PACKAGE_DIR/README.md" << 'EOF'
# HeartIO for macOS

## 使用方法

1. 将 `heartio-rust` 可执行文件放在任意目录
2. 在同一目录下创建或编辑 `heartio.config.json` 配置文件
3. 运行：`./heartio-rust`

## 配置说明

- `heart_rate`: 心率阈值和标签配置
- `osc`: OSC 消息发送配置
- `bluetooth`: 蓝牙设备连接配置
- `server`: Apple Watch 支持的 HTTP 服务器配置
- `system`: 系统设置（防止休眠、GUI界面）

## 依赖要求

- macOS 10.15+ (Catalina 或更新版本)
- 支持蓝牙 LE 的 Mac 设备

如果第一次运行需要授权蓝牙访问权限，请在系统偏好设置中允许。
EOF

echo "Package created: $PACKAGE_DIR"
echo "Contents:"
ls -la "$PACKAGE_DIR"
