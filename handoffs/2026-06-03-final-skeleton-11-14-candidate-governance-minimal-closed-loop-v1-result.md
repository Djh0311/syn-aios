# final-skeleton-11 + 14 候选治理最小闭环 handoff v1

日期：2026-06-03

## 结论

本轮完成 `final-skeleton-11` 和 `final-skeleton-14` 合并批次。

接受为：

- 黑板候选持久确认最小闭环完成。
- 记忆候选生命周期最小闭环完成。
- 两个 sidecar 分离。
- 两套命令分离。
- 两套 UI 文案分离。
- 交叉边界测试覆盖。
- 未写正式事实。
- 未写正式长期记忆。
- 未改 workflow state JSON 结构。

不接受为：

- 正式事实系统完成。
- 正式记忆系统完成。
- 任务包记忆注入完成。
- Obsidian / 知识库集成完成。
- 向量库 / 图数据库完成。
- 秘书核心只读模型完成。
- 黑板候选能直接变成记忆。

## 改动文件

- `prototypes/productized-desktop-shell/src-tauri/Cargo.toml`
- `prototypes/productized-desktop-shell/src-tauri/src/types.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/control_core.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/blackboard_candidate_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/memory_candidate_store.rs`
- `prototypes/productized-desktop-shell/src/lib/types.ts`
- `prototypes/productized-desktop-shell/src/lib/tauri.ts`
- `prototypes/productized-desktop-shell/src/lib/candidateGovernance.ts`
- `prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`
- `CURRENT.md`
- `tasks/README.md`

新增：

- `evidence/2026-06-03-final-skeleton-11-14-candidate-governance-minimal-closed-loop-v1.md`
- `handoffs/2026-06-03-final-skeleton-11-14-candidate-governance-minimal-closed-loop-v1-result.md`

## 可操作状态

后端：

- `load_blackboard_candidate_store`
- `record_blackboard_candidate_decision`
- `load_memory_candidate_store`
- `create_memory_candidate`
- `record_memory_candidate_decision`

前端：

- 项目工作流页显示候选治理条。
- 黑板候选可通过候选层按钮写入 sidecar 状态。
- 记忆候选可通过候选层按钮写入 sidecar 状态。

## 手动测试清单

在应用里测试：

1. 打开应用，进入“项目”。
2. 选择一个已有项目。
3. 进入“项目工作流”。
4. 在工作流画布下方找到“候选治理”区域。
5. 检查显示：
   - `blackboard-candidates.v1.json`
   - `memory-candidates.v1.json`
   - “候选确认只写候选 sidecar；不写正式事实、不写正式长期记忆、不推进 workflow state。”
6. 如果项目黑板有候选，点击“确认黑板候选后续处理”。
7. 在确认弹层里检查边界文案：只写 `blackboard-candidates.v1.json`，不写正式事实、不写正式记忆、不批准权限、不推进 workflow state。
8. 确认后，回到项目工作流页，候选治理区域应显示黑板 revision 增加。
9. 再测试“拒绝黑板候选 / 暂缓黑板候选 / 废弃黑板候选”，确认它们都只走候选 sidecar。
10. 如果已有记忆候选，点击“确认记忆候选保留”。
11. 在确认弹层里检查边界文案：`candidate_confirmed` 只表示确认保留候选，不写正式长期记忆。
12. 确认后，候选治理区域应显示记忆 revision 增加。
13. 测试“隔离记忆候选 / 废弃记忆候选”，确认文案没有“已记住”“正式记忆已写入”。

文件层手动核对：

1. 找到当前 `workflow-state.v0.json` 所在目录。
2. 应能看到：
   - `blackboard-candidates.v1.json`
   - `memory-candidates.v1.json`
3. `workflow-state.v0.json` 不应新增黑板候选或记忆候选字段。
4. `backups/` 里可以出现两个 sidecar 的备份文件。
5. 不应出现正式 `MemoryRecord` 文件或数据库迁移。

## 验证

已通过：

```text
npm run typecheck
npm run test:offline-interaction
npm run build
cargo test --lib
rustfmt --check src/blackboard_candidate_store.rs src/memory_candidate_store.rs
```

记录：

- `npm run test:offline-interaction`：`offline interaction tests passed: 6`
- `cargo test --lib`：93 passed，1 ignored
- `npm run build`：通过；仍有 Vite chunk size warning

## 未验证和风险

- 未做真实浏览器或 Tauri 窗口截图验收。原因：当前可用工具没有浏览器控制工具，本项目也没有 Playwright 依赖；本轮未安装新依赖。
- 没有在真实应用里点确认按钮写实际用户侧 sidecar；Rust 测试只写临时目录。
- 记忆候选创建命令已实现，但项目页目前主要展示和处理已有候选；自动从工作流总结生成记忆候选仍后置。
- JSON sidecar 并发控制足够做第一版，但不是数据库事务。

## 下一步

可以进入 `final-skeleton-15-secretary-core-readonly-model-v1`。

进入前建议继续保持边界：

- 秘书只读。
- 秘书只能生成建议或候选。
- 不直接改事实。
- 不直接派发任务。
- 不写正式记忆。
