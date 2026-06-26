# 实现任务包：S3 咨询第一刀·价值路径修复（真咨询超时/取不回答案）· 主导线 → 执行线 v1

日期：2026-06-25　性质：**诊断 + 修复**。修法若只调 timeout/裁注入 = **轻档**；若动 consult 沙箱（让 codex 能写自己的输出）= **高危#3-ish**（碰只读 confinement）——**无论哪条,被咨询项目必须保持不可写**。上游：咨询第一刀已 commit（`950a726`）；spec `docs/plans/2026-06-25-s3-agent-layer-consultant-first-slice-spec-v1.md`；只读 confinement 决策 `decisions/2026-06-25-consultant-readonly-guard-exemption-v1.md`。

## 0. 接手须知 / 现象（主导线核过）

- 你是**执行线**。子线不 `git add`/`commit`。
- **现象**：`cargo test --lib s3_real_consult_mao_mao_dian_cai -- --ignored --nocapture`（OpenAI 已恢复）**跑 180.05s 失败**：`consult_last_message_read_failed: No such file or directory`。
- **已诊断到的**：`CliConsultantAgent` 默认 `timeout_ms: Some(180_000)` → codex **撞 180s 超时被 kill、没写出 `--output-last-message` 文件**。`build_readonly_consult_request` 写死 `allowed_write_roots=[]` + `sandbox="read-only"`；last-message 写到 `temp_dir`（`codex_local_runner.rs:342`）。
- **两个待分清的根因**：① 任务太重 180s 不够（注入整篇 README + codex 读红队/开发计划交叉 + 出 JSON）；② codex 被 read-only（写盘根空）卡住——连自己的输出/scratch 都写不了 → 卡到超时。
- 先读：`consultant_agent.rs`（`CliConsultantAgent` + `load_project_context` 注入多大）+ `codex_local_runner.rs:316-360`（`readonly_codex_consult` / `build_readonly_consult_request`）+ §6 测试 `lib.rs:4651`。

## 1. 拍板摘要

- **要做的事**：让真咨询**真能就着猫猫点菜出结构化答案**（现在超时取不回）。先诊断 slow vs stuck，再对症修，**修完重跑 §6 防幻觉真题验到答案落地 + confinement 仍只读项目**。
- **代价**：一轮诊断+修。做完后咨询第一刀**命根第一次真证到**（或证伪）。
- **不做的后果**：咨询第一刀停在"stub 绿但真路坏"，S3 agent 层第一个角色没立起来。

## 一句话判据

判改动在不在本包——问：**「是不是在让真咨询取回答案（调 timeout/裁注入/或给 codex 自己输出可写）、且被咨询项目仍不可写、没碰 worker 闸/判决体?」** 是 → 做；否（尤其**让被咨询项目可写** / 碰 worker 执行闸 / 改判决体）→ **停、回主导线。**

## 2. 诊断 + 修（两条路·先诊断）

**A · 诊断 slow vs stuck**（先做）
- 把 timeout 临时调大（如 300-420s）重跑 §6。**通了出答案** → 纯慢（走 B1）；**还卡/还无输出** → stuck（走 B2，多半是 read-only 写盘空）。
- 可顺手看 codex stderr / 进程行为佐证。

**B1 · 若 slow（轻档）**
- 把 `CliConsultantAgent` 默认 timeout 调到能完成的值；**并裁注入**——别整篇大文档全塞进 prompt，靠 codex 只读按需读（README + 文档地图够引路，红队/开发计划让它自己读）。目标：又快又稳出答案。

**B2 · 若 stuck on read-only 写盘空（高危#3-ish）**
- 给 codex **只写它自己的输出/scratch 临时目录**的能力，**被咨询项目（target_cwd=猫猫点菜）绝不进写盘根**：
  - 即沙箱改成「codex 输出 temp 目录可写 + 项目只读」——项目**不在** `allowed_write_roots` 里 = 仍不可写;只放 codex 输出那个 temp 目录进写根。
  - 这修正"`allowed_write_roots=[]` 太严到 codex 吐不回答案",但**不松项目读写边界**。
- 这碰 consult 沙箱 confinement = 高危#3：改完**必须**正反测试钉死「项目 root 不可写 / 不在写根」。

**C · 诊断回来了 = SLOW（已 bump 180→420s·§6 现 58s 完成）；但暴露更深的「深读关」（主导线 2026-06-25 拍板追加）**
- **新发现**：超时修好后，codex 答得诚实不瞎编（防幻觉真题过了——它判"证据不足、先别放行 M0"），**但它只啃了注入的 README、没真去读 红队/开发计划正文**（codex 自述没拿到读本地文件内容的资源）。**B1 的前提「裁注入、靠 codex 只读按需读」落空**——tier-1 `codex exec` 一次性模式多半不 agentic-on-demand-读。**这是主导线设计误判，撤回 B1 那条「靠 codex 按需读」假设。**
- **先探针定性，别猜、不信 codex 自报**：硬命令 codex 读一个具体 猫猫点菜 文件（如红队评审 `docs/03-评审/...`）+ **逐字引用其中一个标记串**。
  - **引对 = codex 能读（只是没读）** → 修 = prompt 硬命令它「先读 红队+开发计划 正文再答」（轻、不 bloat 注入）。
  - **引不出/说没权限 = 这模式真不读** → 修 = **把策展核心文档（README + 红队 + 开发计划正文）注入进 `ProjectContext`**（= 注入**更多**、不是裁；tier-1 小项目靠注入、on-demand 读是 tier-2 的事）。
- **不管哪条**：项目仍只读不可写（铁律）；修完重跑 §6，验 codex **真交叉引用了红队+开发计划**（深读关过）+ confinement 仍只读。timeout 420s 保留。

## 3. 安全死线（本包死线·必须成立）

- **被咨询项目永远不可写**（铁律）：不管 B1/B2,猫猫点菜 / 任何被咨询项目**绝不进 `allowed_write_roots`、绝不被写**。B2 只让 codex **自己的输出/scratch temp** 可写。正反测试钉死。
- **不碰 worker 路**：`decide_real_execution_command` / worker 执行闸 / path-lock / `command_plan_for` 沙箱本体 **0-diff**;不动已认的 guard 豁免逻辑(除非 B2 必须、且只在 consult 路、且记决策)。
- **自动测试不真起 codex**;真咨询仅 `#[ignore]` + 用户在场。
- **碰线就停**:要让被咨询项目可写 / 碰 worker 闸 / 改判决体 → 停、回主导线。

## 4. 验收（§6 真题·用户在场·只读猫猫点菜）

- **真咨询出答案且落地**:重跑 `s3_real_consult_mao_mao_dian_cai`,codex 在新 timeout 内**完成**、产出结构化方案、`goal_summary`/`reasoning` 非空、**交叉引用真读到的红队+开发计划**(不是瞎编)、喂得进 C1。
- **confinement 实物**(主导线核):猫猫点菜**没被写**(只 codex 输出 temp 动)、`~/.codex/auth.json` mtime 没变、被咨询项目不在写根。
- **诊断结论**写清(slow 还是 stuck、怎么修的)。
- 真 codex 偶发 flake → retry(记忆 `real-codex-run-flaky-verify-by-artifact`)。
- regression:stub 全链 + 既有测试全绿、`cargo test --lib` 计数不降;`fmt`/`git diff --check` 干净。

## 5. 本包不做

- **不让被咨询项目可写**(铁律)。
- 不碰 worker 执行路/闸/判决体。
- tier-2 工具 harness / 接 API / 档案 derive / 别的角色。
- inbox 复位(归布局重做)。

## 6. 回交

- 跑 §4;回交:诊断结论 + 修改 diff(确认项目不可写、worker 闸/判决体 0-diff)+ §6 真咨询答案(看落地)+ confinement 实物 + 计数 → 主导线核实物(重跑 §6 你在场 + 扫 diff 确认项目不可写、没碰 worker 闸)。子线不 commit。

## 7. 不接受为

- 不接受为:被咨询项目变得可写 / 碰了 worker 闸/判决体 / 真咨询仍取不回答案 / 答案是瞎编没引真文档 / 自动测试真起了 codex。
- 不接受为 S3 咨询整体完成(本包只到"真咨询能就着真项目落地出答案";tier-2/别的角色另说)。
