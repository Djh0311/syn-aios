# SYN M2a 阶段总派发 Kickoff · v1

date: 2026-08-03
status: ACTIVE — 新会话拿本文件即可执行 M2a，不需要其他上下文。
author: 总指导线（已驳回上一轮"M2 COMPLETE"虚报，见 catch-log 2026-08-03 六条）

---

## 0. 先读：上一轮为什么被驳回

上一轮把 M2 报成"完成"，指导线核实物后驳回。事实：DAT-002..008 交付的 9 个模块（4011 行）中 7 个**外部调用者为 0**，除 lib.rs 的 mod 声明外没有生产文件被改动——是一个自洽但零接线的死代码集群。DAT-007"真实切换"未碰真实 store（指纹坐实）；DAT-008"隔离 App 验收"是进程内函数；DAT-001B 的路径 `$HOME/.syn/` 系虚构。

**本轮的一切规则都为防止重演而设。§3 的硬条款与交付物同等重要，违反即退件。**

权威顺序：当前用户指令 → 本文件 → `docs/harness/CURRENT.md` → M2 阶段计划（§4/§5/§9 为最终依据）→ `AGENTS.md`。

工作目录：`/Users/yoyi/workspace/product-line-syn-fnd-002`，分支 `syn-fnd-002-dev`。不切分支、不 merge、不 push。

## 1. 真实起点（已核实，可采信）

- DAT-001 合同已提交（`49a7e4c`，491+298 行），reference slice 冻结：`workflow-state-sidecar` / aggregate `workflow_state` / command `update_work_item_state`。**注意 §2-T0：合同本身有缺陷要修。**
- 基座集群在 `prototypes/productized-desktop-shell/src-tauri/src/`：`m2_dto / m2_ports / m2_workflow_state / m2_outbox / m2_projector / m2_legacy_adapter / m2_domain_cutover / m2_isolated_app_acceptance / workbench_sqlite_schema_m2`（已在 lib.rs 注册，33 个自含单测真实通过）。端口抽象与模块划分**经指导线抽审认可，可以在此基础上接线**。
- 验证基线：`cargo check --lib` exit 0 / 683 warnings；`cargo test --lib` 1338 passed / 1 failed / 45 ignored（唯一失败是既有 `sqlite_production_preflight...`，修它是本轮 T4 内容）。
- M1 真机验收机制现成可用：`test-fixtures/fnd-006-acceptance/acceptance-record-2026-08-03.md` 的隔离 HOME + `withGlobalTauri` override + console invoke 方法。

## 2. 任务清单（五块，顺序执行）

### T0 · 前置返修：领域词表对齐（合同 + 代码）

生产 work item 状态机是 `draft / ready_to_dispatch / running / ready_for_review / completed / failed / timed_out / retry_pending / needs_changes / paused / blocked`（`lib.rs:950` 附近的状态转移表），而 DAT-001 合同与 `m2_workflow_state.rs` 用的是自创五态（draft/ready/in_progress/completed/failed）。**接线前必须对齐**：以生产状态机为准修合同与 DTO，映射表写进合同附录。同法排查集群内其他自创词表（dispatch 状态、receipt 状态 vs 生产 vocabulary）。

### T1 · 接线：reference slice 走真实命令路径

把 `update_work_item_state` 的真实 Tauri 命令路径接上 UoW：policy → UoW → domain state → event → audit → receipt → current snapshot，同一事务原子提交；policy-denied 走独立 scrubbed denial receipt、零业务 mutation；幂等键重放返回同一 receipt。**判据**：该命令的生产调用点真实经过 m2 组件（grep 外部调用者 > 0 且指向生产命令路径，不是测试）；既有 1300+ 测试不因此破坏。

### T2 · DAT-008 重做：真隔离 App 崩溃恢复验收

按 FND-006 同款机制起真隔离 App（HOME=temp + RUSTUP_HOME/CARGO_HOME 指回 + `withGlobalTauri` override），覆盖：冷启动、写一笔、commit 前强退、commit 后 receipt 丢失、投影失败、重启恢复、重复 command、JSON-leading。**判据**：真机日志 + store 文件前后指纹 + 逐场景记录（照 `acceptance-record-2026-08-03.md` 的格式写 `test-fixtures/m2a-acceptance/`）。进程内函数不叫隔离 App。

### T3 · DAT-001B 重做或降级

按真实 store 路径（`~/Library/Application Support/CodexGovernanceWorkbench/**`）做只读测量出真 manifest；做不到就显式降级为 HOLD 并写明缺什么。**判据**：文档里每条路径都实测存在（附 `ls`/`stat` 输出），`STATIC_OPENING_ONLY` 标签只能出现在实测之后。

### T4 · §0.4 残留

① 真 grant store + mint/load/verify（替换 worker_report 里的格式校验+自铸 grant）；② 修 `sqlite_production_preflight_blocked_creates_no_db_or_report`（preflight 期望拦截实际放行的既有 bug）；③ 进程夹具族环境性失败并案定性；④ code-map advisory 清零（新模块能力映射 + `index.json` invalid domain path）。

## 3. 防虚报硬条款（违反即退件）

1. **每条交付必须附可机械复核的实物证据**：接线类给生产调用点 grep 输出（外部调用者数 + 指向）；真机类给日志文件路径 + 指纹 diff；文档类给每个路径/数字的实测命令与输出。**测试数字不计入接线证据。**
2. **证据等级标签强制**：每项交付自标 STATIC / UNIT / TEMP-INTEGRATION / ISOLATED-RUNTIME / LIVE，标签与证据形式必须匹配（进程内测试最高只能标 TEMP-INTEGRATION）。
3. **禁止词汇**：没有对应实物证据时不得使用"完成/COMPLETE/已通过/已验收/真实切换"；可以说"已实现，未验证"。
4. **CURRENT.md 回写必须符合 v2 合同**（STATUS ≤5、SAFETY ≤2、goal 必填、枚举值正确），回写后自跑 `node scripts/harness-v2/project-context.js --target .` 必须报 OK。
5. **自报数字以指导线复跑为准**：`cargo check --lib` 与 `cargo test --lib` 由指导线前台全量复跑核对，warning 数变化做全集 diff 定性。
6. 交付顺序即 T0→T4；每块完成回传后，**指导线核实物通过才派下一块**。一次交多块时按块分别举证。

## 4. 交付纪律（沿用 M1/M2 kickoff，未变）

完成必附"怎么验的 + 真证据"；commit message 带 `catch:` 标记；`git add` 显式列文件；commit 后另起命令核 `git log` + `rev-parse HEAD^{tree}` 比对 `git write-tree`；catch 记 `docs/harness-catch-log.md`；注释不写找不到对应代码的安全断言；批量验证前台全量落盘。

## 5. M2a 通过标准（全部满足，指导线逐条机械复核）

- T0：合同附录含生产状态机映射表；集群内无自创词表（grep 生产状态名应有命中）。
- T1：`update_work_item_state` 生产路径真实经过 UoW 全链（调用点 grep + 真机 console invoke 一次合法一次被拒，拒绝有 scrubbed denial receipt 落盘）；全量测试不比基线多任何新失败。
- T2：`test-fixtures/m2a-acceptance/` 有逐场景真机记录与指纹证据。
- T3：manifest 全路径实测或显式 HOLD。
- T4：grant 真 store 替换格式校验（grep 坐实）；sqlite preflight 测试转绿；进程夹具族有定性结论；code-map 检查零 WARN。
