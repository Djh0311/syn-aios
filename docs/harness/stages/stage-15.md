# 阶段15 M6 全局主管与内部组织（域层先行，UI 验收载体为新壳）

状态：`ACTIVE / M6_DOMAIN_LAYER_ONLY / NOT_ACCEPTED / NOT_RELEASED`。建立绑定 stage-14 关闭事实（`M5C01-20260818-1939.verdict.md`，M5 产品锚 `c91d8fc`）与用户 2026-08-18 21:49 明确“接下来就是 M6”。本阶段只做 M6 域层与其前置，不宣布 M6 完成、不发布、不激活 Headless Core / Primary / epoch。

总计划：`docs/plans/2026-08-01-syn-stage-6-global-supervisor-and-internal-organization-plan-v1.md`。载体修订依 `decisions/2026-08-17-syn-lightcode-fork-desktop-shell-direction-v1.md` 与 `docs/plans/2026-08-17-syn-lightcode-fork-shell-adoption-plan-v1.md`：M6 域层（合同、service、repository、投影）不依赖壳，先行施工；M6 的产品 UI 与隔离 App 验收载体改为新壳（`syn-shell` fork），待 F2/F3 就绪后另行进行。stage-6 计划验证矩阵里的 “Isolated Tauri” 行按新壳口径理解，域层验证矩阵不变。

输入：`handoffs/2026-08-18-syn-m5-to-m6-and-shell-deferred-debts-v1.md`。ProjectSummary 输入、TemporaryAgent/Advisory 的完整执行 envelope、`m5_runner_entry_registry` 的 `new-grant / guarded-legacy / blocked` 分类均按该交接第 1 节固定，不得放宽。

目标：先完成 M6P00 前置——把 M1 canonical `ProjectId` 的消费面扩到 workflow、项目编排与执行链的正式读写入口，并为 memory relation 的 source owner 建立可判别类型边界；再按 stage-6 计划逐叶施工 M6 域层。M6 跨项目查询不得同时消费 canonical id 与 path-derived id 两套命名空间。

当前用户边界（2026-08-18 用户确认，本阶段全程有效）：

- 以 5600X WSL `/home/synadmin/workspace/syn` 为权威仓库；不 reset、stash、clean、覆盖或丢弃既有未归属 WIP；
- 不接真实个人资料、真实用户项目写入、真实模型/provider、真实消息、账号、凭据、connector 或外部网络业务动作；产品层证据只用隔离 app-data、scratch projects、fake roles/provider/runtime 与白名单合成动作；
- 不 push、merge、rebase、部署、发布。公开 push 只由用户本人明确要求时由总指导执行；
- syn 仓库源码写面同一时间只允许一个施工者。B 线 F2 实施与本阶段的源码写面互斥，不得并行；本阶段不进入 `syn-shell` 仓库；
- 6 个未跟踪 `m6_*.rs`（含 `m6_member_directory.rs.bak`）与 `gen/schemas/linux-schema.json` 只读保全，不得暂存、清理、恢复，也不得升格为 M6 基线或实现输入。

干完的标准：

- M6P00 前置达到自身标准并通过独立验收；
- 其后每个 M6 域层叶各自独立内容提交与定向证据，逐项进入 done；任一实现不得冒充整阶段完成；
- 每叶到节点时 authorization 回精确 closed，并在仓库外 `/home/synadmin/workspace/.syn-gates/open/` 写节点请求文件等独立验收；
- `git diff --check` 通过；M6 写面零未知 delta；stage-12、D0C04/D0C05、M1–M5 冻结合同全程只读保全；
- 阶段关闭另需独立验收结论明确放行，不由本阶段任一叶自行宣布。

允许动：

- `docs/harness/authorization.json`、`docs/harness/plan.md`、`docs/harness/stages/stage-15.md`、`docs/harness/leaves/`、`docs/harness/unfinished/`、`docs/harness/audit/2026-08.jsonl`、`docs/harness/usage/.turn`、`docs/harness/reports/M6*`
- `docs/current-state.md`
- `docs/contracts/`（仅新增增补合同，不改冻结合同正文与旧 hash）
- `docs/plans/2026-08-01-syn-stage-6-global-supervisor-and-internal-organization-plan-v1.md`（仅如实记进度与载体口径）
- `tasks/2026-08-*`
- 产品源码：仅当前 current leaf 明确列出的写域

不许动：

- stage-12、D0C04、D0C05 与 `unfinished/D0C04`、`unfinished/D0C05`（只读保全）
- M1–M5 冻结合同正文与旧 hash；如需补充只能新建增补合同
- M5 已接受的执行合同语义：ExecutionGrant、WorkerReport、receipt/audit/quarantine 边界不得放宽，guarded legacy 不得升格
- OSS-01 与用户自有门面载体（`README.md`、`LICENSE`、`CONTRIBUTING.md`、`SECURITY.md`、`package.json` 与 `src-tauri/Cargo.toml` 的 license/repository 字段，已由 `c1025ba` 独立提交）
- `syn-shell` 仓库、F2/F3/F5 实施、壳采纳
- M7–M11、Headless Core、Primary/authority epoch 激活或实现
- 真实资料/项目写入、真实模型/provider/message/connector、凭据、外部网络业务写、push/merge/rebase/deploy/release
- 伪造 Hook receipt、authorization、stage/leaf、测试或 App 证据

停止与回滚：

- 出现 secret/credential/真实运行数据/未知 symlink/special file、要求伪造证据、M1–M5 冻结合同或 stage-12/D0C04/D0C05 意外修改、候选 commit 与新鲜证据 SHA 不一致、`syn-shell` 写面被触碰时立即停止并交总线；
- authorization 保持精确 closed 两字段，不手填 executionReceipt/session/turn/expiresAt；每次 leaf 切换先 closed 再按真实 receipt 重新签发，禁止旧 active JSON 跨 leaf 续用。

## 叶子

- [ ] M6P00 canonical ProjectId 消费扩面与 relation owner 类型化前置（current）
