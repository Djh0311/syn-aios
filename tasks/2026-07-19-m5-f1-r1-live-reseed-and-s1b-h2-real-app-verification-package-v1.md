# 任务包：M5-F1-R1 真实存储恢复 + S1B-H2 两句 App 验收 v1

日期：2026-07-19  
状态：**已出包，待用户精确高危开工令**  
档位：**重档**（写真实 Workbench DB、替换生产 DB 文件、启动真实 App；用户必须在场）  
执行者：执行线；用户亲手执行 production apply；总指导核对实物  
所属开发线：桌面应用线 / M5 存储恢复 / 主管自然信息流验收  
上游实现包：`tasks/2026-07-19-s1b-h2-supervisor-syn-natural-information-flow-package-v1.md`  
上游案发证据：`evidence/2026-07-19-s1b-h2-real-app-message-to-proposal-failure-preregistration-v1.md`  
既有恢复先例：`evidence/2026-07-16-reseed-db-primary-restored-v2.md`  
任务包生成时 HEAD：`97fca19bc8d3effd4959dec8cc4827e27cac31e6`

## 一句话目标

先把当前 JSON-leading 的真实 Workbench 状态完整备份并经既有 R3-B1 `production_apply` 重 seed 到新 DB，证明 DB=JSON、lag=0、零新增降级；再用包含 M5-F1-R1/H2 当前源码的新 App 发送两句话，只验到正好一张新的 `pending_user_confirmation` 方案卡即停。

## 一、精确授权边界

“出任务包”只授权创建本文与权威指针，**不等于授权执行真实恢复**。开工前用户必须在场并单独发送：

> M5-LIVE-R3 开工；授权在进程和持有者全空后创建新的仓外完整备份，将当前 DB/WAL/SHM 可恢复归档，并用既有 R3-B1 production apply 从现场最新 JSON 重 seed 真实 DB。对账全绿后，授权启动由当前脏源码重新构建的真实 App，只发送 H2 两句话并验到一张 Pending 卡即停。不得修改源 JSON、storage-mode 配置、`.codex`、测试项目文件或安全闸；不得点卡、启动 chain、自动重试或自动回滚。

本授权分成两个硬停点：

1. **恢复闸**：备份、apply、静态对账和启动对账全部绿，才能进入 H2。
2. **落卡闸**：只允许新增一张 Pending 卡；不批准卡、不启动链、不派 worker。

## 二、已知事实、未知项与实施假设

### 已知事实（任务包生成时快照；开工时必须重取）

1. Workbench App、`tauri-capability-probe`、Vite 和孤儿布局测试进程均已退出；workflow-state、DB、WAL、SHM 的 `lsof` 均为空。
2. 真实 JSON `workflow-state.v0.json` 为 revision `274`、audit `1771`、workflows `8`、work_items `58`，mtime `2026-07-19 14:13:02`。
3. 真实 DB 本体 mtime `2026-07-17 15:54:25`，明显早于 JSON；storage-mode 仍声明 `db_primary_json_projection`，现场尚未恢复。
4. proposal store 为 revision `131`：总数 `74`、Pending `17`、user-confirmed `56`、rejected `1`。
5. `/Users/yoyi/workbench-backups` 只有 07-14、07-16 两代备份；尚无本轮新备份、manifest、apply report。
6. M5-F1-R1 独立定向核收 `3/3`；执行线回传 M5-A `10/10`、M5-B `9/9`、M5-C `5/5`。完整 Rust 只能记为 `1023 passed / 1 existing live fixture sandbox-blocked / 44 ignored`，不得写成全绿。
7. 现有 debug App bundle mtime `14:01:12`，早于 R1 源码 mtime `18:35:51`，**禁止用于本轮 live**。
8. exec process registry 当前 entries `0`；本轮没有 stage/commit，唯一 staged 项仍是开工前既有 mockup rename。

### 关键源码冻结 SHA-256

| 文件 | SHA-256 |
|---|---|
| `src-tauri/src/workbench_sqlite_storage_mode_m5f1.rs` | `5d248c34e6332666d4d4ae7405cbf1c12ba84e039285a61bac47c6960b18a092` |
| `src-tauri/src/workflow_db_primary_wiring.rs` | `c61ab2b93fd32d1b6e4c9780e6055dbf3dca7e5dcacdba02a1b306ff04cfc70a` |
| `src-tauri/src/supervisor_resident_oneshot_session.rs` | `86bae55ccc9cd9e1499eae9396b987ea9ef18a31c43f872ad97c0e5e79db2da3` |
| `src-tauri/src/mcp/supervisor_orchestrator_submit_proposal.rs` | `6130ee77e3b6ce4a3730fd049adc2b9bc18718ae49d2401af8d2c035d351962b` |
| `src/views/projects/jiaoban/useJiaobanConversationState.ts` | `47ac7053f55403c55d0a467703937b865c01fe001413bec81dc9776e46558bd2` |

开工前逐个复核。任何 hash 漂移若不能归属于当前主线，报 `BLOCKED_DIRTY_OVERLAP`；不得拿旧 bundle 或重新 checkout 来消除差异。

### 未知项

1. 开工时进程、文件持有者、revision、proposal/chain/thread/generation 和项目 hash 是否漂移。
2. 现场最新 source-root hash；必须由既有 production-apply/preflight 口径重新计算，不能沿用 07-16 的旧 hash。
3. 当前 DB 各表与 11 个 JSON 源的精确差异；必须在 apply 前后分别冻结。
4. 当前脏源码能否重新构建 debug App bundle；构建失败即停止，不能退回旧 App。

### 实施假设

- 沿用已有 R3-B1 `r3_b1_production_apply_confirmed_paths_requires_env_authorization` 入口，不新增脚本、迁移器、Tauri command 或 sidecar。
- apply 阶段源 JSON 严格只读；App 启动与 H2 阶段才允许通过既有 DB-primary→JSON projection 写入预期业务事实。
- storage-mode 配置已指向既有真实路径；本轮不改配置，恢复靠重建 DB 后启动对账重新变绿。
- 任何异常以保留备份和现场证据为先，不自动重跑、回滚或删配置。

## 三、交付结果

1. 一份新的仓外完整备份，含 workflow-state 全根、runtime-artifacts、storage-mode 配置、旧 DB/WAL/SHM、文件清单、size/mtime/SHA-256 与恢复说明。
2. 一次由用户亲手执行的 R3-B1 production apply，产出 report、apply-backup、export verification、rollback manifest。
3. apply 前后源 JSON hash 完全相同；新 DB 对账绿，App 启动后 DB=JSON、lag=0、零新增 `storage_mode_degraded_json_only`。
4. 由当前源码重新构建且可追溯 hash/mtime 的真实 debug App。
5. H2 两句话同一主管 thread 自然流转，正好新增一张目标匹配的 Pending 卡，chain 与测试项目不变。
6. 现场 evidence、raw evidence、`CURRENT.md` 和 catch log 完成最小回写；不 stage、不 commit。

## 四、允许读取

- `/Users/yoyi/workspace/product-line/AGENTS.md`
- `/Users/yoyi/workspace/product-line/CURRENT.md`
- 本包、H2 包、M5-F1/R1 源码与测试、07-16 恢复 evidence
- `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/**`
- `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/runtime-artifacts/**`
- `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/production-db/**`
- `/Users/yoyi/codex-workflow-mario-test/**`（只读 hash/git 状态）
- 进程表、端口、`lsof`、exec process registry 与本轮 runtime artifacts（只读）

不得读取 `.codex` 凭据、token、secret 或无关用户目录。

## 五、允许写入

### 恢复阶段

- 新建且仅使用一个明确解析后的仓外目录：
  - `/Users/yoyi/workbench-backups/workflow-state-backup-20260719-pre-reseed-<HHMMSS>/`
- 将旧 DB/WAL/SHM 在完成复制和 hash 校验后移动到上述新目录的 `stale-db/`；禁止覆盖同名文件。
- 既有 production apply 只可创建：
  - `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/production-db/workbench-state.v1.sqlite`
  - 本包新的 `evidence/raw/2026-07-19-m5-live-reseed-h2/` 下 report/apply-backup/rollback/export 文件。
- 构建产物目录：`prototypes/productized-desktop-shell/dist/`、`src-tauri/target/`；不得因此修改源码。

### H2 阶段（仅恢复闸全绿后）

只允许真实 App 经既有产品路径写入：

- 新的 `storage_mode_initialized` 及正常 DB-primary/JSON projection 审计；
- 两条用户 canonical 消息与对应主管回合事实；
- 一张新的 `pending_user_confirmation` proposal；
- 同一 resident thread/session 的必要审计与 runtime artifacts。

### 文档证据

- 新建 `evidence/2026-07-19-m5-f1-r1-live-reseed-and-s1b-h2-real-app-verification-v1.md`
- 新建 `evidence/raw/2026-07-19-m5-live-reseed-h2/**`
- 最小更新 `CURRENT.md`
- `docs/harness-catch-log.md` 仅在有新 catch 时 EOF 追加；零 catch 必须在 evidence 中明确写出。

## 六、禁止事项

1. 不得修改、补写、格式化或“修复”任何源 JSON；apply 前后必须 byte/hash 相同。
2. 不得修改或删除 `runtime-artifacts/storage-mode.v1.json`；不得用删配置绕过对账。
3. 不得覆盖或清理 07-14、07-16 旧备份；不得使用未解析变量、通配符或宽目录做移动/删除目标。
4. 不得自动 kill 未点名进程。出现持有者时先停，由用户关闭或另给精确 PID 授权。
5. 不得复用旧 bundle、H1 wrapper、副本店或 dev server 冒充真实当前 App。
6. 不得改 H2/M5 源码、安全闸、沙箱、审批、MCP allowlist、path-lock、进程组或用户执行闸。
7. 不得写 `/Users/yoyi/codex-workflow-mario-test`，不得点方案卡、批准方案、启动 chain、派 worker。
8. 不得在首句失败后发送第二句；不得通过重复发送“出方案”或自动刷新重试伪造幂等。
9. apply、启动对账或 H2 任一失败都不得自动重跑、rebase、恢复旧 DB、删配置或继续下一段。
10. 不得 stage、commit、stash、reset、clean 或处理无关 dirty 文件。

## 七、执行顺序与硬闸

### Gate 0：即时停机与脏基线

1. 再查 Workbench App、capability probe、Vite、前端测试、`cargo-tauri`、相关 MCP/codex exec；全部应为空。
2. 对 workflow-state、DB、WAL、SHM 做 `lsof`；任一持有者存在即停。
3. exec process registry 必须为空；只读冻结 `ps`、端口与登记表。
4. 冻结 HEAD、完整 `git status --short`、staged set、五个关键源码 SHA-256。
5. 冻结 JSON revision/hash/mtime/业务计数，DB/WAL/SHM hash/mtime/size，storage-mode 配置 hash。
6. 冻结 proposal 总数/Pending 数、chain 权威计数、generation/thread、测试项目全文件 hash 与 git 状态。

### Gate 1：构建当前源码 App

在 `prototypes/productized-desktop-shell/` 运行当前项目既有 Tauri CLI 的 debug build：

```text
../tauri-capability-probe/.tauri-cli/bin/cargo-tauri build --debug
```

要求：

- build 成功且不启动 App/dev server；失败即停。
- 构建后五个源码 hash 与 Gate 0 相同。
- 新 bundle binary mtime 晚于 R1 源码，冻结 binary SHA-256、mtime、size 和 build 日志。
- 构建后重新执行进程与 `lsof` 检查；不全空即停。

### Gate 2：新的仓外完整备份

1. 先解析并展示唯一新备份绝对路径，确认不存在；不得复用旧目录。
2. 复制以下内容到新备份的 `snapshot/`：workflow-state 全根、外层 runtime-artifacts、production-db 全目录、storage-mode 配置。
3. 生成 manifest：相对路径、类型、size、mtime、SHA-256；核对源/副本文件数、总字节与逐文件 hash。
4. 写明回滚源和“尚未执行 apply”；备份校验不完整即停。
5. 备份全绿后，把真实 DB/WAL/SHM 移入新备份 `stale-db/`；移动前逐个解析绝对路径，目标必须不存在。移动后确认生产 DB 三件均不存在、备份三件 hash 与移动前一致。

### Gate 3：生成 apply 命令，先回传不执行

沿用现有 ignored test：

```text
r3_b1_production_apply_confirmed_paths_requires_env_authorization
```

必须使用这些既有环境字段：

- `R3_B1_APPLY_CONFIRM=CONFIRMED_USER_PRESENT_2026_06_15`（代码既有固定确认值，不改源码）
- `R3_B1_EXPECTED_SOURCE_ROOT_HASH=<按现时源根重新计算>`
- `R3_B1_SOURCE_STATE_ROOT=/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state`
- `R3_B1_PRODUCTION_DB_PATH=/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/production-db/workbench-state.v1.sqlite`
- `R3_B1_BACKUP_ROOT=/Users/yoyi/workspace/product-line/evidence/raw/2026-07-19-m5-live-reseed-h2/apply-backup`
- `R3_B1_REPORT_PATH=/Users/yoyi/workspace/product-line/evidence/raw/2026-07-19-m5-live-reseed-h2/production-apply-report.json`
- `R3_B1_ROLLBACK_MANIFEST_PATH=/Users/yoyi/workspace/product-line/evidence/raw/2026-07-19-m5-live-reseed-h2/rollback-manifest.json`

source-root hash 必须走既有 production-apply/preflight 同口径获取；不得手写另一套 hash 算法。生成完整命令后先回传：命令、cwd、五个 confirmed path、源 hash、备份 manifest hash、生产 DB 不存在证明。**此 Gate 不执行 apply。**

### Gate 4：用户亲手执行一次 apply

总指导核对 Gate 3 后，由用户在终端亲手执行一次。要求：

- 只运行一次；非零退出或报告非 `completed` 即停。
- 不得更换 source-root hash、不加第二次 retry、不用旧 report/apply-backup。
- 冻结完整 stdout/stderr、exit code 和产物 hash。
- 验证 report：`production_apply_performed=true`、`production_db_created=true`、`source_json_written=false`、`production_root_written=false`、`codex_home_touched=false`、`read_cut_enabled=false`、`stop_write_json=false`。
- `before_source_hashes == after_source_hashes`；源根总 hash 与 Gate 3 一致。

### Gate 5：静态对账与启动对账

1. App 仍关闭时，用既有 reconcile/export 口径核 DB 与全部 JSON 源；业务关键面逐项 DB=JSON，零 conflict/hash mismatch。
2. 对账不绿时保持 App 关闭，保留新 DB 和全部证据，停止；不得自动恢复旧 DB。
3. 静态对账全绿后，用户只打开 Gate 1 构建的新 bundle。
4. 核验新增 `storage_mode_initialized` 在 DB/JSON 同笔、lag=0；`storage_mode_degraded_json_only` 总数不得增加。
5. 若启动失败、出现降级、lag 非零或 DB/JSON 再不绿：立即关闭 App，停止，不进入 H2。

### Gate 6：H2 两句话真实 App 验收

恢复闸全绿后重新冻结 proposal/Pending、chain、generation/thread、registry、项目 hash/git 状态，然后用户按顺序发送：

1. `我想给这个游戏里的标题改成小马里奥`
2. `按这个出方案`

第一句必须先证明：用户 canonical 已记录、主管自然答复已记录、回合完成、同一真实 thread 可识别。任一项缺失即停，不发第二句。

第二句必须证明：

- `supervisor_orchestrator.submit_proposal` handler 到达；
- 工具结果与主管答复回到同一 thread；
- proposal 总数恰好 `+1`，Pending 恰好 `+1`，目标与“小马里奥标题”匹配；
- chain 权威计数不变；测试项目 hash/git 状态不变；
- 普通 UI 不出现原始 stderr/MCP 参数；
- 一次普通刷新后不重复落卡；不得重发第二句。

到卡后立即停：不点卡、不批准、不启动链。

### Gate 7：收尾

1. 用户关闭 App；等待本轮 one-shot/MCP 正常退出。
2. registry 为空，`ps` 无本轮孤儿；不得清理无关既存进程。
3. 冻结最终 DB/JSON revision、proposal/Pending、chain、generation/thread、项目 hash。
4. 写 evidence/raw evidence，最小更新 CURRENT；有 catch 才向 catch log EOF 追加，零 catch 明写。
5. 不 stage、不 commit。

## 八、停止条件

出现任一项立即止损并回传，不进入下一 Gate：

- 未收到本文精确开工令；
- 进程、端口、registry 或 `lsof` 未清零；
- 关键源码 hash 漂移、旧 bundle 被使用、当前源码构建失败；
- 新备份目录已存在、manifest 不完整、源/副本 hash 不等；
- 生产 DB 三件未完整可恢复归档；
- source-root hash 不是按既有口径现场计算；
- apply 命令路径、确认值或报告路径有一项不确定；
- apply 非零、报告不完整、安全旗不绿、源 JSON hash 改变；
- 静态或启动对账不绿、出现新降级、lag 非零；
- 首句未入 canonical、主管未完成、thread 不明；
- 卡数不是恰好 +1、chain 或项目发生变化、出现第二个工具或用户批准动作；
- 需要自动重试、rebase、删配置、恢复旧 DB 或扩大真实写面才能继续。

## 九、变更辐射面与五态走查

本包不改源码，但会改变真实存储快照并写入真实对话/方案状态。

- **存储启动**：旧假设“当前 DB 可直接启动”不成立；必须先 reseed，再让启动审计恢复 DB-primary。
- **App 版本**：旧 bundle 不含 R1；必须从当前脏源码重新构建并冻结产物。
- **说**：两句话必须分别入 canonical，失败不吞对话。
- **批**：只生成 Pending 卡；用户批准仍未发生。
- **干**：不涉及；chain 必须不变。
- **交货**：不涉及。
- **卡住**：任何 Gate 失败立即关闭 App/保持现场，不自动重试或伪装成功。

## 十、形状影响

- 任务类型：高危现场恢复 + 真实 App 验收；不是功能代码任务。
- 新增代码落点：无。
- 源码预计行数变化：0。
- Tauri command / sidecar：不新增。
- 棘轮文件：不修改。
- shape gate 豁免：不需要；本包不改源码，沿用开工时冻结的历史 `13 errors / 5 warnings / 5 infos`，若出现源码 diff 则停止而不是申请豁免。
- 构建产物会更新 `dist/`、`target/`，必须与源码 diff 分开报告。
- 本任务基线 commit：`97fca19bc8d3effd4959dec8cc4827e27cac31e6`。
- 本任务完成 commit：不 commit；回传 end commit。

## 十一、验收标准

恢复通过必须同时满足：

1. 新仓外备份逐文件 hash 通过，旧 DB 三件可恢复保存。
2. production apply 一次完成，安全旗全绿，源 JSON 前后 byte/hash 相同。
3. DB 与全部 JSON 源对账绿；启动后 `storage_mode_initialized` 同笔、lag=0、零新增 degradation。
4. live 使用当前源码新 bundle，不使用旧 bundle/dev wrapper。
5. 两句话完整进入同一主管 thread，正好新增一张匹配 Pending 卡。
6. chain、测试项目文件和 git 状态不变；未点卡、未派 worker。
7. registry/`ps` 收尾无本轮孤儿。
8. `git diff --check` 仅对本包新文档/evidence 通过；源码 diff 与开工冻结一致。

## 十二、必须回传（10 项）

1. Gate 0 的进程/持有者、源码 hash、真实状态基线。
2. 新仓外备份绝对路径、文件数/字节数、manifest/hash 与旧 DB 归档位置。
3. 新 bundle 的 build 命令、hash/mtime及为何证明包含 R1/H2。
4. production apply 完整命令、用户执行事实、exit/report/safety flags。
5. 源 JSON 前后 hash、DB↔JSON 静态与启动对账、degradation/lag 结果。
6. H2 两句话的 canonical、主管答复、same-thread 和 handler 到达证据。
7. proposal/Pending `+1`、chain/项目不变、刷新不重复落卡证据。
8. 进程登记表/`ps` 收尾、App 是否已关闭、保留的恢复材料。
9. 改动文件、start/end commit、staged set；确认零代码改动、零新增 command/sidecar、未 commit。
10. 被闸拦过的事项；无新 catch 也必须写“零新增 catch”。

## 十三、总指导回收动作

- 先核备份与 apply 命令，再允许用户执行 Gate 4。
- 再核 DB=JSON/启动绿，才允许 Gate 6 两句验收。
- H2 到一张 Pending 卡后判断接受/修改/暂停；不得顺手进入底1点卡。

