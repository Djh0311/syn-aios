# 会话交接：视觉治理线 G2→G4 全收 · 死视图迁宿主后删 · H2 仍待额度（2026-07-20 下）

> 接棒人=新一代**总指导**（主导线）。读序：`CURRENT.md` → 本文 → 总执行计划 `docs/plans/2026-07-16-master-execution-plan-conversation-first-v1.md`。规则正本 `AGENTS.md`；主导线=唯一 commit 权（问一次）、核实物不信自报、对接用户；执行线吃包不 commit。
> 本会话三个 commit：`8213678`（G2 定式扶正）→ `0e9c61d`（G3 盖章时刻）→ `5dba57e`（G4 死重清扫）。工作树干净，全部未 push。上代交接：`handoffs/2026-07-20-m5-recovery-g1-visual-line-and-h2-resume-handoff-v1.md`。

## 一、本会话干了什么（按大块）

1. **G2 定式扶正收口**（`8213678`）：spec-* 扶正为唯一——事实行 3 式→FactRow（jiaoban-fact 4/memory-kv 4/settings-fact 4·SettingFact 删·元素 47→59）、pill 4 式→Pill（Badge 102 处 31 文件/step-badge 5/canvas-pill 2/prsb 2·Pill 0→111·tone 扩 7=+candidate/unknown/bad 全取既有 token·零语义漂移）+Badge.tsx 删除+退休族 CSS 全删+gate 新规则 `retired_style_family`（selftest 13/13·gate 495→498·白名单 2 条 decisions 登记）+hex 白名单 42→39。亲跑=tsc 0+离线 24 组+shape 13/5/5 零净增+截图 11 对看形过关。包 `tasks/2026-07-20-g2-spec-primitives-restoration-package-v1.md`、证据 `evidence/2026-07-20-g2-spec-primitives-restoration-verification-v1.md`。
2. **G3 盖章时刻收口**（`0e9c61d`）：方案卡批后右上石绿圆章（样张逐值 76px·-12°·multiply·「已批准+SYN·MM-DD」·仅 user_confirmed 出章·批后常驻·fresh 翻 confirmed 那一下 stamp+thud 0.3s·reduced-motion 全禁静态·日期=updated_at_ms 零假数）+交货卡 tone-yellow 三处改朱砂页边批注 `.jiaoban-flag-note`（**用户拍「只改交货卡」**·分级=危险有形 bad pill/黄牌无形批注）+PillRow ariaLabel 补回「这单概览」。离线 25 组（+新册 8 断言组）。**用户最后一眼拍「接受」**；唯一留口=章与首行事实行值轻叠（multiply 双可读·样张同构·已拍定保留）。证据 `evidence/2026-07-20-g3-approval-seal-moment-verification-v1.md`。
3. **G4 死重清扫收口**（`5dba57e`）：RunningWorkflowsView 1196 行死视图**迁宿主后删**（用户拍·勘察发现它是离线测试宿主跑活功能真断言→M1 直渲 DailyMemoryCandidateInbox/M2 直渲 PermissionDialog+字面 action/M3M4 摘死半边·预登记 R1-R8 退役壳+M1-M4 迁移不沉默）+死组件五件（WorkflowStatePanel/Metric/ProjectWorkflowRecoveryPanels/ProjectRuleStatusBar+连坐/CandidateMemoryItem）+ExpandRest+连坐 CSS（共享选择器 5 处只摘成员）+hex 39→36+retired 白名单 2→1+重复定义口径对平（完全相同 0 组零删除/分叉 56 组挂账·审计 112≈56 组 117 处）。14 文件 +104/−1973。证据+预登记+decisions 三件套。
4. **视觉治理线 G1→G4 全收**（一日线）。执行线连续三轮零误报、主动披露成惯例；总指导 catch 三件全自记（见 §五）。

## 二、当前待办（按序）

1. **H2 续验（重档·等 Codex 额度·需用户在场）**：口径同上代交接 §二.1 逐字——Gate 1 二进制（hash `f9d028f3…`）起真实 App，首句已在 canonical，直接发第二句「按这个出方案」，验主管答复回同一 thread+`submit_proposal` handler 到达+proposal +1 且 Pending +1 目标匹配+chain 40 不变+刷新不重复落卡。**到卡即停**。⚠ G2/G3/G4 动了前端源码，**真 App 二进制须重建并重冻结 hash 再续验**（`../tauri-capability-probe/.tauri-cli/bin/cargo-tauri build --debug`，cwd=`prototypes/productized-desktop-shell`）。
2. **底1 真机首单（H2 后）**：从 H2 落的 Pending 卡点批起跑，清 A5+三数复测+P3-A 走查+记忆环①+P1-E 关门脸+记忆中心过目。
3. **知识库第一片**（不烧额度可立即开）：形态 07-18 已拍、勘察已毕待写包（总执行计划 §二排队）；点击回源判不做挂账随此线一并设计（07-18 记忆中心条目）。
4. **底2 包**（底1 真机后写）→ **P2-B 挑会话归置**（待议）。
5. **G 线挂账收尾**（轻档小包，随时）：分叉重复 56 组面审（清单在 G4 decisions §5+evidence §六）→ running-* 死壳 CSS 下把刀 → 两零引用件另拍（lib `buildOperationControlMemoryCaptureInput`、DailyMemoryCandidateInbox 仅剩测试宿主「测试宿主算不算活」）。
6. **架构两刀**（写操作注册表杀上帝开关/状态容器+事件订阅代轮询）：视觉线收官后可另议排期，动行为面不混视觉线。
7. 小尾巴：P1-E 两旧按钮退场、P1-D 走查 3/4（待额度）、病历五号 store 残锁小包、人话工程③清单攒。

## 三、挂账与警报器

- **DB 观察期**：07-20 重 seed 后重计中；M6 停写 JSON=观察期够+用户另授权（audit_ledger 读源切 DB 连带）。
- **memories 渗出**：观察 a 维持；codex 带真家私人 MCP=正常不当洞治（07-18 拍）。
- **额度**：H2/底1/P1-D 走查全卡在它。
- **真 App 看 G1-G4 效果需重建**（同 §二.1 ⚠）。
- **push**：全部未推；**每次 push=用户明确说「push」那一下**（高危#5）。
- **仓外备份**：`/Users/yoyi/workbench-backups/`（07-14/07-16/07-19-003058 三代）。

## 四、总指导闸清单（数字=当前基线）

- `cargo test --lib`（cwd=`src-tauri`）：**1024/0/44**（本线零 Rust，未复跑）。
- `npx tsc --noEmit`=0；离线交互 **25 组全绿**（24+G3 新册 jiaoban-approval-seal-and-flag-note）。
- shape gate：`cd /Users/yoyi/workspace/product-line && node scripts/harness/workbench-shape-gate.js --mode check`：**13/5/5 零净增**（exit 1 正常）；selftest 四册 **18/13/8/13 全绿**；gate 本体 **498 行**（500 软限）。
- 现行机械规则四件：`machine_face_on_ui`、`hardcoded_hex_on_ui`（白名单 **36** 只减不增）、`retired_style_family`（白名单 **1**=ActiveWorkbenchView:277 spec-empty 有意例外）、`converged_helper_redefined`（warn）。
- commit：消息必含 `catch:`、**CURRENT 回写同笔**、问用户一次；工具本地目录（`.claude/`、`.playwright-cli/`、`output/`）不跟仓。

## 五、操作知识（免摸索·本代新增）

- **感受件纪律全流程**（G3 走通）：截图 before/after→总指导看形→留口主动披露→用户最后一眼定夺→catch 明写。playwright 二进制直截法见上代交接 §五。
- **删测=预登记制**（G4 实证）：`evidence/2026-07-20-g4-retired-assertions-preregistration-v1.md` 模板——迁移 M 组（断言平移到被测组件本体）与退役 R 组（壳断言）分列，每条给理由+替代覆盖。
- **共享选择器只摘成员不删块**（`.metric`/`.workflow-state-panel`/prsb 前科面）：CSS 连坐删除前必查选择器是否多成员。
- **死件判定三步**：src 零 import（符号级不是文件名级）→ tests 引用面（**死视图可能是离线宿主跑活断言**，RunningWorkflowsView 前科）→ 连坐帮助函数唯一调用点随死。
- **重复定义对平口径**：同文件同 @media 上下文同选择器；朴素哈希法会误报跨上下文（G4 实证 5 组），须上下文栈递归解析+人工抽检。
- **本代三 catch 入账（防再犯）**：①G2 包允许写入面漏列派生文件（tone helpers/白名单本体）→**勘察清单须含派生写入面**；②G2 计数口径混（行 vs 元素）→口径披露段已成执行线惯例；③G4 主测试渲染块计数 2→实 5→**勘察枚举粒度=渲染块不是调用点**。
- **执行线工作模式**：同上代（kickoff 经用户转发、TASK_TEMPLATE 10 项回传、总指导亲跑四闸+对账逐行+diff 自查写入面）。

## 六、给接棒人的一句话

视觉线四包一日全收、闸四件齐（机器话/hex/退休族/重复 helper），主线没断：**H2→底1 仍是咽喉**（额度+二进制重建，§二.1）；不烧额度的下一件=**知识库第一片**（§二.3），G 线挂账小包随时可清。每步收口必回写 CURRENT，catch 必记账，证据说话。
