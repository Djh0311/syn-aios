# Harness HG-3：U-Gate 去重检查焊进 shape-gate（warning-only）

## 任务名

给 `workbench-shape-gate.js` 加一个 check：扫 `utils/` 之外有没有重复定义已收敛 helper，命中出 **warning（不挡）**；带白名单避免对 R-U 有意 deferred 的项误报。

## 所属开发线

开发治理 harness 改进线（Claude 开发线，worktree `harness-hg`）。基于 HG-1 commit dd0e372。

## 背景

R-U 已把 7 个 helper 收敛进 `src-tauri/src/utils/`：`sha256_hex` / `sha256_hex_bytes` / `short_hash` / `short_hash12` / `sidecar_path` / `remove_file_if_exists` / `normalize_slash_lowercase`。但缺一道门防止有人在 utils 外又写一份。本包加这道 U-Gate（warning-only，不升级 error，不接 CI）。

实测当前形状（worktree 内 grep，HG-3 依据）：6 个 helper 只在 `utils/` 有定义；`sidecar_path` 在 12 个 store 文件还有 per-store 定义，但都是 `store_paths::sidecar_path(path, SIDECAR_NAME, "...")` 的薄委托封装——R-U 有意 deferred（"各店留在原文件的 SIDECAR_NAME 常量"）。故白名单必须含这 12 个，才能保证当前代码 0 warning。divergent `fn normalize`、`fixture_dir`/`manifest_*_fixture_root` 簇名字不同，精确名扫描天然不命中。

## 目标

1. 在 `workbench-shape-gate.js` 加 `scanHelperDuplicates` check + `DEDUP_DEFER_WHITELIST`（12 条 deferred sidecar_path）：扫 `prototypes/productized-desktop-shell/{src-tauri/src,src}` 里 `utils/` 之外对这 7 个名字的定义，命中→`warn`（id `converged_helper_redefined`）。
2. 加一份自测 `workbench-shape-gate.dedup.selftest.js`：临时夹具证明 dup→warn+pass、whitelist→不报，不碰真实产品代码。

## 允许读取

`scripts/harness/workbench-shape-gate.js`、`prototypes/productized-desktop-shell/**`（只读，为设计扫描规则与白名单）。

## 允许写入

`scripts/harness/workbench-shape-gate.js`（**只加 check + 白名单**）、`scripts/harness/workbench-shape-gate.dedup.selftest.js`（新增自测）、本任务包。

## 禁止事项

改产品代码（src-tauri/prototypes）；动 shape-gate 现有任何 ratchet/waterline/command/sidecar 检查（一字不改，只追加）；把去重做成 `error`；接 CI；开 hooks；删脚本。

## 形状影响

- 任务类型：治理任务包（改 harness 门脚本，非产品代码）。
- 新增代码落点：`workbench-shape-gate.js` 追加 ~45 行；新增自测脚本 ~90 行。
- 是否触碰棘轮文件：触碰 `workbench-shape-gate.js` 本体，但**只追加新 check**，不改其输出的 ratchet list / waterline / 现有 finding。
- 预计行数变化：gate 413→约 458 行（仍 < 500 软上限，不触发 `gate_script_soft_limit_exceeded`）。
- 是否新增 Tauri command：否。
- 是否新增 sidecar JSON 种类：否。
- 是否需要 shape gate 豁免：否。
- 本任务基线 commit：dd0e372。
- 本任务完成 commit：见执行结果。

## 验收标准

- 对当前 worktree（U1–U5 已收敛）跑 `--mode check`：Status `pass`、`converged_helper_redefined` warning = 0、`deferred-whitelisted` = 12。
- 自测：临时夹具放一个 utils 外的 `fn sha256_hex`→出 1 个 `converged_helper_redefined` warning 且默认模式 Status 仍 `pass`；把同名放进白名单路径→不报；清理夹具→复原。
- **shape-gate 原有检查行为不变**：对当前代码，去重 check 之外的 finding 集合与改动前逐条一致（用 `--json` 比对）。
- `git diff --check` 干净。

## 执行与验证结果

做了什么：在 `workbench-shape-gate.js` 追加 `DEDUP_HELPERS`/`DEDUP_DEFER_WHITELIST`(12) + `scanHelperDuplicates()` + 一处 `warn` finding(`converged_helper_redefined`) + 一行 print 指标；新增自测 `workbench-shape-gate.dedup.selftest.js`。**未改任何现有检查。**

自验原始结论（worktree `harness-hg`）：

- 行数：485（< 500 软上限，不触发 `gate_script_soft_limit_exceeded`）。
- 当前代码 `--mode check`：`Status: pass`，`converged_helper_redefined` warning = 0，`deferred-whitelisted` = 12（即 12 个 per-store sidecar_path 委托封装）。
- 原有行为不变：改动前后 `summary` 均 `{pass,0,0,9}`；去掉新 check 后**现有 findings 逐条 byte-identical = true**。
- 自测 8/8 通过：utils 外重复 `fn sha256_hex`→恰好 1 个 warning、severity=warn(非 error)、默认模式 Status 仍 pass、指向 dup_probe.rs；白名单路径同款定义→0 warning 且记入 deferred；utils/ 内→豁免 0；干净树→0。临时夹具用毕清理，**全程未碰真实产品代码**。
- `git diff --check` 干净。

有依据的结论：去重门 warning-only、当前 0 误报、白名单覆盖 R-U deferred 的 12 项、对真实重复会出 warning 且不挡、现有门行为零变化。
仍不确定/边界：扫描按精确函数名（`fn <name>`/TS `function|const`）；故意改名规避（如 `fn sha256_hex2`）不在覆盖内——本门只防"同名重复定义"，符合 U-Gate 范围。`--strict` 下新 warning 会按 gate 既有"strict 视 warn 为 fail"规则使 Status=fail（既有逻辑，未改），验收按默认 check 模式。
未动：产品代码、现有 ratchet/waterline/command/sidecar 检查、hooks/CI 开关、其他脚本。
完成 commit：见本包提交（git log harness-hg）。

## 验证：为什么用临时夹具而非往真实代码注入

验收口径"故意加一个重复 fn sha256_hex"用**隔离临时夹具**实现（self-test 在 os.tmpdir 造最小产品树），而非往真实 `src-tauri/src` 注入再回滚——更严格地守住"不碰产品代码"边界，且可重复复核。证据见 self-test 原始输出。
