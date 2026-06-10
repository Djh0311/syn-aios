# Stage J / J1 Codex Control Plane Free Control Entry Evidence v1

日期：2026-06-09

结论：`accepted_with_deferred_items`。

J1-A 已完成主管线实现和自测：工作台智能体页新增普通用户可见的 Codex 控制入口；后端统一 Product Command preview / prepare 支持 `source_kind="codex_control"`；confirm 和 Phase A 继续走既有受控链路；J1-A 不发送 prompt、不执行真实 Codex、不读写 `/Users/yoyi/.codex`。J1-B 真实 `resume` 未授权、未执行。

2026-06-09 17:16 CST 主管线根据长期只读复核线过程反馈追加两项 J1-A 修补：普通 UI 不再展示 `codex exec -C ...` 裸命令计划，改为人话“执行边界摘要”；Codex 控制入口不再提交空 workflow / work item / task package / memory packet 绑定，改为绑定真实项目 workflow 或 J1 临时运行 refs。长期只读复核线复审结论为“通过，带 P2 接受”；P2 归档命名债已在任务包回交产物章节改为实际文件名。

## 1. 产物

- 任务包：`tasks/2026-06-09-stage-j-j1-codex-control-plane-free-control-entry-v1.md`
- Handoff：`handoffs/2026-06-09-stage-j-j1-codex-control-plane-free-control-entry-v1-result.md`
- 后端类型：`prototypes/productized-desktop-shell/src-tauri/src/types.rs`
- 后端实现：`prototypes/productized-desktop-shell/src-tauri/src/real_execution_command.rs`
- 前端类型：`prototypes/productized-desktop-shell/src/lib/types.ts`
- 前端 UI：`prototypes/productized-desktop-shell/src/views/AgentView.tsx`
- 前端样式：`prototypes/productized-desktop-shell/src/styles.css`
- 离线测试：`prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`

## 2. 复核线前置结论

长期只读复核线：`019ea33a-23c4-7c10-8db3-95b8cf910fe7`

J1 任务包复核结论：带 P2 通过，可进入 J1-A 开发，不进入 J1-B。

J1-A 实现复核初次结论：不通过，两个 P1 为普通 UI 暴露裸 `codex exec -C` 命令计划、J1-A 前端空绑定 workflow / run unit / task package / memory packet。

J1-A 修补后复核结论：通过，带 P2 接受；两条 P1 均已关闭，未发现新的 P0/P1。P2 为 evidence/handoff 文件命名与任务包回交产物章节不一致，主管线已将任务包章节改为实际文件名，不新建重复文件。

已关闭的 P2：

- Product Command 不再写成 `command_family="codex_control"` 分叉；J1-A 明确继续使用统一 `command_family="real_execution_product_command"`，`codex_control` 放在 `source_kind` 和可追溯字段中。
- J1-A 测试矩阵补充普通 UI 不得调用 / 暴露 Phase B wrapper，不得复用 legacy / H5 wrapper 冒充自由操控入口。

## 3. 实现摘要

后端：

- 新增 `CodexControlCommandInput`。
- `PreviewRealExecutionProductCommandInput` / `PrepareRealExecutionProductCommandInput` 新增可选 `codex_control` payload。
- `preview_real_execution_product_command_at` 支持 `source_kind="codex_control"`。
- `codex_control` 生成的 request 继续使用 `command_family="real_execution_product_command"`。
- `operation_id="resume"` 可进入 prepare / confirm / Phase A；`operation_id="new_session"` 在 J1-A 只返回 deferred / blocked preview，不写可执行 product command sidecar。
- J1-A preview / prepare / confirm / Phase A 均保持 `prompt_sent=false`、`real_codex_executed=false`、`writes_codex_home=false`、`writes_project_files=false`。
- prompt body 不进入 product command sidecar、continuation sidecar、runtime log、audit 或 memory；只使用 `prompt_summary`、`prompt_ref`、`prompt_hash`。

前端：

- 智能体页主区域新增“Codex 控制”入口。
- 表单支持项目、目标 session、运行模式、sandbox、任务摘要、任务正文。
- J1-A 会把 Product Command 绑定到项目、workflow、work item、task package ref 和 memory packet ref；若项目尚无 workflow，则使用 `workflow:j1-codex-control:<project>` 形式的临时运行绑定，不冒领 J2 自动编排。
- 操作流为：生成预览、写入准备、用户确认、记录 Phase A（不真实执行）。
- 不新增 Phase B 按钮，不暴露裸 CLI；新会话预览普通 UI 只展示执行边界摘要，原始命令计划不铺到普通用户界面。
- UI 明确任务正文保存策略、记忆影响和 read-only 下 allowed root 只是执行边界根。

## 4. 新增测试覆盖

Rust：

- `j1_codex_control_resume_preview_prepare_confirm_and_phase_a_stay_no_real_execution`
  - 覆盖 `codex_control` resume preview / prepare / user confirm / Phase A。
  - 验证 command family 仍为 `real_execution_product_command`。
  - 验证 prompt body 不进入 product command sidecar / continuation sidecar / runtime log。
  - 验证 Phase A flags 全 false：`prompt_sent=false`、`real_codex_executed=false`、`writes_codex_home=false`、`writes_project_files=false`。
- `j1_codex_control_new_session_is_deferred_and_not_prepared`
  - 覆盖 `new_session` 在 J1-A 只能 deferred / blocked。
  - 验证不会写 product command sidecar。

前端：

- `runRealExecutionProductCommandBoundaryScenario` 补充 J1-A UI 断言：
  - 智能体页出现“Codex 控制”入口。
  - 出现“生成预览 / 写入准备 / 用户确认 / 记录 Phase A（不真实执行）”。
  - 出现任务正文保存策略、观察 / 候选来源、不会自动写正式记忆。
  - 既有 forbidden markup 断言继续覆盖 PhaseB wrapper 名称不得出现在普通 UI。

## 5. 验证

已通过：

```text
npm run typecheck
npm run test:offline-interaction
npm run build
cargo test --lib real_execution_command
cargo test --lib session_continuation
cargo test --lib runtime_log
cargo test --lib codex_local_runner
cargo test --lib
cargo fmt -- --check
```

结果摘要：

- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：通过，`offline interaction tests passed: 13`。
- `npm run build`：通过，仅保留既有 Vite chunk size warning。
- `cargo test --lib real_execution_command`：通过，`33 passed; 2 ignored`。
- `cargo test --lib session_continuation`：通过，`17 passed; 4 ignored`。
- `cargo test --lib runtime_log`：通过，`6 passed`。
- `cargo test --lib codex_local_runner`：通过，`11 passed`。
- `cargo test --lib`：通过，`302 passed; 7 ignored`。
- `cargo fmt -- --check`：通过。

J1-A 追加修补后重新验证：

- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：通过，`offline interaction tests passed: 13`。
- `npm run build`：通过，仅保留既有 Vite chunk size warning。
- 未重跑 Rust；追加修补只改 `AgentView.tsx` 和前端离线测试，未改 Rust 后端。

## 6. 边界扫描

已执行 J1-A UI / wrapper 扫描：

```text
rg -n "runRealExecutionProductCommandPhaseB|run_real_execution_product_command_phase_b|executeLegacyWorkflowNodeDispatch|runLegacyWorkflowMachine|previewH5ProjectWorkflowDispatch|h5_project_workflow_dispatch" \
  prototypes/productized-desktop-shell/src/App.tsx \
  prototypes/productized-desktop-shell/src/views \
  prototypes/productized-desktop-shell/src/components
```

结果：

- `AgentView.tsx` 无 PhaseB / legacy / H5 wrapper 命中。
- 命中仅为既有 `App.tsx` legacy action handler，不是 J1-A 新入口。

追加修补后复扫：

- `AgentView.tsx` / `components` / `App.tsx` 无 `命令计划：codex exec -C` 命中。
- `AgentView.tsx` 无 `workflow_id: null`、`work_item_id: null`、`task_package_ref: null`、`memory_packet_ref: null` 命中。
- PhaseB / legacy / H5 wrapper 命中仍仅为既有 `App.tsx` legacy action handler；J1-A 新入口不调用。

已执行普通 UI 文案 / 敏感边界扫描：

```text
rg -n "codex exec|direct CLI|裸 CLI|裸控制台|完整 transcript|已正式记忆|自动写正式记忆|prompt_body" \
  prototypes/productized-desktop-shell/src/views/AgentView.tsx \
  prototypes/productized-desktop-shell/src/components \
  prototypes/productized-desktop-shell/src/App.tsx
```

结果：

- J1-A 新增普通 UI 只出现“不能使用裸控制台”“不会自动写正式记忆”等边界说明。
- `codex exec` 命中均为既有开发者边界 / 权限弹层说明，不是 J1-A 新入口。

已执行 `codex_control` 覆盖扫描：

```text
rg -n "codex_control|CodexControlCommandInput|j1_codex_control|new_session_deferred|prompt_body_runtime_only" \
  prototypes/productized-desktop-shell/src-tauri/src/real_execution_command.rs \
  prototypes/productized-desktop-shell/src-tauri/src/types.rs \
  prototypes/productized-desktop-shell/src/lib/types.ts \
  prototypes/productized-desktop-shell/src/views/AgentView.tsx \
  prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx
```

结果：命中为 J1-A 类型、实现、测试和 UI 显示边界，符合预期。

## 7. 边界确认

本轮没有：

- 执行真实 `codex exec` / `codex exec resume`。
- 发送真实 prompt。
- 读写 `/Users/yoyi/.codex` 或 `.codex/plugins/cache`。
- 读取 auth/token/secret/`.env`/keychain/OAuth/provider credential/full transcript/rollout。
- 启动 Browser / Chrome / Tauri / Vite dev server / 截图工具。
- 同步 `CURRENT.md` / `tasks/README.md` / `AUTHORITY.md` / `STAGE_PLAN.md` / `README.md` checkpoint 入口。

说明：`npm run build` 重新生成了前端 `dist` 构建产物。`git status` 当前不可用，因为 `/Users/yoyi/workspace` 不是 git repo。

## 8. 接受范围

J1-A 接受为：

- `codex_control` source 已接入统一 Product Command preview / prepare。
- 智能体页有普通用户可见 Codex 控制入口。
- 用户可以生成 preview、写入 prepare、记录 user confirmation、执行 Phase A no-op trace。
- J1-A prompt body 不持久化，真实执行 flags 保持 false。
- `new_session` 在 J1-A 保持 deferred / blocked，不冒领真实成功。

不接受为：

- J1 最终完成。
- J1-B 已授权或已执行。
- 真实 Codex resume 已执行。
- 真实 new session 已创建。
- 通用自由 Codex 控制台 / 任意项目自由执行完成。
- 自动化工作流编排 J2 完成。
- 记忆层真实捕获 / 分析 / 候选化 J3 完成。
- planned adapters 真实接入、provider credential / model verification、自动 retry / stop / restart 或真实 Tauri 全量验收完成。

## 9. 下一步

- 主管线同步 `CURRENT.md`、`tasks/README.md`、`AUTHORITY.md`、`STAGE_PLAN.md`、`README.md` 和 Stage J 计划 checkpoint 入口。
- J1-B 真实执行必须另行执行点授权，不继承 J1-A。
