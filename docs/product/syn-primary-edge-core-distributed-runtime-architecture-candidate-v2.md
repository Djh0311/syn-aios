# Syn Primary Core + Edge Core 分布式运行架构候选 v2

日期：2026-08-13

状态：**CANDIDATE / DRAFT / NOT_CURRENT_ARCHITECTURE / NOT_EXECUTION_AUTHORITY**

性质：目标架构候选，只用于后续讨论、合同设计和验收门冻结；不是当前架构正本，不证明多节点、Headless Core、5600X 服务或 Primary 切换已经存在，也不授权连接设备、迁移源码、配置系统、迁数据、Git 或发布。

来源草案：`/Users/yoyi/Library/Containers/com.tencent.xinWeChat/Data/Documents/xwechat_files/wxid_o97veuui2g6m22_75c7/temp/RWTemp/2026-08/d7632f22f514452b259d2a63ea8504ee/2026-08-11-syn-primary-edge-core-distributed-runtime-architecture-draft-v1(1).md`

来源 SHA-256：`9edb90a595eaca481703db75a6b04b3a00aab026e7215d6686fc4c3c26505d7c`

本候选的唯一活动登记入口是 [`candidate-register-v1.md`](candidate-register-v1.md) 中的 `CAND-013`。设备与开发迁移步骤单独放在 [`2026-08-13-syn-5600x-wsl-development-environment-migration-plan-v1.md`](../plans/2026-08-13-syn-5600x-wsl-development-environment-migration-plan-v1.md)，不再与目标架构混写。

## 1. 一句话结论

候选目标是 **一个 Primary Core + 多个 Edge Core**：近期先把后续开发源码和环境安全迁到 5600X 的 WSL；只有在独立 Headless Core、远程 transport、身份、单写者和故障恢复完成实现与验收后，5600X 才可能成为 Interim Primary；未来再用同一套显式 epoch 切换合同迁往 EPYC 独立 VM。

最重要的边界是：

> 开发环境迁到 WSL，不等于 Headless Core 已实现；Headless 候选能启动，不等于 5600X 已成为正式 Primary；配置或路由指向新节点，也不等于数据与 authority epoch 已安全切换。

## 2. 当前事实、用户陈述与目标态分开

### 2.1 当前已核实事实

- 当前产品实现仍以 Mac 上的 Tauri-linked Rust crate 和桌面 UI 为主要形态。
- 产品 crate 仍直接依赖 `tauri-build`、`tauri` 和 `tauri-plugin-log`；仓内没有独立 `syn-core-lib` 或可单独部署的 Headless Primary service。
- 当前没有远程 Node Registry、设备配对、AuthorityLease、跨节点 reconciliation 或 Primary epoch 切换实现证据。
- M1–M4 已完成各自具名范围；M4R01–M4R07 已归档，`stage-07` 已关闭。
- M5–M10 仍为 `PLANNED / NOT_ACTIVE`；本候选不重开 M1–M4，也不自动激活 M5–M10。
- M4R07 v2 只证明本机隔离普通产品的后端/产品链范围；第 8 次 UI、Computer Use、PNG 和 attestation 为 `NOT_EXECUTED / NOT_APPLICABLE`，不提供视觉、远端、Headless 或发布结论。

当前事实入口是：

- [`current-state.md`](../current-state.md)
- [`authority-register-v1.md`](authority-register-v1.md)
- [`workbench-system-architecture-v1.md`](../workbench-system-architecture-v1.md)
- [`docs/harness/plan.md`](../harness/plan.md)及其当前 stage、唯一 leaf 和 [`authorization.json`](../harness/authorization.json)

原草案引用的 `docs/harness/CURRENT.md` 和 `docs/harness/AUTHORITY.md` 不是当前入口；同名文件只存在于历史目录，不能恢复成现行权威。

### 2.2 仍待现场验证的用户陈述与规划输入

- Mac 与 5600X 两端已下载 Tailscale；尚未验证安装、登录、tailnet、设备身份、ACL 或端口可达。
- 5600X 将按规划安装或配置 WSL2；尚未验证 Windows、虚拟化、发行版、磁盘、GPU、systemd 或网络模式。
- 草案中的 CPU、内存、磁盘职责、常开条件、备份链和 Docker 布局均属于现场预检输入，不是当前资产验收结果。

### 2.3 候选目标态

- 5600X：Headless Primary 候选 + 5600X Edge 候选；是否承载正式 Primary 要等独立验收与显式切换。
- Mac：全功能 Edge 候选 + 主 Desktop UI；在 Interim 阶段连接 5600X Primary，在未来连接 EPYC Primary。
- 手机：UI-only 候选，不默认拥有 Edge 能力、租约或高风险批准权。
- EPYC：未来唯一 Primary 的候选承载节点；未采购、部署和验收前不能写成当前资产。

## 3. 修正后的演进时间线

### T0 — 当前 Mac 内嵌形态

- 当前 Core 能力仍与 Mac Tauri 进程、AppState、窗口和本地存储装配存在绑定。
- 当前 Mac UI 调用同机内嵌实现。
- 这是真实现起点，不是目标分布式形态。

### T1 — 只迁开发环境和已提交源码

- 把后续开发所需的完整 Git 字节迁到 5600X WSL，并在 WSL 重建依赖与构建产物。
- Mac 原仓库保持不动，作为已知可回退基线。
- 此时可以通过 Windows Remote-WSL 或 Mac SSH/Remote-SSH 参与 WSL 开发，但普通 SSH 不是 Edge 协议。
- `cargo build` 成功只证明对应 crate 在 WSL 可构建，不证明 Headless Primary 存在。

### T2 — 独立 Headless Core 候选

- 从 Tauri command 和窗口生命周期中抽出平台无关 Core/application service。
- 建立不依赖 Tauri、窗口或 Mac UI 的独立后端进程。
- 定义并验证 in-process transport 与远程 transport、认证、幂等、超时、Stop、重启恢复和错误语义。
- 此阶段仍是候选运行，不接管正式 Primary 权威。

### T3 — 5600X Interim Primary 显式切换

- 只有 T2 验收、备份恢复、旧写者 fencing、单调 authority epoch、回切手册和稳定观察通过后，才允许单独批准切换。
- Mac UI 在这一过渡期连接 5600X Interim Primary；Mac 同时运行全功能 Edge 候选。
- 旧 Mac 内嵌 Primary 必须降级或停写，不能形成两个写 Primary。

### T4 — EPYC Primary 显式切换

- EPYC 独立 VM 先做只读 restore/replay/shadow 和故障演练。
- 短时停写、冻结旧 epoch、恢复新节点并签发新 epoch 后，EPYC 才成为唯一 Primary。
- 5600X 降为 Edge；Mac 与 5600X UI 均连接 EPYC Primary。

## 4. Mac UI 的三态连接

1. **当前态：** Mac UI 调用同机内嵌实现。
2. **Interim 态：** Headless 与切换验收后，Mac UI 连接 5600X Interim Primary；Mac Edge 负责本地工作与受控缓存。
3. **最终候选态：** EPYC 切换后，Mac UI 与 5600X UI 连接 EPYC Primary。

打开 UI、设备登录、成为主要交互端和获得批准能力是四件不同的事。任何 UI 连接成功都不能自动转移事实写入权或批准权。

## 5. 候选核心角色

### 5.1 Primary Core

一个 authority epoch 内唯一的全局协调权威，候选职责包括：

- 用户、设备、Identity、Scope 与 Policy；
- 全局对象注册、稳定 ID、作用域 owner 与单写者约束；
- AuthorityLease、ExecutionGrant 和 ApprovalCapabilityGrant；
- Event、Audit、Outbox、幂等与 reconciliation；
- 节点注册、能力目录、健康状态和受控调度；
- 正式事实、正式记忆和正式 Skill 的治理入口；
- Primary 迁移、authority epoch 切换、恢复和 fencing。

Primary 是逻辑角色，不永久绑定 Mac、5600X、EPYC、Tauri 或某个绝对路径。

### 5.2 Edge Core

受信设备上的本地运行时，可以拥有接近完整的产品能力，但只能在明确 scope、Lease、Grant、风险和预算内执行。候选能力包括：

- 本地 UI、会话、项目工作区、Git、Harness 和工具；
- 本地缓存、索引、artifact、日志和待结算事件；
- Agent、浏览器、Docker、GPU 或本地模型等节点能力；
- 有效租约内的有限离线工作。

Edge 默认不能自签、自扩或自续权限，不能在旧 epoch、租约过期、策略漂移或 owner 冲突时继续正式写入。

### 5.3 UI Client

UI 可以与 Core 同机，也可以连接远端 Primary。UI 在线只证明客户端会话，不证明设备是 Edge、主要交互端、批准端或正式写者。

## 6. 不可退让的分布式边界

- 任一时刻最多一个有效 Primary 和一个有效 authority epoch。
- 同一 `scope + object/fact kind` 在同一时间只有一个正式写 owner。
- 不同步活动 SQLite 文件，不把知识目录复制当分布式数据库，不用最后写入者覆盖权限或业务冲突。
- 非 owner 只能产生 Draft、Proposal、Observation、Candidate、Branch/Diff 或 Pending Reconciliation。
- 代码冲突进入 Git/diff/review；权限、记忆和业务事实冲突进入显式 reconciliation。
- Edge 过期后只允许只读、停止、诊断、草稿和回执上传；新增正式写入 fail closed。
- ERP、数据库、容器和长期服务由自身 OS/VM/容器生命周期负责，不以 Syn 在线作为生存前提。
- 网络可达、设备可信、技术 capability、Agent 输出和全局主管意见都不等于用户授权。

## 7. 最小候选合同

后续正式合同至少需要冻结：

```text
CoreInstance
CoreRole(primary | edge)
AuthorityEpoch

DeviceIdentity
DeviceTrustState
AuthenticatedSession
PrimaryInteractionLease
ApprovalCapabilityGrant

ScopeOwnership
AuthorityLease
NodeIdentity
NodeCapability
ProjectNodeBinding
EdgeHealth

ExecutionGrant
ExecutionAttempt
ExecutionReceipt

LocalEventSpoolEntry
ReplicationCursor
ReconciliationReceipt
```

这些对象必须复用已有 ScopeRef、ProjectId、ObjectRef、ExecutionChannel、Command、Event、Audit 和 Outbox 方向，不能另建绕过控制核心的权限系统。

## 8. Headless 抽离的完成门

只有同时满足以下条件，才可使用 `HEADLESS_CANDIDATE_READY`：

1. Core service 无需 Tauri、窗口系统或 Mac UI 即可独立启动。
2. Tauri 只是一个 adapter/UI host，不再拥有业务核心生命周期。
3. 本地与远程 transport 使用同一 application contract，并有认证、超时、幂等和稳定错误语义。
4. 重启、重复请求、乱序、断线、恢复和旧实例重连测试通过。
5. 持久化 owner、迁移、备份与恢复边界已明确。
6. 结论由当前字节和原始验证绑定；单次构建或临时服务响应不满足此门。

该标记仍不是 `PRIMARY_SWITCH_ACCEPTED`。

## 9. Primary 切换的完成门

只有同时满足以下条件，才可使用 `PRIMARY_SWITCH_ACCEPTED`：

1. 候选 Primary 的备份与恢复演练通过。
2. 切换前已冻结写入边界，旧 Primary 可被确定性 fencing。
3. authority epoch 单调递增，旧 Lease/Grant/写入会被拒绝。
4. 数据总量、关键记录、事件序列、审计和 outbox 核对一致。
5. Mac Edge/UI 已连接新 Primary 并经过约定观察窗口。
6. 回切手册已经演练，且不会通过“直接重启旧节点”制造双主。
7. 有绑定本次节点、字节、epoch、时间窗和结果的真实切换 receipt。

## 10. 与 M1–M10 的关系

- M1–M4 是已完成的历史具名范围，不因本候选出现而重开、倒填或改写。
- 新的 CoreRole、DeviceIdentity、Lease、Grant、epoch 和 reconciliation 先作为 D0 候选合同进入后续独立任务包。
- 若未来内容与 M5–M10 有依赖，只能在对应 stage 被新的用户指令、唯一 leaf 和授权显式激活后实施。
- 开发环境迁移可以先完成，但它不激活任何产品阶段。

## 11. 仍待拍板和验证

- embedded/remote transport 的具体协议与双向认证方式；
- Headless Primary 的数据库继续使用 SQLite，还是在真实并发需求成立后选择 PostgreSQL；
- 设备密钥、密码慢哈希和邮箱验证 provider；
- AuthorityLease 默认时长、离线风险和续租策略；
- 本地知识正文、索引、日志、artifact 和 transcript 的保留与上传策略；
- 5600X 的 WSL 网络入口、磁盘、容量、GPU、常驻、备份和恢复事实；
- EPYC 的虚拟化、网络、备份、RPO/RTO 和灾难恢复规格；
- 手机长期保持 UI-only，还是未来成为受限 Edge。

上述问题未冻结前，不影响先做可逆的开发环境迁移；但会阻止对应的 Headless、Edge 或 Primary 正式切换。

## 12. 候选转正门

本候选只有在以下事项完成后，才可另开 D0b 讨论是否转成现行架构：

- 当前实现差距、迁移成本和安全边界已有新鲜审查；
- 单写者、epoch、Lease/Grant、身份、备份恢复和回退合同已经冻结；
- 与现行产品正本、架构正本和 M1–M10 的冲突已逐项裁决；
- 用户明确批准转正；
- 权威登记、正式决定和现行架构正本在新的授权 stage 内同步更新。

在此之前，本文始终只是 `CAND-013` 的详细材料。
