# Root Treatment / R-U-Gate Dedup Guard Draft v1

日期：2026-06-14

状态：待执行。

性质：R-U 查重门方案草稿。本包只写 2-3 种查重门形态 + 推荐方案，不实现、不接入 harness / CI / shape gate。

Planning baseline：`2813058`。

## 0. 主管线理解

用户要求 U-Gate 的完成定义为“草稿写好”，不是把门建出来接上。

本包只允许：

- 写查重门方案草稿。
- 列出 2-3 种形态。
- 给出推荐路径、适用范围、风险、后续任务包拆分。
- 更新入口 checkpoint。

## 1. 输入事实

R-U 已完成：

- U1：hash helper 去重。
- U2：sidecar path helper 去重。
- U3：fs ops helper 去重。
- U5：前端 `DetailLine` / `SummaryTile` 去重。
- U4：Rust normalization helper 去重，规则不同项保持 deferred。

现有 harness / gate 线索：

- `scripts/harness/workbench-shape-gate.js` 已是 Stage R shape gate 主入口。
- `scripts/harness/task-package-lint.js`、`rules-lint.js`、`pre-completion.js` 等可作为后续 rule / lint 接入参考。
- `docs/plans/2026-06-10-root-treatment-r0-shape-gate-and-governance-task-package-rule-v1.md` 是 shape gate 治理规则前史。

## 2. 允许范围

允许新增：

- `docs/plans/2026-06-14-root-treatment-r-u-gate-dedup-guard-draft-v1.md`
- 对应 evidence / handoff。
- 本任务包和入口 checkpoint 文档。

## 3. 禁止范围

禁止：

- 修改 `scripts/harness/workbench-shape-gate.js` 或任何 harness 脚本。
- 接入 CI / pre-completion / task-finish。
- 新增查重扫描脚本。
- 修改源码、测试、CSS、Rust/Tauri/DB。
- 进入 R3 Level B。
- 执行真实 `codex exec` / `codex exec resume`。
- 读写 `/Users/yoyi/.codex`。
- 解冻 backlog。

## 4. 验证

本包为文档草稿，验证：

- `git diff --check`
- 只读确认未修改 harness 脚本 / 源码。
- 独立复核线确认草稿没有冒充实现。

## 5. 复核判据

独立复核线确认：

- 草稿包含 2-3 种查重门形态。
- 草稿给出推荐方案和理由。
- 草稿明确不实现、不接入。
- 未修改 harness / CI / 源码。

## 6. 不接受为

本包不接受为 U-Gate 已实现、查重门已接入、R-U 全部自动防复发完成、R3 Level B 执行、R5 文档对齐、真实 Codex 执行、`.codex` 读写或 backlog 解冻。

## 7. 停止点

草稿完成、复核通过、commit 和 checkpoint 后，夜间目标三件事全部到位；到此停止，等待用户晨审。
