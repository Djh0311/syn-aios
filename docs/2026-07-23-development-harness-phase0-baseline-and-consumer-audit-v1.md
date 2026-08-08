# 开发 Harness Phase 0：基线与 consumer 审计 v1

> 资料状态（2026-08-09）：已退出的旧开发护栏基线审计，只按历史证据阅读。当前施工规则看 `AGENTS.md` 和轻量开发护栏；本文不定义 Syn 产品，也不提供当前授权。

日期：2026-07-23
性质：只读基线 / consumer 分类；不改变 Harness 运行语义，不授权删除、隐藏、改名、接 Hook 或修改业务代码。
对应：[整改执行计划 v1](plans/2026-07-23-development-harness-routing-code-map-and-authority-governance-remediation-plan-v1.md) Phase 0；[运行模型决策](../decisions/2026-07-23-development-harness-operating-model-v1.md)。

## 1. 冻结快照

| 项 | 基线事实 |
| --- | --- |
| HEAD | `e9ad7f3a204a1ebb11ce26c1e8c05b19c04c0991` |
| staged | 空；`git diff --cached --name-status` 的 SHA-256 为 `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| 工作树 | 以 `git status --porcelain=v1` SHA-256 `dfb0e2f12d1862cec2d7e8ead04d1f95c77bc81fc5deced503f6ddf6e47c5521` 为冻结边界；其中 `AGENTS.md`、`AUTHORITY.md`、`CURRENT.md` 已有并行业务改动 |
| Harness 文件 | 95 个 tracked 文件：88 JS（72 顶层 + 16 `lib/`）、5 JSON、2 Swift；文件清单 SHA-256 `94b040f86babd9171a61443296f562d17931d41e156363d4edbccac82526380e`，内容 manifest SHA-256 `25e446b1bfa90382b98184f90de43476f192d30eb24b64605a05a8f8eb478a81` |
| Harness 脏改 | `scripts/harness/**`、`harness.config.json`、`.githooks/**`、`docs/harness-catalog.md` 均无基线时未提交改动 |
| CLI | `harness.js --help` 显示 35 个命令；脚本 SHA-256 `eb78449528bc0a532ef7d78a9ff0e782d709d5d75cdc20e4f9917b113513c9ab` |
| config | `harness.config.json` SHA-256 `b91dd24b941a7eba69af78bceea8e1e71b83576219a6475acbb0e52eb9616b63`；`balanced`、允许 dirty、CI 非必需、AgentMemory 关闭、managed Hook 关闭 |
| 实际 Hook | `core.hooksPath=.githooks`；仅 `.githooks/commit-msg`，SHA-256 `716d6cc3cf4d533e8c2fd2792eb0e9f50f69bd7741ef0bfe83add586d3413229`，只要求提交信息包含 `catch:` |

冻结命令：

```bash
git rev-parse HEAD
git status --porcelain=v1 | shasum -a 256
git diff --cached --name-status | shasum -a 256
rg --files scripts/harness | sort | shasum -a 256
rg --files scripts/harness | sort | while IFS= read -r f; do shasum -a 256 "$f"; done | shasum -a 256
node scripts/harness/harness.js --help
git config --get core.hooksPath
git ls-files .githooks
```

## 2. 真实 consumer 面，而不是历史文本计数

| consumer 面 | 当前事实 | 处置 |
| --- | --- | --- |
| `.githooks/commit-msg` | 当前唯一真实接线，机械要求 `catch:` | 保留；本整改不改它 |
| `harness.js --help` | 默认展示 35 条命令，含 memory/task/evidence lifecycle 与旧 `capabilities` | Phase 5 前保持兼容；不据此推断每项都活跃 |
| `harness.config.json` | pre-work 7 项、pre-completion 9 项及 strict 变体是配置的手动聚合清单 | `AGENTS.md` 已规定 Harness 默认关闭，因此不能误报为每任务自动路径 |
| `templates/hooks` / `templates/ci` | 只是未启用模板 | consumer-first 时需检查，但目前不是实际接线 |
| `workbench-shape-gate.js` / `stage-k-architecture-gate.js` | 任务按需直调的承重/专项工具 | 保持显式；不能被短路由或 Code Map 接管 |
| 历史 task/evidence/handoff 文本 | 大量命中仅记录过去曾运行或曾计划运行的命令 | 仅作兼容与迁移线索，不视作 current consumer |

本轮发现 8 个历史文本仍引用不存在的脚本：`ad-policy-check.js`、`agent-entrypoint-check.js`、`duplicate-code-check.js`、`harness-observation-installed-lifecycle.test.js`、`harness-observation.js`、`lib.js`、`predev-check.js`、`scope-check.js`。Phase 0 仅登记，不能顺手修历史或新建同名脚本。

## 3. 95 个文件的结构分类与保留理由

以下分类描述**当前可观察的结构与 consumer 线索**，不是尚未实施的 `activeBoundary` 配置，也不表示某项业务能力已验收。各组恰好覆盖 95 个 tracked 文件。

| 组 | 数量 | 文件 / consumer 面 | 当前保留理由 |
| --- | ---: | --- | --- |
| 根 CLI target | 35 | `harness.js` 当前映射的 35 个脚本（详见下节） | 已暴露的兼容表；Phase 5 前不得先删或静默改义 |
| 顶层显式 / 直接入口 | 32 | `browser-evidence-check.js`、`capability-scan.js`、`checkpoint-audit.js`、`ci-gate.js`、`ci-validate.js`、`config-check.js`、`config-migrate.js`、`config-schema.js`、`context-pack.js`、`evidence-check.js`、`evidence-command.js`、`evidence-freshness.js`、`fixture-check.js`、`git-gate.js`、`guard-state-files.js`、`harness.js`、`hook-uninstall.js`、`install-harness.js`、`installed-health.js`、`managed-files-audit.js`、`mcp-doctor.js`、`mistake-check.js`、`mistake-new.js`、`rules-lint.js`、`runtime-docs-diff.js`、`self-test.js`、`stage-k-architecture-gate.js`、`stale-control-check.js`、`status-snapshot.js`、`sync-harness.js`、`ui-verify.js`、`workbench-shape-gate.js` | 可由任务显式调用、被 config/catalog/历史资料提及，或是承重 gate；consumer 审计完成前统一保留 |
| 配套自测 | 5 | `checkpoint-audit.selftest.js`、`workbench-shape-gate.dedup.selftest.js`、`workbench-shape-gate.hardcoded-hex.selftest.js`、`workbench-shape-gate.machine-face.selftest.js`、`workbench-shape-gate.retired-style-family.selftest.js` | 锁定对应检查的机械行为；不能因默认入口收缩而丢失 |
| 内部库 | 16 | `lib/agentmemory-client.js`、`lib/check-runner.js`、`lib/config-loader.js`、`lib/context-pack.js`、`lib/evidence-audit.js`、`lib/hardcoded-hex-rule.js`、`lib/machine-face-rule.js`、`lib/manifest.js`、`lib/memory-governance.js`、`lib/mistake-retrieval.js`、`lib/project-kind.js`、`lib/retired-style-family-rule.js`、`lib/risk-classifier.js`、`lib/security.js`、`lib/skill-index.js`、`lib/task-package-schema.js` | `require` 依赖或规则本体；必须先反查引用，不能按顶层命令可见性删除 |
| eval fixture | 5 | `eval/cases/context-pack.json`、`eval/cases/memory-governance.json`、`eval/cases/mistake-retrieval.json`、`eval/cases/security/prompt-injection.json`、`eval/cases/skill-recommend.json` | `eval-runner.js` 的固定输入，保留以避免测试表面存在而无夹具 |
| Stage K probe | 2 | `stage-k-cgevent-click.swift`、`stage-k-screencapturekit-window-capture.swift` | 历史平台探针；当前零自动 consumer 也不等于可安全删除 |

### 根 CLI 的 35 条兼容入口

| 家族 | 命令数 | 当前为何保留到 Phase 5 |
| --- | ---: | --- |
| 聚合 / 画像 / policy | `doctor`、`pre-work`、`pre-completion`、`profile`、`policy`（5） | 已公开 router 语义；后续只能缩 help，不能破坏直调 |
| init | `init config`、`init docs`、`init hooks`、`init ci`（4） | 仍有模板与安装兼容 consumer；当前不启用不等于可删 |
| mistake | `mistake query`（1） | 保持已有查询路径，后续再按当前规则判断可见性 |
| AgentMemory | 7 个 `memory …` 命令 | config 已关闭且 catalog 标 legacy，但必须先迁模板/历史 consumer |
| task lifecycle | `task start`、`task finish`、`task status`、`task risk`、`task package new`、`task package lint`（6） | 旧 lifecycle 不再是默认流程，但保留命令兼容，不能恢复为新任务必经线 |
| evidence lifecycle | `evidence new`、`evidence retention`、`evidence compact`、`evidence index`、`evidence query`（5） | 同上：历史兼容，不等于每个普通改动都需新 evidence |
| utility / verify | `skill recommend`、`security scan`、`eval`、`verify plan`、`verify run`、`verify suite`（6） | 显式工具与已有调用面，待 consumer-first 审计后才改默认展示 |
| old capability | `capabilities`（1） | 当前仍指向本机能力的 `capability-map.js`；保留直到新 `codebase-map` 完成且 legacy 入口有清晰 deprecation |

## 4. Phase 5 的预分类，不是当前行为变更

| 目标边界 | 已有候选 | 不可越过的边界 |
| --- | --- | --- |
| `mechanical` | `commit-msg catch:`、`workbench-shape-gate.js` 与三条规则库 / 配套自测、config 低层检查 | 不接管 Code Map、计划语义或产品验收 |
| `reportingOnly` | `checkpoint-audit.js` | 字段和 git 对账不等于业务完成判断 |
| `explicitTool` | Stage K、UI/browser evidence、短路由 diagnostic、未来 Code Map | 只由任务需要时人工运行 |
| `legacyIgnored` | memory、task/evidence lifecycle、runtime-doc init、init hooks/CI、旧 capability-map/scan | 先审 consumer、保留兼容期，再决定隐藏或删除 |

## 5. 已验证的漂移与停止线

- `node scripts/harness/config-policy.js --target . --strict --json`：通过。
- `node scripts/harness/config-check.js --target . --strict --json`：通过。
- `node scripts/harness/config-schema.js --target . --strict --json`：失败；仍要求缺失的 `autoRisk`、`verificationRunner`、`taskLifecycle`。这是既有 schema/配置漂移，不可被前两项通过掩盖；Phase 5 必须先处理兼容合同。
- managed Hook 已关闭，但 `.githooks/commit-msg` 实际生效；两种事实不能合并成“Hook 全关”。
- 旧 `docs/harness-catalog.md` 仍按 87 个 JS 叙述，而当前实际为 88 个 JS；它是待 Phase 5 修正的 consumer/audit 输入，不在 Phase 0 偷改。

若后续阶段必须修改当前已 dirty 的 `AGENTS.md`、`AUTHORITY.md`、`CURRENT.md`，必须先冻结文件 hash 与既有 hunk 归属，只允许当次授权范围内的非覆盖式最小合并；无法证明不覆盖、目标文件发生漂移，或需要扫描 / 修改全量 dirty 才能工作时，立即按计划记录 `BLOCKED_DIRTY_OVERLAP` 或 `BLOCKED_FALSE_GATE`，不靠放宽规则取得绿灯。

## 6. Phase 1 准入补正与冻结边界

复核日期：2026-07-23。

复核结论：`PHASE0_EXECUTION_PASS / PHASE1_ENTRY_READY_FOR_USER_DISPATCH`。这只表示 Phase 0 的交付与 Phase 1 前置口径已经闭合；Phase 1 尚未获得实施授权，也没有创建任何短路由或 Code Map 文件。

### 6.1 补正前冻结值

- `HEAD=e9ad7f3a204a1ebb11ce26c1e8c05b19c04c0991`
- staged diff SHA-256（空）：`e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`
- `git status --porcelain=v1` SHA-256：`768035557205a2f573ad1a5fadebe15299be5c3d3ef4bdb8f6cfe806b54deddb`
- 补正前 `AGENTS.md`：`8ceaf7a1fc13d777c1b0170282b78434a8d9c19c1c2e79f6400a52b18336ca94`
- 补正前 `AUTHORITY.md`：`679435b6f0ea6e59d23aec62f1430b32de7ff02914c56e739551fcab990f415a`
- 补正前 `CURRENT.md`：`1c13316f08a8a0b9acf23052de9820432355b275735c192b461e45f4349cd294`
- 补正前整改计划：`db9b5b107d2e96fe17094bda94ec20821ecfae596a37de07aed3f762d43560c3`
- 补正前运行模型决策：`2d421bb3d177f2007d2a0aac699defad3821735b536ebe4c2d00ec6ccc579daf`
- 补正前本审计：`acc0a7b50211f02b05d2fad6a6319f68e129d5e4281a39845f0b15c28d85f130`

### 6.2 并行业务 dirty 的保守归属

当前工作树可直接观察到 shared Conversation Transport、MCP capability registry / supervisor binding、交办接线与对应测试的 implementation-shaped dirty changes。当前权威文档此前仍写“未执行 / 待 kickoff”，而本次 Harness 准入复核没有取得该业务线的授权记录或完成 evidence。

因此统一记为 `DIRTY_IMPLEMENTATION_PRESENT / ACCEPTANCE_UNVERIFIED`：

- 不按“尚未开始”重复实现；
- 不把现有 bytes 推断为已授权、已完成或已验收；
- 由业务 owning line 另行核对 task、验证并写 evidence / `CURRENT.md`；
- Phase 1 不读取其完成语义，也不扫描、修改或验证这些业务 bytes。

### 6.3 Phase 1 唯一允许写入面

用户后续按计划第 16 节单独派发 Phase 1 后，只允许：

- 新建 `docs/project-context.json`；
- 新建 `scripts/harness/project-context.js`；
- 新建 `scripts/harness/project-context.test.js`；
- 对 `AGENTS.md`、`AUTHORITY.md` 做基于下方冻结 hash 的非覆盖式最小指针合并。

Phase 1 不允许修改 `CURRENT.md`、`harness.config.json`、`.githooks/**`、其他 `scripts/harness/**`、业务代码或 `docs/code-map/**`，也不进入 Phase 2 / 3。

### 6.4 Phase 1 派发基线

- `AGENTS.md`：`8ceaf7a1fc13d777c1b0170282b78434a8d9c19c1c2e79f6400a52b18336ca94`（本次前置未修改）
- `AUTHORITY.md`：`e29765ebb54043f5da3337264dd284046f033e5e0b87d625c0c5e28e4ff84dda`（本次前置完成后）
- `CURRENT.md`：`275b3da9befb2d27360bbe04e4a9b8cb539f704f0d326047b9c71469461ddf20`（Phase 1 只读）
- 整改计划：`222f1d4cbc9ba737afb0146a4919d2cee473c2783c0c39e5bd9ef32039236400`
- 运行模型决策：`0a5a1bd7417d5b99f006dd7a7170594e75c9a65a0ba142650e1f9b91551ed383`

派发时必须重新核对这些 hash、`HEAD` 与 staged 为空。任何目标文件漂移、既有 hunk 无法区分归属、需要扩大写入面，或需要依赖业务 dirty 才能让测试通过，都必须停在 `BLOCKED_DIRTY_OVERLAP` / `BLOCKED_FALSE_GATE`，不得覆盖或放宽规则。

严格 config schema 的三个 legacy 字段缺失，以及 shape baseline 的 `16 errors / 5 warnings / 5 infos`，继续作为既有债务保留：它们不是 Phase 1 的清理目标；Phase 1 只要求不引入净新增，并如实报告聚合 gate 结果。

### 6.5 补正后验证

- `HEAD` 仍为 `e9ad7f3a204a1ebb11ce26c1e8c05b19c04c0991`；staged 仍为空；`git status --porcelain=v1` SHA-256 仍为 `768035557205a2f573ad1a5fadebe15299be5c3d3ef4bdb8f6cfe806b54deddb`。
- Harness 清单仍为 95 个文件（88 JS / 5 JSON / 2 Swift）；文件清单与内容 manifest SHA-256 仍分别为 `94b040f86babd9171a61443296f562d17931d41e156363d4edbccac82526380e`、`25e446b1bfa90382b98184f90de43476f192d30eb24b64605a05a8f8eb478a81`。
- `scripts/harness/harness.js`、`harness.config.json`、`.githooks/commit-msg` SHA-256 仍分别为 `eb78449528bc0a532ef7d78a9ff0e782d709d5d75cdc20e4f9917b113513c9ab`、`b91dd24b941a7eba69af78bceea8e1e71b83576219a6475acbb0e52eb9616b63`、`716d6cc3cf4d533e8c2fd2792eb0e9f50f69bd7741ef0bfe83add586d3413229`。
- `docs/project-context.json`、`scripts/harness/project-context.js`、`scripts/harness/project-context.test.js`、`docs/code-map/**` 均不存在；没有越过阶段边界。
- `config-policy --strict --json` 与 `config-check --strict --json` 退出 0；`config-schema --strict --json` 只因既有 `autoRisk`、`verificationRunner`、`taskLifecycle` 缺失退出 1。
- shape baseline 退出 0，结果为 `16 errors / 5 warnings / 5 infos`；shape check 退出 1，计数相同。它证明本轮文档补正没有改变该基线，但不把聚合 gate 报成全绿。
- `git diff --check` 退出 0；目标治理文档无行尾空白。

## 7. Phase 1 完成复核与状态对齐

复核日期：2026-07-23。

复核结论：`PHASE1_IMPLEMENTATION_PASS / PHASE1_STATUS_ALIGNED`。Phase 1 只完成只读、fail-open 的人工短路由，没有进入 Phase 2、Code Map、config / CLI 收缩、Hook 或业务代码整改。

### 7.1 状态对齐前冻结值

- `HEAD=e9ad7f3a204a1ebb11ce26c1e8c05b19c04c0991`
- staged 为空。
- `git status --porcelain=v1` SHA-256：`7a7be2743ac1309e3849d96965d52843c01f722220bac116772027c15a585ea2`
- `AGENTS.md`：`1e0c1ca1ce27e0e3a8dc32c2c2d26b2f74b0024e87e5096a51eddef24cc31771`
- `AUTHORITY.md`：`206f2cd1c306da11bbd9713a08272122ae7c1e1cf5a00f3155e01d94ab8916b9`
- `CURRENT.md`：`f8938bee070c9c62f3277f2396c76321d61109d6c6c65f21d3b1b72cd6f974dc`
- `docs/project-context.json`：`13a1a1a78a5ae57f63c53d3841602735a4942d9964273e7e479b9192fb6986f2`
- `scripts/harness/project-context.js`：`5ffd7c8921eab36fddc07c50d292422f2fb393a633cfa6df6beed1891e6a9fd2`
- `scripts/harness/project-context.test.js`：`b2ca89f05f08132264c0adb57a89d231930845bc806b517008778e201e608d1a`

### 7.2 写入面与并行漂移归属

- Phase 1 新增的运行文件只有 `docs/project-context.json`、`scripts/harness/project-context.js`、`scripts/harness/project-context.test.js`。
- 从当前 `AGENTS.md` 删除 Phase 1 的单行短路由指针后，SHA-256 精确恢复派发冻结值 `8ceaf7a1fc13d777c1b0170282b78434a8d9c19c1c2e79f6400a52b18336ca94`；该合并可独立归属。
- 从当前 `AUTHORITY.md` 删除 Phase 1 的 `docs/project-context.json` 指针后，SHA-256 为 `1a7125382a8451cf35347f735ed8ece290aca12f15c5ba3f4785f5133a422080`，不等于派发冻结值 `e29765ebb54043f5da3337264dd284046f033e5e0b87d625c0c5e28e4ff84dda`；同期 `CURRENT.md` 也从冻结值 `275b3da9befb2d27360bbe04e4a9b8cb539f704f0d326047b9c71469461ddf20` 漂移为上方当前值。
- 这些额外差异对应 shared Conversation Transport 业务线的授权、离线 evidence 与当前下一步收口；最终 `AUTHORITY.md`、`CURRENT.md`、业务总计划和 route 内容相互一致，且 Phase 1 指针 hunk 可分离。但共享 dirty worktree 不能单独证明并行修改的准确先后或执行者，因此本记录只写 `PARALLEL_BUSINESS_DOC_DRIFT_RECONCILED`，不写“目标文件全程零漂移”。

### 7.3 独立复核结果

- `node --test scripts/harness/project-context.test.js`：5 passed / 0 failed。
- 默认 route：退出 0，`READY`，10 行 / 1,275 B；指向 07-22 shared transport 决策与任务包、07-16 业务总计划，唯一下一步为另包、用户在场的真实 App 替代性验收，不派发 resident/private-home 历史线。
- 默认 JSON 不含 diagnostic；只有 `--diagnostic` 输出 pointer、`.git`、`commit-msg` Hook 与 structured / legacy Code Map 的存在性标记。
- 排除两个 Phase 1 新脚本后，旧 Harness 文件清单与内容 manifest SHA-256 仍分别为 `94b040f86babd9171a61443296f562d17931d41e156363d4edbccac82526380e`、`25e446b1bfa90382b98184f90de43476f192d30eb24b64605a05a8f8eb478a81`；`harness.js`、config、`commit-msg` Hook 也保持 Phase 0 冻结值。
- `config-policy --strict --json`、`config-check --strict --json` 退出 0；`config-schema --strict --json` 仍只因既有 `autoRisk`、`verificationRunner`、`taskLifecycle` 缺失退出 1。
- shape baseline 退出 0，shape check 退出 1，二者仍为 `16 errors / 5 warnings / 5 infos`；不把聚合 gate 报成全绿。
- staged 为空；`git diff --check` 退出 0。

Phase 1 到此正式收口。Phase 2 的 `CURRENT.md` 历史分离仍须另行授权，并在开工前重新冻结 dirty overlap；本节不派发 Phase 2。

## 8. Phase 2 完成复核与状态对齐

复核日期：2026-07-23。

复核结论：`PHASE2_IMPLEMENTATION_PASS / PHASE2_STATUS_ALIGNED`。Phase 2 只完成权威索引、当前短视图、计划入口和历史 archive 的职责分离；没有进入 Phase 3、Code Map、config / CLI 收缩、Hook 或业务代码整改。

### 8.1 状态对齐前冻结值

- `HEAD=e9ad7f3a204a1ebb11ce26c1e8c05b19c04c0991`
- staged 为空。
- `git status --porcelain=v1` SHA-256：`01358746409781ead546899d6d3db348a569cf52b60d4e7854bb572f320b2823`
- `CURRENT.md`：`ff0d99a72b104471a66313dce9d512fa356310735a232e9fc3bdce879163480b`（25 行 / 2,261 B）
- `AUTHORITY.md`：`766fa91f179427484b218105f905c72925883d93cb24cd8104ea484155912474`（48 行 / 4,211 B）
- `docs/plans/README.md`：`345b67d12dfc6f8b7692c576cc71f10fbccb47d943c848d62ac178ce5e5293f2`（28 行 / 1,993 B）
- `archive/2026-07-23-current-before-short-view-v1.md`：`8df14369d800aff3e42b08daf808cd9924a615c76f0db8877f4511e91cfa8b21`（83 行 / 58,535 B）
- `docs/project-context.json`：`13a1a1a78a5ae57f63c53d3841602735a4942d9964273e7e479b9192fb6986f2`（Phase 2 未改）

### 8.2 历史冻结与并行漂移口径

- `CURRENT.md` 只保留“现在能用 / 在做 / 唯一下一步 / 锁定项”四块；完整稳定旧正文逐字节保存在上述 archive，`AUTHORITY.md` 明确其只用于历史核对或重建，不参与默认路由。
- Phase 2 派发前冻结的 `CURRENT.md` 哈希 `f8938bee070c9c62f3277f2396c76321d61109d6c6c65f21d3b1b72cd6f974dc` 在并行业务文档漂移后已无法逐字节恢复。执行者先按 `BLOCKED_DIRTY_OVERLAP` 停止，随后依据用户明确的合并授权，以稳定的 `8df143…8b21` 版本建立 archive。
- 因此本记录只确认“合并授权后的稳定基线已冻结”，不声称 archive 与已失效的 `f8938b…f974dc` 基线相同，也不反推并行修改的执行者或准确先后。
- `AUTHORITY.md` 仍是唯一人工索引；07-16 master 仍是唯一业务执行计划，07-23 Harness 计划只作并行开发治理。Stage K、resident/private-home 与停用流程文档均不再是默认入口。

### 8.3 独立复核结果

- `node --test scripts/harness/project-context.test.js`：5 passed / 0 failed。
- 默认 route：退出 0，`READY`，10 行 / 1,275 B；当前决策、任务包、唯一业务计划、唯一下一步和安全边界均可达，resident/private-home 只作历史参照。
- `--diagnostic` 显示 `structuredCodeMap: absent`、`legacyCodeMap: present`；`docs/code-map/**` 与 `codebase-map` 工具均未创建，没有越入 Phase 3。
- `config-policy --strict`、`config-check --strict` 退出 0；`config-schema --strict` 仍只因既有 `autoRisk`、`verificationRunner`、`taskLifecycle` 缺失退出 1。
- shape baseline 退出 0，shape check 退出 1，二者仍为 `16 errors / 5 warnings / 5 infos`；不把既有聚合债务报成全绿。
- `git diff --check` 退出 0；staged 为空；Phase 2 未修改 `scripts/harness/**`、`harness.config.json`、`.githooks/**` 或业务代码。

Phase 2 到此正式收口。Phase 3 已具备单独派发条件，但仍须用户明确 kickoff，并在开工前重新冻结 Code Map 输入、写入面与既有 capability-map 归属；本节不派发 Phase 3。

## 9. Phase 3 完成复核、R1 纠偏与状态对齐

复核日期：2026-07-23。

复核结论：`PHASE3_IMPLEMENTATION_PASS / PHASE3_R1_PASS / PHASE3_STATUS_ALIGNED`。Phase 3 只完成六域 partial Code Map seed 与显式 `query / overlay / check` 工具；没有进入 Phase 4、修改默认路由或 Hook，也没有改业务代码、Harness config、真实 store、sandbox 或 approval。

### 9.1 状态对齐前冻结值与写入边界

- `HEAD=e9ad7f3a204a1ebb11ce26c1e8c05b19c04c0991`；staged 为空。
- `git status --porcelain=v1` SHA-256：`f869e6bc6f073ef5be467a53e0d26b2f8bc7da892cb7db9ce29b08f59bedb2e1`。
- 本次状态对齐前，运行模型决策、本审计、整改计划 SHA-256 分别为 `115e8f2e168f9c036d896f5ab241171c59d755389003958d9f6b6c6d06103e50`、`9c82f0faf20a7590e5f92de36f5e7d47524e88feca91e922d4fbeaddc97c85ed`、`45ac5cec19b9a5a171baa87bd98cb0da67f5aac6eefb057cd1e2fdd8888c8c0c`。
- Phase 3 交付物为 `docs/code-map/**`、`scripts/harness/codebase-map.js`、`scripts/harness/codebase-map.test.js`，以及只增加 historical/superseded 指针的 `docs/2026-07-09-codebase-capability-map-v2.md`；这些文件的联合 manifest SHA-256 为 `f3aa9059b7e8fa9a297d2ef3e2fbd31f3d73cc273bf4427d506d3b4444615380`。
- R1 实际纠偏文件为 `docs/code-map/README.md`、两个 domain JSON、`codebase-map.js` 与测试；其 SHA-256 依次为 `bb2e04c5b38d3bb6965a47b126b5b2d80fdcd032a4d86b5a218d47b1df8b9be5`、`3130696098a141e2ab9e5fd5b7c524902849fca21c7778b31b4bbc9e04d4cef2`、`855e8f4d830b525fae8c40541398dd5af700c8f747bae1d31ac7046627a2c272`、`8dffd7261c39d5224a9e0720e88704ff57c29e9b81b06515ca6159f9bbd605dc`、`a795109b232b35709d2b5f7d91fd3074814b51057d6467bdf9d15b9fff93e3ee`。
- Code Map 文件仍处于共享 dirty 工作树中的 untracked 状态；Git 单独不能证明所有修改的作者或准确先后。本复核依据冻结内容、上轮失败点、R1 diff 语义和直接验证判定，不外推为全工作树归属证明。

### 9.2 初次复核失败与 R1 纠偏

- 初次实现已具备六域、16 条能力和 7/7 工具测试，但 37 个 `publicSymbols` 中有 6 个 CLI 描述性假符号；检查器只验 tracked path 和字符串类型，查询会把假符号当成真实入口。
- 初次 workflow 图还把 active guarded product-command API 挂在 legacy workflow boundary 下；`real execution` 只返回 legacy，存在实际误路由风险，因此没有放行 Phase 3。
- R1 将没有真实公共标识符的 CLI 改为空 `publicSymbols`，并新增 `MAP_PUBLIC_SYMBOL_NOT_FOUND`：只接受 bare identifier，使用 `git show <verifiedAtCommit>:<path>` 核对 TS/JS `export` 或 Rust `pub` / `pub(crate)` 声明。
- R1 把 `legacy_product_command_boundary_spec` / `run_workflow_machine` 与 `prepare_real_execution_product_command_at`、Phase A/B 产品命令拆成两个 capability；后者标为 active 只表示 tracked code capability 存在，不授予真实执行。
- 最终 seed 为六域、17 条能力：conversation 3、Syn/MCP 1、workflow 4、persistence 2、UI 4、development-harness 3；状态为 active 11、legacy 3、needs-confirmation 3。39 个 public symbol 均在 `e9ad7f3…` 源码提交中通过声明核验。
- resident/private-home 继续是 legacy，不作为默认运输或第三套 transport；untracked / unstaged 源码只在 overlay 出现，不写成 canonical。

### 9.3 独立复核结果

- `node --test scripts/harness/codebase-map.test.js`：9 passed / 0 failed；包含 verified commit 假 symbol、real-execution active/legacy 排序、partial no-match、overlay 非写入、staged rename/delete 等回归。
- `node scripts/harness/codebase-map.js check --target . --staged --strict --json`：退出 0，`OK`，无 errors / warnings / staged impacts。
- 十个关键查询均退出 0：交办会话、conversation transport、Stop、poll、readback 首条为 `conversation-transport.agent-manual-relay`；real execution、product command、prepareRealExecutionProductCommand 首条为 `workflow-execution-governance.guarded-real-execution-product-command`；legacy workflow execution、run_workflow_machine 首条为 `workflow-execution-governance.legacy-real-workflow-execution`。
- `overlay --json` 退出 0；当前 `commands.rs` 可同时关联 conversation、guarded product command 与 legacy boundary，`lib/tauri.ts` 关联对应 TS 路线；未提交线索仍与 committed canonical 分离。
- `node --test scripts/harness/project-context.test.js`：5 passed / 0 failed；默认 route 不调用 Git、Hook、Code Map、源码扫描或子进程。
- `config-policy --strict`、`config-check --strict` 退出 0；`config-schema --strict` 仍只因既有 `autoRisk`、`verificationRunner`、`taskLifecycle` 缺失退出 1。
- shape baseline 退出 0，shape check 退出 1，二者仍为 `16 errors / 5 warnings / 5 infos`；不把既有聚合债务报成全绿。
- `git diff --check` 退出 0；staged 为空。Phase 3 和 R1 均未 stage、commit 或 push。

Phase 3 到此正式收口。Phase 4 的重要任务计划对齐已具备单独派发条件，但仍须用户明确 kickoff，并在开工前重新冻结 `AGENTS.md`、`checkpoint-audit.js`、对应 self-test、当前任务包、`docs/project-context.json` 与 dirty overlap；本节不派发 Phase 4。

## 10. Phase 4 完成复核与状态对齐

复核日期：2026-07-23。

复核结论：`PHASE4_IMPLEMENTATION_PASS / PHASE4_STATUS_ALIGNED`。Phase 4 只落地重要任务的机械计划对齐：小改动不强制建包，只读取短路由显式绑定的 `checkpoint.currentImportantTask`，不扫描历史 `tasks/**`，也不复用已经收口的业务 `taskPackage`。本阶段没有进入 Phase 5，没有修改 Harness config、根 CLI、catalog、Hook 或业务代码。

### 10.1 状态对齐前冻结值与写入边界

- `HEAD=e9ad7f3a204a1ebb11ce26c1e8c05b19c04c0991`；staged 为空；共享 dirty 工作树的 `git status --porcelain=v1` SHA-256 为 `c824c201d0b32e5057fb94c7c312636481812f830dd50ed60ed2d1a600f8a102`。
- 状态对齐前，运行模型决策、整改计划、本审计 SHA-256 分别为 `14ef5f2783ff01dcc2779634a2e6a184c0a7b80cad08d84ea1f1d0d32aae1c6a`、`e3a48cd090f557ba9f0d3fd1ed3ee89d0e85e9a14b786e0dad2cde5f554890db`、`f99d9531e5d5d197fd91fbab3ef2ecd348f40896891424fe6840f7378b95a532`。
- Phase 4 实际交付物为 `AGENTS.md`、`scripts/harness/checkpoint-audit.js`、`scripts/harness/checkpoint-audit.selftest.js`、`docs/project-context.json`；四文件联合 manifest SHA-256 为 `8b69f707c54cce7a4661768dba1eca81a543081e6ad11ac2430763faaab04812`。
- 四个交付物 SHA-256 依次为 `9297ef037bf5f309e6e9095e7bcc39e3de84090e678b60dcad1a31502488bd91`、`a646cb60562aa274d5275bc4269cd079cc0d2988f04abc0c240a302524b33d9c`、`6297b6bcacfe0b3cf6bb735ebb421b0a818396d00c054f09ad256ec4b3b368e9`、`06154efd2999875db05c271664c3935af3fa338e6ffe5d78e6d9a46a2aa7f69d`。
- 07-22 已收口 shared transport 实施包未修改，SHA-256 仍为 `58f8a3c4bcfc7739cfc88fc7c640df56837c1364ae925fd6df707b21419b4f7a`。当前没有新的重要任务包获得显式绑定，因此 `docs/project-context.json` 中 `checkpoint.currentImportantTask=null`。
- 本次状态对齐只允许修改运行模型决策、整改计划和本审计，不改 Phase 4 实现文件或任何 Phase 5 写入面。

### 10.2 实现边界

- `AGENTS.md` 增加三条薄规则：重要任务开工前对齐计划、五个机械字段必须完整可查、字段齐全不代表语义正确或完成验收；开工前已有的 Rust build 与短路由 hunk 未被覆盖。
- `checkpoint-audit.js --current` 只读取 `docs/project-context.json` 中的 `checkpoint.currentImportantTask`。无绑定时返回 `NOT_APPLICABLE / NO_CURRENT_IMPORTANT_TASK_PACKAGE` 且退出 0；advisory 缺字段只 warning，只有显式 `mode: strict` 才因必填字段缺失退出 1。
- 对齐块固定为 `authority_chain`、`plan_anchor`、`existing_before_new`、`capabilities_touched`、`forbidden_alternatives`。工具检查字段、计划路径 / heading 与 Code Map ID，保留 legacy / needs-confirmation 提示，但不替用户改路线。
- `FIELDS_PRESENT` 只表示机械字段齐全；输出边界明确否认语义正确、代码完成、真实执行或产品验收。

### 10.3 独立复核结果

- `node scripts/harness/checkpoint-audit.selftest.js`：45/45 通过；其中原有 completion / commit / evidence hash 检查继续通过，并新增无绑定不扫描历史任务、advisory / strict、计划锚点、Map ID、legacy / needs-confirmation 与 `none` 解释测试。
- `node scripts/harness/checkpoint-audit.js --current --target . --json`：退出 0，`NOT_APPLICABLE / NO_CURRENT_IMPORTANT_TASK_PACKAGE`；这只证明当前没有显式绑定，不证明任何业务任务完成。
- `node --test scripts/harness/project-context.test.js`：5 passed / 0 failed；默认短路由仍为 10 行 / 1,275 B，且不调用 Git、Hook、Code Map、源码扫描或子进程。
- `node --test scripts/harness/codebase-map.test.js`：9 passed / 0 failed；`codebase-map check --staged --strict --json` 退出 0，`OK`。
- `config-policy --strict`、`config-check --strict` 退出 0；`config-schema --strict` 仍只因既有 `autoRisk`、`verificationRunner`、`taskLifecycle` 缺失退出 1。
- shape baseline 退出 0，shape check 退出 1，二者仍为 `16 errors / 5 warnings / 5 infos`；不把既有聚合债务报成全绿。
- Phase 5 的 `harness.config.json`、根 CLI、`config-*.js`、catalog 和 `.githooks/**` 未发生 Phase 4 修改；`git diff --check` 退出 0；staged 为空。

Phase 4 到此正式收口。当时下一门是 Phase 5 的 CLI / config / legacy consumer 收缩；它随后经单独派发、复核与 R1 纠偏完成，详见第 11 节。Phase 6 也已随后完成并在第 12 节对齐；当前仅 Phase 7 等待用户单独派发。

## 11. Phase 5 完成复核、R1 纠偏与状态对齐

复核日期：2026-07-23。

复核结论：`PHASE5_IMPLEMENTATION_PASS / PHASE5_R1_PASS / PHASE5_STATUS_ALIGNED`。这只确认 Phase 5 的定向 config、CLI 与 legacy consumer 收缩及其治理回写通过；不表示聚合自测全绿、业务完成、真实 App 验收或产品验收。当时 Phase 6 不在本节范围；其后续已验收实施与 R1 见第 12 节。

### 11.1 实施边界与保留事实

- Phase 5 实际交付物为 `harness.config.json`、`harness.config.example.json`、`scripts/harness/harness.js`、`scripts/harness/config-schema.js`、`scripts/harness/config-check.js`、`scripts/harness/config-policy.js`、`scripts/harness/harness-phase5.test.js` 与 `docs/harness-catalog.md`；R1 只补两份 config、Phase 5 测试与 catalog。
- `activeBoundary` 固定为 `mechanical`、`reportingOnly`、`explicitTool`、`legacyIgnored` 四类，各值为不跨类重复的字符串数组；三项 config 工具共同识别并严格核验它。`autoRisk`、`verificationRunner`、`taskLifecycle` 改为兼容性可选字段，而非回填旧 lifecycle。
- 默认 CLI 恰为 9 项：`context`、`context diagnostic`、`map query`、`map overlay`、`map check`、`checkpoint`、`shape`、`stage-k`、`doctor`。`context diagnostic` 注入 `--diagnostic`、Code Map 子命令注入对应动作、`checkpoint` 注入 `--current`，多词命令采用最长匹配；`maintenance` 未进入默认 help，也没有以 `memory-maintenance.js` 冒充。
- 原 35 条路由仍可直调，`--legacy` 隐藏其中除 `doctor` 外的 34 条；兼容说明不写入 JSON stdout。`.githooks/commit-msg` 未改，SHA-256 仍为 `716d6cc3cf4d533e8c2fd2792eb0e9f50f69bd7741ef0bfe83add586d3413229`。

### 11.2 首次复核失败与 R1 纠偏

- Phase 5 实现复核首先发现 `config-policy` 没有把声明式 `gates.hard` 与 non-mechanical boundary 交叉比对；这会让 `reportingOnly` / `explicitTool` 被误升为 hard gate。实现复核在 Phase 5 内补齐该比对，`config-check` 和 `config-policy` 现都以 `ACTIVE_BOUNDARY_NON_MECHANICAL_HARD_GATE` 拒绝该配置。
- 随后的 R1 首次复核发现默认九项之一 `context diagnostic` 未写入两份 config 的 `activeBoundary.explicitTool`，catalog 也没有相应归类；因此默认入口与边界声明并非完整一一映射，不能只报最终绿。
- R1 将 `context diagnostic` 补入两份 config 与 catalog，新增回归精确锁定默认九项全部且仅声明一次，并验证把它写入 `gates.hard` 时 `config-check`、`config-policy` 都拒绝。定向测试先以 8/9 暴露遗漏，修正后为 9/9 通过。

### 11.3 定向复核结果

- `node --test scripts/harness/harness-phase5.test.js`：exit 0，9 passed / 0 failed。
- 项目 config 与 example config 的 `config-schema --strict --json`、`config-policy --strict --json`、`config-check --strict --json`：均 exit 0；schema 明确把 `autoRisk`、`verificationRunner`、`taskLifecycle` 视为 optional compatibility absent。
- `node --test scripts/harness/project-context.test.js`：5/5；`node --test scripts/harness/codebase-map.test.js`：9/9；`node scripts/harness/checkpoint-audit.selftest.js`：45/45，均 exit 0。
- shape baseline exit 0、shape check exit 1，二者均为既有 `16 error / 5 warning / 5 info`；Stage K exit 0，`0 error / 15 warning / 36 info`。这些不是本阶段全绿结论，也不被修成全绿。
- `git diff --check` exit 0；staged 为空。Phase 5 / R1 未 stage、commit 或 push，未修改 Hook、业务代码、真实 store、默认短路由、Code Map 或 checkpoint 实现。

### 11.4 聚合自测与 JSON 捕获问题（未绿，保留）

- `node scripts/harness/self-test.js` 仍 exit 1，`PASS 177 / FAIL 9`：`evidence-check strict`、`fixture-check`、`harness-doctor strict`、`eval-runner smoke`、`eval-runner context`、`context suite pass`、`context ranking-metrics`、`context-pack source-package-skip`、`harness CLI doctor JSON`。这些遗留聚合失败没有被包装成 Phase 5 通过。
- 当前巨大 dirty 工作树下，`guard-state-files --json` 输出约 114,086,211 B；`check-runner` 对该内部检查的捕获上限为 10 MiB，因而会报 `spawnSync ... ENOBUFS`。这是 guard 内部的捕获失败；`self-test.js` 外层调用的上限另为 20 MiB，不能把两者混写成同一条“32 MiB 捕获链”。
- 与 guard 的内部 `ENOBUFS` 分开，`doctor --json` 及经 `harness.js doctor --json` 路由后的已采样 stdout 未刷新，仍是 65,536 B 的截断片段，`JSON.parse` 对该片段报 `Unterminated string`。该未刷新的 doctor 输出不能反推 guard 的当前捕获上限或失败原因。聚合自测中的 `harness CLI doctor JSON` 项是因 doctor exit 1 失败，并非该自测直接对 doctor 输出作 JSON.parse 断言；这是一项现有聚合 JSON 捕获 / 巨大脏树输出问题，不能归因成 Phase 5 默认 CLI 向 JSON stdout 注入 deprecation 文本。

Phase 5 到此完成状态对齐。Phase 6 随后经单独派发、复核与 R1 纠偏完成，见第 12 节；当前仅 Phase 7 等待用户单独派发，不得因本节、Phase 5 / R1 或 Phase 6 / R1 的通过自动开工。

## 12. Phase 6 完成复核、R1 纠偏与状态对齐

复核日期：2026-07-23。

复核结论：`PHASE6_IMPLEMENTATION_PASS / PHASE6_R1_PASS / PHASE6_STATUS_ALIGNED`。这只确认 Phase 6 的显式、只读 maintenance audit 及其治理回写通过；不表示业务完成、真实 App 验收或产品验收，也不自动启动 Phase 7。

### 12.1 实施边界与当前事实

- Phase 6 仅交付 `scripts/harness/maintenance-audit.js`、`scripts/harness/maintenance-audit.test.js`，并将人工、只读、无自动回写的使用边界写入 `AGENTS.md` 与 Code Map README；不接 Hook、CI、cron 或默认 CLI，不扫描 dirty overlay 或自动更新 CURRENT / Code Map。
- 当前审计覆盖 authority、project-context、CURRENT、Code Map、active boundary、legacy consumer 六项；当前 `maintenance-audit --target . --json` 为六项 PASS，staged canonical impact 为 0。输出有 finding / JSON 上限，超长 dirty fixture 不泄漏路径且小于 64 KiB。

### 12.2 首次复核假绿与 R1 纠偏

- 首次 Phase 6 复核抓到 staged canonical rename/delete 没有映射到受影响 capability：地图引用可在暂存变更下假绿。R1 增加 `STAGED_RENAME_AFFECTS_CAPABILITY` / `STAGED_DELETE_AFFECTS_CAPABILITY`、capability ID 与路径信息，二者均返回 DRIFT。
- 首次复核还抓到默认 CLI help 为空或不可解析时，路由解析返回零项而跳过边界比对。R1 让两种情形都以 `DEFAULT_CLI_BOUNDARY_DRIFT`、exit 1 明确失败，不把“无输出”当作边界一致。

### 12.3 定向复核结果与保留未绿事实

- `node --test scripts/harness/maintenance-audit.test.js`：8 passed / 0 failed；包含原六类漂移、staged canonical rename/delete 与空白/不可解析 default CLI help 回归。
- shape baseline/check 仍为 exit 0 / 1、`17 error / 5 warning / 5 info`；Stage K 仍 exit 0、`0 error / 15 warning / 36 info`。第 11 节的 Phase 5 历史 `16/5/5`、聚合自测 177/9 与 10 MiB / 20 MiB / doctor 截断问题原样保留，不能改写成全绿。
- `git diff --check` 通过；staged 为空。Phase 6 / R1 未修改业务代码、CURRENT、AUTHORITY、CLI、config、catalog、Hook、sandbox 或 approval，未 stage、commit 或 push。

Phase 6 到此完成状态对齐。下一门仅为 Phase 7 回放、观察期与收口，**等待用户单独派发**；不得因本节或此前阶段通过自动开工。
