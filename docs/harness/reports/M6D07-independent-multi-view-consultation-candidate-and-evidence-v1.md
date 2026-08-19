# M6D07 independent multi-view consultation candidate and evidence v1

状态：`SUPERVISOR_SELF_REVIEW_PASS / M6D08_ACTIVE / DOMAIN_RUNTIME_ONLY / NOT_RELEASED`

## Harness

- stage：`stage-15`；leaf：`M6D07 独立多视角会诊（ORG-006A，域层）`；执行来源 receipt：`u-bf5cbe4117b9c087c7f1`。
- 内容候选为 `15bd053cccbaee1302d244afc84eb05578e7fa1c` / tree `1efd6cfd5cd02e2b3510acae9fb200f102b8b5a7`；parent 为 `a3ec73307029a7531345d45a080fcfb11a5a013d`。
- 用户已明确改定：Grok 是主要/优先实现者而非唯一写者，无人值守完成整个 stage-15 是主目标。对 Grok 的本阶段产品派发因私有仓库代码出站策略被系统拒绝，未产生 Grok 修改；同一长驻 Codex 在 current leaf 精确写域内接管，全程只有一个产品源码写者，内容提交按实际贡献不署 Grok。
- 候选精确包含 5 个允许产品路径：新增 `m6_org_multi_view_consultation.rs`；修改 `m6_org_schema.rs`、`commands.rs`、`command_registry.rs`、`lib.rs`。没有修改 M3/M5 实现、冻结合同、前端、manifest 或项目 store。
- 本叶已按七项判据自复核放行并归档。它与 M6D08 同属最后一段，因此不停在单叶检查点；M6D08 随即成为唯一 current leaf。M6D08 完成前不得写阶段交包或宣布 M6 域层/stage-15 完成。

## 产品

- `start` 对 `ROUTINE` 显式保持 single-role，不创建 M6 consultation 或额外 M3 RoleSession；只有 `CROSS_PROJECT_CONFLICT`、`IRREVERSIBLE_DECISION`、`HIGH_IMPACT_RISK` 三类显式重大触发才进入多视角。
- 多视角要求 2–4 个互不重复的 view kind，并为每个 view 创建真实、独立、active 的 M3 RoleSession；RoleSession、Workcell 与 context packet refs 三组均须唯一。所有 view 收到同一个按来源 ref 排序、带 SHA-256、`minimal=true` 的 question packet。
- submit 前的持久查询只加载 consultation header 与目标 view 单行，不能返回 peer payload 或 peer conclusion；提交请求如自报可读 peer、换绑 RoleSession/Workcell/context/question packet、夹带未知字段或引用 question packet 外证据都会 fail closed，且提交计数保持零。
- runtime final answer 只以 ref + SHA-256 进入 M6-owned consultation view；不保存 answer body。每条 claim 的 evidence refs 必须来自原 question packet。全部 view 提交前主动 assemble 仍返回 `IN_FLIGHT`，共识、分歧、证据索引与用户决定项均为空。
- 只有全部独立 view 提交后才确定性生成 consensus、disagreement 与 evidence index；结果唯一进入 `PENDING_USER_DECISION`。consultation 与 decision 均固定 `produces_command=false`、`produces_grant=false`、`produces_fact=false`，不生成项目命令、授权或正式事实。
- 总预算、每 view cap 与 deadline 都是持久显式边界。单 view 或累计超限转为 `BUDGET_EXCEEDED`；超时转为 `TIMED_OUT`；两者均不提交触发 view、不组装部分结果、不创建 decision request。
- M6 SQLite 的 start/submit/assemble、revision CAS、command receipt 与 audit 均持久；精确 replay 收敛，idempotency collision 与 revision drift 可判别。产品代码不打开项目 store、M5 store、project root，也不调用真实 runtime/provider/model。
- 普通生产调用链为 Tauri `generate_handler!` → `commands.rs` 的 `start_global_supervisor_multi_view_consultation` / `submit_global_supervisor_consultation_view` / `assemble_global_supervisor_multi_view_consultation` → `m6_org_multi_view_consultation` → 已安装的 M3 repository 与 M6-owned SQLite。三个 command 都进入 ordinary registry，不是测试专用路径。
- `m6_org_schema` 从 v4 迁移到 v5，新增 consultation、view、pending-decision 与 command-receipt 四张表；`lib.rs` 只增加一个 module 声明，`command_registry.rs` 只增加叶子点名的三个 command。

## 证据

原始证据根：`/home/synadmin/workspace/.syn-gates/evidence/M6D07-15bd053/`。全部命令在绑定候选 `15bd053` 的 detached disposable checkout 上执行，使用独立 `CARGO_TARGET_DIR=/tmp/syn-m6d07-target-15bd053`、隔离 app-data、合成 ordinary product seeds、真实 M3 持久 repository 与 fake/ref-only consultation output；没有 GUI、真实 runner/model/provider、账号、网络业务写、项目事实写或真实成本。`SHA256SUMS` 固定全部原始日志。

- `cargo test --lib m6d07 --offline -- --nocapture`：exit 0；8 passed / 0 failed / 0 ignored / 2157 filtered。覆盖 routine single-role、独立 M3 sessions/workcells/contexts、串台与换绑反例、部分 assemble、完整 sourced indexes、budget、timeout、replay/no-body 与 ordinary command chain。
- 相邻回归全部 exit 0：M6D06 8/8、M6D05 7/7、M6D04 4/4、M6D03 13/13、M6D02 15/15、M4C05 9/9、M3C05 43/43，共 99/99；连同本叶为 107/107。
- `cargo check --lib --offline`：exit 0；rustc 汇总 888 个既有 warnings；本叶初次主树检查暴露并删除了一个仅本叶引入的 unused helper，最终没有增加 warning 数，也没有把仓库 warning debt 扩成清理任务。
- `git diff --check a3ec733 15bd053`：exit 0；`git-show-stat.log` 精确列出上述 5 个产品路径、2334 insertions / 1 deletion；frozen-contract/selected M3-M5 path diff 为空。
- cargo 在 disposable checkout 生成了一个未跟踪 Tauri `gen/schemas/linux-schema.json`；它只在该 disposable checkout 中被精确删除后重取 status，最终 `git status --porcelain=v1 -uall` 为空。主工作树同名受保护文件未改、未暂存、未清理。
- `production-chain.log` 精确定位 ordinary registry 三项、三个 `commands.rs` Tauri wrapper 与 `lib.rs` module 声明；不是仅凭测试函数推断可达。
- `protected-wip-hashes.log` 固定 6 个受保护 `m6_*.rs`（含 `.bak`）与主工作树 `linux-schema.json` 的 7 个 SHA-256，逐项等于本叶开始前基线；它们未暂存、提交、修改、清理、恢复或用作实现输入。

主管七项判据：

1. 写域：内容候选只有 5 个 current leaf 明示路径；`command_registry.rs` 恰为三个点名 command，`lib.rs` 恰为一个 module 声明；没有 M3/M5、前端、manifest、用户载体、M6D08 或禁止域写入。
2. 冻结物：冻结合同与所选 M3/M5 路径 diff 为空；ExecutionGrant、WorkerReport、receipt/audit/quarantine、guarded legacy 与 runtime admission 均未放宽；M3C05、M4C05、M6D02–D06 共 99/99 回归通过。
3. WIP 保全：7 个受保护载体哈希与前置基线一致且未入候选；四个既有 tracked WIP 与 Harness usage/report 噪声未暂存、未归责、未清理、未覆盖。
4. 独立重跑：M6D07 8 项、相邻 99 项、cargo check、diff-check、frozen-diff、ordinary chain 与 disposable-clean 都在 SHA 绑定 checkout 退出 0，原始日志和校验和留在证据根。
5. 实质：三个真实 Tauri command 进入普通 handler；多视角使用真实持久 M3 RoleSession 与 M6 SQLite，不是 fixture-only 或仅文档声明。查询级隔离、换绑反例、项目文件 hash baseline、预算/超时与部分 assemble 反例均由执行测试证明。
6. 不越级：证据只证明 WSL local/offline/synthetic 的 M6D07 域层与 ordinary Tauri composition；没有证明 GUI、新壳、真实 runtime/provider/model/message/account、项目写、部署、发布、阶段验收或 M6 完成。
7. 欠账：本叶标准内没有未满足项。888 warnings 继续归 ENG-01；18 个 M6 command 的 renderer/new-shell consumption 归 M6S01；全量写面、legacy 回滚、startup/gate/schema-carrier/provider-swap 集成债按既定路由在 M6D08 结算。

## 载体

- 产品载体是候选 `15bd053` 的 Rust 域层、M3/M6 ordinary composition 与三个 Tauri command，不是正在运行的 GUI、新壳、真实 provider 集成或发布产物。
- 本报告、归档 leaf、stage/plan/current-state、audit 与 M6D08 authorization 属 Harness 记账；它们不改变候选 tree，也不代替最终 stage-15 独立 verdict。
- 当前结论为 `M6D07 SUPERVISOR SELF-REVIEW PASS / M6D08 ACTIVE / DOMAIN RUNTIME ONLY / NOT RELEASED`。
