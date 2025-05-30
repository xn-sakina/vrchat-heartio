#!/bin/bash

echo "Building HeartIO for macOS..."

cargo build --release

echo "macOS build completed!"
echo "Executable: target/release/heartio-rust"

PACKAGE_NAME="HeartIO-macOS"
PACKAGE_DIR="target/package/$PACKAGE_NAME"
mkdir -p "$PACKAGE_DIR"

cp target/release/heartio-rust "$PACKAGE_DIR/"

echo "Package created: $PACKAGE_DIR"
echo "Contents:"
ls -la "$PACKAGE_DIR"
