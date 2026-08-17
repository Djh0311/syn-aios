# M5R07 项目 UI、隔离 App 与阶段候选报告

- 日期：2026-08-17
- 阶段：stage-14 / leaf M5R07
- 状态：**`AWAITING_INDEPENDENT_ACCEPTANCE`**
- **`NOT_CLOSEOUT` / `NOT_M5_COMPLETE`**
- 不宣布 M5 完成，不关闭 stage-14，不激活 M6 / F2

本包是 exact implementation `f51c3f64ed21d83730f47b26b86587e1c9b7fe6b`（tree `dbdeaedaf28f42bbbff7b38ca8764b3332929d5b`）的 fresh evidence-binding。它不是 closeout，也不是 M5 完成声明。`f51c3f64` 的产品 + Git/Harness scoped independent PASS 保持独立；本包只绑定本次实际跑出的 disposable / isolated-negative 证据。

## Harness

- 唯一 current leaf：`M5R07`
- authorization：closed（`authorized=false`）
- `stage-14` 仍开；本包不 close leaf / stage / M5
- 工作副本 HEAD 必须且实际为 leaf-only `a11f39fc2d28d12ad7475a13f9214df538ace3c5`
- 只更新并提交下面 6 个 exact paths；既有 WIP、产品源码、task、plan、stage、authorization、current-state、audit、done、M6、冻结合同均未动
- 不 reset / stash / clean / `git add -A` / push

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
| workspace leaf-only HEAD | `a11f39fc2d28d12ad7475a13f9214df538ace3c5` |
| `git archive` tar | `/tmp/m5r07-f51-fresh-evidence-iRNwEfkK/candidate.tar` |
| tar SHA256 | `b0c8bd159cab9083bfe26963c40e1f6d41713ee7e9530a738d215bdea3964b76` |
| source extract | `/tmp/m5r07-f51-fresh-evidence-iRNwEfkK/source` |
| launcher exact checkout | `/tmp/m5r07-f51-fresh-evidence-iRNwEfkK/launcher` @ `f51c3f64` / tree `dbdeaeda` / detach 后 `status` clean |
| raw launcher receipt | `/tmp/syn-r4-acceptance-8SDQME/logs/m5r07-launcher-receipt.json` |
| raw receipt SHA256 | `b2427ab97617708db8d56e95c7deb448bf4eb16b28fe37ddf1960ace1b13552d` |
| raw unavailable | `/tmp/syn-r4-acceptance-8SDQME/logs/m5r07-ui-unavailable.json` |
| raw unavailable SHA256 | `bf13fd812815ffddb6245468aa183a63da229f4f0e307df7776d4ed0f818c1f3` |
| stdout SHA256 | `c0bc19f6eef0b3d657dbd1fa6ce1fd510391d5842c39a1e7db07dd6049701022` |

本包 **不自引用尚未生成的 evidence commit SHA**。receipt 只绑定 implementation SHA / tree。

### 当前两栏（不要混读）

| 栏 | 结果 | 证明什么 | 不证明什么 |
|---|---|---|---|
| ordinary disposable backend full-loop | **PASS**（本次实际输出） | archive 出的 exact `f51c3f64` 源码上，定向 `cargo` / `npm` 与 ordinary AppState 产品命令闭环 | 不是 GUI；不是窗口；不是 isolated Tauri full-loop |
| shared isolated real-process unavailable-only | **PASS**（本次实际输出） | 真实进程 + 虚拟 X11 上 M1/M3 未安装、`open_available=false`、`full_loop_claimed=false`、`derived_from=installed_authority_slots` fail-closed | 不是 UI PASS；不是 scene A/B/resume；不是 window capture |

分类标签（isolated 栏只能用这些）：`REAL_PROCESS_VIRTUAL_X11` / `NO_WINDOW_CAPTURE` / `NO_UI_PASS`。

scene A / scene B / resume / second launch / window capture：**全部 `NOT_EXECUTED`**。当前 umbrella **删除**旧 `ui-scene-a` / `ui-scene-b` / `ui-resume` 引用，也删除旧 positive proofs（`rejection_zero_grant` / `exact_joins` / `stale` / `deep_link_resolves` / `restart_recovery` / `receipts_backend_derived` / `m3_role_session` 不得再当作 current）。

### 一、exact source / ordinary disposable（本次命令）

工作目录均为 archive 解出的 source，不是 working copy。

| 命令 | cwd | exit | 本次实际结果 |
|---|---|---|---|
| `cargo check --lib --offline` | `.../source/.../src-tauri` | 0 | `Finished dev profile in 1m 36s` |
| `cargo test --lib --offline m5_ -- --test-threads=1` | 同上 | 0 | `ok. 158 passed; 0 failed; 0 ignored; 0 measured; 1843 filtered out; finished in 91.07s` |
| `cargo test --lib --offline execution_readback_ -- --test-threads=1` | 同上 | 0 | `ok. 13 passed; 0 failed; 0 ignored; 0 measured; 1988 filtered out; finished in 0.39s` |
| `cargo test --lib --offline executed_claim_ -- --test-threads=1` | 同上 | 0 | `ok. 3 passed; 0 failed; 0 ignored; 0 measured; 1998 filtered out; finished in 0.03s` |
| `npm ci --offline --ignore-scripts` | `.../source/prototypes/productized-desktop-shell` | 0 | `added 90 packages, and audited 91 packages` |
| `npm run typecheck` | 同上 | 0 | `tsc --noEmit` 无错误 |
| `npm run build` | 同上 | 0 | vite 7.3.3；`310 modules transformed` |

`ordinary_product_loop_uses_distinct_m3_views_and_survives_reopen`：`ok`。该测试走 `AppState::try_new_with_ordinary_product_ports`，用 `register_exact_alias` 预登记 alias，再走 open / propose / approve / runtime / worker report / independent review / result / reopen。这是后端/产品命令闭环，不是 GUI。

全库 `cargo test`：**未跑、不宣称 PASS**。

### 二、shared isolated 真实进程负向（本次命令）

因 launcher 内部 `git rev-parse HEAD`，另在同一 evidence root：

1. `git clone --shared --no-checkout /home/synadmin/workspace/syn /tmp/m5r07-f51-fresh-evidence-iRNwEfkK/launcher` → exit 0
2. `git checkout --detach f51c3f64ed21d83730f47b26b86587e1c9b7fe6b` → HEAD/tree exact，detach 后 `status` clean
3. 其桌面目录 `npm ci --offline --ignore-scripts` → exit 0；`added 90 packages, and audited 91 packages`
4. 只读端口检查：`ss` 无监听，`127.0.0.1:5173` connect refused → **UNUSED**；未杀任何进程
5. 用户声明 host `DISPLAY=:0` 不可用。实际命令：

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

raw stdout / profile / logs 保留在 evidence root 与 `/tmp/syn-r4-acceptance-8SDQME`。

### 历史（不是当前证明）

旧 `df11a4a3fa19c6d91c8aaa006e395f83c155e772` 与 `faa6ed191f6bef29ddd03b74b4369c4b4e6445fd` **只列历史**，不混入当前两栏。

三个旧 scene 文件保持 byte 不动，明确为 **`faa6ed1` historical / `SUPERSEDED` / `NOT_CURRENT_EVIDENCE`**：

| 文件 | SHA256 | 当前地位 |
|---|---|---|
| `docs/harness/reports/M5R07-isolated-app-evidence/ui-scene-a.json` | `5e730c077cd8c281b729596a19e5421c74d704e539615a9d7f1d7d6b3eeb1909` | historical / SUPERSEDED / NOT_CURRENT_EVIDENCE |
| `docs/harness/reports/M5R07-isolated-app-evidence/ui-scene-b.json` | `5206360ae97f1f47b182a7fc7fdd4ec308c8446599029af4c3e8b629ecb1721a` | historical / SUPERSEDED / NOT_CURRENT_EVIDENCE |
| `docs/harness/reports/M5R07-isolated-app-evidence/ui-resume.json` | `d3970ecc535d6bec58bbccb31c21f0d6f8286aa4a137682bd71661dfa4b7f7b6` | historical / SUPERSEDED / NOT_CURRENT_EVIDENCE |

## 载体

| 项 | 值 |
|---|---|
| exact implementation candidate | `f51c3f64ed21d83730f47b26b86587e1c9b7fe6b` |
| exact tree | `dbdeaedaf28f42bbbff7b38ca8764b3332929d5b` |
| scoped predecessor | `1433d51466e59352cc8859e1c47f176da04f25b0`（gateway/Dispatch scoped PASS；不是本包证据） |
| leaf-only projection HEAD | `a11f39fc2d28d12ad7475a13f9214df538ace3c5` |
| disposable receipt | `docs/harness/reports/M5R07-disposable-checkout-receipt.json` |
| isolated launcher receipt | `docs/harness/reports/M5R07-isolated-app-launcher-receipt.json` |
| isolated unavailable receipt | `docs/harness/reports/M5R07-isolated-ui-unavailable-receipt.json` |
| umbrella isolated-acceptance | `docs/harness/reports/M5R07-isolated-acceptance-receipt.json`（current schema `v4`） |
| evidence commit SHA | 本文件写入时尚未生成；receipt **不自引用** |

## 交给独立验收

- Git：本 evidence-binding 只 add 上述 6 paths；未 push / merge / rebase；既有 WIP 仍原位未 add。
- Harness：唯一 current leaf = M5R07；authorization closed；stage-14 仍开。
- 请 Codex 只读复核 exact `f51c3f64` + tree `dbdeaeda` + 本包 fresh receipts。不要把本报告当成 M5 完成、stage closeout，或 isolated UI / window PASS。
