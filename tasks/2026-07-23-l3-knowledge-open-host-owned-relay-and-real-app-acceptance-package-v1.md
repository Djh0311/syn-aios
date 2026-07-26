# 任务包：L3 `knowledge_open` host-owned relay 与真实 App 验收 v1

- 日期：2026-07-23
- 状态：**R3 GUIDANCE ACCEPTED / R4 在 fresh Gate 0 前安全停止：Syn 首屏自动呈现既有非验收 store 面；未发主管首句、未调用 MCP、未进入十二项**
- 负责人：现有 Codex 开发线（`gpt-5.6-terra`，reasoning=`ultra`）
- 指导/验收：当前总指导对话
- 上游决策：`decisions/2026-07-23-l3-syn-native-knowledge-workspace-route-v2.md`
- 上游计划：`docs/plans/2026-07-23-l3-syn-native-knowledge-workspace-small-stage-plan-v2.md`
- 指导验收：`evidence/2026-07-23-l3-syn-native-knowledge-workspace-guidance-review-v1.md`
- 已验收安全前置：`tasks/2026-07-23-l3-knowledge-open-relay-secret-sink-closure-repair-package-v1.md`

## 0. Kickoff

- 任务：补齐 MCP stdio 子进程到 Syn 主进程原生知识视图的受信任短期 dispatch/ack relay，然后完成 N6 十二项真实 App 验收。
- 负责人：现有 `gpt-5.6-terra / ultra` 开发线。
- 交付物：host relay、前端导航与精确 ack、离线测试、真实 App 截图/日志、更新后的 N6 evidence 与 CURRENT/AUTHORITY。
- 完成标准：`knowledge_open` 只有在目标笔记已由 Syn 原生工作区读入、选中并聚焦后才返回 `opened=true`；失败、超时、错 binding、错 intent、脏草稿拒绝均不得伪造成功；十二项真实 App 场景逐项有证据。

## 1. 已知事实、未知与假设

### 已知

- N0-N5 已经指导线独立离线验收，不重写现有 vault/index/Markdown/图谱/Canvas/附件/恢复。
- MCP server 是同一 desktop binary 的独立 stdio 子进程；当前 `knowledge_open` 只能生成已验证 native-view intent。
- App 的 `knowledge` 路由、`KnowledgeBaseView` 和 `NativeKnowledgeWorkspace` 已有 production wiring。
- 共享主管真实 App 建立链此前曾失败；后续离线修复尚未用新 Gate 0 重验。

### 未知

- 当前工作树启动后，主管首句是否已经能建立并激活可信 binding。
- Tauri 窗口事件、React 导航、目标 Markdown 读取和 MCP tool timeout 的实际时序。
- 原生知识工作区在真实桌面窗口中的布局、键盘、焦点和长内容表现。

### 本包假设

- 只支持当前 macOS/Tauri 桌面宿主；不为未来平台抽象通用 IPC。
- relay 只承载“打开固定 Syn vault 内已验证 Markdown”的短期意图与 ack；不承载正文、绝对路径、vault root、命令、URL 或写动作。
- 如果 fresh Gate 0 仍停在 conversation binding 建立链，本包保留已完成的 relay 离线结果并按停止合同回交，不把旧主管主线顺手扩修。

## 2. 冻结的数据流

实现前先用失败测试锁定以下唯一允许的数据流：

1. Syn 主进程启动一个 host-owned、本机短期 IPC listener；优先使用 Unix domain socket。
2. 启动主管 conversation turn 时，宿主创建一个只存在内存中的短期 relay grant，绑定 `run_id + turn_id + project_id + lease`。
3. relay endpoint/grant 只由宿主写入固定 supervisor MCP server 启动配置；用户、前端和模型的 tool arguments 均不能提供或覆盖。
4. MCP `knowledge_open` 仍只接收精确 `relative_path`；它先走现有固定 vault 校验，再发送有界 intent。
5. 主进程收到 intent 后再次核对：grant 未过期、run/turn/project 精确匹配、conversation binding 为 Active、目标仍是固定 vault 中同一 Markdown。
6. 主进程只向主窗口发一个固定 schema 的 `knowledge-open-intent`，并将窗口切到/唤醒到可显示状态；事件不含 grant、绝对路径、正文或内部 argv。
7. `App` 切换到 `knowledge`，把 `intent_id + relative_path` 交给 `KnowledgeBaseView` 和 `NativeKnowledgeWorkspace`。
8. 原生工作区必须实际完成目标 Markdown 的 typed read、选中状态提交和知识工作区焦点；脏草稿保护可以拒绝切换。
9. 前端通过固定 Tauri command 回传精确 `intent_id + relative_path + outcome`；不能回传任意 route、command 或文件系统目标。
10. 主进程核对 pending intent 后，才向等待中的 MCP 请求返回 ack。
11. 只有精确 ack 成功，`knowledge_open` 才返回 `dispatch_status=opened` 与 `opened=true`；无 listener、过期、错 binding、错 intent、窗口关闭、读取失败、脏草稿拒绝和 timeout 均返回受限失败，不得返回成功态。

## 3. Relay 安全合同

- 不修改 conversation binding payload，不新增 DB/JSON schema，不新增 sidecar，不把路径写进静态全局或工作流 store。
- 允许 host 进程内有界 pending map；每个 grant 最多一个 outstanding intent，完成、拒绝、超时、transport stop 或 App 退出后立即清除。
- 请求/响应各不超过 4 KiB；固定 schema version；拒绝额外字段、重复 intent、空/变体 ID 和超限帧。
- lease 与 UI ack timeout 必须有界；建议 UI ack 8 秒内完成，不能无限阻塞 MCP stdio。
- IPC endpoint、grant、argv、环境和内部错误不得进入 UI、tool result、canonical message、普通日志、截图或 evidence。
- `knowledge_open` 失败不能吞掉主管自然回复，也不能改 binding lifecycle、创建卡、启动 chain/worker 或写知识文件。
- `knowledge_search/read/cite` 行为保持；`knowledge_write/canvas_write/attachment_write` 继续不可见且不可调用。
- 不调用 Obsidian，不用外部 App 代替 Syn 原生视图。

## 4. 精确写入白名单

### 后端

- `prototypes/productized-desktop-shell/src-tauri/src/knowledge_open_relay.rs`（可新增）
- `prototypes/productized-desktop-shell/src-tauri/src/command_registry.rs`（merge-only）
- `prototypes/productized-desktop-shell/src-tauri/src/index_host_app_entrypoints.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`（merge-only）
- `prototypes/productized-desktop-shell/src-tauri/src/manual_relay/conversation_transport.rs`（merge-only）
- `prototypes/productized-desktop-shell/src-tauri/src/mcp/mod.rs`（merge-only）
- `prototypes/productized-desktop-shell/src-tauri/src/mcp/knowledge_capabilities.rs`（merge-only）
- `prototypes/productized-desktop-shell/src-tauri/src/mcp/supervisor_orchestrator.rs`（merge-only）
- `prototypes/productized-desktop-shell/src-tauri/src/mcp/supervisor_conversation_transport_tests.rs`（merge-only）
- 为 relay 新增的一个精确 Rust 测试模块。
- `prototypes/productized-desktop-shell/src-tauri/Cargo.toml` 与 `Cargo.lock`：只有标准库无法生成所需有界 grant 时才允许加入一个最小随机依赖；先复用现有依赖，不得顺手升级。

如 Rust 编译证明 `McpServerConfig` 的可选 relay context 必须机械补齐其他现有 struct literal，可只做 `None`/默认值的等义补齐；开始前列出精确文件，禁止借机改逻辑。

### 前端

- `prototypes/productized-desktop-shell/src/App.tsx`（merge-only）
- `prototypes/productized-desktop-shell/src/components/ActiveWorkbenchView.tsx`
- `prototypes/productized-desktop-shell/src/views/KnowledgeBaseView.tsx`（merge-only）
- `prototypes/productized-desktop-shell/src/views/knowledge/NativeKnowledgeWorkspace.tsx`（merge-only）
- `prototypes/productized-desktop-shell/src/lib/knowledgeOpenRelay.ts`（可新增）
- `prototypes/productized-desktop-shell/src/lib/tauri.ts`（merge-only）
- `prototypes/productized-desktop-shell/tests/knowledge-open-relay.test.tsx`（可新增）
- `prototypes/productized-desktop-shell/scripts/run-offline-interaction-test.mjs`（merge-only）

本包不授权视觉重做；只有真实 App 发现 relay 焦点/状态不可辨认时，才允许在现有知识工作区组件内做最小状态文案，默认不改 `styles.css`。

### 4.1 执行线 2026-07-23 停点（未获授权的最小扩白申请）

本包现有白名单不足以安全完成 R1，故本节**不扩大**白名单，只冻结后续需要指导线明确批准的最小范围：

- `prototypes/productized-desktop-shell/src-tauri/src/exec_process_registry.rs`：仅让带固定 host-owned supervisor MCP marker 的登记条目持久化脱敏摘要和非可逆 command identity hash，并在 reaper 保持 `started_at + PGID + hash` 的精确核对；不得新增 relay sidecar、DB/JSON schema 或可恢复 grant。
- `prototypes/productized-desktop-shell/src-tauri/src/manual_relay.rs`：仅收紧 supervisor relay 的原始 stdout/stderr 捕获、终态清理与 raw-receipt pre-registration 顺序，避免 argv/internal error 落盘或在 managed transport 登记前经 raw poll/stop 回读。

指导线已在 `evidence/2026-07-23-l3-knowledge-open-relay-r1-blocker-guidance-review-v1.md`
确认 blocker，并将扩白冻结到
`tasks/2026-07-23-l3-knowledge-open-relay-secret-sink-closure-repair-package-v1.md`。
返工包现已通过指导线离线验收。本包恢复，但必须先重新冻结共享文件 SHA 并完成 R1-R3 全套离线门；只有 R3 通过后才能按第 5 节进入既有授权的 R4。

### 证据与入口

- `evidence/2026-07-23-l3-knowledge-open-host-owned-relay-offline-verification-v1.md`（历史 R1 安全停点，只读保留）
- `evidence/2026-07-23-l3-knowledge-open-host-owned-relay-offline-verification-v2.md`（新增，恢复后的 R1-R3 离线证据）
- `evidence/2026-07-23-l3-syn-native-knowledge-workspace-real-app-acceptance-v3.md`（新增）
- `evidence/raw/2026-07-23-l3-native-knowledge-real-app/`（新增；只放截图、脱敏日志和 manifest）
- `CURRENT.md`
- `AUTHORITY.md`
- `tasks/2026-07-23-l3-knowledge-open-host-owned-relay-and-real-app-acceptance-package-v1.md`（只更新状态/实际白名单）
- `docs/harness-catch-log.md`（只有真实 catch 时追加）

### 4.2 本恢复轮的实际写面（2026-07-23）

- 代码：`knowledge_open_relay.rs`、`knowledge_open_relay_tests.rs`、`mcp/mod.rs`、`mcp/knowledge_capabilities.rs`、`mcp/supervisor_conversation_transport_tests.rs`、`NativeKnowledgeWorkspace.tsx`；均为本包既有白名单内的最小 relay 合同、失败矩阵或中立失败文案。
- 证据/入口：本任务包、`evidence/2026-07-23-l3-knowledge-open-host-owned-relay-offline-verification-v2.md`、`evidence/2026-07-23-l3-syn-native-knowledge-workspace-real-app-acceptance-v3.md`、`CURRENT.md`、`AUTHORITY.md`。
- 未写入真实 App 原始证据目录、vault、store、binding/DB/JSON schema、Cargo、能力/权限 allowlist 或任何知识写能力。
- §4.3 一次性格式返工：仅 `index_host_app_entrypoints.rs` 的 2 个 block（4 个表达式级换行）与 `lib.rs` 的 6 个 block 写入；冻结前 SHA、反向重建 SHA 与精确 pre/post diff 均匹配，完整目标 rustfmt 和第 7 节离线门已复跑通过。shape 仍单列为历史聚合债，R4 保持 HOLD 等待指导线验收。

### 4.3 指导线一次性格式返工授权（2026-07-23）

指导线已只读复现完整目标 `rustfmt --check`，并与 `HEAD` 中同两份文件的
formatter 输出对照。结论：

- 这些差异在 `HEAD` 版本中已原样存在，是历史格式债，不是本恢复轮 relay 新增；
- 执行线证据中的“`3 + 6` 处”不是精确的 unified diff 计数：
  `index_host_app_entrypoints.rs` 实际为 2 个 rustfmt diff block，其中第二个
  block 含 3 个相邻 `if let` 表达式；按表达式级机械换行计为 4 处；
  `lib.rs` 为 6 个 rustfmt diff block；
- 当前文件冻结 SHA-256：
  - `index_host_app_entrypoints.rs`：
    `011fe5ad6b440d340de50e512e5c99b1f17d5da11e74de048fc61b8e9d94e7d0`
  - `lib.rs`：
    `f74794452496f994220c290fa9cdd111e47262d2e2881ca7f123028e881bdd15`

为保持第 7 节“完整目标 rustfmt 必须通过”的既有合同，现一次性授权：

1. 只对上述两个冻结 SHA 对应的文件运行同版本、同配置的 `rustfmt`；
2. `index_host_app_entrypoints.rs` 只允许 formatter 输出中已冻结的 2 个
   diff block：`rows.into_iter().map(...).collect()` 换行，以及启动段 3 个
   相邻 `if let Err(...)` 的换行；
3. `lib.rs` 只允许已冻结的 6 个测试代码 diff block：
   `assert_eq!`、bindings `all(...)`、两处 authorization 链、`audit_events`
   链及 dispatch event `matches!` 换行；
4. 当前 relay 注册、shutdown、`knowledge_open_relay: None` 和其他既有脏改
   必须保持语义与 token 不变；不得格式化第三个文件，不得借机改逻辑、warning、
   shape 或其他历史代码；
5. 若任一冻结 SHA 漂移、出现第 9 个 rustfmt diff block、出现上述 10 个
   表达式级换行之外的变化，立即停止并回交；
6. 格式化后完整复跑 R3 第 7 节离线门，更新 v2 evidence 的实际结果并回交指导线。
   即使 R3 全绿，本轮也不得自行进入 Gate 0 或 R4；须先由指导线独立验收。

### 4.4 R3 指导验收与 R4 解锁（2026-07-23）

指导线已按
`evidence/2026-07-23-l3-knowledge-open-host-owned-relay-r3-guidance-acceptance-v1.md`
独立复核并接受 R3。§4.3 的格式返工权限已经消费，不得再次使用。

现恢复第 5 节既有 R4 精确授权，但必须从 fresh Gate 0 开始；Gate 0
未通过前不得创建十二项验收命名空间或执行知识场景。对话底座三句重验继续
HOLD，不得与本轮共享真实 App、store、SQLite、sidecar、MCP、进程或 build lock。

### 4.5 R4 fresh Gate 0 安全停点（2026-07-24）

执行线已按第 5 节启动当前工作树 Syn，并在发主管首句前发现：首页自动呈现既有非验收项目/待办等 store 面。该结果触发第 8 节“真实 App 触及既有非验收条目、真实项目、卡/chain/worker”停止条件，最早 blocker 记为 `BLOCKED_REAL_APP_PRE_GATE0_EXISTING_STORE_SURFACE`。

没有发送首句、没有读取/观察 binding 或自然回复、没有调用 `tools/list`、没有启动 Codex CLI/MCP 会话、没有调用 `submit_proposal` 或任一 knowledge tool、没有读取固定 vault manifest、没有创建验收命名空间或进入十二项。未发出业务写操作；但默认 App 启动并非只读、且未获授权独立比对真实 store，故不宣称零隐式启动副作用。截图和原始 UI 日志不保留，避免把既有非验收内容写进 evidence。详见 `evidence/2026-07-23-l3-syn-native-knowledge-workspace-real-app-acceptance-v3.md`。

本停点不归因于 relay、binding 或工具面；下一包须先明示并验证：不会在首屏渲染既有真实 store 的隔离 app-data/store 启动边界、唯一允许的 test project/workflow 身份、无敏感内容的 actual `tools/list` 证据路径，以及 manifest 内部读取/条目呈现的数据最小化口径；在这些边界获授权前，不重试 Gate 0。

## 5. 实施阶段

### R0：冻结红合同

先增加失败测试，至少证明当前代码不能完成：

- MCP intent → host → App → 原生工作区 → host ack → MCP success；
- 错 grant/run/turn/project、Starting/Failed binding、重复 intent、超限帧、过期/timeout 的闭锁；
- UI 只有真正选中同一路径后才能 ack；
- 脏草稿拒绝切换时不能 ack opened。

红灯必须来自 relay 缺失，不得故意破坏既有断言。

### R1：最小 host relay

实现短期 IPC、pending map、lease、request/ack schema、host 复核与退出清理。不得持久化 payload；不得把 IPC 配置暴露为 tool 参数。

### R2：App 导航与 UI ack

接入固定 Tauri 事件和 ack command。App 必须切到知识页；原生工作区 typed read 成功、选中目标并获得焦点后才 ack。读取或导航失败给中立可见状态，并向 host 返回拒绝，不泄露内部错误。

### R3：离线回归

解除原 ignored host-dispatch 红合同并变绿；补齐 failure matrix，然后跑完第 7 节离线门。离线全绿后才能进入真实 App。

### R4：真实 App Gate 0 与十二项验收

本包明确授权在当前机器上：

- 启动当前工作树的 Syn Tauri App；
- 启动本包所需的一次主管 Codex CLI/MCP 会话；
- 只访问 Syn 自管固定 vault；
- 在固定 vault 下创建唯一前缀的验收目录与文件，并只清理本包自己创建的条目；
- 截图并记录脱敏日志。

仍不授权 Obsidian、其他 vault、真实项目文件、登录/付费、辅助功能权限、卡批准、chain/worker、任意 shell/filesystem 能力、stage/commit/push。

Gate 0：

1. 启动前记录 HEAD、status、staged、固定 vault manifest 与进程基线；只记录路径摘要/哈希/计数，不复制私人正文。
2. 启动 Syn 后先发一条新的主管首句，确认 binding 真正到 Active、自然回复可见；工具列表必须精确等于 `submit_proposal + knowledge_search/read/open/cite` 五项，本知识验收只调用四项只读 knowledge tools，不调用 `submit_proposal`。
3. 若首句仍未进入可信建立链或 binding 不是 Active，立即停止真实 MCP 场景，记录 message-scoped 事实并回交 `BLOCKED_EXISTING_CONVERSATION_BINDING_REAL_APP`；不得把 relay 失败与 binding 失败混写。

Gate 0 通过后执行原计划十二项：

1. 新建目录、Markdown 笔记和属性；
2. 双链在反链区出现；
3. 全文搜索和快速打开；
4. 分栏编辑和预览；
5. 全局/局部图打开目标笔记；
6. 新建、编辑、保存并重开 JSON Canvas；
7. 导入允许附件并从笔记/Canvas 引用；
8. 模拟外部改动并确认冲突不覆盖；
9. 主管完成 search/read/open/cite，`knowledge_open` 真实聚焦目标，回复含真实引用；
10. AI 写允许一次、拒绝一次，证明单审计写/零写；
11. 重启 Syn 后恢复知识文件和工作区；
12. 未安装 Obsidian时核心闭环成立。

每项必须记录：操作、预期、实际、截图/日志路径、是否写入、失败阶段。只有 12/12 才能写“阶段目标完成”。

## 6. 必须覆盖的失败矩阵

至少包含：

1. 无 relay config；
2. listener 不存在；
3. grant 过期；
4. binding 为 Starting / Failed / Terminated；
5. run/turn/project 不匹配；
6. relative_path 大小写漂移、symlink、非 Markdown、额外字段；
7. duplicate/replay intent；
8. UI 未挂载或窗口关闭；
9. UI read 失败；
10. 当前草稿 dirty，用户未允许切换；
11. ack intent/path 不匹配；
12. ack timeout；
13. transport Stop/退出清理；
14. 成功打开后 search/read/cite 仍只读，知识文件字节不因 open 改变；
15. 工具失败不吞自然回复，不卡 lifecycle，不增卡/chain/worker。

## 7. 必跑验证

离线：

1. relay/core/协议/失败矩阵定向 Rust 测试；
2. `cargo test knowledge_ --lib`；
3. capability registry、binding、conversation transport、supervisor orchestrator 相关回归；
4. `cargo check --lib`；
5. `npm run typecheck`；
6. `node scripts/run-offline-interaction-test.mjs`；
7. 目标 Rust `rustfmt --check`；
8. `git diff --check`；
9. `git diff --cached --name-only` 必须为空；
10. shape gate 如实报告；当前读数为 `17/5/5`、此前最后已知为 `16/5/5`，不得把 post-change baseline 当成绝对零净增证明，也不得把历史聚合债写成绿色。

真实 App：

- Gate 0 进程、binding、tool list；
- 十二项逐项截图与脱敏日志；
- `knowledge_open` request/host dispatch/UI ack/tool result 的同一 `intent_id` 关联证据，但不得记录 grant；
- 前后 manifest 证明只触及本包创建的验收命名空间；
- 退出后无 Syn/Codex/MCP 残留进程。

## 8. 立即停止条件

- `BLOCKED_DIRTY_OVERLAP`：承重脏文件出现无法归属的并行 hunk。
- `BLOCKED_SECOND_TRUTH_SOURCE`：实现需要把 intent/path 写入 binding、DB/JSON、sidecar 或静态全局。
- `BLOCKED_UNTRUSTED_RELAY_INPUT`：endpoint/grant/route/command 可由前端、用户文本或 tool arguments 选择。
- `BLOCKED_OPEN_SUCCESS_UNCONFIRMED`：没有同 intent UI ack 却出现 `opened=true`。
- `BLOCKED_SUPERVISOR_PERMISSION_EXPANSION`：主管出现 workspace-write、非空写根、任意 shell/filesystem、wildcard/default allow-all。
- `BLOCKED_EXISTING_CONVERSATION_BINDING_REAL_APP`：fresh Gate 0 仍无法得到 Active binding。
- 真实 App 触及其他 vault、既有非验收条目、真实项目、卡/chain/worker，或需要登录/付费/辅助功能权限。

停止时保留 fail-closed 状态，交回最早 blocker、message-scoped 证据和最小下一包；不靠放宽边界换绿。

## 9. 明确禁止

- 不恢复 Obsidian 真嵌入、伴随窗口、Electron 迁移或插件生态复刻。
- 不使用文件轮询/旁路 JSON/SQLite 充当 IPC。
- 不把正文、绝对路径、vault root、argv、环境或 grant 发到 UI/tool result/evidence。
- 不为 relay 新增知识写工具、通用 open-url/open-path/run-command 能力。
- 不顺手修历史 warnings、shape、旧 resident/private-home 或无关 UI。
- 不 stage、commit、push、reset、clean 或 stash。

## 10. 回交格式

回交必须分开写：

1. relay 离线实现与精确数据流；
2. 安全复核：是否新增真相源、是否放宽主管权限；
3. 离线验证实数与历史债；
4. Gate 0 结果；
5. 十二项真实 App 逐项结果；
6. 截图/日志/manifest 路径；
7. 实际改动文件与白名单核对；
8. staged/进程/验收命名空间清理状态；
9. 实际 catch 与剩余问题。

执行线自报完成后，指导线仍需核 diff、关键测试和真实 App 实物；收到回传不等于验收通过。
