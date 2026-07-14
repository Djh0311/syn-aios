# 回交：后端 UI 配套三件——系统状态 / 审计账本 / follow-up 命令面 v1

任务包：`tasks/2026-07-15-backend-ui-support-readmodels-package-v1.md` · 执行线 · 轻档 · 2026-07-15 · **未 commit**

> **一句话**：A/B 两个只读读模型已实现并验过（+12 测试全绿·976/0/45）；**C 判定为「包不出来」——不是没做，是做了会破红线，已停下报总指导裁决**（详见第 5、6 项）。

---

## 1. 做了什么

**A·系统状态读模型（已实现·已验）**
新命令 `load_system_status_read_model()`（无入参，取 `AppState.workflow_state_path`），返回：

```ts
{
  storage_mode: "db_primary" | "json_only",   // 配置态，非「此刻实际写哪」
  storage_healthy: boolean,
  observation_day: number,                    // u32·切换当天=第1天·没进观察期=0
  last_degradation: { at_ms: number, reason_human: string } | null,
  recent_catches: { at_ms: number, summary: string }[],  // 恒空·见第 5 项
  gate_summary: string | null,
  warnings: string[]                          // 软着陆人话·不断面板
}
```

**两个字段要合起来读**（前端接的时候别只看一个）：降级**不改模式**——模式缓存仍是 `db_primary`，只把健康翻 Blocked、写落回 JSON。所以降级态的真实长相 =`storage_mode:"db_primary"` + `storage_healthy:false` + `last_degradation` 有值。顶栏健康点应看 `storage_healthy`，不是看 `storage_mode`。

**B·审计账本读模型（已实现·已验）**
新命令 `query_audit_ledger_read_model(request)`：

```ts
// 入参
{ page: number, page_size?: number, kind_filter?: string }   // page 0 基·page_size 默认 50 上限 500
// 出参
{
  total: number,        // **过滤后**总条数，不是本页条数
  items: { at_ms, source, event_type, human_summary, target_ref: string|null, raw_json }[],
  page, page_size,
  storage_mode: "db_primary" | "json_only",
  kinds: string[],      // 流里出现过的全部 event_type（升序去重）——前端过滤下拉直接用，不用自己猜
  warnings: string[]
}
```

聚合 6 源（时间倒序）：`workflow_state` / `project_consultation_proposal` / `plan_authorization` / `supervisor_orchestrator` / `global_supervisor_review` + `global_supervisor_boundary_review` / `session_continuation`。

**C·follow-up 命令面（勘察完成·实现未做·等裁决）** → 见第 5 项。

## 2. 改了哪些文件

**Rust 面（我只对这几个文件负责）**：

| 文件 | 动作 | 量 |
|---|---|---|
| `src-tauri/src/system_status_read_model.rs` | 新增 | A 读模型 + 6 测试 |
| `src-tauri/src/audit_ledger_read_model.rs` | 新增 | B 读模型 + 6 测试 |
| `src-tauri/src/workbench_sqlite_storage_mode_read_model.rs` | 新增 | 只读健康访问器（~30 行） |
| `src-tauri/src/command_registry.rs` | 改 | +10（2 个 mod 声明 + 2 个命令进 handler） |
| `src-tauri/src/workbench_sqlite_storage_mode.rs` | 改 | **+7·纯加法**（mod 声明 + re-export·逻辑本体零改） |

合计：3 新文件 + 2 文件共 17 行插入 / 0 删除。**`lib.rs` 0-diff。**

**⚠️ 分账**：工作树里另有 **37 M + 11 ??** 是**前端施工线**的并行改动（`.tsx`/`.css`/`.ts`/`backlog.md`），**不是我的**。我的 Rust 面 diff 就是上表五个文件，`git status --short -- '*.rs'` 可核。
另：`git diff --check` 报 `src/components/RightDetailPanel.tsx:793: new blank line at EOF` —— **前端线的文件**，我没碰，留给他们收。我的 Rust 面 `git diff --check` 干净。

## 3. 新增了哪些测试或证据

**+12 测试全绿**（temp 根 + `storage_mode_test_lock` + 收尾清模式缓存 —— 照 M5-C 的测试隔离教训做，不摸 live 配置）：

A（6）：json_only 配置态=健康且观察期 0 天 / 观察期起点取**最早**一条 initialized 且切换当天=第 1 天（故意乱序两条防「取数组首条」） / 降级取**最晚**一条且后端人话原样透出 / 闸摘要由常量派生 / **读模型不许写盘（mtime 前后相等）** / 坏 store 软着陆出 warning。

B（6）：主 store 端到端（倒序 + 人话 + at_ms 解析 + kinds 汇总 + raw_json 全字段 + **sidecar 缺席不刷 warning**）/ 无人话字段回落 event_type / 编排审计 `result_summary` 当人话·`run_id` 当 target_ref / 分页+过滤（total=过滤后总数·越界页空 items 但 total 照报）/ **读模型不许写盘** / 坏源软着陆不炸整页。

## 4. 哪些结论有依据

- **两个读模型纯只读**：不是靠"我没写写入代码"自证，是 **mtime 断言**锁死（A/B 各一条）。
- **命令面事实**：三层查证（`#[tauri::command]` 全仓 grep / `generate_handler!` / 前端 `tauri.ts`）**零 follow-up 命令**——C 的「没有」是核过的，不是印象。
- **红线零碰**：`commands.rs`(path-lock) / `real_execution_command.rs`(S1) / `supervisor_action_controller.rs`(复核实证闸) / h4 / h5 / read_cut / stop_write **全部零改**（`git status` 核空）。
- **存储模式语义零碰**：那 7 行是 mod 声明 + re-export，`git diff` 可逐行核。

## 5. 哪些仍不确定

**① C·follow-up：结论是「按包的写法包不出来」，请总指导裁决**（这是本单最重要的一条，别当小观察）

包里假定「有现成 follow-up 通道，包一层薄命令即可」。核实物后**该假定不成立**：

1. **交办卡住脸根本没有 worker 可续话**。前端乙型脸的数据来自 `AutoAdvanceRoleLoopOutcome`（director 链）；而 `DirectorChainOutcome`(`director_agent.rs:882`) **完全没有 `worker_id` 字段**（全文件 grep `worker_id` 零命中）。且 worker 报文是**任务跑完之后**才消费的（`worker_report.rs:189`）——链停下时那个 worker 进程已经结束了。
2. **follow-up 机器在另一条路径上**。`control_core_follow_up_worker`(`mcp/supervisor_orchestrator.rs:853`) 要 `worker_id`，唯一调用者是 `supervisor_action_controller.rs:128`，且必须裹成 `AuthorizedSupervisorAction`（要 live `run_id` + 授权绑定 + 配额 + **主管 LM 提的 proposal**）。那是站 3a/3b 主管试点的路径，不是交办链。**拿用户的一句话伪造一个主管 proposal 去调它 = 破「主管只提动作·Syn 执行」的 canon**，我没做。
3. **能把一句话送到会话的机器确实有，但每台自带多步人闸**：manual relay（preview→confirm[`risk_acknowledged`]→run·hash 钉死）、受控续话真 resume（`inspect`→phase A→phase B[带 `prompt_body` + H2 授权矩阵 + path-lock]）。**把任何一台压成「单命令一键发送」= 把人闸压掉**，直接违反包里「人闸语义不变」。

所以三条路：伪造主管提案（破 canon）／压掉人闸（破红线）／改主管重拆 prompt 塞用户指示（那是新执行路径 + 动 LM prompt，不是薄包装）——**都出了轻档只读包的范围**。按 AGENTS「范围超出预期→停下说一声」，我停在这里。

**真正要拍的是产品/治理问题，不是接线问题**：用户一句话能不能绕过「方案→批」直接驱动执行？现有产品形态是「重新说目标出新方案→批→跑」（乙型脸上已有的次按钮）。乙型的「直接回它一句」本质是给这个闸开一条捷径——**这个板得用户/总指导拍，执行线不能自己拍**。

供参考的三个选项（未实现·等指令）：
- **甲（最小·推荐）**：乙型回话框维持 disabled 占位（前端已按包规立好形态 + 人话「通道接线中」，零假按钮），先只交付 A/B。等拍板。
- **乙（顺产品形态）**：「直接回它一句」= 拿用户这句话预填「重新说目标」的输入 → 走现成 方案→批→跑。零新执行路径、零人闸改动，但**要多点一次批**（诚实但不够"直接"）。
- **丙（真做 follow-up）**：把交办链接到主管试点那套 worker 生命周期（有 worker_id 可 follow up）——**是真架构活**，不是薄包装，得单独立包 + 拍板。

**② observation_day 的「重开」语义**：我按包里明写的「首条 initialized」实现（取最早）。但 CURRENT 记 07-14 16:28 重 seed 后「观察期重开」——若按「重开」算，起点应取**最近一条**而非最早。两种口径差一天多。包与 CURRENT 不一致，**我按包做了，把这个歧义摆出来**：真要「重开」语义，把 `earliest_event_at_ms` 改 `latest` 即可（一行）。

**③ B 的读源**：包写「db_primary 下从 DB 读」，我**两模式都走 JSON loader**，理由写在文件头且不是偷懒：db_primary 是 **lag=0 投影**（DB≠JSON 则启动对账 fail-closed 起不来），两边是同一份事实；且降级审计 `storage_mode_degraded_json_only` **只写 JSON 不写 DB**（写它时 DB 已冻）——**走 DB 读反而漏掉降级记录**。`storage_mode` 字段如实回传。**挂账**：M6 停写 JSON 后，本模块读源必须整体改走 DB（不是遗漏，是已知待办）。

**④ 真机未验**：A/B 只有 temp 端到端，没在真 App 里跑过（本包不含真机）。真机首秀前不宜宣称"能用"。

## 6. 风险和下一步建议

- **C 等裁决**（第 5 项①）。前端乙型脸已是 disabled 占位、不阻塞其它批次；**建议先拍甲/乙/丙再动**。
- **新 flaky 观察项（登记用）**：`tests::c4a_director_final_mark_lm_unavailable_does_not_complete` 在 4 轮全量里**挂 1 轮**（`marker.calls=0`·终标器没被调到），**solo 复跑过**。它**不在** CURRENT §五 登记的「真进程计时 flaky 三员」里，是**新面孔**。已排除是我引起：跳过我 12 条测试跑全量 → 963/1 且失败回到那个已知的 `codex_local_runner` 收割测试，c4a 没复现；且我的新代码除自身测试外**零运行时调用者**。倾向「负载相关的既有竞态、被 +12 测试的并发挪了时序」，但**没证到根**——建议进 §五 警报器观察。
- **`recent_catches` 恒空**：拦截账本 `docs/harness-catch-log.md` 是仓内人手维护的开发档案，**运行时没有数据源**；app 去读仓内 md 属越界。要真填得另立「运行时拦截事件」写点——本包只读，没做。形状已留，前端可先接。
- 观察期口径（第 5 项②）建议顺手拍一下，一行的事。

## 7. shape gate baseline / check 摘要（仓根跑）

| | Errors | Warnings | Info |
|---|---|---|---|
| 基线（我动手前·`4361eb8`） | **13** | 5 | 5 |
| 收口（现在） | **13** | 5 | 5 |

**零净增 ✓**。`Tauri commands: 134 → 136 total; 0 in lib.rs`（+2 = A/B 两命令·`tauri_command_total_increased` 本就是既有 warn 非 error·新命令未进 lib.rs 合规）。

**⚠️ 包里写「gate 14/5/5」与实测不符**：实测基线是 **13/5/5**。原因 = 前端拆巨石那刀把 `ProjectJiaobanPanel` 拆到 2000 线下，Errors 已 14→13（CURRENT §三.1 有记）。**我按实测 13 对账，没照搬包里的 14**——否则会把 +1 净增藏进"符合包"里。

**过程中真被 gate 抓到一次**（见第 10 项）：健康访问器原本直接加在 `workbench_sqlite_storage_mode.rs`，把它从 2989 顶到 3004 行、破 3000 限 → Errors 13→**14**。已治：照 `m5c` 先例拆出 `workbench_sqlite_storage_mode_read_model.rs` 子模块，父文件回落 2996 行 → 13。**不是绕闸，是按闸的意思拆了模块。**

## 8. start commit / end commit

- start：`4361eb8`（施工分发：前端总包 + 后端 UI 配套三件）
- end：**无 —— 按包规不 commit**。产物留工作树（3 新文件 + 2 文件 17 行插入），等总指导核实物后决定。
- 全量基线：**963 passed + 1 已知 flaky（= 964 口径）/ 45 ignored** → 收口 **976 passed / 0 failed / 45 ignored**（+12 = 我的测试；4 轮全量里 3 轮 976/0）。**只增不减 ✓**
- fmt：收口 `cargo fmt --check` **仅历史三**（`codex_db.rs` / `codex_local_runner.rs` / `mcp/storage.rs`）= 与基线同。
  过程中 `workbench_sqlite_storage_mode_read_model.rs` 出过一条 fmt diff（我自查时抓到·差点在本文里错报成"新文件全合格"），已单文件 `rustfmt` 修掉：该文件**无 `mod` 子声明**，故裸 rustfmt 不会递归（记忆教训：`rustfmt <crate根>` 会重排 mod 子文件、能撞穿 0-diff 死线）；格式化后 `git status -- '*.rs'` 核过**零波及**其它文件。父文件 `workbench_sqlite_storage_mode.rs` 带 `mod` 子声明，**没碰**。

## 9. 是否新增 command、sidecar 或触碰棘轮文件

- **新增 command：2 个** —— `load_system_status_read_model`、`query_audit_ledger_read_model`。落点 `command_registry.rs`（**不在 `lib.rs`**·TASK_TEMPLATE 硬规矩）。
- **新增 sidecar JSON 种类：无**（零新表、零新 sidecar、零 DDL）。
- **棘轮文件**：`lib.rs` **0-diff**；`types.rs` / `styles.css` / `ProjectsView.tsx` / `AgentView.tsx` / `real_execution_command.rs` 全**零改**。`workbench_sqlite_storage_mode.rs` 虽非棘轮但贴着 3000 限 —— 我只加了 7 行且**主动把实现挪出去**保它在线下（2996）。
- **新增写点：0**（两个读模型零写入·mtime 断言锁死）。

## 10. 被闸拦过的事（catch-log 原始情报源）

**有 2 条：**

1. **shape gate·3000 行文件上限（真拦一次·当场改设计）**：健康访问器直接加在 `workbench_sqlite_storage_mode.rs` 上 → 2989→3004 行破限，Errors 13→14。**避免了**：把一个已贴线的核心存储文件顶穿、给后面每个碰它的包留个必踩的雷（M5-C 包红线记过「新增文件 <3000·gate 上包已拦过一次」= 同一道闸的第二次战果）。已按闸意拆子模块解决。

2. **核实物·基线自查（拦住我照搬包里的数字）**：包写「gate 14/5/5」，实测基线 **13/5/5**。**避免了**：按 14 对账 → 我那 +1 净增（破 3000 限那条）会被"符合包"掩盖过去，带病进账本。

**另记两条「差点错报」（不算闸拦，但同源教训，值得进账）：**

- **flaky 定性差点报反**：首轮全量 963/1 挂 `codex_local_runner` 收割测试，我按 CURRENT 的「solo 复跑即准」solo 了一次——**也挂**，差点定性成真回归。第二次 solo 才过。回查 CURRENT §五 原文写的是「残余第二竞态 **solo 偶挂≈1/5**」——即 solo 挂一次本就在登记行为内。**教训**：读登记要读全句，别只记住结论那半句。
- **管道 exit 0 差点盖住 FAILED**：首轮我用 `cargo test | tail` 拿基线，shell exit=0 但正文是 `test result: FAILED`。靠读正文才没错报"基线全绿"。后续改为写文件 + 取原始 exit 码。（此坑记忆里已有记录，本单又撞一次 → 值得进 ledger。）
