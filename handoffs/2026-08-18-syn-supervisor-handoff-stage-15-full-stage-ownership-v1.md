# Syn 开发主管交接：stage-15 全阶段承包 v1

日期：2026-08-18

状态：`ACTIVE_HANDOFF / STAGE_15_ACTIVE / M6_DOMAIN_LAYER_NOT_STARTED`

用途：交给接任的开发主管会话（Codex）。上一任会话上下文过长，用户决定换人并同时改分工。本文件是你的全部起点，不要依赖任何聊天历史，不要凭记忆推进。

## 1. 你的角色（2026-08-18 用户改定）

你**承包整个 stage-15 的实现，包括逐叶验收**。总指导不再逐叶把关，只做三件事：阶段规划、问题裁决、阶段完成后的独立验收。

因此：

- 每一叶做完，**由你自己起独立复核并判定放行**，然后自己做生命周期收口（归档该叶、记账、拉下一叶成为唯一 current leaf）。不要为每一叶去等总指导。
- 一叶通不过你自己的复核，你自己开返修，改完重跑直接受影响的验证与必要回归，直到真实通过。
- 阶段全部叶子做完后停下交包，等总指导做阶段验收（第 5 节）。
- 撞到硬停点（第 6 节）立刻停下上报，不许自行放宽。

叶子的定义与排序权归总指导。stage-15 当前只排了一叶（`M6P00`）；它做完后你**不许自建 M6 域层叶子**，停下让总指导排。

## 2. 开工必读（只读，不要动文件）

1. `/home/synadmin/workspace/syn/AGENTS.md`
2. 本文件
3. `handoffs/2026-08-18-syn-m5-to-m6-and-shell-deferred-debts-v1.md`（M5→M6 输入、执行 envelope、禁止继承边界）
4. `docs/plans/2026-08-01-syn-stage-6-global-supervisor-and-internal-organization-plan-v1.md`（M6 阶段计划，含 2026-08-17 载体修订）
5. `docs/harness/plan.md`、`docs/harness/stages/stage-15.md`
6. `docs/harness/leaves/` 下唯一 current leaf
7. `docs/harness/authorization.json`
8. `/home/synadmin/workspace/.syn-gates/verdicts/` 下最新结论（如果有；那是总指导的阶段结论或历史节点结论，其欠账与返修意见必须执行）

## 3. 当前事实基线（2026-08-18 22:10）

- 仓库 `/home/synadmin/workspace/syn`，`HEAD` = `d33e348`。远端 `main` = `4791654`，本地 3 个未推提交是当前状态，**你不许 push**。
- stage-14 已关闭；M5 = `SCOPED PRODUCT-CHAIN PASS / NOT_RELEASED`，产品内容锚 `c91d8fc72bcbf80186736caff841cb7a9b0660d1` / tree `fe2d982267d474631ca4ea7b3f90ed846f72a89d`。
- stage-15 active：M6 全局主管与内部组织，域层先行，产品 UI 与隔离 App 验收载体改为新壳。
- 唯一 current leaf：`docs/harness/leaves/M6P00-canonical-project-id-consumption-and-relation-owner-typing.md`。它是**前置**，不是 M6 域层施工。
- `authorization.json` 是精确 closed 两字段；按你自己的真实 receipt 签发，每次 leaf 切换先 closed 再重签，禁止旧 active JSON 跨 leaf 续用，禁止手填 executionReceipt/session/turn/expiresAt。
- 环境：WSL；`rustfmt` 1.9.0，仓库已统一到该基线（`c60caa9`）；`cargo check --lib --offline` 增量约 9 秒，有 883 条既有 warning，不要当失败；`sqlite3` 命令不存在，查库用 Python 的 `sqlite3` 模块。

## 4. 每一叶的工作循环

1. **锚定标准**：只按 current leaf 自己写的"做完的标准""允许动""不许动"施工，不得扩大。
2. **拆任务包给 Grok**：产品源码只许 Grok 写，命令 `grok -m grok-4.6 --reasoning-effort high`，一次一个窄任务包。syn 仓库同一时间只允许一个写者。你自己只写 harness 记账、任务包与文档。
3. **你自己复核**：Grok 每次交活你先自查，不合格自己开返修。
4. **独立复核后放行**：在 disposable checkout 上，按下面七项逐条产出你自己的证据，不信任何自报数字。
   - **写域**：`git show --stat` 逐个候选与记账 SHA，比对 leaf 允许写域。越界即不放行。
   - **冻结物**：`git diff` 确认 M1–M5 冻结合同正文与旧 hash 未改，只允许新增增补合同。
   - **WIP 保全**：`git status --porcelain=v1 -uall`，确认既有未归属 WIP 仍在，没被暂存、提交、reset、stash、clean。
   - **重跑**：`git worktree add --detach /tmp/syn-verify-<短SHA> <SHA>`，独立 `CARGO_TARGET_DIR`，重跑该叶要求的定向测试与 `cargo check --lib --offline`，记录真实 passed/failed 与 exit code。
   - **实质**：读 diff 本身——声称的行为是否真实现；是否只存在于 `#[cfg(test)]`、env 门控或 fixture 而普通路径走不到；失败路径是否真 fail-closed（无静默 fallback、默认值兜底、path 派生、自动导入）；调用点是否真在真实 entrypoint 的调用链上。空壳、死代码、只有测试能触发的实现不放行。尤其盯"实现了但运行期没有消费者"的空转。
   - **不越级**：有没有把离线 fixture、disposable checkout、协议推断说成真实运行、GUI 证据、发布或部署；有没有把已获得的 scoped PASS 反向写成 FAIL。
   - **欠账**：即使放行，也把该叶标准没覆盖但影响后续的问题逐条写清，指明由哪个后续叶关掉。
5. **收口**：归档该叶（原子移动一个文件）、在 `stage-15.md` 与 `plan.md` 据实标记、写审计行、写叶报告、把下一叶从 `unfinished/` 拉回 `leaves/`（若总指导已排）。收口只做证据支持的部分，不得顺手关阶段或宣布里程碑完成。
6. **欠账分流纪律**：只有"不修则普通产品对真实用户不可用"的问题才可以成为新的 current leaf；其余一律写成 `docs/harness/unfinished/` 的后续叶文件如实记录，不阻塞排序。跨平台常量、断言收紧、manifest hash 漂移、报告措辞、下游接收方责任都属后者。判不准就按"记录"处理，别按"返修"处理。
7. **原始日志**：留在 `/home/synadmin/workspace/.syn-gates/evidence/<LEAF>-<短SHA>/`。

## 5. 阶段做完怎么交

stage-15 全部已排叶子真实通过后：

1. `authorization.json` 打回精确 closed 两字段；
2. 在 `/home/synadmin/workspace/.syn-gates/open/` 写阶段交包文件，命名 `stage-15-<YYYYMMDD-HHMM>.md`，内容含：阶段名、各叶候选与记账 SHA/tree、每叶做了什么、你自己的复核结论与原始证据（命令、退出码、日志路径）、明确仍未完成的事项与欠账、请求阶段验收的确切范围、全阶段实际写域清单；
3. 原始日志根留在 `.syn-gates/evidence/`；
4. 结束进程。不要自行关闭 stage-15、不宣布 M6 完成、不进入 F2/F3/F5 或壳采纳。

`open/` 里同时只应存在一个交包文件；写之前确认没有未处理的旧文件（有的话说明上一次未被验收，停下并在日志里说明）。

## 6. 必须停下上报的硬停点

- 需要超出当前 leaf 写域，或需要改冻结合同正文/旧 hash；
- 需要 push、merge、rebase、tag、部署、发布；
- 需要真实凭据、真实 provider、真实账号、真实个人资料或外部网络业务写；
- 需要动 `stage-12`、`unfinished/D0C04`、`unfinished/D0C05`、`OSS-01`、用户自有载体或只读保全项；
- 需要进入 `syn-shell` 仓库或启动 F2/F3/F5；
- 需要激活 M6 域层之外的里程碑（M7–M11、Headless Core、Primary/authority epoch）；
- 返修连续不收敛，或标准本身有歧义、判不准；
- 发现 secret/credential/真实运行数据/未知 symlink/special file，或候选 SHA 与证据不一致。

上报方式同第 5 节：authorization 回 closed，在 `open/` 写文件说明，然后停。

## 7. 不许动与自有载体

- **用户自有**：`.claude/harness-lite/*` 与 `AGENTS.md`/`CLAUDE.md` 的规则改动（`0db02ef`）；开源门面 `README.md`、`LICENSE`、`CONTRIBUTING.md`、`SECURITY.md`、`package.json` 与 `src-tauri/Cargo.toml` 的 license/repository 字段（`c1025ba`，精确 7 路径）。不改写、不并进任何候选、不当来源不明 WIP 归责给自己。`OSS-01` 保持 unfinished，不得提升为 current leaf、不得据它 push/tag/release 或提交任何外部表单。
- **总指导自有**：`c60caa9`（rustfmt 1.9.0 基线，32 个源文件，纯格式）。此前工作树里那 21 个未提交的格式改动已不存在，不要再当未归属 WIP。若你或 Grok 用了不同版本 rustfmt 又产生大面积重排，视为噪声，须在交包里如实说明。
- **只读保全**：6 个未跟踪 `m6_*.rs`（含 `m6_member_directory.rs.bak`）与 `src-tauri/gen/schemas/linux-schema.json`。不暂存、不清理、不恢复，不得升格为 M6 基线或实现输入。逐项 hash 见 `docs/harness/reports/M5R08-protected-wip-attribution-v1.md`。
- **M5 已接受语义不得放宽**：ExecutionGrant、WorkerReport、receipt/audit/quarantine 边界；`m5_runner_entry_registry` 的 `new-grant / guarded-legacy / blocked` 分类不得改判，guarded legacy 不得升格。TemporaryAgent/Advisory 只能引用完整执行 envelope，不得从 report 自报、缺字段兼容或 runtime trace 推导正式执行身份。
- **B 线不在你的写面**：新壳在 `/home/synadmin/workspace/syn-shell`（用户自有 fork `Djh0311/syn-shell` 的 `syn` 分支）。不进入该仓库、不读写它的 harness、不启动 F2/F3/F5。F2 起的壳侧实施与 stage-15 争用 syn 源码写面，排序由总指导决定。

## 8. 提交署名

仓库 local git 身份是 `Djh0311 <277674664+Djh0311@users.noreply.github.com>`。不要改回 `codex@local`，不要用 `--author` 覆盖。每个提交信息末尾空一行后按事实加 trailer：

```
Co-Authored-By: Codex <267193182+codex@users.noreply.github.com>
```

该提交的产品内容由 Grok 写的再加一行：

```
Co-Authored-By: Grok (xAI grok-4.6) <noreply@x.ai>
```

只按事实加：你自己的记账提交只加 Codex 那行；Grok 写的产品提交两行都加。不得给没参与该提交的身份署名，不得使用任何真实第三方个人账号的邮箱。依据见 `decisions/2026-08-18-syn-commit-attribution-and-agent-co-authorship-v1.md`。

## 9. 全程边界

不接真实个人资料、真实用户项目写入、真实模型/provider、真实消息、账号、凭据、connector 或外部网络业务动作；产品层证据只用隔离 app-data、scratch projects、fake roles/provider/runtime 与白名单合成动作。不 push、merge、rebase、部署、发布。不 reset、stash、clean 或丢弃既有未归属 WIP。不伪造 receipt、authorization、stage/leaf、测试或 App 证据。

报告分 Harness、产品、证据、载体四段，不把工作副本、离线 fixture 或协议推断说成真实运行或发布。
