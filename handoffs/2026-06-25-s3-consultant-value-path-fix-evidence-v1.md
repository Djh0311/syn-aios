# 回交：S3 咨询第一刀·价值路径修复（诊断 + 注入修复 + §6 真证到）· 执行线 → 主导线 v1

日期：2026-06-25　性质：诊断 + 修复（轻档：只调 timeout + 改注入策略；**没碰沙箱/项目仍只读**）　任务包：`tasks/2026-06-25-s3-consultant-value-path-fix-v1.md`（§2C）　上游：咨询第一刀 `950a726`

## 0. 一句话结论

诊断 + 探针定性 + 对症修，**§6 防幻觉真题第一次真证到**：codex 现在真交叉引用红队+开发计划、找出真缺口、**不瞎编（核实物确认引的串都真在文档里）**、喂得进 C1，**被咨询项目全程只读没被写**。**执行线未 commit。**

## 1. 诊断结论（§2A + §2C·不信 codex 自报，实物定性）

- **超时 = SLOW（确认）**：bump 180s→420s 后 §6 两次都 < 90s 完成（58s / 89s）。第一次 180s 超时多半叠加了那会儿 OpenAI 慢。**420s 保留作 headroom**（主导线拍板）。
- **§2C 探针 = tier-1 codex exec 真不 on-demand 读项目文件（定性）**：新增探针 `s3_diag_codex_reads_redteam_file`，经真 consult 路 `readonly_codex_consult` 硬命令 codex 读「红队正文」（内容**不在**注入的 README）并逐字引专属串。**codex 回「无法读取文件。」、红队专属标记零命中** → 证实 codex 自报为真，**B1「靠 codex 按需读」前提落空**（主导线已撤回该设计误判）。tier-1 `codex exec` 一次性模式没有按需读取工具，只啃注入。

## 2. 修法（主导线 branch 2：注入策展核心文档正文·注入更多·不是裁）

`consultant_agent.rs`：
- **`ProjectContext` 加 `injected_documents: Vec<(路径, 正文)>`** + helper `consultant_load_documents`：把文档地图里的 .md **正文**读进来注入（每篇截断 20000 字、合计 150000 字防爆；按地图序，big 文档截断、关键文档全进）。
- **prompt 重写**：注入各文档正文 +「你**读不到**未注入的文件、**只依据已注入的文档正文作答**、要交叉核对就在注入正文里逐条找依据并原文引用」。（撤掉原来「需要核实就只读地读文档」的错误指引。）
- **timeout 默认 180s→420s**。

## 3. §6 真证到（核实物·不信自报）

`cargo test --lib s3_real_consult_mao_mao_dian_cai -- --ignored --nocapture`（89s 完成）：codex 直答防幻觉真题——**「有：RT-14 漏到 M0 外且 M0 先固化了相反访问模型；RT-06 是『推迟』不是『收口』；RT-11 在 M0 没显式前置」**，逐条**同时引红队原文 + 开发计划任务编号**，还点破 README「19 条全部收口」过乐观。

**不是瞎编（实物核）**：codex 引的串逐个 grep 真文档——
- 红队正文含：`字段裁剪`✓ `一刀切全经云函数`✓ `RT-06`✓ `RT-14`✓ `推迟`✓
- 开发计划正文含：`T12.3`✓ `T0.2`✓ `T0.5`✓ `契约先立`✓ `T9.1`✓

→ **真交叉引用、引的是真内容**。§4 验收全过。

**confinement 实物**：猫猫点菜本次**没被写**（marker 后 find 空）、`~/.codex/auth.json` mtime 仍 Jun 3。**铁律守住**：修法只动注入（prompt 内容），**没碰沙箱**——`codex_local_runner.rs`（含 `readonly_codex_consult`/`command_plan_for`/`build_readonly_consult_request`）**0-diff**，项目仍 `sandbox=read-only`+`allowed_write_roots=[]`。

## 4. 验收门

| 门 | 结果 |
|---|---|
| §6 真咨询出答案且落地·交叉引用·不瞎编 | ✅ 见 §3（89s，RT-14/06/11 真缺口，引串实物核过）|
| confinement（项目只读·auth 没动）| ✅ 猫猫点菜没被写、auth.json Jun 3 |
| 铁律 0-diff（沙箱/readonly_consult/command_plan_for/worker 闸/判决体）| ✅ codex_local_runner.rs / real_execution_command.rs **0 行** |
| stub 全链 + 既有 | ✅ 6 个 s3 stub 全过；新增 `injected_documents` 注入断言 |
| regression | ✅ `cargo test --lib` = **597 passed / 0 failed / 29 ignored**（+1 = 探针 #[ignore]）|
| fmt / git diff --check | ✅ 干净 |
| 范围 | `consultant_agent.rs`（注入修 + timeout）+ `lib.rs`（探针 + 注入断言）|

## 5. 主导线核实物 + 收口

- **重跑 §6**（你在场）：`cargo test --lib s3_real_consult_mao_mao_dian_cai -- --ignored --nocapture` → 看答案真交叉引用红队+开发计划；扫 diff 确认 `codex_local_runner` 0-diff（项目仍只读）。
- **核 confinement**：跑后 `find 猫猫点菜 -newer <marker>` 空、auth.json mtime 没变。
- **一起收口**（主导线拍板「420s + 探针 + 深读修 + §6 重验一起回交」）：执行线不 commit；你核实物 + commit + CURRENT 回写「S3 咨询第一刀价值路径证到（tier-1 靠注入；on-demand=tier-2）」。
- **定性入档建议**：tier-1 `codex exec`（read-only 一次性）**不 on-demand 读项目文件、靠注入**；on-demand 读取工具 = tier-2（spec §3B）。已写进代码注释 + 本证据；建议咨询把 spec §3A/§3B 这条更正（撤 B1 的「按需读」假设）。

## 6. 本包没做（deferred）
tier-2 on-demand 只读工具 harness / 接 API / 档案 derive / 别的角色。注入截断阈值（20K/篇·150K 合计）是 v0 拍的，超大项目可能需分块/检索（tier-2）。
