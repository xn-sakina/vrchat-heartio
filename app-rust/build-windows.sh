set -e

echo "Building HeartIO for Windows x86_64..."

rustup target add x86_64-pc-windows-gnu || true

echo "Building HeartIO for Windows x86_64..."
cargo build --release --target x86_64-pc-windows-gnu

echo "Stripping binary..."
x86_64-w64-mingw32-strip target/x86_64-pc-windows-gnu/release/heartio-rust.exe || true

echo "Windows build completed!"
echo "Executable: target/x86_64-pc-windows-gnu/release/heartio-rust.exe"
