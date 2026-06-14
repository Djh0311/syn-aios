# Harness 脚本索引（scripts/harness/）

> 用得上的脚本索引。**动用或改造任何 `scripts/harness/` 脚本前先查这里**，避免重造已存在但没接线的工具。

- 生成日期：2026-06-14（HG-1）｜数据源：[只读审计附录命中表](harness-script-audit-2026-06-14.md)（另见[上游源码包审计](harness-source-package-audit-2026-06-14.md)）
- 范围：66 个顶层 `.js` + 13 个 `lib/*.js` = **79** 条。索引随包演进：HG-2 会把接通的组从`未接`翻成`已接`、把 `capability-map` 翻成`退役`。

## 一句话判据

> 随手点一个脚本，看它这一行的**状态**：`承重`=放心用；`休眠`=本阶段别依赖（换阶段/开开关才用，别删）；`未接`=能力在但没接线（可补的缺口，接了再用）；`已接`=已进流程可用；`退役`=被取代别用（文件留着）。

## 状态 = 桶 + 接没接 + 该不该用

| 状态 | 含义（桶） | 该不该用 |
| --- | --- | --- |
| `承重` | 几乎每包都跑 | 用 |
| `休眠` | 被 config/阶段关掉（hooks/CI/UI/MCP） | 本阶段别依赖；**别删**，换阶段会用 |
| `休眠·待定` | agentmemory 簇；config `memoryIntegration.enabled=false`，且产品已改走文件记忆 | 别依赖；**是否永久退役待用户拍，本包不决定** |
| `未接` | 装好从没接上电（能力在、没 wire 进 hook/CI/流程） | 可补的缺口；接线后可用。后缀 `HG-2①②③④⑩`=本轮接线目标；`元`=只服务 harness 自身、采用后才谈得上 |
| `已接` | 已接进 AGENTS.md 流程 | 用（HG-2 后才会出现） |
| `退役` / `退役候选` | 被取代/重复 | 别用；文件留着不删 |

统计（HG-1 时）：`承重` 2 ｜ `休眠` 11 ｜ `休眠·待定` 9 ｜ `未接` 56（含元工具、含 HG-2 目标）｜ `退役候选` 1。

> 「怎么调」列统一省略前缀 `node scripts/harness/`；`--target .` 为常用默认。`lib/` 为内部库，被 `require`，不单独 CLI 调。37 个命令另可经 `node scripts/harness/harness.js <子命令>` 路由（`harness.js --help` 看全表）。

## 顶层脚本（66）

| 脚本 | 干啥（一行） | 状态 | 怎么调 |
| --- | --- | --- | --- |
| browser-evidence-check.js | 校验 UI 完成是否带真实浏览器证据（route/interaction/console/network/截图） | 休眠 | `browser-evidence-check.js --target .` |
| capability-map.js | 扫项目可用工具/命令能力，输出能力图 | **退役候选**（与 capability-scan 重复，HG-2 ⑩ 落实） | `harness.js capabilities` |
| capability-scan.js | 扫项目能力信号（preWork 用） | 未接·HG-2⑩ | `capability-scan.js --target .` |
| ci-gate.js | CI 内跑的聚合门 | 休眠（CI 关） | `ci-gate.js --target .` |
| ci-init.js | 从模板初始化 CI 配置 | 休眠（CI 关） | `harness.js init ci` |
| ci-validate.js | 校验 CI 配置 | 休眠（CI 关） | `ci-validate.js --target .` |
| config-check.js | 校验 harness.config.json 形状/一致性 | 未接·HG-2④ | `config-check.js --target .` |
| config-init.js | 从 example 生成项目 config（`--preset auto/advisory/balanced/strict`） | 未接·元 | `harness.js init config` |
| config-migrate.js | 迁移 config schema 版本 | 未接·元 | `config-migrate.js --target .` |
| config-policy.js | 查看/校验 policy 配置 | 未接·HG-2④ | `harness.js policy` |
| config-schema.js | config 的 schema 定义/校验（源码包当 typecheck） | 未接·元 | `config-schema.js --target .` |
| context-pack.js | 为 Strict 恢复打包任务相关运行文档切片 | 未接 | `context-pack.js --target . --task-id <id> --slug <slug>` |
| eval-runner.js | 跑 harness eval 用例（security/skill-recommend/...） | 未接·元 | `harness.js eval --suite smoke` |
| evidence-check.js | 校验证据归档完整性 | 未接·HG-2③ | `evidence-check.js --target .` |
| evidence-command.js | 把命令输出写成证据条目 | 未接 | `evidence-command.js --target .` |
| evidence-compact.js | 压缩超大命令输出证据 | 未接 | `harness.js evidence compact` |
| evidence-freshness.js | 查证据是否过期（maxAgeHours） | 未接·HG-2③ | `evidence-freshness.js --target .` |
| evidence-index.js | 给证据归档建索引 | 未接·HG-2③ | `harness.js evidence index` |
| evidence-new.js | 新建证据归档条目 | 未接·HG-2③ | `harness.js evidence new` |
| evidence-query.js | 查询证据归档 | 未接·HG-2③ | `harness.js evidence query` |
| evidence-retention.js | 规划/执行证据归档保留 | 未接 | `harness.js evidence retention` |
| fixture-check.js | 校验测试夹具（源码包当 build） | 未接·元 | `fixture-check.js` |
| git-gate.js | git 状态/受保护路径门（hooks 内用） | 休眠（hooks 关） | `git-gate.js --target .` |
| guard-state-files.js | 守卫受保护状态文件（CURRENT.md/evidence/...） | 未接·HG-2① | `guard-state-files.js --target .` |
| harness-doctor.js | 聚合只读诊断门（planning/status/evidence 分离） | 未接·元 | `harness.js doctor --target . --strict` |
| harness.js | 统一子命令路由器 + `bin` 入口 | 未接·元 | `harness.js --help` |
| hook-install.js | 安装托管 git hooks | 休眠（hooks 关） | `harness.js init hooks` |
| hook-uninstall.js | 卸载托管 git hooks | 休眠（hooks 关） | `hook-uninstall.js --target .` |
| install-harness.js | 把 harness 安装进目标项目（不拷 package.json） | 未接·元 | `install-harness.js --target <dir>` |
| installed-health.js | 检查已安装 harness 的健康/漂移 | 未接·元 | `installed-health.js --target .` |
| managed-files-audit.js | 审计 manifest 托管文件是否被本地改动 | 未接·元 | `managed-files-audit.js --target .` |
| mcp-doctor.js | 检查 MCP 工具可用性/健康 | 休眠（本阶段无 MCP） | `mcp-doctor.js --target .` |
| memory-agentmemory-query.js | 经治理包装查询 agentmemory | 休眠·待定 | `harness.js memory agentmemory query` |
| memory-agentmemory-save.js | 把批准的记忆写入 agentmemory | 休眠·待定 | `harness.js memory agentmemory save` |
| memory-candidate-lint.js | lint 治理记忆候选 | 休眠·待定 | `harness.js memory candidate lint` |
| memory-candidate-new.js | 新建治理记忆候选 | 休眠·待定 | `harness.js memory candidate new` |
| memory-maintenance.js | 记忆 lint/staleness/后端健康聚合 | 休眠·待定 | `harness.js memory maintenance` |
| memory-review.js | 复核/改记忆候选状态 | 休眠·待定 | `harness.js memory review` |
| memory-stale-check.js | 查记忆候选过期 | 休眠·待定 | `harness.js memory stale-check` |
| mistake-check.js | 查错误账本里相关历史失败 | 未接·HG-2② | `mistake-check.js --target .` |
| mistake-new.js | 新建错误账本条目 | 未接·HG-2② | `mistake-new.js --target .` |
| mistake-query.js | 查询错误账本 | 未接·HG-2② | `harness.js mistake query` |
| pre-completion.js | 完成前聚合检查（hooks 内/手动） | 休眠（hooks 关） | `harness.js pre-completion --target .` |
| pre-work.js | 开工前就绪聚合检查 | 休眠（hooks 关） | `harness.js pre-work --target . --strict` |
| project-profile.js | 探测项目画像信号 | 未接·元 | `harness.js profile` |
| rules-lint.js | lint AGENTS/规则文档（源码包当 lint） | 未接·元 | `rules-lint.js .` |
| runtime-docs-diff.js | 对比运行文档与模板差异 | 未接·元 | `runtime-docs-diff.js --target .` |
| runtime-docs-init.js | 从模板初始化运行文档 | 未接·元 | `harness.js init docs` |
| security-scan.js | 扫文本/文件的注入与密钥模式 | 未接 | `harness.js security scan --file <f> --source issue` |
| self-test.js | harness 自测套件（325 断言，源码包当 test） | 未接·元 | `self-test.js` |
| skill-recommend.js | 按任务文本推荐必读 skill | 未接 | `harness.js skill recommend` |
| stage-k-architecture-gate.js | stage-K 架构门 | **承重**（审计 21 次） | `stage-k-architecture-gate.js --target .` |
| stale-control-check.js | 查控制文件是否过期 | 未接·HG-2① | `stale-control-check.js --target .` |
| status-snapshot.js | 输出项目状态快照 | 未接·HG-2① | `status-snapshot.js --target .` |
| sync-harness.js | 同步已安装项目的规则更新 | 未接·元 | `sync-harness.js --target <dir>` |
| task-finish.js | 收尾任务记录 | 未接 | `harness.js task finish` |
| task-package-lint.js | lint 结构化任务包 | 未接 | `harness.js task package lint` |
| task-package-new.js | 创建结构化任务包 | 未接 | `harness.js task package new` |
| task-risk.js | 推荐项目预设与任务路径 | 未接 | `harness.js task risk` |
| task-start.js | 开始任务记录 | 未接 | `harness.js task start` |
| task-status.js | 报告任务状态 | 未接 | `harness.js task status` |
| ui-verify.js | UI 验证（浏览器证据口径） | 休眠（本阶段离线测试） | `ui-verify.js --target .` |
| verification-plan.js | 规划验证命令 | 未接 | `harness.js verify plan` |
| verification-runner.js | 跑单条验证命令 | 未接 | `harness.js verify run` |
| verification-suite.js | 跑验证套件 | 未接 | `harness.js verify suite` |
| workbench-shape-gate.js | 产品形状/ratchet 门（HG-3 加去重 warning check） | **承重**（审计 371 次） | `workbench-shape-gate.js --mode baseline\|check` |

## 内部库 `lib/`（13，require-only）

| 脚本 | 干啥（一行） | 状态 | 被谁调 |
| --- | --- | --- | --- |
| lib/agentmemory-client.js | agentmemory HTTP 客户端 | 休眠·待定 | memory-agentmemory-* |
| lib/check-runner.js | 检查执行器（聚合门共用） | 未接·元 | doctor/pre-work/pre-completion |
| lib/config-loader.js | 加载 harness.config.json | 未接·元 | 几乎所有脚本 |
| lib/context-pack.js | context-pack 核心逻辑 | 未接 | context-pack.js |
| lib/evidence-audit.js | 证据审计核心逻辑 | 未接 | evidence-check/-freshness |
| lib/manifest.js | 读写 .harness/manifest.json | 未接·元 | install/sync/managed-files-audit |
| lib/memory-governance.js | 记忆治理核心逻辑 | 休眠·待定 | memory-* |
| lib/mistake-retrieval.js | 错误账本检索核心逻辑 | 未接 | mistake-check/-query |
| lib/project-kind.js | 探测项目类型 | 未接·元 | project-profile/risk |
| lib/risk-classifier.js | 风险分类器 | 未接·元 | task-risk |
| lib/security.js | 注入/密钥扫描核心逻辑 | 未接 | security-scan |
| lib/skill-index.js | skill 索引/匹配 | 未接 | skill-recommend |
| lib/task-package-schema.js | 任务包 schema | 未接 | task-package-new/-lint |

## 特别标注（本包不替用户决定）

- **agentmemory 9 件**（memory-agentmemory-query/-save、memory-candidate-new/-lint、memory-review、memory-stale-check、memory-maintenance、lib/agentmemory-client、lib/memory-governance）：`休眠·待定`。config `memoryIntegration.enabled=false`，且产品已改走 Claude 文件记忆 + `handoffs/`，agentmemory 可能**永久不启用**。**是否永久退役待用户拍**——本索引只登记，不决定。
- **capability-map.js**：`退役候选`，与 `capability-scan.js` 目标重复（都扫项目能力，arg 解析近乎雷同）。HG-2 ⑩ 接通 capability-scan 后，在此把 capability-map 翻成 `退役·被 capability-scan 取代`，**文件留着不删**。

## 维护约定

- 接线/退役状态变化时，更新对应行的`状态`列，并在统计行同步计数。
- 真要删脚本前：先确认它在本表是 `退役`，且无其他脚本 `require` 它（查 `lib/` 表的"被谁调"）。
