# 任务包：L3 Syn R4 隔离 runtime profile 与 Gate 0 前置验收 v1

- 日期：2026-07-24
- 状态：**PENDING_AUTHORIZED_I5_HOME_ONLY_UI_DISCOVERY / Gate 0 仍未授权**
- 负责人：现有知识开发线（`gpt-5.6-terra`，reasoning=`ultra`）
- 指导/验收：当前总指导对话
- 上游任务：`tasks/2026-07-23-l3-knowledge-open-host-owned-relay-and-real-app-acceptance-package-v1.md`
- 停点正本：`evidence/2026-07-23-l3-syn-native-knowledge-workspace-real-app-acceptance-v3.md`
- 指导裁决：`evidence/2026-07-24-l3-r4-pre-gate0-existing-store-surface-guidance-review-v1.md`
- 实际结果：I0–I4 已完成。历史 `BLOCKED_I5_MACOS_LOCKED_UI_INSPECTION` 已撤回：上轮仅证明 Home UI 读取没有完成，不能由单条工具文本推断宿主 macOS 锁屏。其后已精确修复 launcher 启动前在隔离根写入 `ui-inspection.json`、触发 Rust 根目录 allowlist 拒绝而 App 以 exit 0 返回的跨语言合同；启动前根目录现严格只含 Rust allowlist 的六项，外部 UI observation 只能在 `logs/` 下于 UI 发现后创建，profile/AppState 启动失败分别以固定非零码闭锁。修复后唯一一次 Home-only isolated smoke 的 build 成功，但 `sky.list_apps()` 返回的 Syn bundle 为非运行态，未形成可安全读取的 UI target；未 attach、未读 Home、未截图、未写 observation，并已终止本线自建进程。指导验收退回的唯一合同缺口也已离线加固：Rust validator 实际消费单一 `PREPARED_ROOT_ENTRY_NAMES` 六项常量，测试严格解析 launcher 的 `PRELAUNCH_ROOT_ENTRY_NAMES` 及其自身 `PROFILE_FILE_NAME` 字面量，并要求同数、无重复、集合完全相等；第七项、六项任一删除/改名和 Rust allowlist 漂移均由内存变异拒绝。此前 discovery-only 专项已静态锁定 fresh `.app` bundle 的构建、identity 回执与直接启动，但在 `sky.list_apps()` 前报告 final Syn 子进程为 `SIGKILL`；v4 原件继续保留。用户后来单独授权一次 pre-list SIGKILL 诊断：新 launcher lifecycle ledger 的红绿合同、18 项 profile 套件、`cargo check --lib`、typecheck、syntax、目标 rustfmt、diff-check 和 staged gate 均通过。该唯一启动仍在 UI 枚举前 `SIGKILL`，但新的 `launcher-receipt-v5.json`（SHA-256 `10ba897b7c452f8739b829fb9a10561cb93dbac040871cdc9e94461133bb3092`）可证明 launcher 未 kill child、parent 未收到 `SIGTERM`/`SIGINT`/`SIGHUP`、child `exit`/`close` 均为 `SIGKILL`；PGID/session 投影为 unavailable，原因未记录。仅 process-scoped macOS fault/error 元数据投影为 `matching_record_present`（SHA-256 `0ba1b0cc32ecdc579e32444885b53af8bb3582dea8ee58d8c8e6b`），不含 raw log 且不能归因。随后用户再授权一次严格单变量对照：沿用其可运行的 `cargo-tauri dev` 链，只注入 fresh synthetic isolated profile；Syn、cargo-tauri 与 Vite 连续存活至少 60 秒，storage-mode 缺失路径明确落在本轮隔离根，随后以 Ctrl-C 正常停止且三类进程和 5173 均无残留。故 isolated profile 已从 SIGKILL 充分条件中排除，问题收窄到 launcher 的 fresh bundle direct-executable 路径。静态 `codesign --verify --strict` 对 dev binary 为 valid，对当时 fresh `.app` 及 executable 均报缺少已声明 resource seal；忽略 resources 后才 valid。后续离线返工先以 0/1 红合同锁住该缺口，再最小修改 launcher：fresh `.app` 必须在最终进程启动前由固定 `/usr/bin/codesign` 完成 ad-hoc seal，并通过 deep/strict verification；seal 或 verify 的启动失败、非零退出或信号均固定闭锁为 `failure_stage=bundle_integrity`。聚焦合同修复后 1/0，完整 profile 套件 19/0；离线 fresh build 后先复现原 strict verify 失败，再执行与 launcher 完全相同的 seal，`.app` 与真实 executable 均 strict verify 通过且 `Contents/_CodeSignature/CodeResources` 存在。第一轮 pre-list 重验中，sealed fresh bundle ready 后连续存活至少 88 秒且未观察到 `SIGKILL`，随后只向精确自建 PID 发 SIGTERM；v6 receipt 记录 build 0、child exit/close SIGTERM、launcher 未 kill child、parent 未收 signal、父子/process-group/session 关系均确认。指导对话随后因未重读已更新的 CURRENT，重复消费一次相同范围重验；第二轮 ready 后连续存活至少 90 秒且仍未观察到 `SIGKILL`，Ctrl-C 后 v7 receipt 记录 child/parent SIGINT、launcher 未主动 kill、关系投影均确认。两轮均未调用 UI、未触碰真实 store，收尾无本线 Syn/cargo-tauri/Vite/5173 残留；重复运行不扩大结论强度。当前停点仍为 `PENDING_AUTHORIZED_I5_HOME_ONLY_UI_DISCOVERY`；详见 `evidence/2026-07-24-l3-syn-r4-isolated-runtime-profile-preflight-verification-v1.md`。

## 0. Kickoff

- 任务：建立仅用于 L3 R4 验收的隔离 runtime profile，让 Syn 启动、migration、reaper、snapshot 与知识路径只接触唯一临时测试根；完成离线门和一次隔离首屏 smoke 后停在 Gate 0 前。
- 负责人：现有 `gpt-5.6-terra / ultra` 知识开发线。
- 交付物：路径路由与 fail-closed profile、唯一 synthetic test project/workflow fixture、离线测试、隔离首屏证据、验证 evidence、CURRENT/AUTHORITY 状态回写。
- 验收标准：隔离模式下首屏只出现本包 synthetic 身份；不能读取/呈现仓库真实 index/tasks、真实 `$HOME/.codex` session store、默认 app-data/vault/canvas；非法 profile 在开窗和任何 store 读取前失败；普通启动行为保持不变。

## 1. 已知、未知与本包假设

### 已知

- R3 已获指导验收；relay、五工具 allowlist 与离线安全合同本包不重做。
- R4 在主管首句前因既有非验收 store 面安全停止，未触发 binding、MCP 或十二项。
- 当前默认首页不只依赖 app-data：还会读取仓库 index/tasks，并可能从 `$HOME/.codex/state_*.sqlite` 加载 session。
- supervisor 的既有受控启动链需要真实 `HOME` 定位认证材料；改变 `HOME`/`CODEX_HOME` 会扩大安全面并破坏当前合同。

### 未知

- startup migration、reaper、snapshot 和前端初始加载中是否还有未归一到固定路径函数的持久化或读取点。
- 最小 synthetic workflow state 的现有合法 schema 与首屏呈现所需字段。
- 隔离首屏 smoke 是否暴露新的真实路径或自动写入点。

### 假设

- 只支持当前 macOS debug 验收构建；不为生产用户增加通用“自定义数据目录”功能。
- 本包可以启动一次**隔离后的** Syn App 做首屏 smoke，但不能发送主管消息、启动 Codex CLI/MCP server 或调用任何工具。
- 路径隔离失败时，宁可不开窗，也不能回落到真实默认路径。

## 2. 权威对齐块

```yaml
authority_chain:
  - AGENTS.md
  - CURRENT.md
  - AUTHORITY.md
  - decisions/2026-07-23-knowledge-and-conversation-parallel-workstreams-v1.md
  - decisions/2026-07-23-supervisor-read-only-exact-five-capability-surface-v1.md
  - decisions/2026-07-23-l3-syn-native-knowledge-workspace-route-v2.md
  - tasks/2026-07-23-l3-knowledge-open-host-owned-relay-and-real-app-acceptance-package-v1.md
  - evidence/2026-07-24-l3-r4-pre-gate0-existing-store-surface-guidance-review-v1.md
plan_anchor: docs/plans/2026-07-23-l3-syn-native-knowledge-workspace-small-stage-plan-v2.md#n6
existing_before_new:
  - AppState 现有 index/tasks/workflow 路由
  - SessionSourceMode::IndexOnly
  - knowledge_vault 固定根与路径锁
  - supervisor 既有临时 HOME/认证复制边界
capabilities_touched:
  - Syn startup runtime path selection
  - isolated acceptance fixture
  - pre-Gate-0 real App smoke
forbidden_alternatives:
  - 修改 HOME 或 CODEX_HOME
  - 通用任意 app-data 覆写
  - 读取或复制真实 codex-index/tasks/session DB/vault 作为 fixture
  - 在本包启动 supervisor/Codex CLI/MCP 或执行 tools/list
  - 继续 Gate 0 或十二项
```

## 3. 冻结的隔离合同

### 3.1 唯一入口

实现一个进程级、验收专用 profile。推荐用一个固定环境变量指向 profile manifest，例如：

`SYN_R4_ACCEPTANCE_PROFILE=/canonical/temp/root/profile.json`

名字可按现有命名风格微调，但必须满足：

1. 只在 debug/验收构建生效；
2. 非 debug 构建发现该变量时拒绝启动，不得忽略后回落真实路径；
3. manifest schema/version/purpose 固定，拒绝额外字段；
4. canonical root 必须是 canonical `std::env::temp_dir()` 下、名称以 `syn-r4-acceptance-` 开头的直接子目录；
5. root 为本用户所有、权限 `0700`、不是 symlink，profile 是 root 的直接普通文件；
6. run identity、project identity、workflow identity 均由 profile/launcher 生成且只能属于该 root；
7. profile 必须在 migration、reaper、AppState、snapshot 与 Tauri window 创建前完成一次初始化；之后 immutable；
8. 变量存在但 profile 缺失、过期、重复初始化、schema 错、权限错、symlink、非空复用或路径逃逸时，统一在开窗前 fail closed。

不得把它扩成产品设置、CLI 任意 `--data-dir` 或可指向用户目录的通用功能。

### 3.2 路由面

profile 激活后，以下路径必须全部派生于同一个 canonical root：

- synthetic `codex-index.json`；
- synthetic/empty `tasks.md`；
- 唯一 test project root；
- workflow state 及其 SQLite/JSON/sidecar/registry/recovery 路径；
- knowledge vault 与 recovery backups；
- Canvas v1；
- 如调用链仍需要 Codex thread DB 路径，则只能是 isolated root 内的空/fixture DB；首选启动 snapshot 固定 `SessionSourceMode::IndexOnly`，不得探测真实 `$HOME/.codex`。

普通模式不带 profile 时，所有现有默认路径和行为必须字节/语义等价。

### 3.3 唯一 synthetic 身份

launcher 只创建一个 fresh fixture：

- 一个 root 内 test project；
- 一个由 run identity 派生的 project ID；
- 一个由 run identity 派生的 workflow ID；
- index 中只含该 project，threads/sessions 为空；
- tasks 为空或只含一条明确的 `SYN R4 ISOLATED ACCEPTANCE` synthetic 条目；
- isolated app-data/vault/canvas 初始为空；
- 禁止从真实 index、task、workflow、Codex DB 或 vault 复制任何正文、标题、ID、路径或时间线。

首屏必须有明确但不泄露绝对路径的验收标识，或者以唯一 synthetic 项目名足以机械判断；不能显示真实项目/待办/session。

### 3.4 不改认证边界

- 不修改进程真实 `HOME`、`CODEX_HOME` 或 supervisor profile/allowlist；
- 不复制认证材料到验收 evidence/fixture；
- 本包不启动 supervisor，所以不得以“验证认证”为由读取认证文件；
- 后续 R4 若启动主管，继续使用既有 host-owned 临时 supervisor home 与短期 relay grant，不由本 profile 另造认证机制。

## 4. 后续 Gate 0 的证据最小化合同

本包只把合同写进代码测试/任务 evidence，不实际调用 `tools/list`。

### 4.1 actual `tools/list`

后续 R4 只能落一份 redacted projection：

```json
{
  "schema_version": 1,
  "source": "actual_mcp_tools_list",
  "tool_count": 5,
  "sorted_tool_names": [
    "knowledge_cite",
    "knowledge_open",
    "knowledge_read",
    "knowledge_search",
    "submit_proposal"
  ],
  "exact_match": true,
  "canonical_projection_sha256": "<sha256>",
  "run_identity_sha256": "<sha256>",
  "turn_identity_sha256": "<sha256>"
}
```

允许保存固定工具名、数量、相等结果和 projection hash；禁止保存 tool descriptions、input schema、endpoint、grant、argv、环境、原始 stdout/stderr、绝对路径或自然回复正文。`source` 必须来自真实 MCP `tools/list` 结果的内存投影，不能由静态 allowlist 伪造。

### 4.2 vault manifest

后续 R4 manifest 只枚举 isolated acceptance namespace 内本轮创建的测试条目，字段限定为：

- root identity hash；
- 相对路径；
- kind；
- byte length；
- SHA-256；
- mtime；
- created_by_this_run。

不得扫描或记录默认真实 vault；不得保存 Markdown/Canvas/附件正文、绝对路径、其他条目名或目录全量清单。start/end manifest 都只能针对本轮 synthetic namespace。

## 5. 精确写入白名单

### 代码

- `prototypes/productized-desktop-shell/src-tauri/src/acceptance_runtime_profile.rs`（可新增）
- `prototypes/productized-desktop-shell/src-tauri/src/acceptance_runtime_profile_tests.rs`（可新增）
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`（merge-only：module、AppState 路由、session source）
- `prototypes/productized-desktop-shell/src-tauri/src/index_host_app_entrypoints.rs`（merge-only：初始化顺序与 pre-window fail closed）
- `prototypes/productized-desktop-shell/src-tauri/src/codex_db.rs`（merge-only：isolated DB 路由或明确拒绝真实 fallback）
- `prototypes/productized-desktop-shell/src-tauri/src/knowledge_vault.rs`（merge-only：app-data/vault/recovery/workflow 路由）
- `prototypes/productized-desktop-shell/src-tauri/src/mcp/storage.rs`（merge-only：Canvas 根路由）
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`（只有 production command 确实绕过 AppState/session mode 时才允许最小接线；开始前列出精确调用点）
- `prototypes/productized-desktop-shell/scripts/run-r4-isolated-app-preflight.mjs`（可新增；只创建/启动/记录自己的 temp fixture）
- `prototypes/productized-desktop-shell/package.json`（只有登记上述脚本命令时可做一行 merge-only 修改）

若只靠上述文件不能拦住启动前真实路径访问，立即停在只读审计结果并申请精确扩白；不得先改文件再补授权。

### 文档/证据

- 本任务包（状态与实际白名单）
- `evidence/2026-07-24-l3-syn-r4-isolated-runtime-profile-preflight-verification-v1.md`（新增）
- `evidence/raw/2026-07-24-l3-syn-r4-isolated-preflight/`（新增；仅 synthetic 截图、redacted receipt、fixture manifest）
- `CURRENT.md`
- `AUTHORITY.md`
- `docs/harness-catch-log.md`（仅真实 catch 时）

### 明确禁止

- `Cargo.toml` / `Cargo.lock`；使用现有 `serde`、`sha2` 与标准库；
- conversation binding payload/schema、capability registry、allowlist、supervisor profile；
- relay grant/endpoint/transport 逻辑；
- 前端知识功能与视觉重做；
- 默认真实 app-data、真实 `.codex`、真实 vault、真实 project/workflow/store；
- stage、commit、push、reset、clean、stash。

## 6. 小阶段

### I0：只读启动路径图与冻结

先列出从 `run()` 到 window 创建前的全部读写点，以及首屏 load/refresh 会调用的路径函数。冻结：

- HEAD/status/staged；
- 白名单文件 SHA；
- 默认真实路径的**固定标签**，不枚举内容；
- 当前 shape baseline。

若发现新直接 `$HOME`、仓库 index/tasks 或任意绝对存储路径访问，先更新调用图并判断是否在白名单；越界即回交。

### I1：红合同

先增加失败测试，至少覆盖：

1. profile 合法时所有 resolved roots 都在同一 isolated root；
2. snapshot 强制 IndexOnly，不调用真实 Codex DB fallback；
3. repository index/tasks 不可见；
4. 默认 app-data/vault/recovery/canvas 不可见；
5. symlink、权限错误、父目录逃逸、过期/重用、额外字段、重复初始化全部 fail closed；
6. profile env 存在但初始化未完成时，任何路径 getter 不得回落真实默认；
7. 普通模式不带 profile 时默认路径与现有行为不变；
8. 非 debug 模式拒绝验收 profile；
9. profile 失败发生在 migration/reaper/snapshot/window 之前。

红灯必须来自隔离能力缺失，不得破坏既有断言。

### I2：最小 runtime profile

实现 immutable profile 与集中路径 resolver，消除同一进程内各模块自行重新解析 `HOME` 的验收模式分叉。避免通用抽象；只提供本包实际需要的 normal/isolated 两态。

### I3：fixture 与 launcher

实现单用途 launcher：

1. 用系统安全 temp API 创建 `0700` fresh root；
2. 生成不含秘密的 profile 与 synthetic fixture；
3. 先在普通构建环境完成 build，再只给最终 Syn 进程注入 profile；
4. 输出 redacted preflight receipt：run hash、resolved-root containment 布尔值、synthetic ID hash、session source、启动/退出结果；
5. 默认保留本轮 isolated root 供指导复核；不自动删除，也绝不清理其他 prefix/root。

launcher 不得启动 Codex CLI/MCP server，不得调用 shell/filesystem MCP，不得修改 `HOME`。

### I4：离线门

必须跑：

- 新 profile 定向 Rust 测试；
- `cargo test knowledge_ --lib`；
- shared supervisor/capability/binding/transport/registry/manual-relay 的既有定向回归；
- `cargo check --lib`；
- `npm run typecheck`；
- 现有 15 组离线交互 runner；
- 白名单目标 `rustfmt --check`；
- `git diff --check`；
- staged 为空；
- shape baseline/check，按历史债如实报告，不要求借本包消债。

production Rust 路径不能只用 `cargo test` 证明。

### I5：隔离 Syn 首屏 smoke

I4 全部达到本包标准后，才允许：

1. 启动一次由 launcher 创建的 isolated Syn；
2. 不发送任何主管消息，不进入对话 Gate 0；
3. 不启动 Codex CLI/MCP server，不调用 `tools/list` 或知识工具；
4. 只确认首屏只见唯一 synthetic 身份，未见真实项目/待办/session；
5. 可打开知识页确认 isolated vault 为空，但不得创建知识内容；
6. 保存一张只含 synthetic 内容的截图和一份 redacted receipt；
7. 正常退出自建 App，并记录自建句柄/退出结果。

若出现任何非 synthetic 条目、真实路径、真实标题、真实 vault 条目或默认 store 警告，立即退出并记录最早泄漏面；不得截图保存泄漏内容，不得继续探索。

### I6：回交

完成 I5 后更新 evidence/CURRENT/AUTHORITY 并回交指导线，状态只能是：

- `ISOLATED PREFLIGHT VERIFIED / PENDING GUIDANCE ACCEPTANCE`；或
- `BLOCKED_<EARLIEST_STAGE>`。

即使 I5 通过，也不得自行恢复上游 R4 Gate 0。指导线验收后另行解锁原包的主管首句、actual `tools/list` 和十二项。

## 7. 完成标准

本包只有同时满足以下条件才算完成：

1. profile 非法时在任何真实 store 读取和开窗前失败；
2. profile 合法时所有 App-owned 状态根都位于唯一 isolated root；
3. snapshot 为 IndexOnly，真实 `$HOME/.codex` 不参与首页；
4. repository index/tasks 不参与首页；
5. 首屏只见一个 synthetic test project/workflow 身份；
6. normal mode 无回归；
7. 离线门按第 6 节如实通过或单列历史债；
8. isolated App smoke 有 synthetic-only 截图/receipt；
9. 没有发送主管消息、启动 MCP、调用工具或执行十二项；
10. 暂存区为空，无 stage/commit/push。

## 8. 停止条件

任一发生立即停止：

- 需要修改 `HOME`/`CODEX_HOME`；
- 需要读取/复制真实 index/tasks/session DB/vault 才能构造 fixture；
- profile 失败后回落真实路径；
- 新发现的承重文件不在白名单；
- 隔离首屏出现非 synthetic 内容；
- 任何 Codex CLI/MCP server、主管首句、工具调用被触发；
- 真实 store/vault/project/workflow 被读取或写入；
- 需要修改 capability/permission/sandbox/approval；
- 需要 stage/commit/push 或删除非本包 temp root。

## 9. 回交格式

执行线必须逐项报告：

1. 实际数据流与初始化顺序；
2. resolved path containment 证明；
3. synthetic project/workflow 身份与 fixture 内容摘要；
4. 红合同与离线验证实数；
5. isolated App smoke 的操作、预期、实际、截图/receipt；
6. 是否出现任何真实 store/vault/路径面；
7. 实际修改文件；
8. 未执行项；
9. Git/status/staged/diff-check/shape；
10. catch-log 是否新增；
11. 最早 blocker 或“等待指导验收”的明确停点。
