# 任务包：L3 Obsidian 完整界面与最大化接入开发 v1

- 日期：2026-07-23
- 状态：**已停止并转入 v2；禁止继续 O4/O4B 与强制 O1**（当前包：`tasks/2026-07-23-l3-syn-native-knowledge-workspace-development-package-v2.md`）
- 负责人：新建 Codex 开发线（`gpt-5.6-terra`，reasoning=`ultra`）
- 指导/验收：当前总指导对话
- 决策：`decisions/2026-07-23-l3-obsidian-full-interface-in-syn-route-v1.md`
- 计划：`docs/plans/2026-07-23-l3-obsidian-full-interface-and-maximal-integration-small-stage-plan-v1.md`
- 本包允许：安装并启动官方 Obsidian、仅使用 Syn 专用 vault、启动真实 Syn 做代表性验收、修改白名单、运行离线/真实验证
- 本包不允许：读取/导入其他 vault、绕过 Gatekeeper、解包/修改 Obsidian、任意 CLI/脚本入口、放宽主管权限、stage/commit/push

> v1 已完成或在途的固定 vault、冲突保护、typed 兼容桥和知识只读能力由 v2 接收；除此之外本包不再授权新工作。

## 0. 对齐块

```yaml
authority_chain:
  - AGENTS.md
  - CURRENT.md
  - decisions/2026-07-23-l3-obsidian-full-interface-in-syn-route-v1.md
  - docs/plans/2026-07-23-l3-obsidian-full-interface-and-maximal-integration-small-stage-plan-v1.md
  - tasks/2026-07-23-l3-obsidian-full-interface-and-maximal-integration-development-package-v1.md
plan_anchor: docs/plans/2026-07-14-post-m5-stage-plan-v2.md#1-轨道表-L3-知识库
existing_before_new:
  - 复用 knowledge_vault.rs 的固定 vault、路径锁、五命令与 Batch 2 audit
  - 复用 KnowledgeBaseView、knowledgeVault.ts、tauri.ts 与现有 AI PendingAction
  - 复用 mcp/capability_registry.rs、可信 conversation binding 与 tools/list/tools/call 双闸
capabilities_touched:
  - obsidian install/readiness/CLI/URI integration
  - shared Markdown vault refresh and conflict protection
  - macOS managed companion-window probe
  - supervisor knowledge_search/read/open/cite
forbidden_alternatives:
  - app.asar 解包、修改、重签名或重新分发
  - 用 iframe/Tauri webview 伪装不存在的 Obsidian Web 版
  - 屏幕截图流、坐标点击或外部窗口冒充真嵌入
  - 任意 shell、任意 Obsidian CLI subcommand、production eval/CDP
  - 复制 knowledge vault、另建第二真相源或绕过 AI 写入确认
  - 为此迁移整个 Syn 到 Electron
```

字段齐全只证明导航完整，不等于路线、代码或验收自动成立。

## 1. 唯一目标

按计划 O1→O6 把当前 L3 第一片扩成真实可用的 Obsidian 集成：官方 Obsidian 使用 Syn 自管 vault；Syn 提供 typed CLI/URI bridge、真实状态、双向刷新、冲突保护和可靠降级；尽量提供受管伴随窗口；主管通过现有 capability plane 只读搜索/读取/打开/引用知识；真实 App 完成代表性日用验收。

开发线必须先重读路线 decision。真嵌入在公开/合规路线下已被否决；不得用更高风险技巧重新打开已裁决路线。O4 伴随窗口失败时按计划转 O4B/独立窗口，不把它变成全包 blocker。

## 2. 用户已给的授权

本包已获得以下明确授权，不必逐阶段重复询问：

- 可以开始开发；
- 可以在勘查时下载安装官方 Obsidian并实地查看；
- 如果不能达到真嵌入，按最大化实现继续；
- 可以并行推进并在合适时拆分开发对话；
- 自主推进到本小阶段计划完成。

以下动作即使在目标内，仍需操作系统或用户现场确认时停下：macOS Gatekeeper/辅助功能/自动化弹窗、登录 Obsidian 账号、购买 Sync/Publish、访问已有 vault。不得代点安全弹窗或绕过系统机制。

## 3. 开工冻结

指导线记录的初始事实：

- `HEAD=e9ad7f3a204a1ebb11ce26c1e8c05b19c04c0991`；staged 为空；工作树有大量已知归属脏改，不 reset/clean/stash/整文件覆盖。
- 本轮勘查前 `/Applications/Obsidian.app` 不存在，PATH 无 `obsidian`。
- Syn Tauri=`2.11.2`；`tauri.conf.json` 是单主窗口；默认 capability 只有 `core:default` 与 set-title。
- `knowledge_vault.rs` SHA-256=`b6b408ff56bb30fb5b293224df9ba1786206b3adf0dc2a1bb7b1f4773707853a`。
- `command_registry.rs` SHA-256=`8bde1852105d6b2d36861d247e5de12e399de175d1e72563dcbeac0350e1bf8b`。
- `KnowledgeBaseView.tsx` SHA-256=`6da9f6ff7cf0570ed67c078e82701f24672f5f0ab7304f0333c36b293c20f972`。
- `knowledgeVault.ts` SHA-256=`f0f106ef61925cf9368c498c001f73fd7c5a18b24aa0d5a8300b562840668587`。
- `tauri.ts` SHA-256=`47bfb2978960856159124cba8f4eed325951db8e0ae649992c314d46d6527fa2`。
- 最近报告 shape 为历史 `16/5/5`；实施者必须亲跑并冻结开工时实际值，不能沿用旧 `13/5/5`。

开发线开工时重新记录 HEAD、porcelain、staged、上述文件 hash、App/CLI 安装状态、当前 Obsidian/Syn 进程与测试 vault 路径。若承重文件继续漂移或出现所有者未知 hunk，先报告并做 merge-only；不得覆盖。

## 4. 写入白名单

清单是上限，不是要求全改。新增文件可在列明的新目录内创建；包外承重文件需要先向指导线说明理由。

### 4.1 后端

- `prototypes/productized-desktop-shell/src-tauri/src/obsidian_integration.rs`（新增）
- `prototypes/productized-desktop-shell/src-tauri/src/obsidian_integration_tests.rs`（可新增）
- `prototypes/productized-desktop-shell/src-tauri/src/knowledge_vault.rs`（已有脏改，merge-only）
- `prototypes/productized-desktop-shell/src-tauri/src/command_registry.rs`（已有脏改，merge-only）
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`（已有脏改，只有确有必要时 merge-only）
- `prototypes/productized-desktop-shell/src-tauri/src/mcp/capability_registry.rs`（已有未跟踪实现，merge-only）
- `prototypes/productized-desktop-shell/src-tauri/src/mcp/supervisor_orchestrator.rs`（已有脏改，merge-only）
- `prototypes/productized-desktop-shell/src-tauri/src/mcp/knowledge_capabilities.rs`（可新增）
- `prototypes/productized-desktop-shell/src-tauri/src/mcp/supervisor_conversation_transport_tests.rs`（已有未跟踪实现，merge-only）
- `prototypes/productized-desktop-shell/src-tauri/Cargo.toml`、`Cargo.lock`（仅 O4 公共 API 确需最小 macOS binding 依赖时）

### 4.2 前端与测试

- `prototypes/productized-desktop-shell/src/lib/obsidianIntegration.ts`（新增）
- `prototypes/productized-desktop-shell/src/lib/knowledgeVault.ts`
- `prototypes/productized-desktop-shell/src/lib/knowledgeBase.ts`
- `prototypes/productized-desktop-shell/src/lib/tauri.ts`（已有脏改，merge-only）
- `prototypes/productized-desktop-shell/src/views/KnowledgeBaseView.tsx`
- `prototypes/productized-desktop-shell/src/styles.css`
- `prototypes/productized-desktop-shell/tests/knowledge-vault-notes.test.tsx`
- `prototypes/productized-desktop-shell/tests/obsidian-integration.test.tsx`（新增）
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`（只改 Obsidian 占位→真实状态相关断言）
- `prototypes/productized-desktop-shell/tests/helpers/offlineMemoryKnowledgeTextFixtures.ts`（只改上述断言共用的知识库文本 fixture）
- `prototypes/productized-desktop-shell/scripts/run-offline-interaction-test.mjs`

### 4.2.1 主管 binding 既有 fixture 的精确扩展

- `prototypes/productized-desktop-shell/src-tauri/src/mcp/supervisor_conversation_binding.rs`（已有未跟踪实现，merge-only；只允许把四个知识只读 capability 加入宿主精确集合，并补 stale/mismatch binding 必须拒绝的测试）

以上三项由开发线开工冻结后提出，指导线核对为实现 O3/O5 契约与回归所必需；不授权保留旧占位文案、热扩旧 active binding、增加写 capability 或改 binding 生命周期。

### 4.3 条件分支 O4B

- `integrations/obsidian/syn-bridge/`（新插件全部文件；只有 O4 不通过且指导线收到裁决事实后才创建）
- `prototypes/productized-desktop-shell/package.json` 与 lockfile（仅插件/测试构建确需依赖时）

### 4.4 文档与收口

- `evidence/2026-07-23-l3-obsidian-embedding-feasibility-and-route-selection-v1.md`
- `evidence/2026-07-23-l3-obsidian-maximal-integration-offline-verification-v1.md`（新增）
- `evidence/2026-07-23-l3-obsidian-maximal-integration-real-app-acceptance-v1.md`（新增）
- `docs/2026-07-08-workbench-current-feature-inventory-for-prototype-v1.md`（已有脏改，收口时 merge-only）
- `docs/plans/2026-07-14-post-m5-stage-plan-v2.md`
- `docs/plans/2026-07-16-master-execution-plan-conversation-first-v1.md`（已有脏改，收口时 merge-only）
- `CURRENT.md`、`AUTHORITY.md`（已有脏改，收口时 merge-only）
- `docs/harness-catch-log.md`（只有真实 catch 时追加）

禁止修改 `workflow_db_primary_wiring.rs`、M5 bridge/repository/schema/CAS、旧 resident/private-home 主路线、conversation profile sandbox、真实业务项目、用户其他 vault 或 `.codex` 凭据。

## 5. 实施顺序

### O1 安装与真实勘查

- 使用官方来源，验证发布摘要、签名、Gatekeeper 和版本；不绕过异常。
- 先用 `/private/tmp` 或明确的新建测试目录，绝不让 Obsidian 自动扫描用户目录。
- 启用并验证官方 CLI；保存可复现命令和失败证据。
- 对完整 UI 代表性功能做一轮实测，作为后续 O3/O4 的事实输入。

### O2 先红后绿的 typed bridge

- 先加 fake executable 测试，锁状态枚举、固定 binary、固定 vault、typed argv、timeout、output cap、错误人话和注入拒绝。
- 再实现最小 Rust module 与 Tauri commands；production 不暴露任意 command/eval/CDP。
- 真实 CLI 探针与离线 fake 测试分开，不用现场绿替代单测。

### O3 页面与同源一致性

- 先加前端场景：未安装、未运行、CLI 未启用、ready、命令失败、刷新冲突、Obsidian 关闭后的降级。
- 再将现有占位卡改为真实状态和操作，不重造 Obsidian 视觉副本。
- Markdown 真相源、AI PendingAction 和 Batch 2 audit 保持；外部改动刷新与冲突拒绝要有断言。

### O4 伴随窗口探针与裁决

- 先在独立、小范围实现公共辅助功能探针；权限缺失要能检测且不循环弹窗。
- 通过计划中的窗口矩阵才可接入页面；否则保留独立打开并转 O4B。
- 任何时候都提供分离/恢复，不允许窗口遮挡失控。

### O5 capability plane

- 在现有 registry 中登记 `knowledge_search/read/open/cite`，复用同一个服务端鉴权判定。
- 主管 profile 保持 read-only + 空写根；知识工具只访问固定 vault。
- 不增加公开写工具；AI 写入闸不变。

### O6 离线与真实 App 验收

- 先完成所有离线闸，再在 Syn 专用 vault 做真实 App 验收。
- 真 vault 首次打开前先备份/manifest；证据只记录路径摘要与计数，不泄露正文。
- 验收完同步 CURRENT/AUTHORITY/master/feature inventory；不 stage/commit。

## 6. 必须通过的离线测试

1. App/CLI 状态六态与版本不兼容；
2. binary/vault/command 均不能由前端改写；引号、换行、`--`、路径穿越、绝对路径均拒绝；
3. timeout、non-zero、stdout/stderr 过大、非 UTF-8/奇异输出的人话收口；
4. open/read/search/command 使用固定 argv，未经过 shell；
5. Obsidian 不可用时现有 knowledge vault 浏览/编辑不回归；
6. 外部文件新增/编辑可刷新，mtime/hash 冲突拒绝覆盖；
7. AI 拒绝时零写，允许时只写一次且 audit 仍走 `knowledge_vault_audit` Batch 2；
8. `tools/list`/`tools/call` 对四个知识能力同源授权，错 binding/项目/slug/变体/wildcard 全拒；
9. 工具失败不吞自然 reply，不生成未确认知识写；
10. 伴随窗口状态机在 permission missing/window missing/app quit/resize failure 时可恢复且默认降级。

## 7. 必跑命令

在 `prototypes/productized-desktop-shell`：

- 新增 Rust 定向测试；
- 相关 `knowledge_vault`、M5-B、capability registry/binding 回归；
- `cargo check --lib`；
- `npm run typecheck`；
- `node scripts/run-offline-interaction-test.mjs`；
- 新前端场景测试；
- 仓根 shape baseline/check，记录开工值与零净增；
- 目标 Rust fmt；若全仓 fmt 被既有差异阻断，只报事实，不格式化包外文件；
- 仓根 `git diff --check`；
- 收口 `git diff --cached --name-only` 为空，列出全部实际改动文件。

## 8. 真实 App 验收

严格执行计划 O6 十项。额外要求：

- 必须展示 Obsidian 的真实标题栏/菜单/设置或其他可识别原版 UI 证据，不能只拍 Syn 状态卡；
- Graph、Canvas、命令面板、核心插件和一个可撤销测试社区插件各至少一项实操；
- Syn 与 Obsidian 双向编辑、冲突、重启恢复各有前后证据；
- 主管知识工具只能读/开/引用，chain/worker、方案卡和执行授权不因本包自动触发；
- 伴随窗口的结论必须是 `passed_managed_companion`、`degraded_independent_window` 或 `reverse_host_plugin` 之一，不能写模糊“嵌入成功”。

## 9. 立即停止条件

- `BLOCKED_GATEKEEPER_OR_SIGNATURE`：官方包不能普通通过签名/Gatekeeper；不绕过，交给用户决策。
- `BLOCKED_EXISTING_VAULT_ACCESS`：下一步会读取/导入非 Syn vault。
- `BLOCKED_OBSIDIAN_REPACKAGING`：必须解包、修改、重签或重分发 Obsidian。
- `BLOCKED_ARBITRARY_COMMAND_SURFACE`：实现需要任意 shell/CLI/eval/CDP 生产入口。
- `BLOCKED_PERMISSION_EXPANSION`：需要放宽 supervisor sandbox/MCP allowlist 或公开写工具。
- `BLOCKED_DIRTY_OVERLAP`：承重文件出现无法归属的并行 hunk。
- `BLOCKED_STORAGE_EXPANSION`：需要第二 vault 真源、M5 schema/bridge 变更或无声覆盖冲突。
- `BLOCKED_USER_ACTION`：需要用户亲手处理 macOS 权限、登录、付费或已有数据选择。

O4 伴随窗口不稳定不触发整包停止；按计划转降级分支。

## 10. 回交格式

每个 O 阶段完成时只报告一次短证据，最后回交必须包括：

1. 实际安装来源、版本、签名/Gatekeeper 与 CLI 状态；
2. 实际改动文件及是否全部在白名单；
3. typed bridge、安全参数、超时/输出限制证据；
4. Syn↔Obsidian 双向 vault 与冲突证据；
5. 伴随窗口/独立窗口/O4B 的真实裁决；
6. capability registry/binding/allowlist 与零越权写证据；
7. Rust、`cargo check --lib`、typecheck、离线 runner、shape、fmt、diff-check 真实输出；
8. 真实 App 十项结果与截图/日志路径；
9. 其他 vault、chain/worker、真实项目、staged/commit/push 均未触碰；
10. `harness-catch-log` 是新增 catch 还是零 catch；历史 warnings/shape 债单列。

开发线自报完成后，指导线仍要核 diff、关键测试与真实 App 实物；“回传收到”不等于验收通过。
