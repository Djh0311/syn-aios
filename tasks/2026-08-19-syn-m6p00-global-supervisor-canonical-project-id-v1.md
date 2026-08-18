# M6P00 子任务：全局主管入口消费 canonical ProjectId

执行记录：本包由 Grok 完成首轮实现，Codex 负责独立复核；任一时刻只有一个源码写者。

## 目标

只修 `global_supervisor_agent` 的正式查询/写入入口：项目根路径仅作为 M1 Project Index 的 exact alias，由普通产品 `AppState` 解析为 canonical `M1ProjectId`；后续链轮查询、方案归属校验与 review 记录写入只使用该 canonical ID。不得再在正式入口中用 `crate::project_id(project_root)` 铸造项目命名空间。

## 允许写域

- `prototypes/productized-desktop-shell/src-tauri/src/global_supervisor_agent.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`（仅必要的 M1 read-port / AppState 接线；能不改就不改）

除此之外不要编辑、格式化、暂存或提交任何文件。工作树里现有改动和所有未跟踪文件都属于别的工作；尤其不得读取或触碰 `m6_*.rs`、`m6_*.rs.bak` 与 `src-tauri/gen/schemas/linux-schema.json`。

## 必须实现

1. `run_global_supervisor_review` 与 `run_global_supervisor_boundary_review` 在任何 consult、review-store 写入或 workflow/proposal 内容返回前，通过 `state.m1_project_index_read_port()` 对 `request.project_root` 做 `resolve_exact_alias`。未安装、未注册、空 alias 或解析失败均返回现有稳定错误码并保持零 consult、零写入。
2. 将解析出的 canonical ProjectId 明确传入同步 core；core、`load_review_input`、review record 与 boundary review record 不得自行从路径派生 project id。
3. B1 链轮查询必须按 canonical project id + workflow id + started_at 精确匹配。若同一 workflow/started_at 同时存在 canonical 记录与 path-derived/foreign 记录，只能读取 canonical 记录。
4. B1/B2 的幂等命中必须核对已有 review 的 `project_id` 等于当前 canonical id；外项目同键记录不得直接返回、覆盖或泄露。
5. B2 读取 proposal 后必须核对 proposal 的 `project_id` 等于当前 canonical id；foreign/path-derived proposal 必须在 consult 和 review-store 写入之前 fail closed。
6. 既有只读 consult 行为、重试/幂等语义、错误人话、store schema、执行合同与其他模块语义保持不变。不得给 canonical 查询增加 path-derived fallback，也不得接受 caller 自报 project id。

## 测试

在同一模块增加或调整定向测试，至少覆盖：

- canonical 与 path-derived/foreign 链轮同键并存时只选 canonical；
- foreign proposal 在 boundary 路径上零 consult、review store 精确零写；
- 已有 foreign review 同键不能被当作 canonical 幂等命中；
- canonical B1/B2 正常成功并在记录中持久化 canonical id；
- M1 index unavailable / alias missing 的正式解析接线有可审计测试（可抽取一个不扩大可见面的内部 helper 测试，不要求 GUI）。

既有 global supervisor 测试必须继续通过。

## 验证

在 `prototypes/productized-desktop-shell/src-tauri` 下运行：

```bash
CARGO_TARGET_DIR=/tmp/syn-m6p00-global-supervisor-target cargo test --lib --offline global_supervisor_ -- --test-threads=1
git diff --check -- prototypes/productized-desktop-shell/src-tauri/src/global_supervisor_agent.rs prototypes/productized-desktop-shell/src-tauri/src/lib.rs
```

不要运行 GUI、真实模型/provider、网络业务写、push/merge/rebase/deploy/release；不要暂存或提交。完成后报告改动文件、关键入口、测试数和退出码。
