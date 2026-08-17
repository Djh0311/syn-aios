# 新总指导交接：M1 前置、M5R07 降级标准与节点协作 v1

日期：2026-08-18
发出者：总指导（Cursor 会话代笔；用户原话为最高权威）
接收者：新任 Syn M5 开发主管（Codex）
状态：**当前用户明确授权的无人值守推进指示。** 取代 2026-08-17 无人值守指示中与本文冲突的部分，其余继续有效。

## 0. 为什么换主管

上一条开发主管会话运行 7 小时 36 分、8894 事件、被上下文压缩 7 次，`docs/plans/2026-08-16-syn-m5-m6-fact-reconciliation-and-product-closure-plan-v1.md` 里 M5R07 的原验收标准在压缩中丢失，导致 M5R07 被切成 U01a/U01b/U01c/U02 一串局部通过、验收口径逐轮下滑。本次交接的第一要务是**重新锚定标准**，不是继续加速。

读取顺序：本文件 → `AGENTS.md` → `docs/product/syn-product-canon-v1.md` 与 `docs/product/authority-register-v1.md` → `docs/plans/2026-08-16-syn-m5-m6-fact-reconciliation-and-product-closure-plan-v1.md` → `docs/harness/plan.md` → `docs/harness/stages/stage-14.md` → 唯一 current leaf → `docs/harness/authorization.json`。

## 1. 三层协作与模型

| 角色 | 谁 | 做什么 | 不做什么 |
|---|---|---|---|
| 总指导 / 独立验收 | Cursor 会话 | 定标准与边界、节点验收、放行 closeout | 不写任何产品源码、不写 leaf |
| 开发主管 | 本会话（Codex，`gpt-5.6-sol` + `model_reasoning_effort=xhigh`） | 写任务包、派 Grok、自查复核、返修循环、harness 记账与文档 | 不写产品源码、不自行放行节点 |
| 写者 | Grok（`grok-4.6` + `--reasoning-effort high`） | 只写产品源码，一次一个窄包 | 不改 harness、不改合同正文、不自评通过 |

syn 仓库源码写面同一时间只允许一个写者，就是 Grok。中间每轮复核由开发主管自己的子 agent 完成，不惊动总指导。

## 2. 节点机制（硬约定）

到达节点时必须**停下**，不得自行继续：

1. 把 authorization 打回精确 closed 两字段；
2. 在 `/home/synadmin/workspace/.syn-gates/open/` 写一个请求文件（命名 `<节点名>-<时间戳>.md`），内容含：节点名、候选 SHA/tree、做了什么、证据在哪、明确未完成项、请求验收的范围；
3. 停止施工，等总指导回复。总指导处理后会把文件移入 `handled/`。

该目录在仓库外，不进任何 Git 载体。不写请求文件就继续往下走，视为违约。

节点只有两个：

- **节点一 · M1 前置完成**（本轮目标）；
- **节点二 · M5R07 收口前**（最终独立验收，过了才放行 closeout）。

## 3. 用户本轮的三个决定

1. **M1 纳入 M5 验收前置。** 不再把 M1 当"另一个 owner 的外部阻塞"绕过。
2. **M5R07 验收要求降级**，只砍界面类证据，不砍组合类证据。详见第 5 节。
3. `.claude/harness-lite/*` 与 `AGENTS.md`/`CLAUDE.md` 的改动**归属用户本人**，已由总指导提交为 `0db02ef`，不再当作来源不明的 WIP 审查。

## 4. 当前事实（2026-08-18 02:15）

- 分支 `main`，HEAD `0db02ef`（本文件写入时）。index 空。
- 今晚新增两条载体：`3aba35e` 壳方向文档 9 路径；`0db02ef` 用户的 Harness Lite 0.8 改动 9 路径。
- M5R07 已有 scoped independent PASS：implementation `f51c3f64`、evidence tip `0e0fcb26`、U01a `f962038`、U01b `70a15a9`、U01c `23642bb`；U02 落在 `0952c83` / `2aac3db`。**全部不是 evidence-binding，也不是 closeout。**
- `docs/harness/authorization.json` 为精确 closed 两字段。
- 工作树剩余未归属载体，须在 M5 closeout 前结清，本轮不动：21 个 M1/M3 时期改过的 `src-tauri/src/*.rs`、7 个 `m6_*.rs`（含一个 `.bak`）、`gen/schemas/linux-schema.json`、`docs/harness/usage/*` 运行时文件。
- `docs/current-state.md` 仍停在 2026-08-16，落后四十余个提交，由开发主管在本轮内更新为真实事实。
- 姊妹仓库 `/home/synadmin/workspace/syn-shell`：`shell-F0` / `shell-F1` 已独立 PASS 并 closed，无 current leaf，F2 以 M5 closeout 为硬门。本会话不碰该仓库。

## 5. M5R07 的新验收标准（用户已批准的降级）

### 5.1 砍掉，不再要求

- 两个隔离 scratch 场景覆盖 11 项交互；
- 逐场景窗口截图；
- 真桌面 computer use 点击（改为可选，不卡验收）；
- 旧壳界面外观、布局与交互手感类证据。

理由：2026-08-17 用户已定 lightcode fork 为长期壳载体，旧 Tauri 壳降为存续期载体，界面将在新壳按 F3 重建。旧壳的像素与交互证据会随旧壳一同过期。**真桌面像素证据挪到新壳 F5 一次性完成**，这是明确记账的欠项，不是取消。

### 5.2 保留，必须真实通过

这些与壳无关，新壳 F2 仍以"Rust 核心为权威服务"消费它们：

1. **真实启动路径取得项目身份**：普通产品启动时老项目能拿到 M1 正式身份，不靠测试 fixture 预登记，不靠 path 派生；
2. **真进程 + 普通产品组合**：真实 Tauri 二进制、普通 `AppState` 装配与 command 注册；
3. **用户拒绝零副作用**：零 spawn、零业务 mutation；
4. **强杀重启后持久状态仍在**；
5. **端口给出精确对象引用**，可回权威事实（只需证明端口层语义，不需证明旧壳点击）；
6. **一次最小端到端贯通**：以现成旧壳作为真实非测试客户端，把 `Proposal → AuthorizationDecision → Authorization → Run/WorkItem + worker RoleSession binding → PreparedAttempt → Grant → Dispatch → runtime → RuntimeReceipt/ExecutedReport → independent Review → ResultUserDecision` 走通**一次**。

第 6 项的目的是证明核心可被真实客户端驱动，为新壳 F2 预演接口，不是验收旧界面。一次即可，不做场景矩阵。

已有的后端定向矩阵继续有效，照跑。

## 6. 本轮目标：M5R00 重开（唯一 current leaf）

`docs/harness/plan.md` 与 stage-14 曾把 M5R00 记为 `NOT_NEEDED / 前置矩阵无 GAP`。经复核该判定错误：非测试代码中登记 M1 精确别名的只有 `m5_ordinary_control_acceptance.rs`（env 门控的验收 fixture）与 `#[cfg(test)]` 内的调用，**普通启动路径上没有任何真实项目登记入口**；老项目身份仍由 `lib.rs` 的 `project_id()` 从项目路径字符串派生。按 8-16 计划，前置矩阵出现 GAP 就必须走 M5R00，因此 M5R00 按真实缺口重开为唯一 current leaf，M5R07 挂起等待。

### 做完的标准

1. 一份 M1 增补合同（不改任何冻结合同正文、不改旧 hash）；
2. 老项目到 M1 正式身份的可信创建 / 迁移路径，来源显式、可重放、幂等；
3. 普通启动路径上真实调用该入口；不可用时 fail-closed，不静默 fallback、不 path 派生、不自动导入；
4. 定向测试覆盖：首次登记、重复登记幂等、重启后同一解析、缺失与损坏时拒绝；
5. `cargo check --lib --offline` 与相关定向测试在 disposable checkout 上通过；
6. 独立内容提交，写域精确，`git diff --check` 通过。

### 允许动

- `docs/contracts/`（仅新增 M1 增补合同）
- `prototypes/productized-desktop-shell/src-tauri/src/m1_project_index.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`（仅普通启动登记调用点与必要 `AppState` 接线）
- `prototypes/productized-desktop-shell/src-tauri/src/m3_project_role_identity_source.rs`（仅因 M1 身份来源变化必须同步时）
- `tasks/2026-08-18-*`（本轮任务包）
- `docs/harness/stages/stage-14.md`、`docs/harness/leaves/`、`docs/harness/unfinished/`、`docs/harness/audit/2026-08.jsonl`、`docs/harness/reports/M5R00-*`、`docs/current-state.md`

### 不许动

- M5R07 已有的 scoped PASS 结论，不得反向写成 FAIL；
- M1–M4 冻结合同正文与 hash；
- `m5_*.rs` 的 execution kernel、`worker_report.rs`、页面布局；
- `m6_*.rs`、`stage-12`、`unfinished/D0C04`、`unfinished/D0C05`；
- 21 个未归属的 `src-tauri/src/*.rs`、`linux-schema.json`（本轮只读保全）；
- 自行关闭 stage-14、宣布 M5 完成、进入 M6 或壳采纳；
- 伪造 receipt、authorization、测试或 App 证据；
- push、merge、rebase、部署、发布。

## 7. 补记事项（本轮内一并完成，只写已发生的事实）

`docs/harness/audit/2026-08.jsonl` 最后一条为 2026-08-16T19:36:45Z，此后实际发生但未记账：8-17 的 M1I01 系列与 M3O01–M3O03 系列共 13 个提交、M5R07 的 U01a/U01b/U01c/U02、今晚的 `3aba35e` 与 `0db02ef`、以及本次 M5R00 重开与 M5R07 挂起。据实补记，**不得补造授权、receipt 或时间线**。

## 8. 自主范围与硬停点

节点之间自主推进，普通工程问题自行修复复跑，不必询问用户。

遇到以下情况立即停止并写节点请求文件，等用户：

- 需要真实凭据、真实 provider、真实账号、真实个人资料或外部网络业务写；
- push、merge、rebase、部署、发布；
- 不 reset / stash / clean / 丢弃既有 WIP 就无法继续；
- 要求伪造 receipt、authorization、stage/leaf、测试或 App 证据；
- `stage-12`、D0C04/D0C05 或 M1–M4 冻结合同将被修改；
- 与用户原话、产品正本或权威登记冲突；
- 同一缺口连续多轮返修仍不收敛。

## 9. 节点二之后的路线（现在不做）

M5R07 通过后由开发主管执行 closeout-only：归档 M5R07 与 stage-14，同步 `current-state`、总计划、M5 计划、计划索引与 Harness plan，形成 M6 输入交接，单独 lifecycle commit，authorization 回 closed。随后按 2026-08-17 指示开 stage-15 的 M6 域层（域层先行，界面不在旧壳做）；壳采纳 F2 起在 `syn-shell` 另立会话推进。M7 及以后须用户明确开始。
