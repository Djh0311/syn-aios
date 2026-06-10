# Stage J / J1 Codex Control Plane Free Control Entry Result v1

日期：2026-06-09

结论：J1-A 已完成并通过长期只读复核线复核，状态为 `accepted_with_deferred_items`。J1-B 真实执行未授权、未执行。

2026-06-09 17:16 CST 追加修补：根据长期只读复核线过程反馈，普通 UI 不再展示 `codex exec -C ...` 裸命令计划，改为“执行边界摘要”；Codex 控制入口会绑定项目 / workflow / work item / task package / memory packet refs，若项目没有既有 workflow 则使用 J1 临时运行绑定。复核线复审结论为“通过，带 P2 接受”；P2 归档命名债已在任务包回交产物章节改为实际文件名。

## 做了什么

- 后端新增 `CodexControlCommandInput`，让统一 Product Command 支持 `source_kind="codex_control"`。
- `resume` 可走 preview / prepare / user confirmation / Phase A no-op trace。
- `new_session` 在 J1-A 只返回 deferred / blocked，不写可执行 sidecar。
- 智能体页新增普通用户可见“Codex 控制”入口。
- UI 支持选择项目、目标 session、运行模式、sandbox、任务摘要和任务正文。
- UI 生成的 J1-A Product Command 不再是游离控制台操作，会带上 workflow / work item / task package / memory packet refs。
- UI 操作流为生成预览、写入准备、用户确认、记录 Phase A（不真实执行）。
- 关闭 J1 任务包复核线 P2：统一 family 不分叉；普通 UI 不得调用 PhaseB / legacy / H5 wrapper。

## 关键边界

- J1-A 不执行真实 `codex exec` / `codex exec resume`。
- J1-A 不发送 prompt。
- J1-A 不读写 `/Users/yoyi/.codex`。
- prompt body 不写 product command sidecar、continuation sidecar、runtime log、audit 或 memory。
- Phase A flags 保持 false：`prompt_sent=false`、`real_codex_executed=false`、`writes_codex_home=false`、`writes_project_files=false`。
- 记忆影响只说明未来 observation / candidate 来源，不自动写 FormalMemory。

## 验证

已通过：

- `npm run typecheck`
- `npm run test:offline-interaction`，`offline interaction tests passed: 13`
- `npm run build`，仅既有 Vite chunk size warning
- `cargo test --lib real_execution_command`，`33 passed; 2 ignored`
- `cargo test --lib session_continuation`，`17 passed; 4 ignored`
- `cargo test --lib runtime_log`，`6 passed`
- `cargo test --lib codex_local_runner`，`11 passed`
- `cargo test --lib`，`302 passed; 7 ignored`
- `cargo fmt -- --check`

追加修补后已重新验证：

- `npm run typecheck`
- `npm run test:offline-interaction`，`offline interaction tests passed: 13`
- `npm run build`，仅既有 Vite chunk size warning

未重跑 Rust；追加修补只改前端 UI 和前端离线测试。

## 扫描

- `AgentView.tsx` 无 `runRealExecutionProductCommandPhaseB` / legacy / H5 wrapper 命中。
- legacy 命中仅为既有 `App.tsx` action handler，不是 J1-A 新入口。
- `codex exec` 命中均为既有开发者边界或权限弹层说明，不是 J1-A 新入口。
- 追加修补后 `AgentView.tsx` / `components` / `App.tsx` 无 `命令计划：codex exec -C` 命中。
- 追加修补后 `AgentView.tsx` 无 `workflow_id: null`、`work_item_id: null`、`task_package_ref: null`、`memory_packet_ref: null` 命中。

## 边界确认

本轮没有执行真实 Codex，没有发送 prompt，没有读写 `/Users/yoyi/.codex`，没有读取 secret/token/`.env`/keychain/OAuth/provider credential/full transcript/rollout，没有启动 Browser / Chrome / Tauri / Vite dev / 截图工具。

`npm run build` 重新生成了前端 `dist`。`git status` 不可用，因为当前 `/Users/yoyi/workspace` 不是 git repo。

## 仍不得声明

不得声明 J1 最终完成、J1-B 已授权或已执行、真实 Codex resume 已执行、真实 new session 已创建、任意项目自由执行完成、J2 自动化工作流编排完成、J3 记忆捕获完成、planned adapters 真实接入、provider/model verification 或真实 Tauri 全量验收完成。

## 下一步

主管线同步 `CURRENT.md`、`tasks/README.md`、`AUTHORITY.md`、`STAGE_PLAN.md`、`README.md` 等 checkpoint 入口。J1-B 必须另行执行点授权。
