# Harness 脚本库实证审计（scripts/harness/）

- 审计日期：2026-06-14
- 范围：仓库根 `scripts/harness/`（66 个顶层 `.js` + 13 个 `lib/*.js` = **79 个 Node 脚本**，另 2 个 `stage-k-*.swift` 辅助文件、`eval/` 下 5 个 JSON 用例）。**不含** `prototypes/` 下的产品代码。
- 性质：纯只读调查。未改动、未删除、未运行任何脚本，未提交。本文件是唯一新增文件。
- 证据基础：脚本调用点普查（`package.json` / git hooks / CI / `harness.js` 路由 / 流程文档 / 脚本互相 require）＋ 全量历史轨迹扫描（`evidence/` 424 文件、`handoffs/` 370 文件、`tasks/` 254 文件，共 1048 份历史文件中谁的输出真出现过）。

---

## 拍板摘要（先看这段）

- **在批准什么**：把 79 个 Node 治理脚本归四桶——「承重 / 本阶段休眠 / 从没接上 / 真死」，并据此决定删不删、补不补。本文件只给证据和建议，**删改由你拍**。
- **代价**：审计本身零成本（只读）。后续若采纳建议，「补接线」是工程投入，「删」几乎无收益且有风险。
- **不批准会怎样**：现状继续——一套 79 脚本的治理库 99% 处于无人调用、连维护者都不知其存在的状态，下一次「明明有工具却重造」的事故迟早再发生（最近 verification 工具差点被重造就是先例）。
- **本审计的核心结论一句话**：这套库**不是「写坏了」，是「从没接上电」**——真正该按的不是删除键，是「要么接线、要么明文退役」的开关；而比这更紧的，是先补一份「能用的脚本索引」。

## 一句话判据（怎么用本报告）

> 拿任意一个脚本，问三步：**① 历史 1048 份完成报告里它的输出出现过吗？**（出现→承重）**② 没出现是因为它对应的子系统被 config/阶段关掉了吗？**（是→本阶段休眠，别删）**③ 既没出现、也没被关掉，而这件事团队今天正在手工做？**（是→从没接上，是缺口该补；否且与他脚本重复→真死，删候选）。

---

## 一、核心结论

### 1. 真实的验证脊柱只有 5 条命令，且其中只有 1 条来自这套 harness

历史 1048 份 `evidence/handoffs/tasks` 文件里，真正反复出现的验证手段（按命中文件数）：

| 真实在跑的命令 | 命中文件数 | 来源 |
| --- | ---: | --- |
| `npm run typecheck` | 544 | 产品 `prototypes/.../package.json` |
| `test:offline-interaction` | 530 | 产品自带 `scripts/run-offline-interaction-test.mjs` |
| `cargo test` | 522 | Rust/Tauri 工具链 |
| `npm run build` | 463 | 产品 package.json |
| **`workbench-shape-gate.js`** | **371** | ← **本 harness 唯一承重脚本** |
| `stage-k-architecture-gate.js` | 21 | 本 harness（stage-K 专用） |

### 2. 这套「设计先行铺全」的 harness，整体 0 落地

| 安装套件里的脚本 | 历史命中文件数 |
| --- | ---: |
| `harness.js`（总入口路由器） | **0** |
| `harness-doctor.js`（聚合诊断门） | **0** |
| `config-check.js` / `mcp-doctor.js` / `capability-map.js` / `evidence-freshness.js` / `verification-suite.js` … | **各 0** |
| 其余 75 个安装脚本（含全部 13 个 `lib/`） | **各 0** |

> 唯一一个非零的安装脚本看似是 `lib/manifest.js`（7 命中），逐条核对后**全部是 `manifest.json` 等散文里的子串误命中**，真实落地 = 0。

### 3. 四道「激活面」全部是关闭/未接状态——这就是为什么 0 落地

| 激活面 | 状态 | 证据 |
| --- | --- | --- |
| **Git hooks** | **关** | `harness.config.json → policy.hooks.enabled=false`；`.git/hooks/` 下只有 `*.sample`，`hook-install.js` 从没装过 |
| **CI** | **无** | `policy.ci.required=false, providers=[]`；无 `.github/workflows`、无 `.gitlab-ci.yml`；提交者 `Codex <codex@local>`，本仓库从不跑 CI |
| **`harness.js` 总入口** | **从没被调** | 历史 0 命中；37 个子命令无一在任何完成报告里出现 |
| **流程文档路由** | **不指向它** | `AGENTS.md`/`README.md`/`STAGE_PLAN.md`/`principles.md`/`DEV_LINES.md` 提到 `scripts/harness` **各 0 次**；只有 `CURRENT.md`（10×shape-gate、1×stage-k）和 `TASK_TEMPLATE.md`（2×shape-gate）指名，且**只指那两个手写门**。安装套件仅被 `harness.config.json` 引用，而那个文件没有任何东西去执行它。 |

另有三个子系统被 config 显式关停：`memoryIntegration.enabled=false`（agentmemory 整套）、`hooks.enabled=false`、`ci.required=false`。

### 4. 真正在用的两个门，恰恰不属于这套安装库

- `workbench-shape-gate.js`（371）和 `stage-k-architecture-gate.js`（21）**都不在安装 manifest 里**（`.harness/manifest.json` 查无），是 root-treatment 期间**手写的、零依赖（XREF=0）、standalone** 的产物。
- `workbench-shape-gate.js` 内容是写死本产品的 ratchet：基线 commit、命令数基线（97）、逐文件行数 waterline、允许的 sidecar JSON 种类。它随前端每次重构被持续修订（git log 里它有大量 commit）；而**全部 79 个安装脚本只在 `ed01c6f「establish root treatment governance baseline」一次性入库后再没被碰过**（mtime 全是 `May 31 11:11`）。
- 含义：团队需要一道门时，**没有去碰 `verification-suite.js`/`config-check.js`/`harness-doctor.js`，而是手搓了一个**。这是「整套通用门被非采纳地架空」，而非「被替代」。

---

## 二、四桶分类（每个脚本恰好一桶 + 证据）

### 桶 A — 承重（几乎每包都用）｜2 个

| 脚本 | 命中 | 结论 |
| --- | ---: | --- |
| `workbench-shape-gate.js` | 371 | 产品架构 ratchet 门，每包必跑，`CURRENT.md`/`TASK_TEMPLATE.md` 指名。**唯一真正的承重件。** |
| `stage-k-architecture-gate.js` | 21 | stage-K/root-treatment 专用架构门，在其阶段内承重。 |

（真实脊柱的另外四条 `cargo test` / `npm typecheck` / `test:offline-interaction` / `npm build` 不属于本 harness，属产品工具链。）

### 桶 B — 本阶段休眠（现在用不上、换阶段/开关一开就会用，**别删**）｜约 20 个

按「被哪个开关压着」分组：

- **Hooks 子系统**（`policy.hooks.enabled=false`）：`pre-work.js`、`pre-completion.js`、`hook-install.js`、`hook-uninstall.js`、`git-gate.js`
  - 一旦把 hooks 打开，这组就是 pre-commit/pre-push 的执行骨架。现在纯休眠。
- **CI 子系统**（`ci.required=false`，无 CI）：`ci-init.js`、`ci-gate.js`、`ci-validate.js`
  - 治理冻结期当然歇着；将来若上 CI（如 `templates/ci/github-actions/harness.yml`）即可用。
- **UI/浏览器验证**（本阶段用 offline 交互测试，真实浏览器验收推迟）：`ui-verify.js`、`browser-evidence-check.js`
- **MCP**（`tools.mcp.required=[]`，本阶段不依赖 MCP）：`mcp-doctor.js`
- **Agentmemory 整套**（`memoryIntegration.enabled=false`）：`memory-agentmemory-query.js`、`memory-agentmemory-save.js`、`memory-candidate-new.js`、`memory-candidate-lint.js`、`memory-review.js`、`memory-stale-check.js`、`memory-maintenance.js`、`lib/agentmemory-client.js`、`lib/memory-governance.js`
  - **⚠ 特别提示**：本产品的记忆模型已改走「Claude 文件记忆 + `handoffs/`」，config 把 agentmemory 设为 enabled=false 且无重启计划。这 9 个**可能是永久休眠**——若哪天正式宣布放弃 agentmemory，应整组转入「真死/删」复评。现按休眠保留，但它不是「显然会回来」那种休眠。

### 桶 C — 从没接上（本身可能有用，只是没 wire 进 hook/CI/流程；是【可补的缺口】，不是删）｜约 56 个

**这是绝大多数脚本的真实归属。** 再分两层：

**C1 — 高价值缺口（团队今天正在手工做这件事，工具却闲置）：**

| 子系统 | 脚本 | 为什么是「该补的缺口」 |
| --- | --- | --- |
| 证据 | `evidence-check/-freshness/-new/-index/-query/-retention/-compact/-command.js`、`lib/evidence-audit.js` | 项目**手写了 424 份 evidence 文件**，全程 0 用这套证据工具 |
| 错误账本 | `mistake-check/-new/-query.js`、`lib/mistake-retrieval.js` | `AGENTS.md` 把 learning-from-mistakes 列为强制，工具却 0 调用 |
| 验证编排 | `verification-plan/-runner/-suite.js` | 团队天天手跑 cargo/npm 验证，从不经这些 planner |
| 状态/控制守卫 | `guard-state-files.js`、`status-snapshot.js`、`stale-control-check.js` | 本项目**最在意受保护路径**，恰恰这道守卫从没接上——缺口最刺眼 |
| 配置校验 | `config-check.js`、`config-policy.js` | `harness.config.json` 在被频繁改，却没人校验它 |
| 任务生命周期 | `task-start/-finish/-status.js`、`task-package-new/-lint.js`、`task-risk.js`、`lib/task-package-schema.js` | 项目**手写了 254 份 task 文件**，不经这套 |
| 能力普查 | `capability-scan.js` | 「工具明明在却没人知道」的典型受害者（见 capability-map 重复） |
| 技能路由 | `skill-recommend.js`、`lib/skill-index.js` | 风险路由全靠人读 SKILL.md |
| 上下文恢复 | `context-pack.js`、`lib/context-pack.js` | Strict 恢复靠手翻控制文件，没用打包器 |
| 安全扫描 | `security-scan.js`、`lib/security.js` | `AGENTS.md` 强调 untrusted-input 隔离，扫描器闲置 |

**C2 — 元工具缺口（只有当 harness 自身被采用才谈得上价值；优先级低）：**

`harness.js`（路由器）、`harness-doctor.js`、`installed-health.js`、`managed-files-audit.js`、`self-test.js`、`sync-harness.js`、`install-harness.js`、`rules-lint.js`、`runtime-docs-diff.js`、`runtime-docs-init.js`、`config-init.js`、`config-migrate.js`、`config-schema.js`、`eval-runner.js`、`project-profile.js`，以及 `lib/check-runner.js`、`lib/config-loader.js`、`lib/manifest.js`、`lib/project-kind.js`、`lib/risk-classifier.js`。

- 这些是「给一套没人开机的基础设施做体检/迁移/自检」的工具。值得注意：`self-test.js`（73KB）能验证整套库是否还工作，但它**自己也从没跑过**——所以连「这套冻结的库今天还能不能通过自检」都是未知。

> 注：全部 13 个 `lib/*.js` 均**只被「从没落地」的脚本 require**，因此运行时传递性使用 = 0。

### 桶 D — 真死（被取代/重复/错抽象，删候选）｜实证下≈ 0–1 个

**按「有实物证据的重复/取代」严格判，几乎没有干净的死脚本。** 唯一一个具体的重复：

| 重复对 | 证据 | 建议 |
| --- | --- | --- |
| `capability-map.js`（17KB，`harness.js` 的 `capabilities` 子命令）↔ `capability-scan.js`（13KB，`preWork` 清单） | 两者目标相同（扫目标目录的工具/命令能力），arg 解析与 ignore 列表近乎逐行雷同 | **二选一**：留一个、退役另一个。具体留哪个需做一次完整 diff 再定；我未运行/未深 diff，不替你点名处决。 |

**为什么 D 桶几乎是空的——这正是本审计最该传达的事**：这套库的失败模式不是「写得烂被替代」，而是**「装好了从没通上电」**。把一堆「没接线」误判成「死了」然后删，会删掉本可低成本接上的真能力（最近差点重造 verification 工具就是这个误判的预演）。

---

## 三、为什么没用上（根因，本任务的重点）

1. **激活面从一开始就没打开**：hooks=off、CI=none、`harness.js` 没人调、流程文档不指向它。脚本再好，没有任何东西在某个时刻去 `node` 它，就等于不存在。
2. **「设计先行一次性铺全」与「按需手搓」的错配**：真到需要一道门时，团队手搓了 `workbench-shape-gate.js`（贴合本产品、零依赖、可随重构改），而不是去配置通用门。通用套件做了「正确但通用」的事，产品需要的是「具体到本仓库」的事——抽象层级对不上。
3. **冻结期的阶段错位**：browser/ui/mcp/ci/deploy/agentmemory 这些子系统本就被当前后端治理冻结期/config 开关压着，休眠是合理的——但它们和「该接没接」的脚本混在同一个 0 命中里，外观上一律像「没用」，掩盖了真正的缺口。
4. **可发现性塌方（最关键）**：唯一的发现入口是 `harness.config.json` 和 `harness.js --help`，而 agent 实际只读 `AGENTS.md`/`CURRENT.md`/`TASK_TEMPLATE.md`——这些文件不指向脚本库。于是连维护者都不知道库里有什么，已存在的工具被反复「重新发现」甚至险些重造。

---

## 四、Meta：可发现性 —— 比删更要紧的事

**连维护者都不知道存在的好脚本，等于死脚本。** 本仓库恰恰如此：79 个脚本里 77 个无人知晓，不是因为坏，是因为没有任何「人会读到的地方」列出它们。

**建议（按性价比排序，均需你拍板，本审计不擅自执行）：**

1. **先补一份「能用的脚本索引」**（最高性价比）：一页表，列每个脚本「干什么 / 怎么调 / 当前桶 / 是否已接线」，放进 agent 真会读的位置（`AGENTS.md` 或 `CURRENT.md` 或 `TASK_TEMPLATE.md` 引一行指过去）。这直接消除「重造已存在工具」的事故根因——**比删任何脚本都值。**
2. **对 C1 高价值缺口做「接线 or 明文退役」二选一**：尤其 `guard-state-files`、`mistake-check`、`evidence-check/-freshness`、`config-check`——它们对应的纪律团队天天在手工做，接一条线即得真收益。不接，就在索引里明文标「退役/不采用」，消除歧义。
3. **`capability-map` vs `capability-scan` 合一**：唯一的具体重复，做次 diff 后留一弃一。
4. **agentmemory 9 件给个了断**：要么宣布永久放弃→转删候选，要么记录「保留待启用」的触发条件，别让它永远悬在「休眠」。
5. **删，留到最后**：在「索引 + 接线决策」做完前，删任何脚本都是把「没接线」误当「死」。当前实证下的删候选仅 `capability-*` 二选一中的那一个。

---

## 五、我做了什么 / 没做什么（边界诚实）

- **做了**：调用点普查、1048 份历史文件的全量基名命中统计、config/manifest/git 史交叉验证、两个承重门通读、重复对与 73KB `self-test` 抽查。
- **没做**：未运行任何脚本（包括 `self-test.js`，故「冻结套件今天是否仍自检通过」未知）；未对 `capability-map/scan` 做完整逐行 diff（故未替你点名处决具体哪个）；未碰 `.codex`/浏览器/真实执行。
- **一个证据口径的诚实**：本审计以「基名是否在完成报告里被引用」为落地判据。理论上某脚本可能被临时手跑而未记录；但本仓库证据纪律极强（shape-gate 输出被记录了 371 次），若某脚本是流程一部分却 1048 份文件 0 命中，「系统性使用却不留痕」在此文化下不成立。

---

## 附录 — 全量命中频次表

口径：**RUN** = `evidence/handoffs/tasks` 中命中的文件数（真实轨迹）；**DOCS** = 治理/流程文档+config 中被指名的文件数；**XREF** = `scripts/harness/` 内被别的脚本 require/spawn 的文件数。按 RUN 降序。

| RUN | DOCS | XREF | 脚本 | 桶 |
| ---: | ---: | ---: | --- | --- |
| 371 | 2 | 0 | workbench-shape-gate.js | A 承重 |
| 21 | 1 | 0 | stage-k-architecture-gate.js | A 承重 |
| 0 | 3 | 6 | pre-completion.js | B 休眠(hooks) |
| 0 | 3 | 5 | pre-work.js | B 休眠(hooks) |
| 0 | 2 | 4 | hook-install.js | B 休眠(hooks) |
| 0 | 0 | 3 | hook-uninstall.js | B 休眠(hooks) |
| 0 | 1 | 4 | git-gate.js | B 休眠(hooks) |
| 0 | 0 | 4 | ci-init.js | B 休眠(CI) |
| 0 | 1 | 4 | ci-gate.js | B 休眠(CI) |
| 0 | 0 | 2 | ci-validate.js | B 休眠(CI) |
| 0 | 1 | 2 | ui-verify.js | B 休眠(UI) |
| 0 | 2 | 6 | browser-evidence-check.js | B 休眠(UI) |
| 0 | 2 | 6 | mcp-doctor.js | B 休眠(MCP) |
| 0 | 0 | 4 | memory-agentmemory-query.js | B 休眠(mem⚠) |
| 0 | 0 | 1 | memory-agentmemory-save.js | B 休眠(mem⚠) |
| 0 | 0 | 2 | memory-candidate-new.js | B 休眠(mem⚠) |
| 0 | 0 | 2 | memory-candidate-lint.js | B 休眠(mem⚠) |
| 0 | 0 | 2 | memory-review.js | B 休眠(mem⚠) |
| 0 | 0 | 2 | memory-stale-check.js | B 休眠(mem⚠) |
| 0 | 0 | 2 | memory-maintenance.js | B 休眠(mem⚠) |
| 0 | 0 | 1 | lib/agentmemory-client.js | B 休眠(mem⚠) |
| 0 | 0 | 1 | lib/memory-governance.js | B 休眠(mem⚠) |
| 0 | 2 | 7 | evidence-check.js | C1 缺口 |
| 0 | 2 | 6 | evidence-freshness.js | C1 缺口 |
| 0 | 0 | 4 | evidence-new.js | C1 缺口 |
| 0 | 0 | 4 | evidence-index.js | C1 缺口 |
| 0 | 0 | 4 | evidence-query.js | C1 缺口 |
| 0 | 1 | 1 | evidence-retention.js | C1 缺口 |
| 0 | 0 | 1 | evidence-compact.js | C1 缺口 |
| 0 | 0 | 2 | evidence-command.js | C1 缺口 |
| 0 | 0 | 0 | lib/evidence-audit.js | C1 缺口 |
| 0 | 2 | 5 | mistake-check.js | C1 缺口 |
| 0 | 0 | 2 | mistake-new.js | C1 缺口 |
| 0 | 0 | 2 | mistake-query.js | C1 缺口 |
| 0 | 0 | 1 | lib/mistake-retrieval.js | C1 缺口 |
| 0 | 2 | 6 | verification-plan.js | C1 缺口 |
| 0 | 3 | 5 | verification-runner.js | C1 缺口 |
| 0 | 1 | 5 | verification-suite.js | C1 缺口 |
| 0 | 2 | 6 | guard-state-files.js | C1 缺口 |
| 0 | 2 | 6 | status-snapshot.js | C1 缺口 |
| 0 | 1 | 4 | stale-control-check.js | C1 缺口 |
| 0 | 2 | 5 | config-check.js | C1 缺口 |
| 0 | 0 | 4 | config-policy.js | C1 缺口 |
| 0 | 1 | 4 | task-start.js | C1 缺口 |
| 0 | 1 | 4 | task-finish.js | C1 缺口 |
| 0 | 0 | 4 | task-status.js | C1 缺口 |
| 0 | 0 | 2 | task-package-new.js | C1 缺口 |
| 0 | 0 | 3 | task-package-lint.js | C1 缺口 |
| 0 | 1 | 3 | task-risk.js | C1 缺口 |
| 0 | 0 | 0 | lib/task-package-schema.js | C1 缺口 |
| 0 | 1 | 2 | skill-recommend.js | C1 缺口 |
| 0 | 0 | 0 | lib/skill-index.js | C1 缺口 |
| 0 | 1 | 2 | context-pack.js | C1 缺口 |
| 0 | 0 | 0 | lib/context-pack.js（注：实为 13KB，被 context-pack.js 调） | C1 缺口 |
| 0 | 0 | 3 | security-scan.js | C1 缺口 |
| 0 | 0 | 1 | lib/security.js | C1 缺口 |
| 0 | 2 | 7 | harness.js（路由器） | C2 元工具 |
| 0 | 3 | 4 | harness-doctor.js | C2 元工具 |
| 0 | 1 | 3 | installed-health.js | C2 元工具 |
| 0 | 1 | 3 | managed-files-audit.js | C2 元工具 |
| 0 | 2 | 2 | self-test.js（73KB，自身从没跑过） | C2 元工具 |
| 0 | 0 | 4 | sync-harness.js | C2 元工具 |
| 0 | 0 | 5 | install-harness.js | C2 元工具 |
| 0 | 1 | 5 | rules-lint.js | C2 元工具 |
| 0 | 1 | 3 | runtime-docs-diff.js | C2 元工具 |
| 0 | 0 | 4 | runtime-docs-init.js | C2 元工具 |
| 0 | 0 | 4 | config-init.js | C2 元工具 |
| 0 | 0 | 2 | config-migrate.js | C2 元工具 |
| 0 | 0 | 2 | config-schema.js | C2 元工具 |
| 0 | 0 | 2 | eval-runner.js | C2 元工具 |
| 0 | 0 | 4 | project-profile.js | C2 元工具 |
| 0 | 0 | 1 | lib/check-runner.js | C2 元工具 |
| 0 | 0 | 0 | lib/config-loader.js | C2 元工具 |
| 0 | 0 | 0 | lib/manifest.js（7 命中全为 manifest.json 误命中） | C2 元工具 |
| 0 | 0 | 1 | lib/project-kind.js | C2 元工具 |
| 0 | 0 | 1 | lib/risk-classifier.js | C2 元工具 |
| 0 | 3 | — | capability-scan.js | C1 缺口 / 见 D |
| 0 | 0 | 3 | capability-map.js | **D 真死候选（与 capability-scan 二选一）** |

> XREF 反映「内部接线丰度」，不反映落地；XREF 高而 RUN=0 的脚本（如 `harness.js` XREF=7、`evidence-check` XREF=7）恰恰说明「库内部互相接得很全，但整体没接到外部世界」。
