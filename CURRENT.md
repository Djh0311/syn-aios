# Current Authority（精简版 · 2026-06-19）

> 完整历史进 `archive/2026-06-18-current-full-history-before-slim-v2.md`（+ git）。本文只留四块：现在能用什么 / 在做什么 / 下一步 / 锁着的。规则见 `AGENTS.md`（精简版 v2）。

## 一、现在真能用什么（验过的）

- **甲·中转 relay 后端**：手动把一句话发给 codex 的安全机制做好了——三本分（原话逐字 / 发对项目 / 一次一发）、命根子（真发 codex 限项目沙箱 `--sandbox workspace-write` + `--add-dir` + 拒 `--full-auto`/绕审批）、stop 真杀、回执。代码在 main（`b99f16c`…`e53f32a`）。
- **命令行中转已验成**：`codex exec --sandbox read-only` 把用户原话真发 mariotest（`/Users/yoyi/codex-workflow-mario-test`），codex 真收到真回（session `019ed9f7`、gpt-5.5）。→「话 → codex 真到达」成立。
- **前端**：会话引擎重做（点开大对话不卡，`b56bad8`）；`ProjectsView`/`AgentView`/`types.ts` 已拆瘦。

## 二、在做什么

- **治理流程精简（2026-06-19）**：`AGENTS.md` 已砍成「高危清单 + 两档流程 + 完成必附验证」✅；本 `CURRENT.md` 砍版（本次）。
- **relay GUI 待重做**：现有 GUI direct relay（点 codex 会话 Enter 直发）真机体验差、用户叫停。交接：`handoffs/2026-06-18-codex-relay-jia-zhongzhuan-handoff-for-restart-v1.md`。

## 三、下一步

- relay GUI 在新对话重做 / 修真机体验——见上面交接文档 + 总图 `docs/plans/2026-06-18-master-roadmap-phased-v1.md`。
- **UI 信息架构问题**（现有前端全平铺、没层级、没渐进披露）——方案已出、待落地：`docs/plans/2026-06-20-ui-internal-field-disclosure-sweep-plan-v1.md`（全 views 内部字段渐进披露收口）。智能体聊天框已作样板落地（证据 `docs/evidence/2026-06-20-agent-view-info-surfacing-cleanup-v1.md`）。

## 四、锁着的 / 没接（要碰先按 `AGENTS.md` 高危档走）

- **真跑 codex 进真实项目**（非 temp）：用户在场明确授权那一下，不可省。
- **乙·自动连环 / 多项目接力**：终局，没开（风险到这才真大）。
- R3 真库切换、统一记忆层、Stage L 剩项、K3-B1/B2：deferred，各需另窗另批。

---

*阶梯：甲·手动中转（现在·轻护栏）→ 中间·半自动 → 乙·自动连环（终局·重闸到那时再加）。*
