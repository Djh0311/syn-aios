# 共享主管 Conversation Binding 建立链离线验证 v1

日期：2026-07-23  
范围：仅本地源码、测试夹具与已获准的真实 store 私有临时副本；未启动 App，未写入真实 source。

> 后续纠正：本证据原先把四个阶段写成完整失败分类，并把 transport-start 失败概括为“终结已建 binding”。该口径遗漏 store 准备和激活失败，也没有证明终结写失败时的安全收口。当前完整口径与四类注入结果见 [阶段语义与失败收口返工验证 v1](2026-07-23-shared-supervisor-conversation-binding-phase-semantics-and-failure-closure-rework-verification-v1.md)。本文件保留为历史部分证据，不能再用来声称全链路已覆盖或终结一定成功。

## Phase A：冻结与副本审计

- 基线 HEAD：`e9ad7f3a204a1ebb11ce26c1e8c05b19c04c0991`；暂存区为空。工作树原本已脏，未 reset、clean、stash、stage 或 commit。
- 运行态检查没有发现本任务相关 App/开发服务器进程；真实 SQLite、WAL/SHM 与 workflow/supervisor/registry 文件均无 holder，之后才制作私有临时副本。
- 只在副本上做 SQLite `integrity_check`，结果为 `ok`。
- 副本前置计数：SQLite supervisor sessions/bound=`25/0`，audit=`263`；JSON sessions/audit/bound=`25/263/0`；exec registry entries/audit/warnings=`0/9/86`。

这证明该时点的副本在计数与完整性上自洽；它不提供真实 App 当次失败的 message-scoped 子错误，因此不能反推唯一根因。

## 静态链路审计与修复范围

后端启动路径的顺序为：可信 context/现有 thread 校验、run id、read-only binding 构造、binding 持久化、可选 host thread 激活、transport 启动。

当时修复后的部分建立错误只会落入以下一项固定阶段；它不是当前完整阶段表：

| 阶段 | 安全含义 | 失败时的动作 |
| --- | --- | --- |
| `binding_construct` | context、身份或 binding 形状不成立 | 返回失败 receipt，不启动 transport |
| `binding_persist_db` | DB-primary delta 未提交 | 返回失败 receipt，不投影、不发布工具 |
| `binding_project_json` | DB 已提交但 JSON projection 未完成 | 返回失败 receipt，不发布工具 |
| `transport_start` | binding 之后的运输启动未完成 | 历史代码尝试终结已建 binding；该尝试不等于终结已确认 |

收据仅包含 turn 与固定阶段/人话；前端在 runtime boundary 再次 allowlist 阶段，未知值和未知异常都不会进入共享状态。

## 离线验证结果

| 检查 | 结果 |
| --- | --- |
| `cargo test --lib 'mcp::supervisor_conversation_binding::tests' -- --nocapture` | 5 passed |
| `cargo test --lib 'mcp::supervisor_orchestrator::tests::shared_supervisor_' -- --nocapture` | 8 passed，1 ignored（副本回放需要显式私有副本） |
| 真实私有副本回放 ignored test | 1 passed；启动对账后断言 `25/0`，建立后断言 JSON/DB `26/1`、lifecycle=`Starting` |
| DB-primary 失败注入 | 失败阶段=`binding_persist_db`；JSON/DB 均保持 `25/0`；工具面为空 |
| JSON projection 失败注入 | 失败阶段=`binding_project_json`；JSON 保持 25，DB 为 `26/1`；工具面为空，DB-primary health 被冻结 |
| `cargo test --lib 'mcp::supervisor_orchestrator::tests::m5b_' -- --nocapture` | 2 passed |
| `cargo check --lib` | passed；既有 unused/dead-code warnings 598 条 |
| `npm run typecheck` | passed |
| `node scripts/run-offline-interaction-test.mjs` | 15 passed |
| `rustfmt --check`（本任务触及文件） | 仅同一文件中三处改前既有格式差异；本包新增/改动行已格式化 |

## Gate 与差异

- `workbench-shape-gate` 的 Phase A 与收口复核均为 Errors/Warnings/Info=`16/5/5`。这是历史债务基线；本包零净增，但不能把 gate fail 说成通过。
- 全仓 `cargo fmt --check` 首先报告白名单外既有脏文件，因此没有批量格式化。
- 本包没有产生新的 harness catch；没有改写 `docs/harness-catch-log.md`。

## 裁决

历史离线链路只证明：在副本的 25-session 形状和当时的受控失败注入下，构造、DB-primary 持久化、JSON projection 与部分 transport 前置错误可被区分，且没有制造工具或产品副作用。它**没有**证明 store 准备、激活、终结失败均已被唯一分类，也没有证明 transport 失败后的 durable lifecycle 一定为 `Failed`。

仍未证明：真实 App 首句为何没有进入建立链，也未证明真实三句验收通过。下一步必须由新的真实 App 授权包执行；本证据不能替代那次验收。
