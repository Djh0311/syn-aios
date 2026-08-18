# SYN-M5R07 D1/D2 M1 正式身份消费与普通启动来源铺设窄包 v1

日期：2026-08-18
阶段：stage-14 / current leaf M5R07
基线：`bf64a8e3e02982c1403567294d4ff76680b08e2e`
写者：Grok `grok-4.6 --reasoning-effort high`
状态：IMPLEMENTATION TASK / NOT ACCEPTANCE / NOT CLOSEOUT

## 目标

只修 M5R00 独立验收结论带入 M5R07 的两个欠账：

1. D1：普通产品运行期的任务包记忆注入消费 M1 正式 canonical `ProjectId`，不再从 `project_root` 调 `project_id()` 派生；
2. D2：M5R07 ordinary disposable Tauri runner 在启动前明确铺设合成的 `m1-ordinary-project-identity-source-v1.json`，并让 acceptance 进程走 M5R00 已验收的真实普通 Tauri constructor 来源重放路径。

本包不补完整 M5R07 端到端 evidence-binding，不 close stage-14，不改 execution kernel、页面或 M6。

## 写域

只许修改：

- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`（仅 D1 普通 command 的 M1 解析与参数传递）
- `prototypes/productized-desktop-shell/src-tauri/src/workflow_state_lifecycle_task_package.rs`（仅把已解析的 canonical ProjectId 传入 task memory packet）
- `prototypes/productized-desktop-shell/src-tauri/src/m5_ordinary_control_acceptance.rs`（仅 D2 改走真实普通 constructor，M1 fixture 从创建改为只解析）
- `prototypes/productized-desktop-shell/scripts/run-m5-ordinary-control-acceptance.mjs`（仅 D2 合成来源铺设与可信 receipt 字段）

不许改其他文件，不许 stage/commit，不许 reset/stash/clean，不许覆盖或归责现有混合 WIP。

## D1 精确要求

- 生产 `generate_task_package_file` command 必须从 `AppState` 已安装的 M1 read port 按 `request.project_root` exact alias 解析类型化 M1 ProjectId；authority 未安装、alias 缺失、registry 损坏等全部在任务包/记忆/工作流写入之前 fail-closed。
- 将已解析 canonical ID 作为服务器可信参数一路传给 `task_memory_packet_input_from_task_package`；该函数不得再调用 `project_id(project_root_value)`。
- renderer/request 不得提交或覆盖 canonical ProjectId；不得 fallback 到 path hash、slug、legacy index 或自动登记。
- 尽量保留旧的纯函数测试入口；若测试 helper 必须兼容，可增加只在 `#[cfg(test)]` 下使用的明确 legacy test wrapper，但普通 production caller 必须机械可见地走 M1 解析。
- 新增直接测试，至少证明：传入的 opaque canonical ID 原样进入 `TaskMemoryPacketBuildInput.project_id`；普通 command 的 M1 authority/alias 不可用时在任何 task package 或 workflow mutation 前拒绝；不存在 `project_id(project_root_value)` 的生产消费。

## D2 精确要求

- Node runner 在第一进程启动前，于其合成 ordinary app-data 根的直接子路径创建 `m1-ordinary-project-identity-source-v1.json`，字段严格符合 `m1.ordinary-project-identity-source.v1`：唯一 source/entry、`source_revision > 0`、`mode=migrate_legacy_project`、显式 synthetic `source_ref`、`exact_alias=canonical projectRoot`。
- 来源必须在 runner 自己的临时隔离根产生，不读取或迁移真实项目，不从启动代码/legacy index 自动生成。receipt 记录来源相对位置、SHA-256、schema/mode/source_ref，并明确 `SYNTHETIC_INPUTS`、`NO_REAL_USER_DATA`、`NOT_DEPLOYED`。
- `m5_ordinary_control_acceptance::construct_ordinary_app_state` 必须调用与真实 Tauri 普通入口相同的 `AppState::try_new_with_tauri_ordinary_product_seeds`，不能再直接调用 `try_new_with_ordinary_product_ports` 绕过来源重放。
- `install_server_fixture_for_locator` 不得再调用 M1 `register_exact_alias`；它只允许解析由普通 constructor 已重放的 alias，并继续铺设现有 M3 三角色 fixture。来源缺失/损坏/alias 未登记必须失败。
- 第二进程复用同一 app-data/source/registry，canonical ProjectId、binding、role session 与持久状态保持一致；不得产生第二个 M1 id 或重复 registry revision。
- 更新 receipt 中旧的 `ORDINARY_DISPOSABLE_FIXTURE_ONLY / not_legacy_composition` 口径，使其准确表达：真实普通 Tauri constructor + 合成隔离输入 + acceptance-only M3/终态 fixture；不得冒充真实用户老项目、日常运行、发布或 stage closeout。

## 必须保全

- M5R07 U01a/U01b/U01c/U02 已有 scoped PASS 不反写 FAIL；本包只是 successor correction。
- M1–M4 冻结合同与 M1 增补合同正文不改。
- shared-isolated 仍只作 authority-unavailable negative；不在本包安装 authority。
- 用户拒绝路径仍零 Grant、零 durable operation、零 runtime effect。
- 不改 `worker_report.rs`、M5 execution kernel、页面布局、`m6_*.rs`、stage-12、D0C04/D0C05。
- 不接真实凭据/provider/账号/资料，不作外部网络业务写，不 push/merge/rebase/deploy/release。

## 交付与自检

完成后只返回：

1. 实际修改路径；
2. D1 的生产调用链与 fail-closed 点；
3. D2 的来源产生方式、普通 constructor 调用链与 fixture 剩余职责；
4. 新增/调整的测试名；
5. 执行过的命令、exit code 与摘要；
6. 仍未完成的 M5R07 项。

至少运行（使用仓外 target，避免把构建产物作为候选）：

```bash
cd /home/synadmin/workspace/syn/prototypes/productized-desktop-shell/src-tauri
CARGO_TARGET_DIR=/tmp/syn-m5r07-d1d2-target cargo test --lib m5_ordinary_control --offline
CARGO_TARGET_DIR=/tmp/syn-m5r07-d1d2-target cargo test --lib task_memory_packet --offline
CARGO_TARGET_DIR=/tmp/syn-m5r07-d1d2-target cargo check --lib --offline
```

如果过滤名不匹配，改用能直接覆盖新增测试的最窄过滤并如实报告；不得用“0 tests”冒充通过。
