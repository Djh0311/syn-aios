# 阶段15 M6 全局主管与内部组织（域层先行，UI 验收载体为新壳）

状态：`CLOSED / M6_DOMAIN_LAYER_SCOPED_PASS / NOT_RELEASED`。总指导 2026-08-20 02:54 决定关闭，主管代执行记账。

关闭依据：M6P00 独立检查点 PASS（`stage-15-m6p00-20260819-0342`）；CP1 / CP2 / CP3 独立 verdict PASS（`stage-15-cp1-20260819-0521`、`stage-15-cp2-20260819-0733`、`stage-15-cp3-20260819-0924`）；最终域层候选 `a3d575975033f7eb5ec92dab18c24fe97ddb8001` 获阶段终包独立 verdict **PASS**（`stage-15-20260819-1123`）。放行只到 M6 **域层**（本地离线合成证据链）：合同冻结、Global Supervisor RoleSession、只读跨项目 query/advisory、Secretary consult Handoff、稳定成员目录、临时 agent 历史投影、多视角会诊与域层集成回归。**不构成完整 M6**：ORG-007 双项目隔离 App 验收与顶层入口 UI 在 `unfinished/M6S01`（载体为新壳，待 F3）；真实 provider/model/账号、发布、部署、真实日用均未成立。verdict 点名欠账已分流至 ENG-01 / M6S01 / ACC-01，不随关闭消失。

关闭处置（2026-08-20）：本文件由 `docs/harness/stages/` 原子移动至 `docs/harness/done/2026-08/`。以下为关闭时点的原文存档。

---

原状态行（存档）：`ACTIVE / M6_DOMAIN_LAYER_CANDIDATE / PENDING_INDEPENDENT_STAGE_VERDICT / NOT_RELEASED`。建立绑定 stage-14 关闭事实（`M5C01-20260818-1939.verdict.md`，M5 产品锚 `c91d8fc`）与用户 2026-08-18 21:49 明确“接下来就是 M6”。M6P00 与 M6D01–M6D08 已逐叶主管自复核，最终域层内容锚为 `a3d575975033f7eb5ec92dab18c24fe97ddb8001`；本阶段仍须经最终独立 verdict 才能关闭。本阶段只做 M6 域层与其前置，不宣布完整 M6 完成、不发布、不激活 Headless Core / Primary / epoch。

总计划：`docs/plans/2026-08-01-syn-stage-6-global-supervisor-and-internal-organization-plan-v1.md`。载体修订依 `decisions/2026-08-17-syn-lightcode-fork-desktop-shell-direction-v1.md` 与 `docs/plans/2026-08-17-syn-lightcode-fork-shell-adoption-plan-v1.md`：M6 域层（合同、service、repository、投影）不依赖壳，先行施工；M6 的产品 UI 与隔离 App 验收载体改为新壳（`syn-shell` fork），待 F2/F3 就绪后另行进行。stage-6 计划验证矩阵里的 “Isolated Tauri” 行按新壳口径理解，域层验证矩阵不变。

输入：`handoffs/2026-08-18-syn-m5-to-m6-and-shell-deferred-debts-v1.md`。ProjectSummary 输入、TemporaryAgent/Advisory 的完整执行 envelope、`m5_runner_entry_registry` 的 `new-grant / guarded-legacy / blocked` 分类均按该交接第 1 节固定，不得放宽。

目标：先完成 M6P00 前置——把 M1 canonical `ProjectId` 的消费面扩到 workflow、项目编排与执行链的正式读写入口，并为 memory relation 的 source owner 建立可判别类型边界；再按 stage-6 计划逐叶施工 M6 域层。M6 跨项目查询不得同时消费 canonical id 与 path-derived id 两套命名空间。

叶子（全部归档于 done/2026-08/）：

- [x] M6P00 canonical ProjectId 消费扩面与 relation owner 类型化前置（内容 `4147454`、记账 `cf1cb25`；独立检查点 `stage-15-m6p00-20260819-0342` PASS）
- [x] M6D01 跨项目与成员合同冻结（ORG-001；内容 `80ddebd`）→ CP1
- [x] M6D02 顶层 Global Supervisor 持久 RoleSession（ORG-003；内容 `651a8fb`）→ CP1 独立 verdict PASS
- [x] M6D03 只读跨项目 query 与 advisory（ORG-002；owner 前置 `977770f`、内容 `60a8e19`）→ CP2
- [x] M6D04 Secretary consult Handoff（ORG-004；内容 `ec1ba99`）→ CP2 独立 verdict PASS
- [x] M6D05 稳定成员目录（ORG-005；内容 `a58815f`）→ CP3
- [x] M6D06 临时 agent 历史投影（ORG-006；内容 `274cb08`）→ CP3 独立 verdict PASS
- [x] M6D07 独立多视角会诊（ORG-006A；内容 `15bd053`）
- [x] M6D08 M6 域层集成回归与阶段候选（内容 `a3d5759`）→ 阶段终包独立 verdict `stage-15-20260819-1123` PASS

明确不在本阶段：ORG-007 双项目隔离 App 验收与顶层入口 UI，载体在新壳，记在 `unfinished/M6S01-dual-project-isolated-app-acceptance-on-new-shell.md`。因此本阶段通过也只到 M6 域层，不构成 M6 完成。

文件名纪律（存档）：6 个未跟踪 `m6_*.rs` 占用了 `m6_cross_project_query.rs`、`m6_global_supervisor_session.rs`、`m6_member_directory.rs`（含 `.bak`）、`m6_organization_identity.rs`、`m6_temporary_agent_history.rs` 这几个名字。本阶段新增域层源码统一用 `m6_org_*` 前缀，禁止使用上述任一名字。该纪律对后续阶段继续有效，直至 ENG-01#12 裁定那 6 个文件去处。

其余边界、检查点纪律与停止条款为施工期条款，随关闭失效，原文见 git 历史。
