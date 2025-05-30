#!/bin/bash

# Windows 交叉编译脚本
# 需要先安装 Windows 交叉编译工具链：rustup target add x86_64-pc-windows-gnu

echo "Building HeartIO for Windows x86_64..."

# 添加 Windows 目标平台
rustup target add x86_64-pc-windows-gnu

# 设置交叉编译环境变量
export CC_x86_64_pc_windows_gnu=x86_64-w64-mingw32-gcc
export CXX_x86_64_pc_windows_gnu=x86_64-w64-mingw32-g++
export AR_x86_64_pc_windows_gnu=x86_64-w64-mingw32-ar

# 编译为 Windows 可执行文件
cargo build --release --target x86_64-pc-windows-gnu

echo "Windows build completed!"
echo "Executable: target/x86_64-pc-windows-gnu/release/heartio-rust.exe"
