# Harness 目录（当前工作树）

> 这是开发 Harness 的导航与 consumer 边界，不是任务完成、业务测试、真实 App 验收或产品授权的证明。脚本默认由人显式调用；先查本页和当前 `AGENTS.md`，再决定是否运行。

## 当前清单

当前工作树共有 **100** 个 Harness 文件：**93 JS、5 JSON、2 Swift**。

- 顶层 JS：77（含 `harness-phase5.test.js`）
- `lib/` 内部 JS：16（只被其他 Harness 脚本 `require`，不是独立 CLI）
- `eval/cases/` JSON：5
- Stage K Swift 探针：2

此前“87 个脚本”的数字是历史盘点，不能当作当前清单。Git tracked 文件数与工作树清单不同：本页按当前真实工作树列出，不把已有未提交的短路由或 Code Map 工具伪装成已提交 canonical。

### 顶层 JS（77）

```text
browser-evidence-check.js          capability-map.js                 capability-scan.js
checkpoint-audit.js                checkpoint-audit.selftest.js      ci-gate.js
ci-init.js                         ci-validate.js                    codebase-map.js
codebase-map.test.js               config-check.js                   config-init.js
config-migrate.js                  config-policy.js                  config-schema.js
context-pack.js                    eval-runner.js                    evidence-check.js
evidence-command.js                evidence-compact.js               evidence-freshness.js
evidence-index.js                  evidence-new.js                   evidence-query.js
evidence-retention.js              fixture-check.js                  git-gate.js
guard-state-files.js               harness-doctor.js                 harness-phase5.test.js
harness.js                         hook-install.js                   hook-uninstall.js
install-harness.js                 installed-health.js               managed-files-audit.js
mcp-doctor.js                      memory-agentmemory-query.js       memory-agentmemory-save.js
memory-candidate-lint.js           memory-candidate-new.js           memory-maintenance.js
memory-review.js                   memory-stale-check.js              mistake-check.js
mistake-new.js                     mistake-query.js                  pre-completion.js
pre-work.js                        project-context.js                project-context.test.js
project-profile.js                 rules-lint.js                     runtime-docs-diff.js
runtime-docs-init.js               security-scan.js                  self-test.js
skill-recommend.js                 stage-k-architecture-gate.js      stale-control-check.js
status-snapshot.js                 sync-harness.js                   task-finish.js
task-package-lint.js               task-package-new.js               task-risk.js
task-start.js                      task-status.js                    ui-verify.js
verification-plan.js               verification-runner.js            verification-suite.js
workbench-shape-gate.dedup.selftest.js
workbench-shape-gate.hardcoded-hex.selftest.js
workbench-shape-gate.js
workbench-shape-gate.machine-face.selftest.js
workbench-shape-gate.retired-style-family.selftest.js
```

### 内部库、数据与遗留探针

```text
lib/agentmemory-client.js          lib/check-runner.js               lib/config-loader.js
lib/context-pack.js                lib/evidence-audit.js             lib/hardcoded-hex-rule.js
lib/machine-face-rule.js           lib/manifest.js                   lib/memory-governance.js
lib/mistake-retrieval.js           lib/project-kind.js               lib/retired-style-family-rule.js
lib/risk-classifier.js             lib/security.js                   lib/skill-index.js
lib/task-package-schema.js

eval/cases/context-pack.json       eval/cases/memory-governance.json
eval/cases/mistake-retrieval.json  eval/cases/security/prompt-injection.json
eval/cases/skill-recommend.json

stage-k-cgevent-click.swift        stage-k-screencapturekit-window-capture.swift
```

八个仅存在于历史资料的名字——`ad-policy-check.js`、`agent-entrypoint-check.js`、`duplicate-code-check.js`、`harness-observation-installed-lifecycle.test.js`、`harness-observation.js`、`lib.js`、`predev-check.js`、`scope-check.js`——不是当前文件；不要为它们补空壳。

## Active boundary

`harness.config.json` 和 example 都使用相同的四分类。每项只能属于一个分类；`reportingOnly` 与 `explicitTool` 不能被配置升级成 hard gate。

| 分类 | 当前含义 | 当前入口 / 例子 |
| --- | --- | --- |
| `mechanical` | 有明确退出码的窄结构或安全检查 | `commit-msg catch:`、config schema/check/policy、shape |
| `reportingOnly` | 提供导航或报告，不自动阻塞 | `context`、`checkpoint` |
| `explicitTool` | 任务或阶段需要时人工调用 | `context diagnostic`、Code Map、Stage K、doctor |
| `legacyIgnored` | 保留兼容但不再默认展示或推荐 | memory、task/evidence lifecycle、runtime-doc init、managed Hook/CI init、旧 capability scan |

`preWork` 现在只推荐短路由；`preCompletion` 只推荐窄 config 检查（example 另保留 `rules-lint`，因为安装包自测依赖它的 source-package skip）。这些推荐不代替任务相关测试、build、UI 或真实 App 验收。

## 根 CLI

默认 `node scripts/harness/harness.js --help` 只展示以下 **9** 个入口：

| 命令 | 固定转发 |
| --- | --- |
| `context` | `project-context.js` |
| `context diagnostic` | `project-context.js --diagnostic` |
| `map query` | `codebase-map.js query` |
| `map overlay` | `codebase-map.js overlay` |
| `map check` | `codebase-map.js check` |
| `checkpoint` | `checkpoint-audit.js --current` |
| `shape` | `workbench-shape-gate.js` |
| `stage-k` | `stage-k-architecture-gate.js` |
| `doctor` | `harness-doctor.js` |

其余参数会原样转给目标工具；多词命令采用最长匹配，因此 `context diagnostic` 不会被 `context` 吞掉。`maintenance` 不在默认入口：它属于后续维护审计边界，不能用已退役的 `memory-maintenance.js` 冒充。

`node scripts/harness/harness.js --legacy` 展示 **34** 个隐藏兼容入口；它们仍可直接调用，路由不向 JSON stdout 加入 deprecation 文本：

```text
pre-work, pre-completion
init config, init docs, init hooks, init ci
profile, policy, mistake query
memory candidate new, memory candidate lint, memory review, memory stale-check,
memory maintenance, memory agentmemory query, memory agentmemory save
task start, task finish, task status, task risk, task package new, task package lint
evidence new, evidence retention, evidence compact, evidence index, evidence query
skill recommend, security scan, eval
verify plan, verify run, verify suite
capabilities
```

`doctor` 已在默认九项中，故不重复计为 legacy。保留这些兼容入口不等于推荐新工作走旧 lifecycle。

## Consumer 事实

- **真实自动接线**：Git `core.hooksPath=.githooks`，唯一 Hook 为 `.githooks/commit-msg`，只机械检查 commit message 是否含 `catch:`。它不调用 Harness CLI、config、Code Map 或文档检查。
- **未启用模板**：`templates/hooks/**`、`templates/ci/**` 仅模板；当前没有实际 CI workflow，且 config 为 `hooks.enabled=false`、`ci.required=false`。
- **显式 / 内部调用**：`pre-work`、`pre-completion` 与 `doctor` 只有被人工调用时才聚合；`self-test` 和 `lib/**` 的引用是兼容回归或内部依赖，不是自动接线。
- **AgentMemory**：`memoryIntegration.enabled=false`。相关命令、库和模板作为兼容/历史材料保留，不形成当前默认流程。
- **Code Map 与短路由**：它们是当前人工导航工具，不进入 Hook，也不替代源码核对、业务验证或真实运行验收。

本轮不删除任何 Harness 文件，也不修改 `.githooks/**`、业务代码、真实 store 或运行数据。
