# HeartIO Rust 实现

这是 HeartIO 项目的 Rust 重写版本，提供跨平台的心率监控功能，支持蓝牙心率设备和 Apple Watch。

## 功能特性

- ✅ 蓝牙 LE 心率设备监控（支持标准心率服务）
- ✅ OSC 消息发送（用于 VRChat 等应用）
- ✅ SQLite 数据库存储心率数据
- ✅ HTTP 服务器支持 Apple Watch 数据接收
- ✅ 实时 GUI 界面显示日志和状态
- ✅ 可配置的心率阈值和标签
- ✅ 系统资源优化（防止休眠）
- ✅ 跨平台支持（Windows 和 macOS）

## 系统要求

### macOS
- macOS 10.15+ (Catalina 或更新版本)
- 支持蓝牙 4.0+ (LE) 的 Mac 设备
- 蓝牙访问权限（首次运行时系统会提示）

### Windows
- Windows 10 版本 1703 或更新版本
- 支持蓝牙 4.0+ (LE) 的 Windows 设备
- 蓝牙访问权限（在 Windows 设置中允许）

## 构建说明

### 依赖要求
```bash
# 安装 Rust (如果尚未安装)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# macOS 依赖
# 无需额外依赖，系统内置 Core Bluetooth 框架

# Windows 依赖（可选，用于交叉编译）
rustup target add x86_64-pc-windows-gnu
```

### 构建命令

#### macOS 本地构建
```bash
./build-macos.sh
```

#### Windows 本地构建
```bash
build-windows.bat
```

#### Windows 交叉编译（在 macOS/Linux 上）
```bash
./build-windows.sh
```

#### 手动构建
```bash
# Debug 版本
cargo build

# Release 版本
cargo build --release

# 特定平台
cargo build --release --target x86_64-pc-windows-gnu  # Windows
cargo build --release --target x86_64-apple-darwin    # macOS Intel
cargo build --release --target aarch64-apple-darwin   # macOS Apple Silicon
```

## 使用说明

### 配置文件

应用启动时会在可执行文件同一目录下查找 `heartio.config.json` 配置文件。如果不存在，会自动创建默认配置。

示例配置：
```json
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
```

### 运行应用

#### macOS
```bash
./heartio-rust
```

#### Windows
```bash
heartio-rust.exe
```

或直接双击可执行文件。

### GUI 界面

应用启动后会显示一个实时日志窗口，显示：
- 连接状态
- 心率数据
- OSC 消息发送状态
- 错误和警告信息

### 蓝牙设备连接

1. 确保心率设备已开启并处于配对模式
2. 应用会自动扫描并连接符合标准的心率设备
3. 如需连接特定设备，在配置文件中设置 `device_name`

### Apple Watch 支持

1. 确保配置中启用了 `apple_watch_support`
2. 使用 HTTP POST 请求发送心率数据到 `http://localhost:3000/heartrate`
3. 请求格式：`{"heartRate": 75}`

### OSC 集成

应用会将心率数据通过 OSC 协议发送到指定地址，可用于：
- VRChat 头像参数控制
- 其他支持 OSC 的应用

发送的参数：
- `HeartRate`: 当前心率值（整数）
- `HeartRateZone`: 心率区间（0-4，对应很低到很高）

## 开发说明

### 代码结构

```
src/
├── main.rs              # 应用入口点
├── config.rs            # 配置文件管理
├── bluetooth.rs         # 蓝牙心率设备监控
├── osc.rs              # OSC 消息发送
├── database.rs         # SQLite 数据库操作
├── server.rs           # HTTP 服务器（Apple Watch 支持）
├── system.rs           # 系统工具（防止休眠等）
├── gui.rs              # 实时 GUI 界面
└── heart_rate.rs       # 心率监控协调器
```

### 主要依赖

- `btleplug`: 跨平台蓝牙 LE 库
- `rosc`: OSC 协议实现
- `sqlx`: 异步 SQL 数据库访问
- `axum`: 现代 HTTP 服务器框架
- `egui/eframe`: 跨平台 GUI 框架
- `tokio`: 异步运行时
- `serde_json`: JSON 序列化

### 与 TypeScript 版本的差异

1. **配置管理**: 使用 JSON 文件代替 `.env` 文件
2. **数据库**: 使用 `sqlx` 代替 `better-sqlite3`
3. **HTTP 服务器**: 使用 `axum` 代替 Express
4. **蓝牙**: 使用 `btleplug` 代替 `@stoprocent/noble`
5. **OSC**: 使用 `rosc` 代替 `node-osc`
6. **GUI**: 新增实时日志显示界面
7. **内存管理**: Rust 自动内存管理，无需内存泄漏检测

## 故障排除

### 常见问题

1. **蓝牙权限问题**
   - macOS: 在系统偏好设置 > 安全性与隐私 > 隐私 > 蓝牙中允许应用访问
   - Windows: 在设置 > 隐私 > 蓝牙中允许应用访问

2. **找不到心率设备**
   - 确保设备处于配对模式
   - 检查设备是否支持标准心率服务 (UUID: 180D)
   - 重启蓝牙服务

3. **OSC 消息发送失败**
   - 检查目标应用是否在监听指定端口
   - 确认防火墙设置
   - 验证 IP 地址和端口配置

4. **Apple Watch 连接问题**
   - 确保 HTTP 服务器正常启动
   - 检查端口是否被占用
   - 验证请求格式

### 日志查看

应用运行时会在 GUI 界面显示详细日志。如需更多调试信息，可以设置环境变量：

```bash
RUST_LOG=debug ./heartio-rust
```

## 许可证

遵循与原 TypeScript 项目相同的许可证。

## 贡献

欢迎提交 Issue 和 Pull Request！
