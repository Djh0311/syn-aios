# 阶段15 M6 全局主管与内部组织（域层先行，UI 验收载体为新壳）

状态：`ACTIVE / M6_DOMAIN_LAYER_ONLY / NOT_ACCEPTED / NOT_RELEASED`。建立绑定 stage-14 关闭事实（`M5C01-20260818-1939.verdict.md`，M5 产品锚 `c91d8fc`）与用户 2026-08-18 21:49 明确“接下来就是 M6”。本阶段只做 M6 域层与其前置，不宣布 M6 完成、不发布、不激活 Headless Core / Primary / epoch。

总计划：`docs/plans/2026-08-01-syn-stage-6-global-supervisor-and-internal-organization-plan-v1.md`。载体修订依 `decisions/2026-08-17-syn-lightcode-fork-desktop-shell-direction-v1.md` 与 `docs/plans/2026-08-17-syn-lightcode-fork-shell-adoption-plan-v1.md`：M6 域层（合同、service、repository、投影）不依赖壳，先行施工；M6 的产品 UI 与隔离 App 验收载体改为新壳（`syn-shell` fork），待 F2/F3 就绪后另行进行。stage-6 计划验证矩阵里的 “Isolated Tauri” 行按新壳口径理解，域层验证矩阵不变。

输入：`handoffs/2026-08-18-syn-m5-to-m6-and-shell-deferred-debts-v1.md`。ProjectSummary 输入、TemporaryAgent/Advisory 的完整执行 envelope、`m5_runner_entry_registry` 的 `new-grant / guarded-legacy / blocked` 分类均按该交接第 1 节固定，不得放宽。

目标：先完成 M6P00 前置——把 M1 canonical `ProjectId` 的消费面扩到 workflow、项目编排与执行链的正式读写入口，并为 memory relation 的 source owner 建立可判别类型边界；再按 stage-6 计划逐叶施工 M6 域层。M6 跨项目查询不得同时消费 canonical id 与 path-derived id 两套命名空间。

当前用户边界（2026-08-18 用户确认，本阶段全程有效）：

- 以 5600X WSL `/home/synadmin/workspace/syn` 为权威仓库；不 reset、stash、clean、覆盖或丢弃既有未归属 WIP；
- 不接真实个人资料、真实用户项目写入、真实模型/provider、真实消息、账号、凭据、connector 或外部网络业务动作；产品层证据只用隔离 app-data、scratch projects、fake roles/provider/runtime 与白名单合成动作；
- 不 push、merge、rebase、部署、发布。公开 push 只由用户本人明确要求时由总指导执行；
- syn 仓库源码写面同一时间只允许一个施工者。Grok 是优先产品执行者但不是唯一写者；Grok 不可用、卡住或不收敛时，同一长驻 Codex 可在 current leaf 精确写域内接管，以无人值守完成计划为主。B 线 F2 实施与本阶段的源码写面互斥，不得并行；本阶段不进入 `syn-shell` 仓库；
- 6 个未跟踪 `m6_*.rs`（含 `m6_member_directory.rs.bak`）与 `gen/schemas/linux-schema.json` 只读保全，不得暂存、清理、恢复，也不得升格为 M6 基线或实现输入。

干完的标准：

- M6P00 前置与 M6D01–M6D08 各自达到本叶写下的标准，经主管自复核放行；
- 每叶各自独立内容提交与定向证据，逐项进入 done；任一实现不得冒充整阶段完成；
- 每到检查点 authorization 回精确 closed，并在仓库外 `/home/synadmin/workspace/.syn-gates/open/` 写交包；同一长驻 Codex 前台阻塞启动零上下文 Cursor Opus 验收官，PASS 才继续下一段；
- `git diff --check` 通过；M6 写面零未知 delta；stage-12、D0C04/D0C05、M1–M5 冻结合同全程只读保全；
- 阶段关闭另需独立验收结论明确放行，不由本阶段任一叶自行宣布。

允许动：

- `docs/harness/authorization.json`、`docs/harness/plan.md`、`docs/harness/stages/stage-15.md`、`docs/harness/leaves/`、`docs/harness/unfinished/`、`docs/harness/audit/2026-08.jsonl`、`docs/harness/usage/.turn`、`docs/harness/reports/M6*`
- `docs/current-state.md`
- `docs/contracts/`（仅新增增补合同，不改冻结合同正文与旧 hash）
- `docs/plans/2026-08-01-syn-stage-6-global-supervisor-and-internal-organization-plan-v1.md`（仅如实记进度与载体口径）
- `tasks/2026-08-*`
- 产品源码：仅当前 current leaf 明确列出的写域

不许动：

- stage-12、D0C04、D0C05 与 `unfinished/D0C04`、`unfinished/D0C05`（只读保全）
- M1–M5 冻结合同正文与旧 hash；如需补充只能新建增补合同
- M5 已接受的执行合同语义：ExecutionGrant、WorkerReport、receipt/audit/quarantine 边界不得放宽，guarded legacy 不得升格
- OSS-01 与用户自有门面载体（`README.md`、`LICENSE`、`CONTRIBUTING.md`、`SECURITY.md`、`package.json` 与 `src-tauri/Cargo.toml` 的 license/repository 字段，已由 `c1025ba` 独立提交）
- `syn-shell` 仓库、F2/F3/F5 实施、壳采纳
- M7–M11、Headless Core、Primary/authority epoch 激活或实现
- 真实资料/项目写入、真实模型/provider/message/connector、凭据、外部网络业务写、push/merge/rebase/deploy/release
- 伪造 Hook receipt、authorization、stage/leaf、测试或 App 证据

停止与回滚：

- 出现 secret/credential/真实运行数据/未知 symlink/special file、要求伪造证据、M1–M5 冻结合同或 stage-12/D0C04/D0C05 意外修改、候选 commit 与新鲜证据 SHA 不一致、`syn-shell` 写面被触碰时立即停止并交总线；
- authorization 保持精确 closed 两字段，不手填 executionReceipt/session/turn/expiresAt；每次 leaf 切换先 closed 再按真实 receipt 重新签发，禁止旧 active JSON 跨 leaf 续用。

## 检查点纪律（2026-08-19 00:35 用户改定为同一 Codex 阻塞续跑）

本阶段的叶子已一次排完，主管在同一段内连续施工、逐叶自复核；到检查点暂停仓库施工并先验收当前段：

| 检查点 | 覆盖叶 | 交包文件 |
|---|---|---|
| （前置） | M6P00 | `stage-15-m6p00-<YYYYMMDD-HHMM>.md` |
| CP1 | M6D01、M6D02 | `stage-15-cp1-<YYYYMMDD-HHMM>.md` |
| CP2 | M6D03、M6D04 | `stage-15-cp2-<YYYYMMDD-HHMM>.md` |
| CP3 | M6D05、M6D06 | `stage-15-cp3-<YYYYMMDD-HHMM>.md` |
| 阶段交包 | M6D07、M6D08 | `stage-15-<YYYYMMDD-HHMM>.md` |

到检查点的硬动作，缺一不可：把 `authorization.json` 打回精确 closed 两字段；在 `/home/synadmin/workspace/.syn-gates/open/` 写该检查点的交包文件（含所覆盖各叶的候选与记账 SHA/tree、每叶做了什么、主管自复核七项的原始证据与退出码、仍未完成事项与欠账、请求验收的确切范围、实际写域清单）；原始日志留在 `.syn-gates/evidence/`；确认 `docs/harness/leaves/` 为空后，由当前同一 Codex 前台阻塞启动 Cursor Opus 独立验收官。`open/` 里同时只应存在一个交包文件；写之前确认没有未处理的旧文件，有就按硬停点处理。

不得跳过检查点、不得把两个检查点合并交包、不得在未获 PASS 时进入下一段第一叶。验收期间仓库保持只读，Codex 每两分钟发一次会话心跳；收口后 `docs/harness/leaves/` 应为空，下一段第一叶等 PASS 后才拉入。

检查点验收的执行载体：`checkpoint-loop.sh` 已退役，不退出、不等待驱动、不另起第二个 Codex。当前 Codex 使用 `/home/synadmin/workspace/.syn-gates/verifier-prompt-checkpoint.md` 生成提示词并前台阻塞运行 `cursor-agent --trust --force --output-format text`；验收官把结论写到 `verdicts/<交包名>.verdict.md`，首行顶格 `VERDICT: PASS` 或 `VERDICT: FAIL`。Codex绝不写、改、删 verdict。PASS 时把非阶段终包移入 `handled/`，按 verdict 点名欠账分流，拉入下一段第一叶并用同一真实用户 receipt 重签 authorization；FAIL 时只在当前段和 verdict 点名范围返修后重交。同一检查点连续两次 FAIL、结论首行不可读或文件未生成时，写 `stage-15-halt-<YYYYMMDD-HHMM>.md` 并停止。阶段终包 PASS 后保留在 `open/`，停止并留给总指导做最终处置。

## 叶子

顺序依 stage-6 计划第 5 节依赖图（ORG-001 → ORG-003 → ORG-002 → ORG-004；ORG-001 → ORG-005 / ORG-006；ORG-003+005+006 → ORG-006A），并按"syn 源码同一时间单写者"串行化：

- [x] M6P00 canonical ProjectId 消费扩面与 relation owner 类型化前置（内容 `4147454`、记账 `cf1cb25`；独立检查点 `stage-15-m6p00-20260819-0342` PASS）
- [x] M6D01 跨项目与成员合同冻结（ORG-001，只写合同；内容 `80ddebd`，主管自复核 PASS）→ CP1
- [x] M6D02 顶层 Global Supervisor 持久 RoleSession（ORG-003；内容 `651a8fb`，主管自复核 PASS）→ **CP1 独立 verdict `stage-15-cp1-20260819-0521` PASS**
- [x] M6D03 只读跨项目 query 与 advisory（ORG-002；owner 前置 `977770f`、内容 `60a8e19`，主管自复核 PASS）→ CP2
- [x] M6D04 Secretary consult Handoff（ORG-004；内容 `ec1ba99`，主管自复核 PASS）→ **CP2 独立 verdict `stage-15-cp2-20260819-0733` PASS**
- [x] M6D05 稳定成员目录（ORG-005；内容 `a58815f`，主管自复核 PASS）→ CP3
- [x] M6D06 临时 agent 历史投影（ORG-006；内容 `274cb08`，主管自复核 PASS）→ **CP3 awaiting independent verdict**
- [ ] M6D07 独立多视角会诊（ORG-006A）
- [ ] M6D08 M6 域层集成回归与阶段候选 → **阶段交包**

明确不在本阶段：ORG-007 双项目隔离 App 验收与顶层入口 UI，载体在新壳，记在 `unfinished/M6S01-dual-project-isolated-app-acceptance-on-new-shell.md`。因此本阶段通过也只到 M6 域层，不构成 M6 完成。

文件名纪律（总指导钉死，避免覆盖只读保全）：6 个未跟踪 `m6_*.rs` 占用了 `m6_cross_project_query.rs`、`m6_global_supervisor_session.rs`、`m6_member_directory.rs`（含 `.bak`）、`m6_organization_identity.rs`、`m6_temporary_agent_history.rs` 这几个名字。本阶段新增域层源码统一用 `m6_org_*` 前缀，禁止使用上述任一名字。
