# 总线无人值守指示：M5R07 验收循环 → stage-15 与壳采纳 F0 v1

日期：2026-08-17
发出者：用户（经 Cursor CLI 会话代笔；用户原话为最高权威）
接收者：Codex CLI 总线指导与独立验收主管
状态：**当前用户明确授权的无人值守推进指示。**

## 0. 用户本次授权（原话要点）

用户 2026-08-17 明确表示：核心需求和要做的事情已经定得差不多，希望总线了解情况后按本指示**无人值守推进**：

1. 等 grok（开发主管会话）完成 M5R07 修正包后，总线独立完成验收；
2. 不通过就给 grok 开窄修正包返修，循环直到通过；
3. 通过后完成 closeout，然后**直接开始 stage-15（M6 域层先行）与壳采纳 F0**，不需要回来重新请示；
4. 本指示即为 stage-15 与壳采纳阶段的用户启动授权。

## 1. 先建立上下文（只读核对，不凭转述施工）

按顺序读取：

1. `decisions/2026-08-17-syn-lightcode-fork-desktop-shell-direction-v1.md`——lightcode fork 壳方向决定（lightcode 骨架 + Syn 功能 + Syn 皮肤；布局权威在新壳、槽位替换；Electron 栈不回迁；Rust 核心为权威服务、壳线程不是身份根；M6 域层先行、M6 UI 落新壳；移动端后置；GitHub 插件首期）。
2. `docs/plans/2026-08-17-syn-lightcode-fork-shell-adoption-plan-v1.md`——壳采纳 F0–F5 分期计划。
3. 同轮修订的权威文件：`docs/workbench-system-architecture-v1.md`（§4 壳层修订）、`docs/workbench-frontend-display-boundary-v1.md`（§0.1 修订：验收载体、移动端后置、布局条款降级）、`docs/plans/2026-08-01-syn-personal-ai-workbench-master-development-plan-v1.md`（KEEP 行）、`docs/plans/2026-08-01-syn-stage-6-global-supervisor-and-internal-organization-plan-v1.md`（验证矩阵载体注记）、`docs/plans/README.md`、`docs/product/authority-register-v1.md`。
4. 这些文档变更不触碰 M5R07 修正包写面，不改变 M5 验收标准；M5 验收仍按 stage-14 既有口径，不降级、不加码。

## 2. 无人值守推进链

### 第一步：M5R07 验收循环

- grok 交付新 candidate（六缺口修正包：真实 AppState/command 接线、正式 RoleSession、Proposal→Authorization→Grant→Dispatch 正式链、ProjectSummary source refs/stale、隔离双场景 Tauri 交互、disposable checkout receipt）后，总线独立验收：核 Git、代码、测试、证据、receipt 与 candidate SHA 绑定。
- **不通过**：写明缺口，开新的最窄修正包给 grok，保持 stage-14 开启、M5R07 current、authorization closed；循环直到通过。修正包不得扩大范围或引入新目标。
- **通过**：执行 closeout——归档 M5R07 与 stage-14，同步 current-state、总计划、M5 计划、计划索引、Harness plan，形成 M6 输入 handoff，单独 lifecycle commit，authorization 回 closed。

### 第二步：壳方向文档单独提交

- 把第 1 节列出的 2026-08-17 文档变更（含本文件）做成独立 doc carrier commit；不与 M5 candidate 混合，不吞入七个 m6_*.rs 未跟踪文件或其他 WIP。

### 第零步（2026-08-17 15:50 用户并行化修订）：B 线壳采纳 F0 立即启动，不等 M5

用户明确串行太慢，确认并行开发表：F0/F1 在姊妹仓库 `/home/synadmin/workspace/syn-shell` 施工，与 syn 仓库零写面冲突，**从现在起即可与 M5 验收循环并行**。为 B 线另建一个开发主管会话（不占用 grok 的 M5 写位）；F2 合同草案、旧前端风格 token 提取、M6 域层任务包起草（进 unfinished/）也可在验收间隙并行做。F2 冻结与实施、F3/F4/F5、stage-15 施工仍以 M5 closeout 为硬门。syn 仓库源码写面同时只允许一个施工者。

### 第三步：并行开两条线（本指示即用户启动授权）

**A. stage-15（M6 域层先行）**

- 按 `docs/plans/2026-08-16-syn-m5-m6-fact-reconciliation-and-product-closure-plan-v1.md` 与 M6 阶段计划建立真实 stage-15；域层（合同、service、repository、投影）先行施工，为 grok 建立开发主管任务。
- M6 产品 UI 与隔离 App 验收载体为新壳，UI 部分待壳采纳 F2/F3 就绪后再排；建 stage-15 时按新壳口径改写验证矩阵 "Isolated Tauri" 行。
- 既有七个 m6_*.rs 未跟踪文件按 M5/M6 修正计划处置：可复用、修正或淘汰，不默认继承完成状态。

**B. 壳采纳 F0（源码入库与可构建基线）**

- 按壳采纳计划 F0 建立阶段与 leaf：fork/vendor `github.com/SDSLeon/lightcode` 上游 exact SHA（clone 该开源仓库的只读网络读取由本指示授权；仍不 push 任何远端）；建议落位为姊妹仓库 `/home/synadmin/workspace/syn-shell`，最终形态以真实构建事实确认。
- 复现构建与运行；盘点模块地图、导航槽位（供 F3 槽位替换）、remote/relay 安全面（对照架构正本 §5.10）；产出保留/裁剪清单、LICENSE/NOTICE 与品牌替换清单。
- F0 完成后可按计划顺序续 F1（品牌与风格基线）；F2 起涉及壳 ↔ Syn 核心接口合同，完成 F1 后如证据与合同基础充分可继续，存疑则停点报告。

## 3. 全程边界（不因无人值守放宽）

- 不 push、merge、rebase、deploy、release（clone lightcode 上游只读除外）；
- 不接真实个人资料、真实用户项目写入、真实模型/provider、真实消息、账号、凭据、connector 或外部网络业务写动作；
- 不 reset、stash、clean、覆盖或丢失既有 WIP；不 `git add -A` 吞混合 WIP；
- stage-12、D0C04/D0C05、M1–M5 冻结合同只读保全；
- 不伪造 receipt、测试或 App 证据；证据只到其真实证明范围；
- 每个阶段/叶按 Harness Lite 正常建立、审计、退场；报告分 Harness、产品、证据、载体。

## 4. 停点（遇到即停，等用户）

- 需要真实凭据、真实 provider、外部网络业务写、push 或发布；
- lightcode 上游授权或品牌事实与已核实的 Apache-2.0 不符；
- 无法在不丢 WIP 的前提下继续；
- 本指示与用户原话、产品正本或权威登记冲突；
- M5R07 连续多轮返修仍无法收敛（谨慎判断后向用户报告循环卡点，而不是无限空转）。
