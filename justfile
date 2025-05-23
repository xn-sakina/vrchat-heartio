
cli_dir := "cli"
cli_graph_dir := "cli-graph"
app_desktop_dir := "app-desktop"
app_apple_watch_dir := "app-apple-watch"

# Install all deps
install:
	cd ./{{cli_dir}} && pnpm i
	cd ./{{cli_graph_dir}} && pnpm i
	cd ./{{app_desktop_dir}} && pnpm i
	cd ./{{app_apple_watch_dir}} && pnpm i
