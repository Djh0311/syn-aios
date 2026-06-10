#!/bin/zsh
set -e

APP_DIR="/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell"
export CARGO_HOME="/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-home"
export CARGO_TARGET_DIR="/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-target"

cd "$APP_DIR"

echo "Starting Codex Governance Workbench..."
echo "Window title: Codex 治理工作台"
echo "Close this Terminal window or press Ctrl+C to stop the dev app."
echo

npm run tauri:dev
