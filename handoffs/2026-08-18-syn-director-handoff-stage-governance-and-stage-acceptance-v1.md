# Syn 总指导交接：阶段治理与阶段验收 v1

日期：2026-08-18

状态：`ACTIVE_HANDOFF / STAGE_15_ACTIVE / M5_CLOSED / NOT_RELEASED`

用途：交给接任的总指导会话。上一任的会话上下文过长，用户决定换人并同时改分工。本文件是你的全部起点，不要依赖任何聊天历史。

## 1. 你的角色（2026-08-18 用户改定）

你**只做三件事**：

1. **阶段工作规划。** 按产品正本与阶段计划决定一个阶段包含哪些叶、什么顺序、每叶的完成标准与写域边界。叶子由你排，不由主管自建。
2. **问题处理。** 主管撞到硬停点（第 6 节）时由你判定和排解。
3. **阶段完成验收。** 主管把整个阶段做完并交包后，由你独立验收整个阶段。

你**不再参与逐叶/节点验收**。每一叶的验收由开发主管自己负责，包括起独立复核。旧的"主管举手 → 总指导侧验收官 → 放行"循环（`/home/synadmin/workspace/.syn-gates/loop.sh`）在这个分工下已不适用，除非用户另行改回。

你也不代替主管实现：不写产品源码，不做逐叶记账。你的产出是阶段计划、stage 文件、leaf 定义、问题裁决和阶段验收结论。

## 2. 当前事实基线（2026-08-18 22:10）

- 权威仓库：`/home/synadmin/workspace/syn`（5600X WSL）。`HEAD` = `d33e348`。
- 远端 `origin` = `git@github.com:Djh0311/syn-aios.git`，公开。远端 `main` = `4791654`。本地有 3 个未推提交：`9890f47`、`c60caa9`、`d33e348`。**未推是当前状态，不是待办**——push 只在用户本人明确要求时做。
- SSH 到 `github.com:22` 当前不通（走代理 fake-ip，TCP 通但握手挂死）。可用路径是 HTTPS + `gh` 凭据助手；`gh` 已登录 `Djh0311`，scopes `gist, read:org, repo`。
- stage-14 已关闭（`M5C01-20260818-1939.verdict.md`）。M5 结论是 `SCOPED PRODUCT-CHAIN PASS / NOT_RELEASED`，产品内容锚 `c91d8fc72bcbf80186736caff841cb7a9b0660d1` / tree `fe2d982267d474631ca4ea7b3f90ed846f72a89d`。
- **stage-15 已建立并 active**（`docs/harness/stages/stage-15.md`，提交 `d33e348`）：M6 全局主管与内部组织，域层先行，产品 UI 与隔离 App 验收载体改为新壳。
- 唯一 current leaf：`docs/harness/leaves/M6P00-canonical-project-id-consumption-and-relation-owner-typing.md`。
- `docs/harness/authorization.json` 是精确 closed 两字段。主管按自己的真实 receipt 签发，你不手填。
- 仓外协作目录 `/home/synadmin/workspace/.syn-gates/`：`open/`（阶段交包）、`verdicts/`（历史验收结论）、`handled/`、`evidence/`、`loop-logs/`。当前 `open/` 为空。

## 3. 生命周期文件事实

文件位置就是生命周期事实，退场只原子移动一个文件：

- `docs/harness/plan.md`：总计划与阶段勾选、当前优先级。
- `docs/harness/stages/`：active stage 文件（现在只有 `stage-15.md`，`stage-12.md` 仍开着）。已关闭的进 `docs/harness/done/<年-月>/`。
- `docs/harness/leaves/`：**唯一** current leaf。
- `docs/harness/unfinished/`：只规划、未开工或已记录的后续叶。当前 13 个，见第 5 节。
- `docs/harness/audit/2026-08.jsonl`：审计行，一行一 JSON。
- `docs/harness/reports/`：叶与阶段报告。

## 4. 阶段完成验收怎么做

主管交包后，你独立验收整个阶段。判定依据只有 stage 文件与其各叶自己写的"做完的标准"，不拿更高标准卡人，也不因为主管说"已完成"就放过。逐项都要你自己跑出证据：

1. **写域**：`git show --stat` 逐个候选与记账 SHA，比对 stage 与各 leaf 的允许写域。越界即 FAIL。
2. **冻结物**：`git diff <阶段起点>..<阶段终点> -- docs/contracts/`，确认 M1–M5 冻结合同正文与旧 hash 未被改，只允许新增增补合同。
3. **WIP 保全**：`git status --porcelain=v1 -uall`，确认既有未归属 WIP 仍在，没被暂存、提交、reset、stash、clean。
4. **重跑**：`git worktree add --detach /tmp/syn-verify-<短SHA> <SHA>`，独立 `CARGO_TARGET_DIR`，重跑阶段各叶要求的定向测试与 `cargo check --lib --offline`。记录真实 passed/failed 与 exit code；与主管自报不符以你的为准并明确指出差异。
5. **实质**：读候选 diff 本身。声称的行为是否真的实现；是否只存在于 `#[cfg(test)]`、env 门控或 fixture 而普通路径走不到；失败路径是否真 fail-closed（有无静默 fallback、默认值兜底、path 派生、自动导入）；调用点是否真在真实 entrypoint 的调用链上。空壳、死代码、只有测试能触发的实现一律 FAIL。特别盯"实现了但运行期没有消费者"这种空转。
6. **不越级**：有没有把离线 fixture、disposable checkout、协议推断说成真实运行、GUI 证据、发布或部署；有没有把已获得的 scoped PASS 反向写成 FAIL。
7. **欠账**：即使 PASS，也把阶段标准没覆盖但影响后续的问题逐条列出，写清该由哪个后续叶关掉。

结论写到 `/home/synadmin/workspace/.syn-gates/verdicts/<STAGE>-<YYYYMMDD-HHMM>.verdict.md`，第一行顶格 `VERDICT: PASS` 或 `VERDICT: FAIL`，之后写候选 SHA/tree、你实际执行的命令与真实输出摘要、逐条核验结论、欠账清单，FAIL 则给出精确到文件和行为的返修意见。

环境备注：`sqlite3` 命令不存在，用 Python 的 `sqlite3` 模块查库。`cargo check --lib --offline` 约 9 秒（增量），有 883 条既有 warning，不要把 warning 当失败。`rustfmt` 是 1.9.0，仓库已统一到该基线。

## 5. 后续欠账与 unfinished 真实清单

M5 verdict 的 8 项欠账已按用户 18:40 纪律路由完毕，无一项被提升为返修：

| 欠账 | 去处 |
|---|---|
| canonical ProjectId 未扩到 workflow/执行链 | `M6P00`（已是 current leaf） |
| relation source foreign owner 不可判别 | 同上 |
| `UNENROLLED` 首启缺主动引导 | `unfinished/F3-m1-unenrolled-guidance-and-status-projection.md` |
| 测试 helper 可能 `0 == 0` 空转 | `unfinished/ENG-01-post-m5-nonblocking-hardening-and-worktree-hygiene.md` |
| `validate_preview_input` path-derived 死代码 | 同上 |
| 883 条 warning 待分类 | 同上 |
| 13 个历史 worktree 注册项 | 同上 |
| 验收期间新增 OSS 门面 | 用户自有载体，已由 `c1025ba` 提交；`unfinished/OSS-01-*` 保持 unfinished |

`unfinished/` 现有 13 个文件：`D0C04`、`D0C05`（stage-12 的 SSH 通道，只读保全，不恢复不关闭）、`ENG-01`、`F3-m1-unenrolled-guidance`、`M1I01` 系列 5 个、`M3O01`–`M3O03`、`OSS-01`。

另有两项非阻断记账欠项：真桌面窗口像素证据欠在新壳 F5；旧壳 M5R07 acceptance driver 的"不得继承"是新壳 F3 的待接收边界。见 `handoffs/2026-08-18-syn-m5-to-m6-and-shell-deferred-debts-v1.md`。

## 6. 主管必须上报、由你处理的硬停点

- 需要超出当前 leaf 写域，或需要改冻结合同正文/旧 hash；
- 需要 push、merge、rebase、tag、部署、发布；
- 需要真实凭据、真实 provider、真实账号、真实个人资料或外部网络业务写；
- 需要动 `stage-12`、`unfinished/D0C04`、`unfinished/D0C05`、`OSS-01`、用户自有载体或只读保全项；
- 需要进入 `syn-shell` 仓库或启动 F2/F3/F5；
- 需要激活 M6 域层之外的里程碑（M7–M11、Headless Core、Primary/authority epoch）；
- 逐叶返修连续不收敛，或标准本身有歧义、判不准；
- 发现 secret/credential/真实运行数据/未知 symlink/special file，或候选 SHA 与证据不一致。

主管遇到这些必须停下写文件上报，不许自行放宽。你裁决后更新 stage/leaf 文件，让边界从文件生效，而不是只在对话里说一句。

## 7. 不许动与自有载体

- **用户自有**：`.claude/harness-lite/*` 与 `AGENTS.md`/`CLAUDE.md` 的规则改动，已提交为 `0db02ef`；开源门面 `README.md`、`LICENSE`、`CONTRIBUTING.md`、`SECURITY.md`、`package.json` 与 `src-tauri/Cargo.toml` 的 license/repository 字段，已提交为 `c1025ba`（精确 7 路径）。不改写、不并进任何候选、不当来源不明 WIP 归责。
- **上一任总指导自有**：`c60caa9` 把 tauri crate 32 个源文件统一到 rustfmt 1.9.0 基线，纯格式、零 token 级改动，`cargo fmt --check` 与 `cargo check --lib` 均通过。未加工具链钉子（按版本号钉会让 rustup 重下整套副本，不值），所以不同 rustfmt 版本仍可能产生重排噪声，真正的修法是 fmt 门，属 `ENG-01`。
- **只读保全**：6 个未跟踪 `m6_*.rs`（含 `m6_member_directory.rs.bak`）与 `src-tauri/gen/schemas/linux-schema.json`。不暂存、不清理、不恢复，也不得升格为 M6 基线或实现输入。
- **提交署名**：仓库 local git 身份是 `Djh0311 <277674664+Djh0311@users.noreply.github.com>`，不要改回 `codex@local`，不要用 `--author` 覆盖。按事实加 trailer：Codex 参与加 `Co-Authored-By: Codex <267193182+codex@users.noreply.github.com>`；产品内容由 Grok 写的再加 `Co-Authored-By: Grok (xAI grok-4.6) <noreply@x.ai>`；你自己（Cursor 侧）写的加 `Co-authored-by: Cursor <cursoragent@cursor.com>`。依据见 `decisions/2026-08-18-syn-commit-attribution-and-agent-co-authorship-v1.md`。

## 8. B 线（新壳）现状

- 姊妹仓库 `/home/synadmin/workspace/syn-shell`，独立 git 历史，不是 syn 的分支。
- 上游是 `github.com/SDSLeon/lightcode`（Poracode，Apache-2.0，Electron/TS，约 2500 源文件），现为 remote `upstream`。
- 用户自有 fork `github.com/Djh0311/syn-shell` 为 remote `origin`；我们的线推在 `syn` 分支（`45dd5c1c`，已设为默认分支），fork 的 `master` 保持纯跟踪上游，避免 force push。
- F0（源码入库与可构建基线、模块地图、LICENSE/NOTICE、品牌替换清单）与 F1（品牌与风格基线）已独立 PASS 并 closeout，没有 current leaf，authorization closed。
- F2（壳 ↔ Syn 核心桥）硬门 M5 closeout 已满足，但 **F2 实施与 stage-15 争用 syn 仓库源码写面**，同一时间只允许一个施工者，必须排序不得并行。F2 的合同草案只写文档，可与 A 线并行。
- 方向依据：`decisions/2026-08-17-syn-lightcode-fork-desktop-shell-direction-v1.md`、`docs/plans/2026-08-17-syn-lightcode-fork-shell-adoption-plan-v1.md`。

## 9. 用户长期边界

- 用户用自然语言给目标和边界，直接生效；plan、stage、leaf、route、receipt 都不能扩大用户原话。
- 不接真实个人资料、真实用户项目写入、真实模型/provider、真实消息、账号、凭据、connector 或外部网络业务动作。产品层证据只用隔离 app-data、scratch projects、fake roles/provider/runtime 与白名单合成动作。
- 不 push、merge、rebase、部署、发布，除用户本人明确要求。
- 不 reset、stash、clean 或丢弃既有未归属 WIP；不伪造 receipt、authorization、stage/leaf、测试或 App 证据。
- 报告分 Harness、产品、证据、载体四段，不把工作副本、离线 fixture 或协议推断说成真实运行或发布。
- 用户要"平实的话"，不要术语堆砌，不要把过程当结论。
