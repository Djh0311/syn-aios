# Root Treatment / R2-T0 Inline Tests Migration Reassessment After R3-A13 v1

日期：2026-06-12

状态：已完成，hash 已回填。

Planning baseline commit：`329b2d9bda1adcd6b67356a6fe752d8cca472817`

Implementation commit：`05ccd9fe5e7a794a95b3bd7648be332895ab97ad`

Review result：`CLEAR`，P0/P1/P2 无；复核线程继续复用 `019eb850-0698-7f70-a9b2-e7d0d668ccf5`。

Checkpoint commit：`53a8d2aee4236eb216f4798cfe3f1ccd15ba9687`

本文是 Root Treatment / Stage R 的 R2 后段 inline tests 迁移复评任务包。它落实 `handoffs/2026-06-12-root-treatment-execution-strategy-review-claude-to-codex-v1.md` 的 P1-2 要求：以 R3-A13 Level A 事务采纳完成为输入，重新判定 `lib.rs` inline tests 迁移是否解锁，并写成显式决定。

## 0. 全局主管理解

已知事实：

- R2 closing / R3 preflight review 当时结论为 `DONE_WITH_CONCERNS`，指出 `lib.rs` inline tests 巨石约 12,247 行、213 个 `#[test]`。
- 当时不建议立即迁移 inline tests 的主要原因是：
  - 共享 fixture / stub runner 底座未拆。
  - 跨 store transaction 语义、R3 SQLite / test support 设计未冻结。
- R3-A13 Level A 已完成，证明 memory candidate adoption、formal memory record、formal memory version、memory audit event 和 workflow audit event 可在同一个 fixture / temp SQLite transaction 内提交。
- R3-A13 Level B 未执行，真实 workbench state root 未读取，真实 production DB 未创建，JSON / sidecar 未停写。
- R4-A50 已改 shape gate 为 historical-low ratchet，后续任务必须说明降低哪个棘轮指标。

核心判断：

```text
R2 inline tests 迁移已“部分解锁”，但不是全量解锁。
```

## 1. Execution Mode

Execution Mode：Supervisor-led reassessment, no product code change。

Multi-Agent Policy：

- 主管线负责当前态扫描、复评结论、任务包、evidence、handoff 和 checkpoint。
- 复核线只读审查当前复评结论和后续任务建议。
- 本任务不修改 Rust / TS / UI / DB / sidecar schema / workflow state schema。

## 2. Scope

允许：

- 读取 R2 closing / R3-A13 / R4-A50 任务包、evidence、handoff 和正式计划。
- 静态扫描 `lib.rs` inline tests 当前行号、测试数量、fixture / runner 分布。
- 运行只读 / 测试命令建立当前基线。
- 写当前任务包、evidence、handoff。
- checkpoint 同步当前入口文档。

禁止：

- 迁移任何 inline test 源码。
- 修改产品代码、UI、CSS、Rust/Tauri 产品路径、DB、sidecar schema、workflow state schema 或真实执行路径。
- 执行真实 `codex exec` / `codex exec resume`，发送 prompt，读写 `/Users/yoyi/.codex`。
- 读取 secret/token/`.env`/keychain/OAuth/provider credential/full transcript。
- 启动 Tauri / Browser / Chrome / Vite dev / screenshot。
- 执行 R3 Level B、K3-B1 retry、K3-B2、多 agent 并行真实执行。
- 解冻 backlog 功能。

## 3. Reassessment Decision

结论：`PARTIALLY_UNLOCKED_WITH_GUARDS`

可以启动：

- R2-T1 inline tests 迁移专项。
- 第一批只允许迁移低耦合、纯 fixture / temp 文件 / native parser / read-model 类测试。
- 第一批可以使用 crate-root `#[cfg(test)] include!("...")` 作为保守过渡，以避免一次性扩大生产函数可见性。
- 每个迁移包必须让 `lib.rs` 行数下降，并写明预计降低多少行。

暂不允许：

- 全量搬迁 `#[cfg(test)] mod tests`。
- 迁移会改变存储语义、transaction boundary、真实执行 gate、K3-B / J / H 真实执行冻结语义的测试。
- 单独迁移已低于阈值的新测试 helper，除非它是降低 `lib.rs` 棘轮指标的同一包内部准备工作。

## 4. Unlocked Slices

优先级 1：

- `2505-2894` transcript catalog / dispatch readback stats 测试。
- 原因：边界清晰，主要使用 temp sqlite / temp rollout fixture，不需要真实 `.codex`，且与 R3 transaction 语义低耦合。
- 推荐落点：`src-tauri/src/lib_transcript_readback_tests.rs` 先由 crate-root test module include；后续再评估迁到 `codex_transcript.rs` / `codex_db.rs` local tests。
- 预期收益：降低 `lib.rs` 约 350-500 行。

优先级 2：

- `1703-2520` diagnostics / provider / session continuation / adapter boundary read-model tests 中不涉及 K3-B runtime prompt guard 的子集。
- 原因：多数是纯只读 descriptor / summary 派生。
- 限制：K3-B command guard 相关测试先留在 `lib.rs`，避免和 Stage L / K 冻结语义混在一起。

优先级 3：

- `6802-7293` workflow_state init / bootstrap / task draft / audit helper 中 store-local 测试。
- 原因：workflow_state focused tests 当前通过；store helpers 已有独立模块。
- 限制：不改 workflow state JSON shape，不迁 real-state ignored test。

优先级 4：

- `5709-6656` memory lint / maintenance / mature pattern 本地 store / preview 测试。
- 原因：多数不依赖 R3 Level B；可按 store-local 分批迁。
- 限制：跨 candidate -> formal adoption 的端到端组仍暂缓。

## 5. Still Blocked / Deferred Slices

继续暂缓：

- `7304-9300` 中的 memory candidate adoption 跨 candidate + formal store 组。
- `11374-12306` workflow node dispatch execute / readback / user reviewed instruction / timeout / failure 组。
- `12306-13165` workflow machine / director review / offline role 中依赖 runner fixture 的端到端组。
- `13200-13965` ignored real task package file generation、fixture factories、stub runners、`read_json_file` 共享底座。

暂缓理由：

- R3-A13 只完成 Level A，不等于生产 DB / read-cut / stop-write 解锁。
- Stub runner / `CodexResumeRunner` 相关测试是多域共享底座，直接搬迁容易制造可见性扩散和真实执行边界混淆。
- ignored real-state test 读取真实工作台状态，治理期不应顺手改动或重新激活。

复评触发点：

- R2-T1 / T2 完成 test support 和 transcript/readback 低风险迁移后。
- R3 Level B window plan 完成并明确真实迁移窗口边界后。
- 如迁移中发现必须扩大生产可见性或改测试语义，立即停止并重新复评。

## 6. Next Task Recommendation

下一任务建议：

```text
R2-T1 Rust Inline Transcript / Readback Test Extraction
```

建议边界：

- 只迁 `lib.rs` 中 transcript catalog / dispatch readback stats 相关测试和其局部 fixture。
- 允许新增 `src-tauri/src/lib_transcript_readback_tests.rs`，由 `#[cfg(test)] mod tests` 内 `include!` 引入。
- 不迁 cross-store memory adoption、workflow execution runner、workflow machine、ignored real-state test。
- 不改产品函数签名，不新增 public API，不改测试断言含义。

验收建议：

- `lib.rs` 行数下降约 350-500 行。
- 新测试文件低于 Rust 3,000 行阈值。
- `cargo test --lib transcript`
- `cargo test --lib dispatch_readback_stats`
- `cargo test --lib`
- `node scripts/harness/workbench-shape-gate.js --mode check`
- `git diff --check`

## 7. Acceptance Boundary

可接受为：

- R2 后段 inline tests 迁移复评完成。
- 明确 `PARTIALLY_UNLOCKED_WITH_GUARDS` 决策。
- 明确第一批可迁和暂缓测试域。
- 明确 R2-T1 推荐任务方向。

不可接受为：

- inline tests 迁移已开始或完成。
- `lib.rs <= 3,000` 已达成。
- R2 全部完成。
- R3 Level B 执行或完成。
- 生产 SQLite 迁移、read-cut、stop-write、rollback production workflow 或多 agent 并行真实执行解锁。
- 真实 Codex 执行、`.codex` 读写、UI / 产品行为修改或 backlog 功能解冻。

## 8. Review Result

只读复核线 `019eb850-0698-7f70-a9b2-e7d0d668ccf5` 结论：`STATUS: CLEAR`。

复核结论：

- P0：无。
- P1：无。
- P2：无。
- 复核确认当前工作树只有三份 R2-T0 文档未跟踪，tracked diff 为空。
- 复核确认 `PARTIALLY_UNLOCKED_WITH_GUARDS` 有 R2 closing、R3-A13 和 R4-A50 依据链支撑。
- 复核确认 R2-T1 推荐方向符合棘轮收益规则，没有解锁 R3 Level B、production DB/read-cut/stop-write、多 agent 并行真实执行、真实 Codex 或 `.codex` 读写。
