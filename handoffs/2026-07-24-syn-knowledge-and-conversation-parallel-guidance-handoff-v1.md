# Syn 知识库与对话底座双线指导交接 v1

- 日期：2026-07-24
- 状态：**HISTORICAL RUNTIME HANDOFF / 07-25 界面路线修订见 CURRENT 与决策 v2**
- 交接对象：下一位总指导
- 当前仓库：`/Users/yoyi/workspace/product-line`
- 当前执行拓扑：知识库线与对话底座线同时 active；共享承重文件和真实运行验收串行

> 本文保留 07-24 runtime 与双线安全边界。07-25 用户已把知识库界面改为 Obsidian 核心桌面高保真单壳路线；界面目标、执行顺序与授权以 `CURRENT.md` 和修订后的决策/计划 v2 为准。I5 技术证据可复用，但不得按本文旧顺序自动续跑。

## 1. 用户当前真正要得到的结果

用户已明确停止“把 Obsidian 完整界面嵌入 Syn”以及 Electron 迁移路线，当前目标是：

1. 在 Syn 内实现可真实日用的原生知识工作区；
2. 保留 Markdown、Frontmatter、普通附件和 JSON Canvas 等开放格式；
3. 主管通过 Syn 统一 MCP capability plane 只读搜索、读取、打开和引用知识；
4. 对话底座与知识库方向并行推进，但不得并发写共享承重文件或并发操作同一真实 store；
5. 最终结论必须来自真实 Syn App 验收，离线测试不能替代真实可用性。

当前路线正本：

- `decisions/2026-07-23-l3-syn-native-knowledge-workspace-route-v2.md`
- `decisions/2026-07-23-knowledge-and-conversation-parallel-workstreams-v1.md`
- `decisions/2026-07-23-supervisor-read-only-exact-five-capability-surface-v1.md`

## 2. 接手阅读顺序

按以下顺序恢复，不要从旧计划或单份 evidence 反推当前权限：

1. `AGENTS.md`
2. `CURRENT.md`
3. `AUTHORITY.md`
4. 本交接
5. `tasks/2026-07-24-l3-syn-r4-isolated-runtime-profile-preflight-package-v1.md`
6. `evidence/2026-07-24-l3-syn-r4-isolated-runtime-profile-preflight-verification-v1.md`
7. `tasks/2026-07-23-shared-conversation-transport-real-app-reacceptance-package-v2.md`
8. `evidence/2026-07-23-shared-conversation-transport-parallel-restart-audit-v1.md`

历史 Obsidian 真嵌入决策、伴随窗口方案、Electron 迁移和旧 resident/private-home 对话路线只能作历史参考，不得重新派发。

## 3. 已拍板且不可自行改写的合同

### 3.1 知识产品路线

- Syn 原生知识工作区仍是主产品；主界面高保真对齐 Obsidian 核心桌面结构与交互，Obsidian 本体只保留开放格式兼容和可选外部打开。
- 不嵌入 Obsidian，不复制其商标/受限资产，也不复刻插件生态、主题兼容、私有 API、Sync、Publish 或移动端。
- 前端不得获得任意文件路径；文件、Canvas、附件和搜索走 typed host command。
- 写入必须经过固定 vault、冲突检测、用户确认和既有 audit；主管 MCP 不获得知识写能力。

### 3.2 主管 MCP 工具面

`supervisor-read-only + project_supervisor` 的工具集合必须精确为：

1. `submit_proposal`
2. `knowledge_search`
3. `knowledge_read`
4. `knowledge_open`
5. `knowledge_cite`

五项只有在可信 binding 为 Active，且 project/root/workflow/run/thread 全部一致时才能通过同一服务端 gate 被 `tools/list` 和 `tools/call` 使用。历史“只见 `submit_proposal`”已失效。

### 3.3 并行边界

- 两线使用独立任务、evidence 和验收结论。
- 文档、只读审计和不重叠实现可以并行。
- `commands.rs`、`manual_relay.rs`、`manual_relay/conversation_transport.rs`、`exec_process_registry.rs` 等共享承重文件同时只允许一条线写。
- 同一真实 store 的 Syn/Codex/MCP 验收不得并发。
- 一条线的离线绿、真实绿或失败都不能替另一条线结算。

## 4. 知识库线当前状态

### 4.1 已完成但只属离线证据

- N0–N5 已实现并完成离线验证：固定 vault、Markdown 工作区、文件组织、双链/反链、搜索、图谱、JSON Canvas、附件、刷新与恢复。
- N6 的 `knowledge_search/read/open/cite` 只读 capability 和精确五项 registry 已离线收口。
- `knowledge_open` host-owned relay、同 intent UI ack、secret sink、spawn 前 raw 闭锁、失败清理与 recovery route 已完成离线返工并获指导验收。
- 以上均不代表真实 App 十二项通过。

主要证据：

- `evidence/2026-07-23-l3-syn-native-knowledge-workspace-offline-verification-v2.md`
- `evidence/2026-07-23-l3-knowledge-open-relay-secret-sink-closure-offline-verification-v1.md`
- `evidence/2026-07-23-l3-knowledge-open-host-owned-relay-r3-guidance-acceptance-v1.md`

### 4.2 为什么需要 isolated runtime profile

第一次 R4 尝试在 Gate 0 前发现 Syn 首页自动呈现既有非验收 store 面，执行线立即退出，没有保存泄漏内容。指导线确认仅改 app-data 不够，隔离还必须覆盖 index/tasks、workflow state、Codex DB、vault/recovery/Canvas 和 logs。

当前 I0–I4 已实现 debug-only、fail-closed 的 isolated runtime profile，且离线门已通过。真实 UI 仍未验收。

### 4.3 已确认的 I5 根因

2026-07-24 的单次 isolated Syn build 成功，但进程在 UI target discovery 前 exit 0。执行线最初把它写成 `normal_exit`；指导线已在同日独立核到确定性原因：

1. `scripts/run-r4-isolated-app-preflight.mjs` 在 Syn 启动前，把 `ui-inspection.json` 写入 isolated root；
2. `acceptance_runtime_profile.rs::validate_root_layout` 明确要求 root 只能包含 `profile.json`、`fixture`、`workflow-state`、`app-data`、`codex-db`、`logs`；
3. 多出的 `ui-inspection.json` 固定触发 `acceptance_runtime_profile_reused`；
4. `index_host_app_entrypoints.rs::run()` 在 profile/AppState 初始化失败时只打印错误并 `return`，所以进程仍以 0 退出；
5. launcher 把任意 exit 0 都映射成 `normal_exit`，从而掩盖了启动校验失败。

因此准确停点不是未知的正常退出，而是：

`BLOCKED_I5_PRELAUNCH_FIXTURE_ROOT_CONTRACT_MISMATCH`

当前 `CURRENT.md`、`AUTHORITY.md`、I5 task/evidence 中仍可能暂时保留旧状态；知识开发线已被要求在返工收口时纠正。下一位指导不得再把它归因为 macOS 锁屏或未知 UI 退出。

### 4.4 当前正在执行的任务

- Codex 任务 ID：`019f8b76-425e-71e3-ac2a-4be78b0cf51b`
- 模型：`gpt-5.6-terra`
- reasoning：`ultra`
- 当前状态：执行中

已授权范围：

1. 增加跨语言红合同，证明 launcher 的 prelaunch root entries 与 Rust allowlist 完全一致；
2. 启动前不创建额外 root sidecar；优先将 UI observation 路径放在既有 `logs/` 下，并只由外部 UI 观察器在 App 完成 profile 初始化、发现目标后创建；
3. profile/AppState 早退必须被识别为 startup failure，未经 UI 完成证据的 exit 0 不得再叫 `normal_exit`；
4. 不落原始 stderr/stdout、命令、环境或短期秘密；
5. 跑聚焦 Rust 测试、launcher syntax、typecheck、`cargo check --lib`、目标 rustfmt、`git diff --check`，并确认 staged 为空；
6. 仅在离线门全部通过后，执行一次新的 Home-only isolated smoke；
7. smoke 只验证进程可发现、Home 只见 synthetic 身份和 receipt containment；失败即停，不重试。

仍然禁止：

- Gate 0、主管首句、Codex CLI/MCP server；
- `tools/list`、任何 MCP 工具调用和十二项验收；
- 真实 store/vault、Obsidian、非测试真实项目；
- relay、binding、capability/profile 权限面或知识写能力扩张；
- stage、commit、push、reset、clean、stash。

## 5. 知识线回报后的指导验收

不得直接接受执行线自报。至少核对：

1. launcher 启动前的 isolated root 直接子项与 Rust allowlist 完全一致；
2. `ui-inspection` 文件不存在于 profile 校验前，且不会让 `logs/` 在校验前非空；
3. profile/AppState 失败不能落成 `normal_exit`；
4. 新测试不是只搜字符串，而是能抓住 Node launcher 与 Rust validator 漂移；
5. 没有通过放宽 root allowlist、关闭 fail-closed 或保存 raw stderr 绕过问题；
6. diff 没碰 relay、binding、capability registry、真实 store 路由或知识写权限；
7. 离线门实数、warnings、shape 和历史 rustfmt 债分开报告；
8. smoke 只发生一次；截图和 receipt 只含 synthetic 信息；
9. 自建 App 已安全退出；没有借 smoke 进入 Gate 0；
10. staged 为空，未 commit/push。

若 smoke 成功，只能写“isolated Home preflight 通过”。它不等于 Gate 0、五工具真实发现、`knowledge_open` 真实打开或十二项验收通过。

## 6. 对话底座线当前状态

### 6.1 已完成

- Agent 页与交办页共享 Conversation Transport 已离线接线。
- supervisor profile 固定 `read-only + 空写根`，忽略用户侧写权限配置。
- 服务端精确 allowlist、可信 turn binding、共享前端状态和分层 receipt 已离线验证。
- binding 建立链已细分为 construct、store prepare、DB-primary persist、JSON projection、activate、transport start、terminate-unconfirmed，并有失败闭锁测试。

### 6.2 真实缺口

2026-07-23 的真实 App 替代性验收未通过：

- 第一条消息只新增 canonical recorded；
- JSON/SQLite 都没有形成 durable supervisor conversation binding；
- 没有 injected、自然回复、卡、chain 或 worker；
- 第二、第三句按停止合同未发送。

七阶段临时 fixture 只能证明失败分类和闭锁，不能解释或证明真实首句已经修复。真实根因仍未知。

### 6.3 当前 HOLD 合同

`tasks/2026-07-23-shared-conversation-transport-real-app-reacceptance-package-v2.md` 已冻结三句真实 App 重验合同，但当前没有启动授权。后续重验必须：

1. 等知识线共享文件写入稳定并经指导验收；
2. 重新冻结源码、binary、store、registry/lock 和五工具面；
3. 使用精确五工具集合；
4. 第一条不全绿就不发第二条，第二条不全绿就不发第三条；
5. 不现场修码、不重发、不补卡、不点卡；
6. 与知识真实 App 验收串行。

不要恢复旧 resident/private-home 主路线。

## 7. 历史债与不能宣称的内容

- aggregate shape 截至 2026-07-24 的记录为 `17 errors / 5 warnings / 5 info`，属于历史结构债，不能写成绿色。
- `cargo check --lib` 截至 2026-07-24 仍有约 598 条项目既有 warnings。
- `codex_db.rs`、`knowledge_vault.rs`、`mcp/storage.rs` 曾存在白名单外格式债；以执行线最新实测为准，不能用 scoped rustfmt 代替全树结论。
- N0–N6 离线通过不等于知识库小阶段完成。
- isolated Home smoke 通过不等于 Gate 0 或十二项通过。
- 五工具静态 allowlist 通过不等于真实 MCP `tools/list` 已发现五项。
- conversation binding fixture 通过不等于真实首句 durable binding 已形成。

## 8. 下一位总指导的第一组动作

> 07-25 修订：以下是 07-24 的历史接手顺序。当前不得从第 1 项自动续跑；先按 `CURRENT.md` 完成 N2R-R0 参考冻结和新的精确实现授权。

1. 先查看 Codex 任务 `019f8b76-425e-71e3-ac2a-4be78b0cf51b` 是否已完成或需要用户介入；不要重复派发同一返工。
2. 收到回报后按第 5 节只读核实代码、测试、receipt、截图和 staged 状态。
3. 若 I5 失败，记录最早可证 blocker，不批准自动第二次 smoke。
4. 若 I5 成功，先向用户报告“isolated Home preflight”结果，再由用户决定是否授权知识线 Gate 0 与十二项。
5. 对话线可继续只读准备，但在共享写面稳定和用户明确授权前不得启动真实重验。
6. 两条真实 App 线一次只能运行一条。

## 9. 本交接生成时的仓库事实

- 工作树已有大量跨任务未提交改动和新增文件，归属复杂；不得清理、覆盖或宽泛格式化。
- 暂存区在指导线 2026-07-24 本轮检查时为空。
- 本交接轮只新增本文，没有修改代码、`CURRENT.md`、`AUTHORITY.md`、任务包、evidence 或 memory。
- 本文生成后应运行 `git diff --check`；不得 stage 或 commit。

## 10. 大白话

知识库的离线功能已经很多，但还没有完成真实 App 验收。现在不是卡在锁屏，也不是 Syn 莫名正常退出，而是验收 launcher 自己提前多写了一个文件，触发了我们设计的 fail-closed 校验；同时 App 把启动失败当成 exit 0，launcher 又把 exit 0 叫作正常退出。开发线正在精确修这两个问题，离线全过后只准再跑一次隔离首页检查。

对话底座的离线结构也已经搭好，但真实 App 第一条消息仍没有形成 durable binding。它的三句重验合同已经写好，当前仍 HOLD，不能因为知识线有进展就自动开跑。
