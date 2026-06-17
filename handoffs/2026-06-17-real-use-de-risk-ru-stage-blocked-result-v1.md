# 交接：RU 真实使用去险阻断分类结果 v1

日期：2026-06-17

来源：Codex 执行线

状态：`blocked_classified`，未提交，待咨询线复核。

## 一句话结论

RU 没有完成，也不应硬冲完成：真实 `mariotest` 项目与真实 state root 已只读核实，但默认 Tauri GUI / snapshot 会读 `/Users/yoyi/.codex`，而当前没有不读 `.codex` 且经 M2 门写正式记忆的非 GUI 入口；所以 RU1 只能算只读部分核实，RU2 未执行，RU3 只能给阻断建议。

## 证据入口

- Evidence：`evidence/2026-06-17-real-use-de-risk-ru1-ru2-blocked-v1.md`
- 计划正本：`docs/plans/2026-06-17-real-use-de-risk-dogfood-stage-plan-v1.md`
- Kickoff：`handoffs/2026-06-17-real-use-de-risk-ru-stage-claude-to-codex-kickoff-v1.md`

## 已核实的真实事实

- `WORKBENCH_STATE_ROOT`：`/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state`
- 真实文件仅两份：
  - `workflow-state.v0.json`：`4bd5434fdca9e82c8fafc42989e1a267ed7d677bfe2972273fb3afaa26829972`
  - `plan-authorizations.v1.json`：`6962e4781f49246525d4cde37d3133924a66faa12b8aab90db106c3c9f401b0e`
- 记忆 sidecar 当前不存在：`memory-capture-events.v1.json`、`observations.v1.json`、`memory-candidates.v1.json`、`formal-memories.v1.json` 等均未出现。
- `mario test` 项目真实存在：
  - project_id：`project:users-yoyi-documents-mario-test`
  - root_path：`/Users/yoyi/Documents/mario test`
  - permission_level：`user_confirmed_write`
- `mario test` workflow 真实存在：
  - workflow_id：`workflow:users-yoyi-documents-mario-test:default`
  - title：`mario test 四角色编排测试工作流`
  - state：`draft`
  - model_policy：`codex_threads_user_confirmed`
  - node_count：7
  - edge_count：7
  - mariotest audit by project/workflow id：253
- `mario test/.workbench` 有历史探针文件，但本 RU 窗口未复用为“本窗执行”证据。

## 阻断点

1. 默认 GUI / snapshot 路径会访问 `.codex`：
   - `load_workbench_snapshot` 与 `query_workbench_page_read_model` 均经 `build_snapshot()`。
   - `build_snapshot()` 固定使用 `SessionSourceMode::RealWithSqliteFallback`。
   - 该模式调用 `codex_db::default_state_db_path()`，从 `$HOME/.codex` 寻找 `state_*.sqlite` 并 read-only 打开。
   - RU 硬封印禁止读写 `/Users/yoyi/.codex`，所以执行线未启动 `tauri dev` / 默认 GUI。
2. RU2 没有安全替代入口：
   - M2 采纳链路存在于 Tauri command 与前端 PermissionDialog / pending action 中。
   - 当前未发现可直接用于 RU 的 CLI / runner / MCP 入口，能够在不启动默认 GUI、不读 `.codex`、不改源码时完成 `capture -> observation -> candidate -> M2 adoption`。
   - 手工写 JSON 会绕过 M2 门，不能接受。

## 分项状态

- RU1：`partial_readonly_verified`，真数据根和 `mariotest` 实物已核，但 GUI 真机跑通 / 重开仍在未执行。
- RU2：`blocked_not_executed`，未写候选或正式记忆；正式记忆仍为 0 条真实 sidecar。
- RU3：`blocked_deferred`，只产出阻断分类与下一步建议；不能判定 L5 完工线达成，不能建议直接开 B。

## 建议给咨询线的决策选项

- 推荐：另开窄任务，新增 confirmed-path RU/Dogfood 后端入口或 ignored runner。要求显式传真实 workflow-state path，构造上禁止读 `.codex`，只做 capture / candidate / M2 adoption，先 fixture 测试，再由用户确认真实窗口。
- 备选：由用户明确变更本 RU 硬封印，接受默认 GUI read-only 读取 `/Users/yoyi/.codex/state_*.sqlite` 的风险后，再手动 GUI 实操。执行线不能自行改这个授权。

## 本窗口边界

- 未启动 GUI / `tauri dev`。
- 未执行真实 `codex exec` / `codex exec resume`。
- 未启动 K3-B1/K3-B2。
- 未真 retry / stop / restart / resume。
- 未切 R3 真库产品全局读写。
- 未读写 `/Users/yoyi/.codex`。
- 未读 secret/token/.env/keychain/OAuth/provider credential/full transcript/rollout/prompt body。
- 未写真实 workbench 数据根、未写 `mariotest` 项目、未写产品源代码、未改 `CURRENT.md`。

## 交回请求

请咨询线复核本阻断分类是否成立，并决定下一步是开窄入口任务，还是由用户重新授权 GUI 读取 `.codex` 的手动实操窗口。本交接不请求提交；当前仍停在未提交状态。
