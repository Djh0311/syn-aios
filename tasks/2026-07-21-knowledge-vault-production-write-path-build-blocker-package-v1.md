# 任务包：知识库生产写路修复——test-only helper 构建阻断 + DB-primary 审计接桥 v1

日期：2026-07-21  
状态：**已执行完成，未 stage、未 commit；结果见 `evidence/2026-07-21-knowledge-vault-production-write-path-build-blocker-verification-v1.md`，决定见 `decisions/2026-07-21-knowledge-vault-audit-production-write-path-v1.md`**  
档位：**轻档代码修复（存储一致性敏感）**——只改源码与测试，不启动真实 App、不读写现场 workflow-state/DB/vault；若需要改 M5 生产桥、安全闸或现场数据，立即停下回总指导  
执行者：执行线；总指导回收核实物与唯一 commit  
所属开发线：蓝图能力层 L3 知识库 / M5 生产写路兼容 / 构建阻断修复  
下游状态：本包构建阻断已清；S1B-H2 仍须重冻结修后 debug binary，并在用户在场且额度可用时完成真实 App 两句续验  
上游实现：`b9f7e34`（知识库第一片）  
当前基线：`e9ad7f3`（P2-B 收口；出包时工作树干净）

## 一句话目标

让知识库三类审计不再调用只在 `#[cfg(test)]` 下存在的 `workflow_state_store::atomic_write`，改走既有低频 Batch 2 生产写路，使 non-test 构建恢复，同时守住 workflow-state schema/revision/CAS、DB-primary→JSON 投影、Blocked fallback 和知识库现有权限/路径边界。

## 一、案发事实与结论

### 已知事实

1. `cargo check --lib` 在 `src/knowledge_vault.rs:251` 稳定失败：

   ```text
   error[E0425]: cannot find function `atomic_write` in module `crate::workflow_state_store`
   note: found an item that was configured out
   src/workflow_state_store.rs:128 #[cfg(test)]
   ```

2. `workflow_state_store::atomic_write` 仅为测试夹具入口；`cargo test --lib` 开启 test cfg，所以知识库首片曾得到 **1030/0/44** 假安心；non-test 的 `cargo check/build/tauri build` 不编译该函数，因此真实 App 无法构建。
3. 同次编译的 8 个 unused-import warning 是既有警告，不是本次失败原因，不在本包清理。
4. 当前生产写路已有 `write_m5b_batch2_workflow_state(path, phase, value)`：
   - DB-primary：先记 repository delta，再投影 JSON；
   - JSON-only：走 `write_validated_workflow_state`；
   - Blocked JSON-only：合并降级审计后走既有 CAS，不 rebase 外部冲突。
5. 知识库审计是低频辅助写，归 **Batch 2**；首选接法为：

   ```rust
   crate::write_m5b_batch2_workflow_state(
       workflow_state_path,
       "knowledge_vault_audit",
       &value,
   )?;
   ```

   `knowledge_vault.rs` 虽挂在 `command_registry::knowledge_vault` 子模块，仍可调用 crate root 对后代可见的私有 helper；必须由 `cargo check --lib` 实证，不靠口头判断。
6. 原决策 `decisions/2026-07-20-knowledge-vault-first-slice-v1.md` 挂账写了“Batch bridge 私有不可达、模式默认关”，已被源码调用先例与当前 DB-primary 现场事实推翻。本包收口必须新建决策纠偏，不能继续沿用该假设。

### 未知项

1. 首选调用在当前 module/privacy 结构下是否直接通过 non-test 编译——必须由执行线先做最小替换后跑 `cargo check --lib`。
2. 知识库现有测试用 `{"audit_events":[]}` 作为假 workflow-state；生产 validated writer 会正确拒绝该假形。测试需要改为最小合法 `workflow_state_v0` fixture，具体复用/构造方式由执行线据现有测试先例选择。

### 实施假设

低频知识库审计归 Batch 2，且无需修改 `workflow_db_primary_wiring.rs`、`workbench_sqlite_storage_mode.rs` 或 repository 本体。若首选接法要求改这些生产桥文件，说明勘察前提不成立：**停，不自行扩大任务包。**

## 二、目标与成功标准

1. non-test 路径可编译：`cargo check --lib` 与 debug Tauri build 都通过。
2. 知识库 create/edit/AI write 三事件继续落 `audit_events`，字段与 actor/source_summary 语义不变。
3. JSON-only 写每次消费恰好一个 revision；schema 校验与 CAS 冲突仍 fail-closed。
4. DB-primary 下 audit delta 进入 DB，JSON 投影完成，重启 reconcile 仍 green/lag=0。
5. Blocked JSON-only 继续沿既有降级策略；不得在知识库里另写 fallback/retry/rebase。
6. 知识库权限闸、vault 路径锁、5 个 Tauri command、前端、P2-B 和 H2 源码均零行为变化。

## 三、施工清单

### A. 修生产调用

在 `src-tauri/src/knowledge_vault.rs`：

1. `append_audit_event` 保留现有事件构造、备份和错误传播；只把 test-only raw writer 调用改为 §一.5 的 Batch 2 生产 writer。
2. phase 固定为 `knowledge_vault_audit`，不按事件类型动态造新 phase，避免观测词表无界增长。
3. **不得**删除 `workflow_state_store.rs` 的 `#[cfg(test)]`；不得新增另一个 raw atomic writer；不得直接调用 `write_validated_workflow_state` 绕开 DB-primary 路由。

### B. 修测试夹具与反例

在 `src-tauri/src/knowledge_vault.rs` 的现有测试中：

1. 把最小 `{"audit_events":[]}` 假状态替换为合法 `workflow_state_v0` fixture（含现行 validator 要求的 schema/version/revision 与数组字段）。
2. 锁 create→create→edit 三事件仍按原顺序落账，actor 不漂。
3. 锁 AI source_summary 空时 vault 零写；允许后事件/actor/reason 不漂。
4. 新增 revision 断言：三次成功审计后 revision 恰好 `+3`；不能因备份或投影多消费 revision。
5. 新增坏 schema/陈旧 revision 反例时，不得为了“让测试绿”放宽生产 validator/CAS。

### C. DB-primary 直接证据

优先在现有 `src-tauri/src/workbench_sqlite_storage_mode_m5b_tests.rs` 追加一个窄测试，复用 `db_primary_fixture`：

1. 构造一条 `source_kind=knowledge_vault` 的 workflow audit candidate；
2. 通过 **Batch 2** helper、phase=`knowledge_vault_audit` 写入；
3. 断言 DB `workflow_audit_events` 恰有该 event、JSON 恰有该 event、revision `+1`；
4. 清缓存模拟重启，`reconcile_db_vs_json` 为 green。

该测试证明生产桥合同；知识库实际调用由 `knowledge_vault.rs` diff + JSON-only 单测 + non-test compile 三面合取。**不得**为了从测试跨模块调用 private `append_audit_event` 而扩大 `command_registry`/模块可见性。

### D. 落档纠偏

执行线新增：

- `evidence/2026-07-21-knowledge-vault-production-write-path-build-blocker-verification-v1.md`
- `decisions/2026-07-21-knowledge-vault-audit-production-write-path-v1.md`

新决策只纠偏 07-20 decision 的“DB-primary 挂账/私有不可达/test helper 可用于生产”部分；vault 形态、权限闸、路径锁、渲染器和 5 命令继续有效。`CURRENT.md` 与 `docs/harness-catch-log.md` 由总指导回收时同笔更新，执行线不写。

## 四、允许读取

- `AGENTS.md`、`TASK_TEMPLATE.md`、本包与 kickoff。
- `src-tauri/src/knowledge_vault.rs`
- `src-tauri/src/workflow_state_store.rs`
- `src-tauri/src/workflow_db_primary_wiring.rs`
- `src-tauri/src/workbench_sqlite_storage_mode.rs`
- `src-tauri/src/workbench_sqlite_storage_mode_m5b_tests.rs`
- `src-tauri/src/workflow_state_json_helpers.rs`
- 上游 task/evidence/decision 与 `CURRENT.md`、`docs/harness-catch-log.md`。

派生面已核：生产调用首选只改 `knowledge_vault.rs`；DB-primary 直接证据只追加 `workbench_sqlite_storage_mode_m5b_tests.rs`；不需改 bridge、registry、lib.rs 或前端。

## 五、允许写入

1. `prototypes/productized-desktop-shell/src-tauri/src/knowledge_vault.rs`
2. `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_storage_mode_m5b_tests.rs`
3. `evidence/2026-07-21-knowledge-vault-production-write-path-build-blocker-verification-v1.md`
4. `decisions/2026-07-21-knowledge-vault-audit-production-write-path-v1.md`

超出以上四个文件，必须先停并回总指导说明“为什么首选生产路不成立”。

## 六、禁止事项

1. 禁止删除/放宽 `#[cfg(test)]`；禁止把 raw atomic writer 暴露到 production。
2. 禁止修改 M5 bridge/repository/storage-mode 生产文件、schema validator、revision/CAS、Blocked fallback、安全闸或沙箱。
3. 禁止启动真实 App、读写真实 workflow-state/DB/vault、继续 H2、发送对话、生成/批准方案卡。
4. 禁止修改知识库前端、命令注册、5 个 Tauri command、PermissionDialog、PendingAction、路径锁和 AI 权限语义。
5. 禁止顺手清 8 个 warning、全仓 rustfmt、重构模块或新增抽象。
6. 禁止 stage、commit、push；执行线只回交工作树与证据。

## 七、变更辐射面

改变的假设：知识库 audit 不再“直接 JSON raw 写”，而是低频 Batch 2 生产写。

- **workflow-state revision**：每次知识库审计由 validated writer 消费一个 revision；测试必须对平。
- **DB-primary**：audit 先落 DB 再投影 JSON；reconcile/lag 必须保持 green/0。
- **JSON-only**：现有知识库测试必须从假 JSON 升为合法 schema，事件语义不变。
- **Blocked fallback**：沿用 bridge 既有行为，知识库不自行重试或 rebase。
- **vault 文件与权限**：零变化；但 audit 写失败时现有命令错误传播语义不在本包重设计。
- **构建闸**：以后所有含 Rust 生产代码的包，`cargo test` 之外必须有 `cargo check --lib` 或等价 non-test build，防 test cfg 再遮蔽。

## 八、五态旅程走查

- 说：不涉及。
- 批：不涉及；知识库 AI 写入确认弹窗语义不变。
- 干：不涉及；不得继续 H2 或 chain。
- 交货：不涉及。
- 卡住：修的是 App 构建阻断；修复失败只回传编译/存储证据，不自动改桥或现场数据。

## 九、形状影响

- 任务类型：**紧急缺陷 / 构建阻断**。
- 新增生产代码：原则上 0；`knowledge_vault.rs` 一处调用替换，测试 fixture/断言小幅增加。
- 预计行数：`knowledge_vault.rs` 净增约 10–40 行（主要为测试）；M5B 测试净增约 20–50 行；两份落档文件新增。
- 棘轮文件：不触碰 `lib.rs`、`real_execution_command.rs`、`styles.css`、前端主测试等 ratchet 文件。
- 新增 Tauri command：0；新增 sidecar：0；新增依赖：0；shape 豁免：0。
- 基线 commit：`e9ad7f3`；完成 commit：总指导核收后填写。

## 十、验收标准

### 构建闸（本包新增关键闸）

1. `cargo check --lib`（cwd=`src-tauri`）exit 0；允许既有 8 warnings，禁止 E0425/其他 error。
2. `cargo test --lib` 回到至少 **1030 passed / 0 failed / 44 ignored**；新增测试后 passed 只增不减。若沙箱仅撞已登记 PID/进程权限，两败必须原样披露并由总指导在外层复跑，执行线不得改测试或权限。
3. `../tauri-capability-probe/.tauri-cli/bin/cargo-tauri build --debug`（cwd=`prototypes/productized-desktop-shell`）exit 0；只构建，不启动 App。

### 存储语义

4. JSON-only：三事件、actor/reason、revision `+N` 对平；坏 schema/陈旧 revision fail-closed。
5. DB-primary：目标 event 在 DB/JSON 各一、revision `+1`、restart reconcile green、lag=0。
6. 静态反查：`knowledge_vault.rs` 不再引用 `workflow_state_store::atomic_write`；`workflow_state_store.rs` 的 `atomic_write` 仍保持 `#[cfg(test)]`。
7. M5 定向：M5-B、M5-C、M5-F1 既有测试全绿，证明没有破 projection/fallback。

### 常规闸

8. `npm run typecheck`、`npm run test:offline-interaction` 全绿（前端应 0-diff，防搭车）。
9. shape baseline/check 保持 **13/5/5**，机器脸/hex/退休样式零新增；`git diff --check` 通过。
10. `git diff --name-only` 仅 §五四文件；`src-tauri` 除两册允许文件外零改，真实数据目录零触碰。

## 十一、停止条件

出现任一项立即停下回传，不自行扩大：

- 首选 Batch 2 调用因 privacy/链接结构无法通过 non-test 编译；
- 必须修改 `workflow_db_primary_wiring.rs`、`workbench_sqlite_storage_mode.rs`、repository、schema/CAS 才能继续；
- DB-primary 测试出现 DB/JSON 分叉、revision 多增、reconcile 非 green；
- Blocked fallback 需要新增 retry/rebase；
- 需要真实 App/现场 store 才能证明修复；
- 发现知识库权限/路径边界或 vault 文件原子性存在另一个独立问题。

最后一项单独报新问题，不顺手混修。

## 十二、必须回传（TASK_TEMPLATE 10 项）

1. 做了什么。
2. 改了哪些文件。
3. 新增了哪些测试或 evidence/decision。
4. 哪些结论有直接依据。
5. 哪些仍不确定。
6. 风险和下一步建议。
7. shape gate baseline/check 与各构建/测试数字。
8. start commit=`e9ad7f3`；end commit=`未提交（执行线禁 commit）`。
9. command/sidecar/依赖/棘轮影响（预期全 0）。
10. 被闸拦过的事；无也必须写“无”。

另附“口径披露”：逐条说明是否偏离首选 Batch 2 接法、是否触碰允许写入外文件、是否观察到独立缺陷。

## 十三、总指导回收动作

1. 不信回传，亲跑 `cargo check --lib`、全量 Rust、Tauri debug build、M5 定向、typecheck/离线、shape、diff-check。
2. 逐字核：没有删除 `#[cfg(test)]`、没有 raw production write、没有修改 M5 生产桥、安全闸或现场数据。
3. 核 DB-primary 测试的 event/DB/JSON/revision/reconcile 断言，不接受只靠静态 grep。
4. 判断接受/需要修改/暂停/废弃；回写 `CURRENT.md` 与 catch-log，枚举式 staging；commit message 必含 `catch:`。
5. 本包正式核收、debug binary 重建并重冻结后，才恢复 S1B-H2。
