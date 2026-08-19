# M6D06 temporary agent history projection candidate and evidence v1

状态：`SUPERVISOR_SELF_REVIEW_PASS / CP3_AWAITING_INDEPENDENT_ACCEPTANCE / DOMAIN_RUNTIME_ONLY / NOT_RELEASED`

## Harness

- stage：`stage-15`；leaf：`M6D06 临时 agent 历史投影（ORG-006，域层）`；执行来源 receipt：`u-bf5cbe4117b9c087c7f1`。
- 内容候选为 `274cb08629e09689357cd1522c1ad23f1aea9e08` / tree `b49f177d88b9f5a06306b093436fbc9728d2e5c9`；parent 为 `a6a7efed123b7854a9bca30f145cc587bbf40a00`。
- 用户已明确改定：Grok 是优先实现者而非唯一写者，无人值守完成整个 stage-15 是主目标。对 Grok 的本叶产品派发因私有仓库代码出站策略被系统拒绝，未产生 Grok 修改；同一长驻 Codex 随即在 current leaf 精确写域内接管，全程只有一个产品源码写者，内容提交按实际贡献只署 Codex。
- 候选精确包含 6 个允许产品路径：新增 `m6_org_temporary_agent_projection.rs`；修改 `m6_org_member_directory.rs`、`m6_org_schema.rs`、`commands.rs`、`command_registry.rs`、`lib.rs`。没有修改 M5 实现、冻结合同、前端、manifest 或项目 store。
- 本叶已按七项判据自复核放行并归档。M6D05+M6D06 的 CP3 段已收口，`leaves/` 为空且 authorization 为精确 closed；必须前台阻塞运行独立 Cursor Opus 验收，PASS 前不得进入 M6D07。

## 产品

- 新增 M6-owned 持久 TemporaryAgent 投影。refresh 只读打开既有 M5 SQLite，并从 WorkItem、Attempt、ExecutionGrant、Dispatch、RoleSession binding、authoritative RuntimeReceipt、Claim、command receipt、event 与 audit 逐层 exact join；M5 store 不被写入。
- 完整 envelope 固定包含 `project_id + orchestration_id + workflow_run_id + work_item_id + node_id + dispatch_id + attempt_id + grant_id + worker_role_session_id + authoritative receipt + trusted actor + hashes`。任一字段、绑定或 hash 缺失/不一致都 fail closed；可信 actor 必须同时匹配 grant、role binding、command receipt、event 与 audit，绝不采用 report 自报 actor。
- canonical readback hash 在消费时重算；command/event/audit 三类载体均验证。TemporaryAgentId 由完整执行 envelope 稳定派生，重复 refresh 收敛且不自动晋升。
- Task、Result、Failure、Source 四类检索只保存 source refs 与 hashes，不复制 report body。旧式缺字段 executed claim 进入 `REF_ONLY` quarantine；manual/offline claim 不被提升为执行事实，也不静默进入目录。
- runtime child 只有绑定同一 durable operation 的完整 attempt/grant/actor/receipt 才形成 `ChildRunRef`；session-looking 名称与 parent/child 关系都不能推导它。`ChildRunRef` 不创建 StableMember、TemporaryAgent 或组织层级。
- TemporaryAgent 与 StableMember 严格分型。唯一晋升入口要求显式人工动作，可新建 StableMember 或绑定既有 StableMember，保存 `promoted_from` 与 promotion binding；晋升前后逐字节核对原 TemporaryAgent 序列化 payload 未被改写。
- 普通生产调用链为 Tauri `generate_handler!` → `commands.rs` 三个真实 command → `m6_org_temporary_agent_projection` → M6-owned SQLite；refresh 的执行事实读取再经只读 M5 SQLite，promotion 分支调用 `m6_org_member_directory` 的专用显式边界。三个 command 都进入 ordinary registry，不是测试专用路径。
- `m6_org_schema` 从 v3 迁移到 v4，新增 temporary projection、quarantine、promotion binding 与 receipt 表；M5 schema/执行语义、M6D05 普通成员注册 preimage 和既有 advisory/Handoff 表未改。`lib.rs` 只增加一个 module 声明，`command_registry.rs` 只增加叶子点名的三个 command。

## 证据

原始证据根：`/home/synadmin/workspace/.syn-gates/evidence/M6D06-274cb08/`。全部命令在绑定候选 `274cb08` 的 detached disposable checkout 上执行，使用独立 `CARGO_TARGET_DIR=/tmp/syn-m6d06-target-274cb08`、隔离 app-data、合成 M5 attempts/receipts 和 observing fake runtime；没有 GUI、真实 runner、provider、账号、网络业务写或项目事实写入。`SHA256SUMS` 固定全部原始日志。

- `cargo test --lib m6d06 --offline -- --nocapture`：exit 0；8 passed / 0 failed / 0 ignored / 2149 filtered。覆盖完整 envelope 与 12 类字段/hash 缺失、report actor 自报拒绝、ChildRun exact/名称反例、显式 create/bind promotion、搜索 refs/no-body、legacy quarantine/manual ignored、ordinary command registry。
- `cargo test --lib m6d05 --offline -- --nocapture`：exit 0；7 passed / 0 failed / 0 ignored / 2150 filtered。
- `cargo test --lib m6d04 --offline -- --nocapture`：exit 0；4 passed / 0 failed / 0 ignored / 2153 filtered。
- `cargo test --lib m6d03 --offline -- --nocapture`：exit 0；13 passed / 0 failed / 0 ignored / 2144 filtered。
- `cargo test --lib m6d02 --offline -- --nocapture`：exit 0；15 passed / 0 failed / 0 ignored / 2142 filtered。
- `cargo test --lib m4c05 --offline -- --nocapture`：exit 0；9 passed / 0 failed / 0 ignored / 2148 filtered。
- `cargo test --lib m3c05 --offline -- --nocapture`：exit 0；43 passed / 0 failed / 0 ignored / 2114 filtered。六组相邻回归共 91/91，本叶连同相邻回归共 99/99。
- `cargo check --lib --offline`：exit 0；rustc 汇总 888 个既有 warnings，日志 `warning:` 文本行 889；本叶没有把 warning debt 扩成清理任务。
- `git diff --check a6a7efe 274cb08`：exit 0；`git-show-stat.log` 精确列出上述 6 个产品路径、2474 insertions / 5 deletions；frozen-contract/M5 selected-path diff 为空；验证后 disposable status 为空。
- `production-chain.log` 精确定位 ordinary registry 三项、三个 `commands.rs` Tauri wrapper 与 `lib.rs` module 声明；不是仅凭测试函数推断可达。
- `protected-wip-hashes.log` 固定 6 个受保护 `m6_*.rs`（含 `.bak`）与主工作树 `linux-schema.json` 的 7 个 SHA-256，逐项等于本叶开始前基线；它们未暂存、提交、修改、清理、恢复或用作实现输入。

主管七项判据：

1. 写域：内容候选只有 6 个 current leaf 明示路径；`command_registry.rs` 恰为三个点名 command，`lib.rs` 恰为一个 module 声明；没有 M5、前端、manifest、用户载体、后续 M6 叶或禁止域写入。
2. 冻结物：冻结合同与所选 M5 执行路径 diff 为空；ExecutionGrant、WorkerReport、receipt/audit/quarantine、guarded legacy 和 runner entry 分类均未放宽；M3C05、M4C05、M6D02–D05 共 91/91 回归通过。
3. WIP 保全：7 个受保护载体哈希与前置基线一致且未入候选；四个既有 tracked WIP 与 Harness usage/report 噪声未暂存、未归责、未清理、未覆盖。
4. 独立重跑：M6D06 8、M6D05 7、M6D04 4、M6D03 13、M6D02 15、M4C05 9、M3C05 43 个测试和 cargo check/diff-check 都在 SHA 绑定的 disposable checkout 退出 0，原始日志和校验和留在证据根。
5. 实质：三个真实 Tauri command 进入普通 handler；refresh 真实只读既有 M5 持久执行事实并写 M6 投影/quarantine，search 真实查询持久 refs，promotion 真实走显式 StableMember create/bind 边界，不是 fixture-only 或测试空转。
6. 不越级：证据只证明 WSL local/offline/synthetic 的 M6D06 域层与 ordinary Tauri composition；没有证明 GUI、新壳、真实 runner/provider/model/message/account、项目写、部署、发布、CP3 或 M6 完成。
7. 欠账：本叶标准内没有未满足项。888/889 warnings 继续归 ENG-01；renderer/new-shell consumption 归 M6S01；M6D07/M6D08 只有在 CP3 独立 PASS 后按既定叶序执行。

## 载体

- 产品载体是候选 `274cb08` 的 Rust 域层、M5 read-only/M6 ordinary composition 与三个 Tauri command，不是正在运行的 GUI、新壳、真实 provider 集成或发布产物。
- 本报告、归档 leaf、stage/plan/current-state、audit 与 closed authorization 属 Harness 记账；它们不改变候选 tree，也不代替 M6D05+M6D06 的 CP3 独立 verdict。
- 当前结论为 `M6D06 SUPERVISOR SELF-REVIEW PASS / CP3 AWAITING INDEPENDENT ACCEPTANCE / NOT RELEASED`。
