# 会话交接：M5 恢复收口 · 人话工程①② · 三栏归真设计线 G1 落地 · H2 待额度续验（2026-07-19→20）

> 接棒人=新一代**总指导**（主导线）。读序：`CURRENT.md` → 本文 → 总执行计划 `docs/plans/2026-07-16-master-execution-plan-conversation-first-v1.md`（防跑偏总则执行任何包前重读）。规则正本 `AGENTS.md`；主导线=唯一 commit 权（问一次）、核实物不信自报、对接用户；执行线吃包不 commit。
> 本会话四个 commit：`24bde2e`（S1B/S1C/H1/H2/R1 收口基线）→ `ee684e5`（人话工程①②）→ `1037819`（设计方向落档）→ `a1d2a67`（G1 token 归真+红章拍板）。工作树干净。

## 一、本会话干了什么（按大块）

1. **M5-LIVE-R3 现场核收（重档·恢复闸全绿接受，落卡闸额度挂起）**：执行线按包 `tasks/2026-07-19-m5-f1-r1-live-reseed-and-s1b-h2-real-app-verification-package-v1.md` 七闸走完——新仓外备份 467 文件全 PASS、用户亲手 R3-B1 apply 一次 completed 九旗全绿、静态 17 面+启动对账 DB=JSON lag=0 零新增降级（revision 274→276、audit 1773=1773）、H2 首句「我想给这个游戏里的标题改成小马里奥」已入 canonical（resident user msg 8）后主管因 **Codex 额度上限**未运行，止损挂起。总指导十项逐件亲验实物（git/hash/report 九旗/生产 JSON+DB 直查/二进制 hash/registry rev1120 entries=[]/mario 项目），零误报。证据 `evidence/2026-07-19-m5-f1-r1-live-reseed-and-s1b-h2-real-app-verification-v1.md` + `evidence/raw/2026-07-19-m5-live-reseed-h2/`。**真实存储=DB-primary 健康，观察期重计中。**
2. **人话工程①②收口**（`ee684e5`）：前端 11 个错误翻译函数逐字收编进新单一真源 `src/lib/humanize.ts`（272 行·6 迁出文件 re-export 保导入面）；后端删 2 个 `humanize_consult_error` 薄壳直调 `run_error_translation`；App notice `messageOf` 接薄委托（命中族出人话·未命中逐字不变）；shape gate 新规则 `machine_face_on_ui`（直渲 error 级新增零容忍·state 形 warn·白名单 6 条 decisions 登记）。总指导亲核=cargo 1024/0/44+tsc 0+离线全绿+shape 13/5/5+三函数体对 HEAD 逐字 IDENTICAL。③清单与 *Label 枚举族/重复簇挂账（包 `tasks/2026-07-20-human-language-engineering-modules-and-machine-face-gate-rule-package-v1.md` §十三）。
3. **前端双审计+设计方向拍板**（`1037819`）：架构+视觉两份审计（带 file:line）发现**实现层漂离自己的宪法**——`--bg:#f3f0ea` 运行时死值、≥1181px 桌面皮违七律（白卡+朱红选中）、88 token 四时代、237 硬编码 hex、定式组件零 adoption、上帝开关/五层 prop drilling/读模型双轨/1196 行死视图。出两版样张（`prototypes/design-mockups/`：**三栏归真版** `jiaoban-redesign-specimen-v1.html` + **手账概念版** `jiaoban-journal-concept-v1.html`），**用户拍：三栏归真**，手账留档下一代。决策 `decisions/2026-07-20-jiaoban-design-direction-three-column-truth-restoration-v1.md`；视觉治理线 G1→G4 排入总执行计划 §二；**架构两刀（写操作注册表/状态容器+事件订阅）另议不混视觉线**。
4. **G1 token 归真收口+红章拍板**（`a1d2a67`）：六 `:root` 坍一单正典 76 token（`--bg` 定回 `#f3f0ea`、`--warning` 定回 `#8a4010`、中性=live、alias 桥接零引用面改动）+14 死 token 清零+`--ui-*` 全系退役+桌面皮死段 12 区退役/活违宪 6 组治平/孤儿 6+1 先迁后删（`.hide-dev-detail` 实测默认隐藏仍在）+hex 262 裸值+9 转义归 token（violations 0·白名单 42≤86 只减不增）+字体 21 档→7 档/650·800 清零/mono 归一/幽灵 webfont 五族清零+gate `hardcoded_hex_on_ui` 上线（selftest 13/13·gate 495 行）。**用户最后一眼拍「红章+绿动作」**：`.brand-mark` 朱砂红品牌专属例外（文化=印章本红），批准/按钮/选中仍石绿，红全 App 仅两坑=品牌印章+黄牌/危险；decisions §补充拍板，枚举⑨作废。

## 二、当前待办（按序）

1. **H2 续验（重档·等 Codex 额度·需用户在场）**：额度恢复后，用 Gate 1 构建的二进制 `prototypes/productized-desktop-shell/src-tauri/target/debug/codex-governance-workbench`（hash `f9d028f3…`，含 R1/H2）启动真实 App——首句已在 canonical，**直接发第二句「按这个出方案」**，验：主管答复回同一真实 thread、`submit_proposal` handler 到达、proposal 恰好 +1 且 Pending +1 目标匹配「小马里奥标题」、chain 40 不变、刷新不重复落卡。**到卡即停：不点卡、不起链、不派 worker。** 续验前按包 §Gate 0 重冻结基线（进程/lsof/registry/五源 hash/真实现场计数）。实现包 `tasks/2026-07-19-s1b-h2-supervisor-syn-natural-information-flow-package-v1.md`。⚠ 构建产物 target/ 不入仓；若源码此后有改动须重建并重冻结 hash。
2. **底1 真机首单（H2 后）**：从 H2 落的 Pending 卡点批起跑，清 A5+三数复测+P3-A 走查+记忆环①+P1-E 关门脸+记忆中心过目。
3. **G2 定式扶正（视觉治理线下一件·轻档离线）**：spec-* 扶正为唯一——事实行 4 式→`FactRow`（`spec-fact-row` 全 App 仅 1 处直连；`.jiaoban-fact` 12.5px/`.memory-kv`/`.settings-fact` 三式）、pill 5 式→`Pill`（语义色已归 token，G1 已铺路）；迁移后删旧式+gate 防再造；顺清 styles.css 死壳残段与 112 处文件内重复定义。勘察起点=07-20 视觉审计（本会话 agent-2 报告，关键数已进 G1 包与 decisions）；G2 出包前照例重核行号（G1 已动 styles.css，旧行号漂移）。
4. **G3 盖章时刻**（批准=石绿印章落纸，样张已拍·reduced-motion 退化·需用户最后一眼）→ **G4 死重清扫**（RunningWorkflowsView 1196 行死视图/占位页/re-export 门面）。
5. **后续排队**：底2 包 → 知识库第一片（形态 07-18 已拍）→ P2-B 挑会话归置；小尾巴：P1-E 两旧按钮退场、P1-D 走查 3/4（待额度）、病历五号 store 残锁小包、人话工程③清单攒（机器话清单底稿=`docs/plans/2026-07-14-syn-frontend-stage1-audit-v1.md` §四.3）。

## 三、挂账与警报器

- **DB 观察期**：07-20 重 seed 后重计中；M6 停写 JSON=观察期够+用户另授权（audit_ledger 读源切 DB 连带）。
- **memories 渗出**：观察 a 维持；codex 带真家私人 MCP=正常不当洞治（07-18 拍）。
- **额度**：H2/底1/P1-D 走查全卡在它；失败单先定位挂在哪一环再下结论（总指导误判前科在档）。
- **真 App 看 G1 效果需重新构建**：当前 debug 二进制是 G1 前的；用户自然重启 App 前若想看到红章/真米色，须 rebuild（`../tauri-capability-probe/.tauri-cli/bin/cargo-tauri build --debug`，cwd=`prototypes/productized-desktop-shell`）。
- **push**：全部未推；**每次 push=用户明确说「push」那一下**（高危#5）。

## 四、总指导闸清单（数字=当前基线）

- `cargo test --lib`（cwd=`prototypes/productized-desktop-shell/src-tauri`）：**1024/0/44**；删测走预登记制。
- `npx tsc --noEmit`（cwd=`prototypes/productized-desktop-shell`）=0；离线交互 `node scripts/run-offline-interaction-test.mjs` 24 组全绿。
- shape gate **固化命令单独一条 Bash**：`cd /Users/yoyi/workspace/product-line && node scripts/harness/workbench-shape-gate.js --mode check`：**13/5/5 零净增**（历史债，exit 1 正常）；规则三件套自测=`workbench-shape-gate.{machine-face,hardcoded-hex,dedup}.selftest.js` = **18/13/8 全绿**；gate 本体 495 行，新规则必须拆 `scripts/harness/lib/`（500 软限）。
- 现行机械规则：`machine_face_on_ui`（UI 禁直渲机器错误串）、`hardcoded_hex_on_ui`（禁新硬编码 hex·白名单 42 条只减不增）、`converged_helper_redefined`（warn）。
- commit：消息必含 `catch:`（hook 强制）、**CURRENT 回写同笔**、问用户一次；工具本地目录（`.claude/`、`.playwright-cli/`、`output/`）不跟仓。

## 五、操作知识（免摸索）

- **playwright 截图**：`npx playwright@1.61.1` 与缓存浏览器（`~/Library/Caches/ms-playwright/chromium-1217`）版本不配，直接用二进制：`~/Library/Caches/ms-playwright/chromium-1217/chrome-mac-arm64/Google Chrome for Testing.app/Contents/MacOS/Google Chrome for Testing --headless --disable-gpu --screenshot=<out> --window-size=<W,H> --hide-scrollbars <url>`。感受件纪律：截图对照 before/after 先总指导看形，再用户最后一眼（G1 先例 `output/playwright/g1-token-truth/`）。
- **真实现场直查**：workflow-state=`~/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json`（python json 读，audit 按数组序取尾）；DB=`…/production-db/workbench-state.v1.sqlite`（`sqlite3 -readonly`，表名 `workflow_audit_events`/`workflow_chain_runs`/`project_proposals`）；registry=`…/workflow-state/exec-process-registry.v1.json`；proposal store=`…/workflow-state/project-proposals.v1.json`。
- **本日两 catch 入账（防再犯）**：①总指导勘察「等」字漏列 warn 档观察件（gate 拦下）→**勘察清单逐文件枚举，禁用「等」**；②写包数字口径两连失（函数数与表不符、hex「255」与分列 262 不符、死段行数估计差 10 倍）→**包文数字必须与自带分列对平，估计标「待实测」**。
- **执行线工作模式**：kickoff 经用户转发到另一会话；回传按 TASK_TEMPLATE 10 项；总指导亲跑四闸+对账表逐行核+grep 迁移面+函数体抽样对 HEAD（python extract 比对，勿用 awk 简版——曾假报 DIFF）。
- **仓外备份**：`/Users/yoyi/workbench-backups/`（07-14/07-16/07-19-003058 三代，rollback 源齐全）。

## 六、给接棒人的一句话

主线没断：**H2→底1 是对话优先的咽喉**，额度一恢复就续验（§二.1，到卡即停）；视觉治理线 G2 随时可开（§二.3，轻档）。每步收口必回写 CURRENT，catch 必记账，证据说话。
