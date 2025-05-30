
cli_dir := "cli"
cli_graph_dir := "cli-graph"
app_apple_watch_dir := "app-apple-watch"
app_rust_dir := "app-rust"

# Install all deps
install:
	cd ./{{cli_dir}} && pnpm i
	cd ./{{cli_graph_dir}} && pnpm i
	cd ./{{app_apple_watch_dir}} && pnpm i

# Build Rust APP
build-macos:
	cd ./{{app_rust_dir}} && ./build-macos.sh

build-windows:
	cd ./{{app_rust_dir}} && ./build-windows.sh
