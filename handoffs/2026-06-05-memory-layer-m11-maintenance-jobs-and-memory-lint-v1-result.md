# Handoff：Memory Layer M11 Maintenance Jobs And Memory Lint v1

日期：2026-06-05

## 本轮结果

M11 已完成。记忆层现在有 maintenance run、维护 finding、维护报告、任务包 blocking 协作和记忆中心维护摘要。

完成内容：

- 复用 `memory-lint.v1.json`，新增 `maintenance_reports[]`。
- 新增 `maintenance_run` intent、maintenance report、check summary、recommendation 和 index status。
- 扩展 deterministic lint engine，覆盖 stale / missing source / duplicate / permission revoked / relation source revoked / secret / private / entity drift / derived index stale / mature pattern signal。
- open blocking finding 继续阻断 task memory packet 召回相关正式记忆。
- 记忆中心复用既有 `记忆` 入口展示维护摘要和运行入口。
- 确认弹层新增 `run-memory-maintenance`，明确只写 lint sidecar，不改正式记忆。
- 权威入口已更新到 M11 完成，下一步指向 M12 成熟模式、跨项目记忆和完整验收。

## 接受范围

只接受为：

- 维护任务运行入口可用。
- 维护任务会写 run record / report summary。
- 维护 finding 能覆盖过期 / stale、缺来源、重复 / 实体漂移、权限撤回、私密风险、索引状态和 mature pattern signal。
- blocking finding 能阻止相关正式记忆进入任务包召回。
- 记忆中心有最小维护摘要和人工复核建议展示。

不接受为：

- M12 成熟模式、跨项目记忆或完整验收完成。
- M13 中间版本记忆系统最终验收完成。
- 自动修复、自动合并、自动废弃、自动冻结、自动归档或自动删除正式记忆完成。
- mature pattern 自动成为正式记忆、技能或全局规则完成。
- 向量库、图数据库、GraphRAG、自动索引重建系统或完整运维后台完成。
- 真实 worker / Codex 已执行。

## 验证

通过：

- `npm run typecheck`
- `npm run test:offline-interaction`，9 scenarios passed
- `npm run build`
- `cargo test --lib memory_lint`，9 passed
- `cargo test --lib memory_maintenance`，3 passed
- `cargo test --lib task_memory_packet`，10 passed
- `cargo test --lib`，217 passed / 1 ignored
- `rustfmt --check src/memory_lint_engine.rs src/memory_lint_store.rs src/control_core.rs src/types.rs src/lib.rs`

说明：

- Rust 测试仍有既有 `JsonRpcError::invalid_params` dead_code warning。
- `npm run build` 仍有 Vite chunk size warning。

## 未完成 / 风险

- 普通 in-app Browser smoke 已完成：`记忆` 入口里的维护任务卡片和边界文案可渲染，产品页面 console error / warn 为空。
- PNG 截图落盘失败：Browser runtime 写 evidence PNG 时返回 `EPERM`；本地 Vite smoke server 已关闭。
- 真实 Tauri 数据桥窗口验收未完成；不能声称 M11 UI 已完成真实 Tauri 验收。
- M11 维护报告是 finding / review 摘要，不是正式事实源；后续不要让维护报告绕过正式记忆状态机。
- Mature pattern signal 只是 M12 的输入信号，不是成熟模式正式化。
- 权限撤回当前通过 blocking / needs_review finding 影响未来召回；完整生命周期处理仍需后续任务按确认链路拆。

## 当前权威入口

- `CURRENT.md`
- `README.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `docs/plans/middleware-version-stage-plan-v1.md`
- `docs/plans/memory-layer-implementation-slice-v1.md`
- `tasks/2026-06-05-memory-layer-m11-maintenance-jobs-and-memory-lint-v1.md`
- `evidence/2026-06-05-memory-layer-m11-maintenance-jobs-and-memory-lint-v1.md`

## 下一步

进入 M12：成熟模式、跨项目记忆和完整验收。

M12 建议优先明确：

- mature pattern signal 如何生成 `MaturePatternCandidate`，以及谁能确认。
- 跨项目主题报告如何下钻来源，但不能直接影响 worker。
- 用户确认后成熟模式 / 全局记忆如何通过 M9 lifecycle 或正式记忆写入链路落地。
- M1-M12 的完整验收如何区分“第一条闭环完成”“正式记忆系统完成”和“中间版本记忆系统完成”。

继续保持：

- 不执行真实 worker / Codex。
- 不读写用户 Codex 会话数据；本轮仅为 Browser smoke 按插件要求读取了 `/Users/yoyi/.codex/plugins/cache/openai-bundled/browser/26.602.30954/skills/control-in-app-browser/SKILL.md`，未写 `/Users/yoyi/.codex`。
- 不让 knowledge doc、candidate、observation、relation candidate、LLM summary、maintenance report 或 graph/index 报告绕过正式记忆状态机。
