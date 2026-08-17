# M5R07 项目 UI、隔离 App 与阶段候选报告

- 日期：2026-08-17
- 阶段：stage-14 / leaf M5R07
- 状态：**`AWAITING_INDEPENDENT_ACCEPTANCE`**
- **`NOT_CLOSEOUT` / `NOT_M5_COMPLETE`**
- 不宣布 M5 完成，不关闭 stage-14，不激活 M6 / F2
- 本文件是 `5d023c6309e968da18f7d0791609f8c244fae338` 的 evidence carrier 追加修正，不是独立验收通过，也不是 closeout。

`f51c3f64ed21d83730f47b26b86587e1c9b7fe6b`（tree `dbdeaedaf28f42bbbff7b38ca8764b3332929d5b`）产品实现与 fresh root `/tmp/m5r07-f51-fresh-evidence-iRNwEfkK` 实跑结果保持不变。本轮不改产品源码、不重跑真实业务。`f51c3f64` 的产品 + Git/Harness scoped independent PASS 保持独立；本修正只补载体。

## Harness

- 唯一 current leaf：`M5R07`
- authorization：closed（`authorized=false`）
- `stage-14` 仍开；本包不 close leaf / stage / M5
- 5d 错误把 current leaf 混入 evidence 包并改写用户来源。本 correction 将 `docs/harness/leaves/M5R07-project-ui-isolated-app-and-stage-candidate.md` 精确恢复到 `a11f39fc2d28d12ad7475a13f9214df538ace3c5` 的 blob `3f3d5d0c4ba58008931bddd90ad5c97dcba8740c`。本轮不再做 leaf projection。
- cumulative `a11f39fc..tip` leaf net unchanged（零 diff）
- 不 reset / stash / clean / `git add -A` / push
- 不改 plan / stage / authorization / current-state / task / 产品源码 / 其他 WIP
- `docs/harness/reports/M5R07-isolated-ui-unavailable-receipt.json` 未改：JSON 一致性核对其无旧 scene path / hash / positive-proof 直接引用，不必须纳入本 correction

## 产品

`f51c3f64` 是 terminal final implementation candidate，已获产品 + Git/Harness **scoped independent PASS**。本包不重跑、不改写该 scoped PASS，也不把它升级成 closeout。

产品范围仍是正式 M5 authority → Grant → Dispatch readback → runtime → `RecordExecutionAttemptReadback` → terminal-gated EXECUTED claim。ordinary M1 composition 与 M6 排除。

必须分开读的两件事：

1. **ordinary disposable AppState 测试**（含 `ordinary_product_loop_uses_distinct_m3_views_and_survives_reopen`）是 **server fixture 预登记 M1 alias + M3 authority 的后端/产品命令闭环**，不是 GUI，不是窗口，不是截图。
2. **shared isolated 真实 Tauri 进程**只证明 `try_new_with_isolated_product_profile` 按冻结合同 `m1-m3-shared-appstate-acceptance-profile-isolation-v1.md` 不安装 M1/M3，因此 `open_available=false`、`full_loop_claimed=false`、authority unavailable **fail-closed**。window / UI screenshot 为 **`NOT_EXECUTED`**。

**ordinary M1 legacy `ProjectRecord` → canonical/alias composition 仍是 stage-14 blocker**，排除于 M5 实现。不得把该 blocker 反向写成 `f51c3f64` 产品 FAIL，也不得据此 close stage。

## 证据

Fresh evidence root（保留供复核）：`/tmp/m5r07-f51-fresh-evidence-iRNwEfkK`

| 项 | 本次实际值 |
|---|---|
| implementation SHA | `f51c3f64ed21d83730f47b26b86587e1c9b7fe6b` |
| implementation tree | `dbdeaedaf28f42bbbff7b38ca8764b3332929d5b` |
| workspace leaf-only HEAD at fresh run | `a11f39fc2d28d12ad7475a13f9214df538ace3c5` |
| `git archive` tar | `/tmp/m5r07-f51-fresh-evidence-iRNwEfkK/candidate.tar` |
| tar SHA256 | `b0c8bd159cab9083bfe26963c40e1f6d41713ee7e9530a738d215bdea3964b76` |
| source extract | `/tmp/m5r07-f51-fresh-evidence-iRNwEfkK/source` |
| launcher exact checkout | `/tmp/m5r07-f51-fresh-evidence-iRNwEfkK/launcher` @ `f51c3f64` / tree `dbdeaeda` |
| `pre_run_after_detach_clean` | `true` |
| `post_run_tracked_diff` | `false` |
| raw launcher receipt | `/tmp/syn-r4-acceptance-8SDQME/logs/m5r07-launcher-receipt.json` |
| raw receipt SHA256 | `b2427ab97617708db8d56e95c7deb448bf4eb16b28fe37ddf1960ace1b13552d` |
| raw unavailable | `/tmp/syn-r4-acceptance-8SDQME/logs/m5r07-ui-unavailable.json` |
| raw unavailable SHA256 | `bf13fd812815ffddb6245468aa183a63da229f4f0e307df7776d4ed0f818c1f3` |
| stdout SHA256 | `c0bc19f6eef0b3d657dbd1fa6ce1fd510391d5842c39a1e7db07dd6049701022` |

本包 **不自引用尚未生成的 evidence commit SHA**。receipt 只绑定 implementation SHA / tree。本 correction 不是独立验收通过。

### 当前两栏（不要混读）

| 栏 | 结果 | 证明什么 | 不证明什么 |
|---|---|---|---|
| ordinary disposable backend full-loop | **PASS**（fresh 实跑，未重跑） | archive 出的 exact `f51c3f64` 源码上，定向 `cargo` / `npm` 与 ordinary AppState 产品命令闭环 | 不是 GUI；不是窗口；不是 isolated Tauri full-loop |
| shared isolated real-process unavailable-only | **PASS**（fresh 实跑，未重跑） | 真实进程 + 虚拟 X11 上 M1/M3 未安装、`open_available=false`、`full_loop_claimed=false`、`derived_from=installed_authority_slots` fail-closed | 不是 UI PASS；不是 scene A/B/resume；不是 window capture |

分类标签（isolated 栏只能用这些）：`REAL_PROCESS_VIRTUAL_X11` / `NO_WINDOW_CAPTURE` / `NO_UI_PASS`。

scene A / scene B / resume / second launch / window capture：**全部 `NOT_EXECUTED`**。当前 umbrella 只承载上述两栏；旧 scene path / hash / positive-proof 不再直接引用。历史只指向 `docs/harness/reports/M5R07-history/faa6ed1/manifest.json`（`excluded_from_current_evidence=true`）。

### 一、exact source / ordinary disposable（fresh 命令 + 本轮补齐 raw log）

工作目录均为 archive 解出的 source，不是 working copy。log SHA256 从 fresh root 现有日志实算，不是截断 hash。

| 命令 | cwd | exit / exit_path | log_path | log SHA256 |
|---|---|---|---|---|
| `cargo check --lib --offline` | `.../source/.../src-tauri` | 0 / `logs/cargo-check-lib-offline.exit` | `logs/cargo-check-lib-offline.log` | `9891d6fbd635119f6d17d52e844e03992a9c7ddf2ea43743c505108265bbfcdc` |
| `cargo test --lib --offline m5_ -- --test-threads=1` | 同上 | 0 / `logs/cargo-test-lib-offline-m5_.exit` | `logs/cargo-test-lib-offline-m5_.log` | `5328f67fdb2fffa4fd7b8ca1db23afc1fb39fb0fd6525843a3bf61fba5e59f56` |
| `cargo test --lib --offline execution_readback_ -- --test-threads=1` | 同上 | 0 / `logs/cargo-test-lib-offline-execution_readback_.exit` | `logs/cargo-test-lib-offline-execution_readback_.log` | `9fb24541c04309811016eb4f1c00366d495d663f26bda5221b96f86c3ae91007` |
| `cargo test --lib --offline executed_claim_ -- --test-threads=1` | 同上 | 0 / `logs/cargo-test-lib-offline-executed_claim_.exit` | `logs/cargo-test-lib-offline-executed_claim_.log` | `8ae4415c04ae25190946c57fe57e8127d066d87c19bdbc04275654b660c96765` |
| `npm ci --offline --ignore-scripts` | `.../source/prototypes/productized-desktop-shell` | 0 / `logs/source-npm-ci-offline.exit` | `logs/source-npm-ci-offline.log` | `61af5c7261c029e4169dab5e96497ca88d8e039041cb621d96dbb788ff4c0cc3` |
| `npm run typecheck` | 同上 | 0 / `logs/source-npm-typecheck.exit` | `logs/source-npm-typecheck.log` | `7c6406bc913dea2176ae15987bb4f4995031066c29195bce737e3fde360f90f0` |
| `npm run build` | 同上 | 0 / `logs/source-npm-build.exit` | `logs/source-npm-build.log` | `b472e5a5425a451ece337f7112f640ed804cb6ffe957491cf41f435be2d2588d` |

上述 `logs/` 均位于 `/tmp/m5r07-f51-fresh-evidence-iRNwEfkK/logs/`。`exit` 绑定同名 `.exit` 文件（内容 `0\n`）。命令摘要未变：`Finished dev profile in 1m 36s`；`ok. 158 passed; 0 failed; 0 ignored; 0 measured; 1843 filtered out; finished in 91.07s`；`ok. 13 passed ... 1988 filtered ... 0.39s`；`ok. 3 passed ... 1998 filtered ... 0.03s`；`added 90 packages, and audited 91 packages`；`tsc --noEmit` 无错误；vite 7.3.3，`310 modules transformed`。

`ordinary_product_loop_uses_distinct_m3_views_and_survives_reopen`：`ok`。该测试走 `AppState::try_new_with_ordinary_product_ports`，用 `register_exact_alias` 预登记 alias，再走 open / propose / approve / runtime / worker report / independent review / result / reopen。这是后端/产品命令闭环，不是 GUI。

全库 `cargo test`：**未跑、不宣称 PASS**。

### 二、shared isolated 真实进程负向（fresh 命令 + 本轮补齐 preparation / final state）

因 launcher 内部 `git rev-parse HEAD`，另在同一 evidence root：

1. `git clone --shared --no-checkout /home/synadmin/workspace/syn /tmp/m5r07-f51-fresh-evidence-iRNwEfkK/launcher` → exit 0
2. `git checkout --detach f51c3f64ed21d83730f47b26b86587e1c9b7fe6b` → HEAD/tree exact；`pre_run_after_detach_clean=true`
3. source target reuse symlink：`prototypes/productized-desktop-shell/src-tauri/target` → `/tmp/m5r07-f51-fresh-evidence-iRNwEfkK/source/prototypes/productized-desktop-shell/src-tauri/target`
4. launcher `npm ci --offline --ignore-scripts` → exit 0；`added 90 packages, and audited 91 packages`；`log_path=/tmp/m5r07-f51-fresh-evidence-iRNwEfkK/logs/launcher-npm-ci-offline.log`；`log_sha256=f6b3d37d0e75a22e330744fb9f9580b68a0ef9e3119dae0b6246e35710edd3d6`；`exit_path=.../launcher-npm-ci-offline.exit`
5. 独立 `cargo build --offline --bins`：cwd `/tmp/m5r07-f51-fresh-evidence-iRNwEfkK/launcher/prototypes/productized-desktop-shell/src-tauri`；argv `["cargo","build","--offline","--bins"]`；exit 0；`log_path=/tmp/m5r07-f51-fresh-evidence-iRNwEfkK/logs/launcher-cargo-build-bins-offline.log`；`log_sha256=840b3b42dd8c3eaf57c12d76717cb869b23581a501419631b7f8db568baf3698`；`exit_path=.../launcher-cargo-build-bins-offline.exit`
6. 只读端口检查：`ss` 无监听，`127.0.0.1:5173` connect refused → **UNUSED**；未杀任何进程
7. host `DISPLAY=:0` 未使用；本次选择 Xvfb :99。实际命令：

```text
timeout 240s xvfb-run -a -s "-screen 0 1280x800x24" node scripts/run-m5-isolated-app-acceptance.mjs
```

cwd：`/tmp/m5r07-f51-fresh-evidence-iRNwEfkK/launcher/prototypes/productized-desktop-shell`

started `2026-08-17T22:42:16+08:00` / ended `2026-08-17T22:42:23+08:00` / **exit 0**

进程 receipt `display`：`:99`

`m5-x11-screenshot.py`：**未调用**

进程事实：

- `m1_authority_installed=false`
- `m3_authority_installed=false`
- `open_available=false`
- `full_loop_claimed=false`
- `derived_from=installed_authority_slots`
- `isolated_authority_unavailable=true`
- `scene_a` / `scene_b` / `resume` = `null`（无 scene 文件）
- `window_scene_b` / `window_resume` = `null`
- `receipts_backend_derived=false`（scene 未执行；fail-closed 来自 installed authority slots，不是旧 scene backend_store 证明）

跑后状态（tracked 无 diff；仅下列 2 项 generated untracked）：

- `post_run_tracked_diff=false`
- `post_run_generated_untracked` 恰好 2 项（相对 launcher checkout）：
  - `prototypes/productized-desktop-shell/src-tauri/target`（source target reuse symlink）
  - `prototypes/productized-desktop-shell/src-tauri/gen/schemas/linux-schema.json`

raw stdout / profile / logs 保留在 evidence root 与 `/tmp/syn-r4-acceptance-8SDQME`。

### 历史（不是当前证明）

历史只引用 `docs/harness/reports/M5R07-history/faa6ed1/manifest.json`。该 manifest：`candidate_sha=faa6ed191f6bef29ddd03b74b4369c4b4e6445fd`；`evidence_binding_commit_sha=f05d47b5d4ce8843dcca3bf3aa203948bacfa8cf`；`status=HISTORICAL_SUPERSEDED_NON_AUTHORITY`；`current_evidence=false`；`excluded_from_current_evidence=true`。禁止把它写成 current f51 evidence、M1/M3 authority PASS 或 isolated full-loop PASS。本报告不再列出 generic old scene 表。

## 载体

| 项 | 值 |
|---|---|
| exact implementation candidate | `f51c3f64ed21d83730f47b26b86587e1c9b7fe6b`（未改） |
| exact tree | `dbdeaedaf28f42bbbff7b38ca8764b3332929d5b` |
| scoped predecessor | `1433d51466e59352cc8859e1c47f176da04f25b0`（gateway/Dispatch scoped PASS；不是本包证据） |
| rejected carrier | `5d023c6309e968da18f7d0791609f8c244fae338` |
| leaf restore | a11 blob `3f3d5d0c4ba58008931bddd90ad5c97dcba8740c`；`a11..tip` net leaf unchanged |
| disposable receipt | `docs/harness/reports/M5R07-disposable-checkout-receipt.json` |
| isolated launcher receipt | `docs/harness/reports/M5R07-isolated-app-launcher-receipt.json` |
| isolated unavailable receipt | `docs/harness/reports/M5R07-isolated-ui-unavailable-receipt.json`（本轮未改） |
| umbrella isolated-acceptance | `docs/harness/reports/M5R07-isolated-acceptance-receipt.json`（current schema `v4`） |
| historical manifest | `docs/harness/reports/M5R07-history/faa6ed1/manifest.json` |
| evidence commit SHA | 本文件写入时尚未生成；receipt **不自引用** |

本 correction 将显式 stage：leaf 恢复、current report、disposable、launcher、umbrella、历史目录 / rename / manifest。不把本 correction 写成独立验收通过或 closeout。

## 交给独立验收

- Git：本 correction 只 add 上列路径；未 push / merge / rebase / amend；既有 WIP 仍原位未 add。
- Harness：唯一 current leaf = M5R07；authorization closed；stage-14 仍开；状态仍为 `AWAITING_INDEPENDENT_ACCEPTANCE`。
- 请 Codex 只读复核本 correction 载体。不要把本报告当成 M5 完成、stage closeout、独立验收通过，或 isolated UI / window PASS。
