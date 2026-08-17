# 决定：以 lightcode（Poracode）fork 作为 Syn 桌面壳方向 v1

日期：2026-08-17
状态：**当前有效。方向决定，不含施工授权。**

## 1. 用户本轮确认

用户在 2026-08-16/17 对话中明确确认：

1. 用户实际使用 Poracode（数小时真实使用）后认为其能力正是 Syn 底层需要的一部分：多编程智能体统一编排、跨设备消息通道、Windows/WSL 桥接、MCP 接入、工具插件与线程生命周期管理；
2. 角色与事实（身份、正式事实、记忆、权限、审计）仍由 Syn 自己设计实现，不外包给壳；
3. 在"fork 为新壳"与"只摘编排层"两条路线中选择**路线 1：把 lightcode 源码拿过来，做成 Syn 自己的形状**；
4. 现有 Tauri/React 前端用户本就不满意，不再投入大改；但既有**功能界面**（首页 Attention/收件、秘书对话与 brief、项目、知识库、记忆中心、技能、审计账本、设置等）要在新壳中重建保留；
5. 现有前端的**视觉风格**继承过来，套到 lightcode 的 UI 结构与按钮排布上。即"lightcode 的骨架 + Syn 的功能 + Syn 的皮肤"；
6. **布局权威在新壳**（2026-08-17 补充确认）：新壳（lightcode）的布局一定要保留；旧 Syn 功能界面做适配，融入新壳的布局与信息架构，不把旧壳布局搬进新壳。
7. **2026-08-17 后续确认**：壳保持 lightcode 的 Electron/TS 栈，不做 Tauri 回迁（渲染层同为 React，差异全在 Node 生态的主进程编排层，回迁等于数月级重写并失去上游跟进）；M6 域层先行施工、M6 产品 UI 落在新壳；移动端从"禁止"改为后置目标；GitHub 插件纳入壳采纳首期（读面优先，外部写动作仍按 connector 治理门）；导航采用"槽位替换"——Syn 入口占用或替换新壳不适用的按钮/槽位，不加第二套导航体系。

## 2. 已核实的事实基础

- Poracode 桌面应用的上游源码仓库为 `github.com/SDSLeon/lightcode`，公开、活跃（最近 push 2026-08-16），主语言 TypeScript。
- 授权为 **Apache-2.0**：允许修改、改名、闭源商用；义务是保留 LICENSE/NOTICE 归属声明并标注修改。"Poracode" 品牌与商标不在授权内，须整体替换。
- 代码规模约 2500 个源文件：`src/main`（Electron 主进程：threads、agent 适配、MCP、SSH、remote、schedules、computer-use、SQLite/drizzle）、`src/renderer`（React UI）、`src/server`、`src/supervisor`、`src/mobile` + Capacitor Android/iOS 工程、chrome-extension、`packages/codex-protocol` 等。
- 本地已装 Poracode 1.6.2 的运行事实：`agent-plugins/`（claude、codex、cursor、copilot、gemini、grok、opencode、qoder、commandcode 的 lifecycle hook forwarder）、`wsl-helpers/`（bridge、cursor-sdk-worker、mcp-filter/probe）。

## 3. 本次决定

### 3.1 壳与核心的分工

- **lightcode fork 成为 Syn 的桌面（及未来移动）壳载体**，提供多 agent 编排、传输通道、远程接入与 UI 骨架；
- **Syn 治理核心保持原生**：M1–M5 已建立的 Rust 核心（RoleSession、事实所有权、秘书、Attention、执行授权链、ProjectSummary）作为权威服务保留，壳通过受控接口消费；
- 依产品正本第 12 节与[原生治理核心决定](2026-08-16-syn-native-governance-core-and-governed-self-upgrade-direction-v1.md)：壳是可替换的 Runtime/入口层。**壳的线程记录不得成为角色身份根**；替换、停用或销毁壳后，角色身份、正式事实、长期记忆与用户控制权必须仍由 Syn 核心恢复。壳线程 ID 只能作为外部引用记入 receipt。

### 3.2 前端处置

- 现有 Tauri/React 前端不再作为长期投入方向；
- 既有功能界面在新壳中按 Syn 核心的 typed read model 重建，不复制前端为事实来源；
- **重建以新壳布局为基准**：lightcode 的整体布局、导航结构与按钮排布保留；旧 Syn 功能界面适配进该布局，功能行为对齐旧壳（parity 只对行为，不对旧布局）；
- 现有前端的视觉风格（配色、质感、排版）提炼为风格基线，套用到 lightcode 的 UI 结构与按钮排布上；
- 旧壳的物理退役时点与兼容边界由后续阶段计划结算，不因本决定立即删除。

### 3.3 授权合规

- fork 中保留上游 LICENSE 与归属声明，新增 NOTICE 标注 Syn 的修改；
- 移除/替换 Poracode 品牌资产；
- 上游为活跃项目，fork 后是否持续跟进上游由实施阶段按成本决定，不预设义务。

## 4. 对当前路线的影响

- M5R07 独立验收与 stage-14 收口照原计划进行，不因本决定跳过或降级；M5 核心正是将来壳要消费的权威服务；
- M6 及以后各阶段的产品 UI 载体预期转向新壳，具体排序在实施计划中结算；
- 实施依 [`docs/plans/2026-08-17-syn-lightcode-fork-shell-adoption-plan-v1.md`](../docs/plans/2026-08-17-syn-lightcode-fork-shell-adoption-plan-v1.md) 进入真实 Harness 阶段后开始。

## 5. 不代表什么

本决定不表示：

- 已经创建 fork 仓库、已经开始改造或已有任何施工授权；
- M5 独立验收可以跳过，或 M5 候选序列被放弃；
- 已经批准接真实账号、真实 provider、外部网络业务动作、push 或发布；
- lightcode 的移动端、chrome-extension、computer-use 等全部能力都已纳入 Syn 首期范围；
- Poracode/lightcode 的可靠性、安全性与长期维护成本已通过 Syn 验收。

实现仍需按当前用户指令、对应阶段、唯一任务包、验证和退场规则逐步进入。
