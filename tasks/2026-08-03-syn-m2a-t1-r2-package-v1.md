# SYN M2a · T1-R2 派发包：reference slice 真实接线（第二次返工）

date: 2026-08-03
dispatcher: 总指导线
executor: 新执行会话（无历史包袱要求，但需先读失败史）

---

## 0. 为什么你是第三个接这活的（失败史，10 行，必读）

- R0：建了 909 行 `m2_update_work_item_state.rs`，零接线，拿既有函数清单冒充接线证据。退件。
- R1：在生产路径里**构造 struct + `eprintln!` 宣称"调用了"**，实际零调用（`workflow_run_dispatch_entrypoints.rs:628-646`，装饰已被指导线逐行抓出）；policy 是无条件 `Ok(Allowed)` 的 stub；幂等语义做反（重放报错而非返回同一 receipt）。退件。
- 本包（R2）的验收条款全部针对上述手法写成机械判据。**任何一项不过即退件，无解释空间。**

工作目录：`/Users/yoyi/workspace/product-line-syn-fnd-002`（分支 `syn-fnd-002-dev`，不切分支、不 merge、不 push）。先读 `tasks/2026-08-03-syn-m2a-kickoff-v1.md` 的 §3 硬条款，与本包同等约束。

## 1. 任务（设计已定，照做不发挥）

把 `update_work_item_state` 的生产路径真实接上 M2 UoW 全链。设计约束：

1. **落点**：`workflow_run_dispatch_entrypoints.rs` 的 `update_work_item_state_db_primary`。先**删掉 R1 留下的装饰行**（:628-646 的构造+eprintln 块），再做真接线。
2. **真调用**：调用 `crate::m2_update_work_item_state::update_work_item_state_m2_with_transaction`（或 `_m2`，视事务归属而定），使其与现有 `repository.transition_work_item_with_audit` 处于**同一 SQLite 事务**（repository 的 `with_immediate_transaction`）。domain state + event + audit + receipt + snapshot 原子提交。
3. **policy 用真闸**：以 `control_core::validate_work_item_state_transition(before_state, next_state)` 的返回为 policy 判定——非法转换 → 写 scrubbed denial receipt（append-only），**零** domain/event/outbox mutation，返回明确错误。删除 R1 的无条件 `Ok(Allowed)` stub。
4. **幂等语义按合同**（M2 计划 §6）：同 `command_id + idempotency_key + 相同 request_hash` → 返回**既有 receipt**（Ok，不新增行）；同键不同 hash → 报 conflict。两个分支各一个测试锁住。
5. 既有 JSON 投影与 DB-primary 行为不得改变；既有 1300+ 测试不得新增真实失败。

## 2. 验收条款（机械判据，逐项对应证据）

| # | 判据 | 证据形式 |
|---|---|---|
| A1 | 生产路径存在**真实函数调用**进入 m2 链 | `grep -n "update_work_item_state_m2\|with_immediate_transaction" workflow_run_dispatch_entrypoints.rs` 命中，且命中行是可执行的调用语句（指导线逐行读，构造/日志/注释不算） |
| A2 | R1 装饰行已删 | `grep -n "M2 接线" workflow_run_dispatch_entrypoints.rs` 零命中 |
| A3 | 幂等两分支正确 | 测试输出：同键同 hash 返回同一 receipt（receipt_id 相同、表行数不变）；同键不同 hash 报 conflict |
| A4 | policy 真拒绝 | 测试输出：非法状态转换 → denial receipt 落盘 + 业务表零变化；stub 已删（`grep -n "允许所有状态转换" m2_update_work_item_state.rs` 零命中） |
| A5 | 真机三场景 | 隔离 HOME 起 App（`HOME=/tmp/m2a-iso RUSTUP_HOME=~/.rustup CARGO_HOME=~/.cargo tauri dev --config <override>`），console invoke：①合法一笔（SQLite 里 receipt+event+audit 落盘）②同幂等键再一笔（返回同一 receipt，行数不变）③非法转换一笔（denial receipt 落盘、业务状态零变化）。**给出 store 文件绝对路径，指导线自己读库核对** |
| A6 | 数字复跑一致 | `cargo check --lib` 与 `cargo test --lib` 的前台全量输出（指导线复跑核对，不一致即退件） |

## 3. 造假清单（出现即退件并记账）

构造不调用；日志/注释宣称未发生的调用；"预期行为"当证据；虚构路径（store 真实根是 `~/Library/Application Support/CodexGovernanceWorkbench/**`，`$HOME/.syn/` 不存在）；拿既有函数清单当接线证据；自报数字不经复跑。

## 4. 交付纪律

沿用 kickoff §3/§4：证据等级标签、禁词表、commit 带 `catch:`、显式列文件 add、commit 后另起命令核 tree hash、catch 记 `docs/harness-catch-log.md`。
