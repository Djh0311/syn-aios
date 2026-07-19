# Gate 1 构建记录（2026-07-20 00:24 +0800）

## 命令

- cwd：`/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell`
- `../tauri-capability-probe/.tauri-cli/bin/cargo-tauri build --debug`
- 全量日志：`gate1-build.log`（vite build ✓ + cargo dev profile 1m17s，`Built application at: …/target/debug/codex-governance-workbench`，BUILD_EXIT=0）

## 产物冻结（本轮 live 唯一合法 App）

- 路径：`/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src-tauri/target/debug/codex-governance-workbench`
- SHA-256：`f9d028f3d9ebb942877506bc12b586d1d1cb60fac536a61fa4ba97b4b1db01f3`
- size：66399400；mtime：2026-07-20 00:24:05（晚于 R1 源码 mtime 2026-07-19 18:35:51 ✓）

## 含 R1/H2 论证

- cargo 增量构建：本次 lib 重新编译并重链（旧二进制 66379576B → 新 66399400B，hash 不同）；
- 五个关键源文件 SHA-256 构建前后与 Gate 0 冻结逐一相等（无构建期源码漂移）；
- R1 无独有字符串字面量可 grep（test-only 符号不进非 test 产物），以"冻结源 → 重链产物"因果链证明。

## 旧 bundle 处置

- `target/debug/bundle/macos/CodexGovernanceWorkbench.app` 内部二进制仍为 14:01 旧件
  （SHA-256 `793401a5867ec22c455f6d2aeadaf3f922abef6035f17a09f59c7be075b31273`）。
- 原因：项目 `tauri.conf.json` 既有配置 `bundle.active=false`，`cargo-tauri build` 不打 .app。
- 按禁止事项#5 该 .app 本轮禁止使用；Gate 5 用户启动 = 终端直接执行上述新裸二进制。

## 构建后复查

- `ps`（workbench/tauri/vite/cargo/rustc）：无本轮残留（PID 14847 远古 probe 与 98835 无关服务仍在，非本轮）。
- `lsof` DB 本体与 workflow-state.v0.json：无持有者（exit=1）。

**Gate 1 绿，进入 Gate 2。**
