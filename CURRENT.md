# Current Authority（精简版 · 2026-06-20）

> **本文是唯一「每次工作完必更」的活正本**（四块：能用 / 在做 / 下一步 / 锁着）。per-task 状态以本文为准；`docs/plans/2026-06-18-master-roadmap-phased-v1.md` 只在阶段切换时动。规则见 `AGENTS.md`。完整历史进 `archive/` + git。

## 一、现在真能用什么（验过的）

- **甲·中转 relay（A 线 ③b 已收口）**：GUI 在 Syn 里**真发 codex 成功**（里程碑 `9b7360a`「第一次真启动 Codex 干活」）；绑会话 Enter 直发、`codex exec` 沙箱限项目 `--sandbox workspace-write` + `--add-dir` + 拒审批绕过、在场 env 闸、stop 真杀、回执。后端 `manual_relay.rs`。命令行中转亦验成。
- **⭐ 智能体页 UI = 整体完成**（本会话收口、真机验过、已入库）：codex 布局；信息呈现收口（envelope/receipt 进开发者详情、失败转人话）；会话流拆固定虚拟化 + 消抖 + 按轮分组 + 过程折叠；会话列表过滤 subagent（604→136）+ 无项目统一（`Documents/Codex`→「直接聊天」）+ 标题截断 + 拖拽改宽 + 侧栏收窄 + 头部按 scope 条件化。
- **运行工作流画布 = 画布优先重画 P1–P4 完成**（本次收口、typecheck 0 + offline 15+r4 亲验）：任务包页从「指标仪表盘」改成「工作流执行画布」——标题区 + 状态带 + 左只读 React Flow 节点画布（状态卡牌 + 连线状态着色 + 顶部阶段进度段带 + minimap）+ 右栏当前节点详情 + 底部模式/操作；统计大盘降级收起。复用 `projectCanvas.ts`（`ProjectWorkflowReactFlowCanvas` + `deriveProjectWorkflowCanvasReadModel`），纯前端零后端、空数据不撒谎、点节点不执行。落点 `RunningWorkflowsView.tsx` + `styles.css`。**唯一没验**：真机对图（需起 Tauri）。
- **前端其余（B 线已收口）**：拆瘦 App 1104→695 / 记忆页 1340→676 / 工作流侧栏 953→340；全 views 内部字段渐进披露（11 面板）。

## 二、在做什么

- **运行工作流画布·真机对图打磨**（B 线最后尾巴）：代码层 P1–P4 已完成并亲验（typecheck+offline）；剩**起 Tauri 对着 image 1 微调视觉**（密度/间距/连线着色/段带）。这就是「打磨 UI」那一步，纯视觉、不碰数据/后端。

## 三、下一步

1. **运行工作流画布·真机对图打磨**：起 Tauri 看画布 vs image 1，差距在哪调哪（纯视觉微调）。← UI 打磨在这一步。
2. relay GUI 体验最后打磨（A 线，已基本随对话模块重做一起收）。
3. 再往后才是 **中间·半自动**（A 线阶段 1，依赖甲成熟 + 底座）。

## 四、锁着的 / 没接（要碰先按 `AGENTS.md` 高危档走）

- **真跑 codex 进真实项目**（非 temp）：用户在场明确授权那一下，不可省。
- **乙·自动连环 / 多项目接力**：终局，没开（风险到这才真大）。
- **底座**：R3 真库切换、统一记忆层、真攒记忆 —— deferred，各需另窗另批。

---

*阶梯：甲·手动中转（已收口）→ 中间·半自动（下下步）→ 乙·自动连环（终局）。**本文每次 commit 必回写**（AGENTS.md §五）。*
