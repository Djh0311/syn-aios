# 实现任务包：S3 咨询第一刀 v0「契约 + CLI/tier-1 impl + 静态档案 + ProjectContext(猫猫点菜)」· 主导线 → 执行线 v1

日期：2026-06-25

出自：主导线（Claude）。性质：**建是轻档**（Rust 代码 + stub 测试，不真起 codex）；**首次真咨询是「读真项目」单独步**——第一次让 codex（**只读**）上非测试真实项目，需主导线核实物 confinement + 用户在场。上游：spec `docs/plans/2026-06-25-s3-agent-layer-consultant-first-slice-spec-v1.md`（**先读·一切据它**）。

## 0. 接手须知

- 你是**执行线**。流水线：实现 + stub 测试（**自动测试绝不真起 codex**）→ 主导线核实物 → 真咨询（`#[ignore]` / 用户在场·只读·猫猫点菜）。子线不 `git add`/`commit`。
- 先读：spec 全文 + `codex_local_runner`（**只读沙箱模式 + `--add-dir` 怎么用**，照 `manual_relay` 的限项目套路）+ `create_project_consultation_proposal`（C1 输入 schema，产出要喂得进它）+ `AGENTS.md` 高危#1/#3。
- **全程中文。子线不 commit。** 一句话：**建「契约 trait + CLI 只读 impl + 静态档案 + ProjectContext 装配」,咨询全程只读、codex 硬钉只读沙箱限项目、不碰执行闸、产出喂 C1；不建 tier-2 工具 loop、不接 API、不 derive。**

## 1. 拍板摘要

- **要做的事**：落 spec §2–§4 的**第一刀 v0**——稳定契约 + CLI/tier-1 impl（驱动 codex **只读**读项目文档、产出咨询方案）+ v0 静态档案 + ProjectContext 装配（curated core 注入 + 找入口文档 + 防御式降级），对 `/Users/yoyi/project/猫猫点菜小程序` 跑得通，方案喂现有 C1。
- **代价**：一轮 Rust。做完后**咨询 agent 第一次能就着一个真项目、落地答出能进循环的方向**（证 LM+harness 这套有用）。
- **关键澄清**：v0 用 **CLI/tier-1**——codex 自带 loop、自己读文档（**只读沙箱**）；**不建 spec §3B 那套 6 只读工具 harness loop（那是 tier-2/后续）**。不接 API、不 derive 档案、不做别的角色。

## 一句话判据

判某改动在不在本包内——问：**「是不是在建『契约 + CLI 只读 impl + 静态档案 + ProjectContext 装配』、咨询只读、codex 硬钉只读沙箱限当前咨询项目、没碰执行闸/没改沙箱、产出喂 C1，且没建 tier-2 工具 loop / 没接 API / 没 derive 档案 / 没做别角色?」** 是 → 做；否（尤其要让 codex 可写/可执行、改沙箱、走 worker 执行闸、建 tier-2 loop、接 API）→ **停、回主导线。**

## 2. 建什么（据 spec §2–§4）

1. **契约 trait**（缝·稳定）：
   ```
   trait ConsultantAgent { fn consult(&self, ctx: &ProjectContext, question: &str) -> Result<ConsultationProposal, String>; }
   ```
   `ConsultationProposal` 字段 = 目标/范围/为什么/风险与不确定/必停点/建议下一步——**且能映射进 C1 `create_project_consultation_proposal` 的输入**（先读 C1 schema 定字段）。
2. **`CliConsultantAgent`（tier-1 impl）**：拼 prompt（**静态档案 + ProjectContext + question**）→ 经 `codex_local_runner` **只读沙箱**起 codex（codex 自己读项目文档）→ 抠出 `ConsultationProposal`。
3. **v0 静态档案**：照 spec §3C 那段（结构核 + 行为层，**写死、先不 derive**）。
4. **ProjectContext 装配**（spec §3A + §4）：
   - 找入口文档（root 或 `docs/` 下 README/CURRENT/index）整篇注入；扫文档/结构地图。
   - 黑板/记忆**有才注、空就跳**；无 git 跳 git-log 用版本/mtime。**有啥塞啥、不假设齐全。**
   - 对猫猫点菜实测：注入 `docs/README.md` 全文 + 文档地图，黑板/记忆空，无 git。
5. **喂 C1**：`ConsultationProposal` → `create_project_consultation_proposal`，下游循环不动。

## 3. 安全死线（本包死线·必须成立）

- **咨询 codex 硬钉只读**：经 `codex_local_runner` **只读沙箱**（复用现成只读模式、**`command_plan_for` 字节不改**），**结构上不暴露 workspace-write/执行选项**——这条路**永远不能写/不能跑命令**。要它能写 = 另起高危决策、回主导线。
- **限当前咨询项目 + 不碰凭据**：`--add-dir` 只圈当前被咨询项目目录；**不给 `~/.codex`/凭据/其它项目**（照 `manual_relay` 限项目套路）。
- **不走 worker 执行闸**：只读咨询 ≠ 真执行，**不经** `decide_real_execution_command`（那是写/执行的闸）——但**自带 confinement**（上两条），不是无防护。**不得借此路写/执行**（只读沙箱挡死）。
- **更新「无漏网」分类**：把这条**只读咨询 codex 路**登记进入口×守卫表，归类 **「read-only-confined」**（既非 path-lock 也非授权矩阵，但**结构性只读**）——让将来审计看得见、知道它只读。
- **不改**：执行闸 / `decide_real_execution_command` / `command_plan_for` 沙箱 / 既有封堵 / A 线 store **0-diff**。
- **自动测试不真起 codex**（stub `ConsultantAgent`）；真 codex 仅 `#[ignore]` + 用户在场。
- **碰线就停**：要可写/可执行 / 改沙箱 / 走 worker 闸 / 读凭据 / 越出项目 → 停、回主导线。

## 4. TDD 验收门（测试钉死）

- **契约 + 装配（stub）**：用 stub `ConsultantAgent`（不起 codex）测全链——ProjectContext 装配正确（入口文档找到、地图对、空黑板/无 git 优雅降级）、`ConsultationProposal` 映射进 C1 入口、喂 C1 成功。
- **只读 confinement 单测**：CLI impl 构造的 codex 命令计划 = **只读沙箱**、`--add-dir` 只含咨询项目、**无凭据/无其它目录**；正反断言（给写选项应拒/不可达）。
- **regression**：执行闸/沙箱/A 线/既有封堵 0-diff、`cargo test --lib` 计数不降。
- **全量**：`cargo test --lib` / `cargo fmt -- --check` / `git diff --check`。

## 5. 本包不做（deferred）

- **tier-2**：spec §3B 那套 6 只读工具 harness loop（等 API）。
- 接 API / provider 适配器。
- 档案行为层 **derive**（v0 静态写死）。
- 别的角色（主管/worker/秘书/全局主管）/ NL 拆解 / 一句话启动 / 套模版 / 反馈边 / 每日记忆采集。
- 任何让 codex 可写/可执行的能力。

## 6. 真咨询验证（单独步·`#[ignore]` 或用户在场·只读·猫猫点菜）

建完 + stub 验通 + 主导线核实物**之后**，单独一步：对 `/Users/yoyi/project/猫猫点菜小程序` 真跑一次咨询——注入 ProjectContext + 问 spec §5 那道**防幻觉真题**（"红队 19 条说全收口，抽查开发计划 M0,有没有红队点了、开发计划没接的?"）。验：
- 答案**落地引用真读到的文档**（红队 + 开发计划交叉）、给出能进循环的结构化方向、喂得进 C1。
- **confinement 实物**：codex 只读、没写任何文件、`--add-dir` 没越出项目、`~/.codex/auth.json` mtime 没变。
- 注意真 codex 偶发 flake → retry（见记忆 `real-codex-run-flaky-verify-by-artifact`）。
- `#[ignore]` 默认不跑。

## 7. 回交

- 跑 §4 各门；回交：实现 diff（确认执行闸/沙箱/A 线 0-diff、只读 confinement）+ stub 全链证据 + 只读单测证据 + 真咨询那次的答案 + confinement 实物 + 「无漏网」表更新 → 主导线核实物（重跑计数 + 扫 diff + 真咨询你在场跑一遍·核只读）。子线不 commit。

## 8. 不接受为

- 不接受为：codex 能写/能执行 / 改了沙箱·执行闸 / 走了 worker 闸 / 读了凭据 / 越出项目 / 建了 tier-2 loop / 接了 API / derive 了档案 / 自动测试真起了 codex / 产出喂不进 C1。
- 不接受为 S3 整体完成（这只是「一个角色·CLI·静态档案·能就着真项目落地答」第一刀）。
