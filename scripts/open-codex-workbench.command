#!/bin/zsh
set -euo pipefail

APP_DIR="/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell"
TAURI_CLI="../tauri-capability-probe/.tauri-cli/bin/cargo-tauri"

# 构建缓存合并（2026-07-13）：原先把 CARGO_HOME/CARGO_TARGET_DIR 重定向到
# tauri-capability-probe 下，使主 App 的构建缓存存了两份（探针 .cargo-target ~26G +
# shell 自己的 src-tauri/target ~32G）。改为用默认 ~/.cargo 与 shell 的 src-tauri/target，
# 与命令行 `cargo` / `tauri:dev` 共用一份，省掉重复的 ~26G。仍用探针本地的 cargo-tauri CLI。

if [[ ! -d "$APP_DIR" ]]; then
  echo "App directory not found: $APP_DIR"
  exit 1
fi

cd "$APP_DIR"

if [[ ! -x "$TAURI_CLI" ]]; then
  echo "Tauri CLI not found or not executable: $APP_DIR/$TAURI_CLI"
  exit 1
fi

find_free_port() {
  local start="$1"
  local max=$(( start + 200 ))
  local port

  if (( max > 65535 )); then
    max=65535
  fi

  for (( port = start; port <= max; port++ )); do
    if ! lsof -nP -iTCP:"$port" -sTCP:LISTEN >/dev/null 2>&1; then
      print -r -- "$port"
      return 0
    fi
  done

  return 1
}

START_PORT="${CODEX_WORKBENCH_PORT:-5173}"
if [[ "$START_PORT" != <-> ]] || (( START_PORT < 1024 || START_PORT > 65535 )); then
  START_PORT=5173
fi

PORT="$(find_free_port "$START_PORT")"
CONFIG_DIR="$(mktemp -d "${TMPDIR:-/tmp}/codex-workbench-tauri-conf.XXXXXX")"
CONFIG_FILE="$CONFIG_DIR/tauri.conf.json"

cleanup() {
  rm -rf "$CONFIG_DIR"
}
trap cleanup EXIT INT TERM

cat > "$CONFIG_FILE" <<JSON
{
  "build": {
    "devUrl": "http://127.0.0.1:${PORT}",
    "beforeDevCommand": "npm run dev -- --port ${PORT} --strictPort"
  }
}
JSON

echo "Starting Codex Governance Workbench..."
echo "Window title: Codex 治理工作台"
echo "App directory: $APP_DIR"
echo "Dev URL: http://127.0.0.1:${PORT}"
if (( PORT != START_PORT )); then
  echo "Port ${START_PORT} is busy; using ${PORT} instead."
fi
echo "Close this Terminal window or press Ctrl+C to stop the dev app."
echo

if [[ "${CODEX_WORKBENCH_DRY_RUN:-}" == "1" ]]; then
  echo "Dry run only. Temporary Tauri config:"
  cat "$CONFIG_FILE"
  exit 0
fi

exec "$TAURI_CLI" dev --config "$CONFIG_FILE"
