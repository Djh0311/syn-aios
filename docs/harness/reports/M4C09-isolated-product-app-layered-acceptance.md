# M4C09 隔离产品 App 分层验收报告

日期：2026-08-10<br>
状态：`COMPLETED / ISOLATED PRODUCT APP / SYNTHETIC ONLY`

结论：C09 的静态、Rust 单元/聚焦回归、TypeScript、离线前端、non-test build、隔离 debug App 首启/强退/同 profile 重启和可见交互均通过。运行只使用 C09 launcher 生成的隔离 profile、两个固定 synthetic source owner 和 fake model；真实模型、provider、Codex 消息、账号、connector 与外部网络写入均未进入。该结论只覆盖机械证据与隔离产品 App，不等于真实日常使用。

## 1. 实现边界

- `SYN_R4_ACCEPTANCE_PROFILE` 继续只提供文件系统隔离基座；C09 还要求 debug build、有效 profile 和 `SYN_M4C09_ISOLATED_ACCEPTANCE=1`。M2 reference slice、M3C07 与 C09 mode 互斥，父环境 marker 在 build/child 前清理并由 launcher 固定重建。
- C09 在隔离 profile 内安装普通 M3 Secretary RoleSession 与普通 M4 SQLite repository。C09 RoleSession adapter 先逐项核对 M3 server-resolved opaque binding，再映射固定 canonical PersonalScope；renderer 不提供 role、scope、owner、profile、provider 或 model 选择。
- C09 只替换三个同名 Tauri wrapper；runtime 未安装时原样委托普通 `secretary_agent` bridge。隔离 runtime 激活后，全局 invoke handler 只放行启动读取、M4 只读/协调和无参数 C09 status IPC；dispatch、provider、connector、项目写和 legacy effect IPC 在反序列化与 effect 前统一拒绝。
- 固定入口 `scripts/run-m4-isolated-app-acceptance.mjs` 不接收参数，只能调用 `run-r4-isolated-app-preflight.mjs --m4c09-isolated-acceptance`。launcher 生成一个 synthetic profile，复用同一 build/profile/reentry capability，并限制最多 4 次拉起。
- C09 没有补 C08 legacy tuple adapter，也没有实施 M5–M10。

## 2. 机械与构建证据

| 层 | 命令 | 最终结果 |
| --- | --- | --- |
| C09 Rust | `cargo test --lib m4c09_` | exit 0；3 passed / 0 failed |
| M4 聚焦保留语义 | `cargo test --lib m4c0` | exit 0；98 passed / 0 failed |
| non-test Rust | `cargo check --lib` | exit 0；只保留仓库既有 warning |
| TypeScript | `npm run typecheck` | exit 0 |
| 既有离线交互矩阵 | `npm run test:offline-interaction` | exit 0；摘要 15 组，含既有 M4C06/M4C08 |
| C09 新增静态契约 | esbuild bundle 后执行 `tests/m4c09-isolated-product-app-acceptance.test.ts` | exit 0；`m4c09-isolated-product-app-acceptance: ok` |
| 前端生产构建 | `npm run build` | exit 0；306 modules；仅既有 `>500 kB` chunk warning |
| launcher 语法 | `node --check` 两个 C09 launcher 脚本 | exit 0 |
| 定向格式 | `rustfmt --edition 2021 --check src/m4_acceptance.rs` | exit 0；未递归格式化 `commands.rs` / `lib.rs` |
| diff 机械检查 | `git diff --check` | exit 0 |

`npm run test:offline-interaction` 的既有显式 entrypoint 清单尚不包含新增 C09 文件，因此 C09 静态契约单独 bundle/执行；本报告没有把既有 15 组摘要冒充 C09 测试。

## 3. 隔离产品 App 现场

固定 wrapper 构建并启动本仓库 debug App。run hash 为 `ffa3e7b69959ab2a2170156cc605b8799d8493f21e0465ba6e93dc4aadb74a33`，profile 只以 SHA-256 `9b5f157993a751110af2fb88d23c535fcfcfa179b1f6d3eb618fa629f37152c5` 记录。

| 场景 | 可见/持久结果 |
| --- | --- |
| 首启空态 | 普通 M4 read path 先读到空 attention；`zero_item_read_model_calls=true`；scheduler/run report 的 material event、agent turn、model invocation 均为 0 |
| 两个 source owner 与去重 | `OWNER_ALPHA` / `OWNER_BETA` 各产生一个 inbox 与一个 open loop；exact duplicate replay；表计数固定为 source event 2 / inbox 2 / open loop 2 |
| 日报重跑 | 连续两次 refresh 的 report id、version 与 daily/scheduler 表计数不变；`daily_report_rows=1`、`scheduler_run_rows=1` |
| fake model 故障 | 首次 explicit synthetic trigger 记录 `M4C09_FAKE_MODEL_FAILURE`，exact replay 不再调用；deterministic brief hash 不变；durable invocation 仅 1 行 |
| 生命周期 | 首启可见点击 alpha ACKNOWLEDGE、alpha CARRY_OVER、beta SNOOZE；重启后 alpha 为 `ACKNOWLEDGED`、beta 为 `SNOOZED`，carry-over receipt 精确 1 行 |
| RoleSession 恢复 | 三次 launch 的 receipt 使用同一 role-session hash；重启 receipt 为 `same_role_session_recovered=true` |
| 可见 deep link | 点击“在来源模块中查看此关注项”进入项目来源面，并显示“已转到来源负责模块；事实详情只在来源模块读取。” |
| 机械解释 | “请秘书解释”返回机械解释；UI 显示 `NOT_REQUESTED`，并明确解释不是来源事实 |

launcher 共记录 3 次同 profile launch：第 0 次按验收要求 `SIGKILL`；第 1 次在 UI inspection 仍 pending 时正常 exit 0，launcher 继续按合同拉起；第 2 次完成 UI inspection 后 `SIGTERM`。最终为：

- `same_profile=true`
- `same_profile_reused=true`
- `initial_restart_eligible=true`
- `ui_inspection_completed=true`
- `runtime_receipt_complete=true`
- `relaunch_limit_reached=false`
- `startup_failure_family=null`
- build exit 0
- `environment_unchanged=true`
- `home_initial_view_config_pinned=true`

退出后按本仓库完整 App 路径和两个 launcher 名称精确检查，未发现本次验收残留进程。

## 4. 脱敏证据

仓库只保存 JSON 脱敏回执；两张截图不入库，只记录 SHA-256，并作为本任务内可见 Computer Use 证据。捕获时 runtime receipt、UI inspection 与截图均为 `0600`，receipt 为普通文件、非 symlink、link count 1。

| 证据 | SHA-256 |
| --- | --- |
| `M4C09-isolated-product-app-evidence/launcher-receipt.json` | `036d00250e68df7e057877dc628c46eb8be4ac421869d96397bce066e838d1eb` |
| `M4C09-isolated-product-app-evidence/runtime-receipt.json` | `53717738a24396644dd915f65b0d7a87e0594b531f901841598bffcdcf34e210` |
| `M4C09-isolated-product-app-evidence/ui-inspection.json` | `669f4b1902935c1856b86e9780c34172dd3531b783b297645f24646b62bb23c6` |
| 首启 synthetic 窗口截图（不入库） | `99b4af3384629c48e1cdb25b3611f8eba1bba2408640064454491bf4a6a93f6e` |
| 重启后 deep link / 机械解释截图（不入库） | `8a98afd316b6e90da016a373b400569eb3261d9fc3f314bfe9ed5a691659d322` |

runtime receipt 只含固定 code、hash、bool 与 count。最终 isolation 计数为：

- `real_model_attempts=0`
- `real_provider_attempts=0`
- `external_connector_attempts=0`
- `external_network_writes=0`
- `real_codex_message_attempts=0`

## 5. 审查与证据强度

本任务在实现冻结后由 GPT-5.6 Sol / ultra 重新按 leaf 做了独立审计回合，逐项核对 tracked diff、三个新增文件、三份脱敏 JSON 与最终测试，结论为 P0=0、P1=0、P2=0。该回合与施工步骤分离，但仍属于同一主管任务。

另行尝试的 ephemeral Codex 第二会话在生成前命中账号用量上限；Oracle CLI 的当前版本也不具备可靠的 GPT-5.6 Sol / ultra 选择能力，浏览器登录态同步未形成模型输出。二者均未被计入审查通过证据，也没有留下仓库或临时浏览器进程。后续总线主管复核仍可作为额外独立证据，但不改变本报告对现有代码、测试与隔离 App receipt 的事实记录。

## 6. 明确限制

- 本次运行是 synthetic fixture + fake model + 隔离 profile；没有真实个人资料、真实用户项目、真实日常节奏、真实模型/provider、真实 Codex 消息、真实账号或外部 connector。
- UI 证明普通产品 App 在隔离组合下可见、可点、可重启恢复；它不是长期真实使用观察，也不证明发布包或部署。
- C08 普通产品当前没有 legacy tuple adapter，五类 legacy inventory 仍按设计 quarantine。该事实既不是 C09 缺陷，也未在本叶修改。
- 没有 push、merge、rebase、发布或部署。
