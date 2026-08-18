# M6D02 顶层 Global Supervisor 持久 RoleSession（ORG-003，域层）

阶段：stage-15 M6 全局主管与内部组织（域层先行，UI 验收载体为新壳）

状态：`CURRENT` / `NOT_STARTED`。stage-15 检查点 CP1 的第二叶。前置 M6D01 内容 `80ddebdf17889035bc7acde423e32ad6de6f17bb` 已获主管自复核 PASS；本叶做完即到 CP1，必须收口交包并由同一 Codex 前台阻塞独立验收。

来源收据：stage-6 计划第 4 节 SYN-ORG-003、第 3 节 `GlobalSupervisorSession` 不变量（global scope、只读默认、provider handle 非授权）；M6D01 冻结的合同为唯一判据；用户 2026-08-18 22:41 的排叶与检查点要求。

目标：在 M3 既有 RoleSession 之上建立顶层全局主管的持久会话，默认只读、global scope、与项目主管和秘书会话严格分离，并且在普通产品路径上真有消费者。本叶不做跨项目查询、不做 advisory、不做前端页面。

做完的标准：

1. 新增 `m6_org_global_role_session.rs`，在 M3 RoleSession 之上建立 global scope 会话；持久化必须走 M3 既有 repository / schema 机制，**不得新建平行内存结构**（这是 M6 旧原型的已知病，见 plan.md 中 M6 候选原型盘点）；
2. 会话默认只读：会话本身不携带任何项目写能力，provider handle 不构成授权；任何试图用 global 会话直接写项目的路径必须编译期或运行期 fail-closed；
3. 会话上下文只含最小 summaries 与 refs 的容器边界，不含原始项目文件、transcript、secret、未裁剪 memory；
4. 与 Project Supervisor / Secretary 会话严格分离：scope 判别基于可判别类型或显式字段，不靠名称巧合；项目主管会话不能被当作 global 会话使用（反例测试）；
5. **真实生产消费者**：至少一个真实 Tauri command 在 `commands.rs` 注册、在 `lib.rs` 接入 `AppState`，普通启动路径可达。不得只存在于 `#[cfg(test)]`、env 门控或 fixture；报告须指出从普通 entrypoint 到本叶实现的完整调用链；
6. 重启后同一会话解析为同一身份（不靠路径派生、不靠自动重建）；来源缺失或损坏时 fail-closed，无静默 fallback、默认值兜底或自动导入；
7. 定向测试覆盖：持久化往返、重启同一身份、scope 隔离反例、只读默认、损坏/缺失 fail-closed；
8. `cargo check --lib --offline` 与本叶定向测试在 disposable checkout 上通过，记录真实 passed / failed 与退出码，证据绑定候选 SHA；
9. 独立内容提交，写域精确，`git diff --check` 通过；
10. 本叶做完即到 **CP1 检查点**：主管自复核放行并收口后，把 authorization 打回精确 closed，在 `/home/synadmin/workspace/.syn-gates/open/` 写 `stage-15-cp1-<YYYYMMDD-HHMM>.md` 交包；同一长驻 Codex 前台阻塞启动零上下文 Cursor Opus 验收官并每两分钟心跳。PASS 才处理交包、分流欠账并进入 M6D03；FAIL 只按点名范围返修。同一检查点连续两次 FAIL、verdict 缺失或首行不可读则写 halt 交包并停止。不得绕过 CP1。

证据：只在 disposable checkout 上产出定向证据，绑定候选 SHA。本叶不做 GUI、不做窗口截图、不接真实 provider 或账号。

允许动：

- `prototypes/productized-desktop-shell/src-tauri/src/m6_org_global_role_session.rs`（新建）
- `prototypes/productized-desktop-shell/src-tauri/src/m6_org_schema.rs`、`m6_org_store.rs`、`m6_org_dto.rs`（新建，M6 域层自有持久化与 DTO）
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`（仅 `mod` 声明、`AppState` 接线与 command 注册）
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`（仅本叶 command 接线）
- `m3_role_session.rs`、`m3_role_session_repository.rs`、`m3_role_session_schema.rs`：**仅**可见性调整（如 `pub(crate)`）与新增 trait 实现，不改既有语义、不改既有字段含义；每一处此类改动须在本叶报告里逐条列出并说明为何不可避免
- `docs/contracts/`（仅新增增补合同）
- `tasks/2026-08-*`、`tasks/2026-08-19-*`
- `docs/harness/stages/stage-15.md`、`docs/harness/leaves/`、`docs/harness/unfinished/`、`docs/harness/done/2026-08/`、`docs/harness/authorization.json`、`docs/harness/audit/2026-08.jsonl`、`docs/harness/reports/M6D02-*`、`docs/harness/plan.md`、`docs/current-state.md`

不许动：

- 跨项目查询、advisory、成员目录、临时 agent、会诊（分属后续叶）
- 直读项目 store / projection / project root；跨项目读取只允许后续叶经 M5 `ProjectSummaryQueryPort`
- M1–M5 冻结合同正文与旧 hash；M5 已接受执行语义（ExecutionGrant、WorkerReport、receipt / audit / quarantine 不放宽，`m5_runner_entry_registry` 分类不改判）
- 6 个未跟踪 `m6_*.rs`（含 `.bak`）与 `gen/schemas/linux-schema.json`：只读保全，不暂存、不清理、不恢复、不作实现输入，**不得被同名新文件覆盖**
- 前端源码、页面布局、旧壳 UI、`syn-shell` 仓库、F2/F3/F5、壳采纳
- stage-12、`unfinished/D0C04`、`unfinished/D0C05`、`OSS-01`、用户自有载体
- 真实凭据 / provider / 模型 / 账号 / 个人资料 / 外部网络业务写
- 自行关闭 stage-15、宣布 M6 完成、跳过 CP1、越过检查点继续下一叶
- 伪造 receipt、authorization、stage/leaf、测试或 App 证据
- push、merge、rebase、部署、发布
