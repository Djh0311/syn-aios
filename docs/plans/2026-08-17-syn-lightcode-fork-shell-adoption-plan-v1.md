# Syn lightcode fork 壳采纳计划 v1

日期：2026-08-17<br>
里程碑：`Shell adoption（跨 M6+ 载体切换）`<br>
状态：**PLANNED / NOT_ACTIVE / NO_EXECUTION_AUTHORITY**<br>
方向来源：[`decisions/2026-08-17-syn-lightcode-fork-desktop-shell-direction-v1.md`](../../decisions/2026-08-17-syn-lightcode-fork-desktop-shell-direction-v1.md)<br>
上游锚点：`github.com/SDSLeon/lightcode`（Apache-2.0，master，2026-08-16 仍活跃；启动时以当时 exact SHA 冻结为 F0 基线）<br>
拟用 Harness 生命周期：M5 独立验收与 stage-14 收口后，另建真实 shell adoption 阶段；本计划存在不产生施工权限。<br>

## 0. 权威定位与启动边界

本计划把"lightcode 的骨架 + Syn 的功能 + Syn 的皮肤"落成可分期实施的路线，不是产品正本，不改写 M1–M5 冻结合同，不降低 M5/M6 退出条件。

读取顺序不变：当前用户指令 → AGENTS.md → 产品正本/权威登记/架构 → Harness plan → 活动 stage 与唯一 current leaf → authorization → current-state 与源码新鲜验证 → 本计划。

启动前置（2026-08-17 用户确认并行化修订）：F0/F1 在姊妹仓库施工、与 syn 仓库零写面冲突，可与 M5 验收循环**并行立即启动**（用户 2026-08-17 已授权 clone 上游与无人值守推进）；F2 的合同**草案**可提前起草，但 F2 冻结与实施、F3/F4/F5 仍以 M5R07 独立验收完成、stage-14 closeout 为硬门。

与 M6 的排序（2026-08-17 用户确认）：M6 域层（合同、service、repository、投影）不依赖壳，可在 stage-15 建立后与壳采纳并行施工；M6 的产品 UI 与隔离 App 验收载体为新壳，待 F2/F3 就绪后进行。M6 阶段计划验证矩阵中的 "Isolated Tauri" 行在 stage-15 建立时按新壳口径改写。

## 1. 目标与明确不做

### 1.1 最终目标

1. 建立 Syn 自有的 lightcode fork 仓库，满足 Apache-2.0 归属义务，替换 Poracode 品牌；
2. 新壳跑通并接入 Syn 治理核心：角色身份、事实、权限、审计由 Syn Rust 核心作为权威服务持有，壳只消费受控接口；壳线程 ID 不成为身份根；
3. 在新壳中重建既有功能界面（首页 Attention/收件、秘书对话与 brief、项目、知识库、记忆中心、技能、审计账本、设置），消费 Syn typed read model；**布局权威在新壳**：保留 lightcode 的布局、导航结构与按钮排布，旧界面按新壳信息架构适配，不搬旧壳布局；
4. 把现有前端视觉风格提炼为风格基线，套用到 lightcode 的 UI 结构与按钮排布；
5. 多 agent 编排能力（agent 适配、线程、MCP、WSL 桥）与 M5 执行授权链（ExecutionGrant、WorkerReport、独立审查）对接：壳可以驱动 agent，但执行授权与完成判定在 Syn 核心；
6. 每期形成独立 Git 载体、隔离验收证据与 Harness 退场；旧 Tauri 壳的退役边界单独结算。

### 1.2 本计划不做

- 不跳过或降级 M5 独立验收；不改写 M1–M5 冻结合同正文；
- 不接真实账号、真实 provider 计费凭据、真实用户资料；证据只用隔离 app-data、scratch 项目与 fake/白名单动作；
- 首期不纳入 chrome-extension、computer-use、Outlook 插件；移动端（Capacitor iOS/Android）为**后置目标**（2026-08-17 用户确认解除显示边界移动端禁令，改为后置；启动前仍需单独计划与授权）；GitHub 插件**纳入首期**，读面优先，涉及外部写动作（建 PR、评论等）仍按 M8 connector/action 授权边界单独结算；
- 不物理删除旧 Tauri 壳与其数据；退役另行结算；
- 不 push、merge 上游、部署、发布；fork 与上游的同步策略按成本另定。

## 2. 仓库形态（F0 内确认）

建议 fork 作为独立姊妹仓库（如 `/home/synadmin/workspace/syn-shell`），保留独立 git 历史以便追踪上游；不并入 syn 主仓避免 monorepo 膨胀。syn 主仓通过文档与接口合同引用它。最终形态在 F0 用真实构建事实确认。

## 3. 分期

- **F0 源码入库与可构建基线**：fork/vendor 上游 exact SHA；在本机（Windows 构建 + WSL 开发链路）复现构建与运行；盘点 main/renderer/server/supervisor 模块与数据层（better-sqlite3/drizzle）、导航槽位（侧栏、面板、快捷入口，供 F3 槽位替换）与 remote/relay 安全面（对照架构正本 §5.10 的 pairing/E2E/allowlist 约束）；产出模块地图与保留/裁剪清单；完成 LICENSE/NOTICE 与品牌替换清单。
- **F1 品牌与风格基线**：从现有 Tauri 前端提炼视觉风格 token（配色、质感、排版、间距）；替换 Poracode 品牌资产；把风格套用到壳的全局主题层，不逐页重画。
- **F2 Syn 核心桥**：定义壳 ↔ Syn 核心的受控接口合同（RoleSession/身份、typed read model、动作提交、receipt/audit 回传）；Syn Rust 核心以权威服务进程形态供壳消费；壳线程 ↔ RoleSession 的映射只作外部引用；隔离环境证明"壳销毁重建后角色脉络由核心恢复"。
- **F3 功能界面适配重建**：按 F2 合同在新壳重建八类既有功能界面；以新壳布局为基准做适配设计，采用**槽位替换**——基于 F0 的导航槽位盘点（侧栏、面板、快捷入口），Syn 入口占用或替换其中不适用的按钮/槽位，不加第二套导航体系，面板、右侧详情、quick composer 等交互模式原样沿用；parity 清单只对功能行为对齐旧壳，不对旧布局；前端不拥有协调真值。
- **F4 编排与执行治理对接**：壳的多 agent 线程/派发接入 M5 ExecutionGrant、WorkerReport 与独立审查链；壳内 agent 完成自报不构成 Syn 验收；WSL 桥与 hook forwarder 按 Syn receipt 规范回传。
- **F5 隔离验收与切换边界**：隔离 app-data 全链路验收（含 SIGKILL/重启恢复）；确定新壳成为日常主壳的条件与旧 Tauri 壳的兼容/退役边界；形成 M6 及后续阶段在新壳上的载体交接。

各期独立进 done，任一期完成不冒充整计划完成。

并行泳道（2026-08-17 修订）：

- A 线（syn 仓库）：M5 验收循环 → stage-14 closeout → grok 转 stage-15 M6 域层；syn 仓库源码写面同时只允许一个施工者。
- B 线（syn-shell 姊妹仓库）：F0 → F1 → F2 实施 → F3 → F5；与 A 线并行，F2 冻结/实施起依赖 M5 closeout。
- C 线（只读 + 新增文档）：F2 合同草案、旧前端风格 token 提取、M6 域层任务包起草；随时可做，不碰源码。
- 汇合点：M6 产品 UI 需要 F3 就绪的新壳 + M6 域层完成。

## 4. 风险与停点

- 上游活跃演进与本地改造冲突：F0 冻结 exact SHA，同步策略后置；
- Electron/TS 与 Rust 核心的进程间接口是新增攻击面与故障面：F2 合同必须含 fail-closed 与恢复语义；
- 规模风险（~2500 文件）：只保证保留清单内模块可维护，裁剪模块记录而非静默删除；
- 出现需要真实凭据、外部网络业务动作或上游授权变化时立即停点回用户。
