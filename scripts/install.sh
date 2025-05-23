set -e

file_dir=$(dirname "$0")
root_dir=$(realpath "$file_dir/..")

apple_watch_dir="$root_dir/apple-watch"
desktop_dir="$root_dir/desktop-app"
graph_dir="$root_dir/graph"
cli_dir="$root_dir/cli"

# install
cd $apple_watch_dir && pnpm i
cd $desktop_dir && pnpm i
cd $graph_dir && pnpm i
cd $cli_dir && pnpm i
