#!/bin/zsh
set -euo pipefail

APP_DIR="/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell"
TAURI_CLI="../tauri-capability-probe/.tauri-cli/bin/cargo-tauri"

export CARGO_HOME="/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-home"
export CARGO_TARGET_DIR="/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-target"

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
  # 杀掉本次 beforeDevCommand 起的 vite，否则它变僵尸占着端口，下次重启端口漂移+连回旧进程看不到新代码
  local vite_pid
  vite_pid="$(lsof -tiTCP:"$PORT" -sTCP:LISTEN 2>/dev/null || true)"
  [[ -n "$vite_pid" ]] && kill $vite_pid 2>/dev/null || true
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

# 每次启动先清掉 webview 的 HTTP 缓存，确保改了前端代码 WKWebView 不会拿旧页面/旧模块。
# (这是“改了代码、桌面端看不到变化”的头号元凶；真实数据在 Application Support，不受影响。)
# 2026-07-14 修:此前只清进程名目录=清错窝——WKWebView 真缓存跟 bundle id(local.codex.governance.workbench)
# 走,且 ~/Library/WebKit/ 下的网络/模块缓存此前完全没清(「改了看不到」复发的真元凶)。四个窝全清:
rm -rf "$HOME/Library/Caches/codex-governance-workbench" \
       "$HOME/Library/Caches/local.codex.governance.workbench" \
       "$HOME/Library/WebKit/codex-governance-workbench" \
       "$HOME/Library/WebKit/local.codex.governance.workbench" 2>/dev/null || true

exec "$TAURI_CLI" dev --config "$CONFIG_FILE"
