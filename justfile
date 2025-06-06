cli_dir := "cli"
cli_graph_dir := "cli-graph"
app_apple_watch_dir := "app-apple-watch"
app_rust_dir := "app-rust"
app_python_dir := "app-python"

python_cmd := `which python || which python3`

# Install all deps
install:
	cd ./{{cli_dir}} && pnpm i
	cd ./{{cli_graph_dir}} && pnpm i
	cd ./{{app_apple_watch_dir}} && pnpm i

# Build Rust APP for MacOS
build-macos:
	cd ./{{app_rust_dir}} && ./build-macos.sh

# Build Rust APP for Windows
build-windows:
	cd ./{{app_rust_dir}} && ./build-windows.sh

# Run Nodejs Version CLI
start:
	cd ./{{cli_dir}} && pnpm start

# Run Graph
start-graph:
	cd ./{{cli_graph_dir}} && \
	{{python_cmd}} ./scripts/export.py && \
	pnpm dev
	