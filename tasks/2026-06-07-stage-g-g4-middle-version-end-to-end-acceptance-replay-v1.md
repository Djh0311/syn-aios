# Task Package: Stage G / G4 Middle Version End-to-End Acceptance Replay v1

状态：已完成 / accepted_with_deferred_items。  
用途：对中间版本 C / D / E / F / G 主链路做离线端到端回放和验收矩阵冻结；本文不新增产品能力，不执行真实 Codex，不读写 `/Users/yoyi/.codex`。

## 0. 先说薄弱点

- G3-B 没有完成 13 / 13 张真实 Tauri 截图，G3-C 已将缺口冻结为 deferred；G4 必须携带该缺口，不能包装成真实 Tauri 全量通过。
- G4 容易被误做成单次 demo；本任务必须覆盖 C / D / E / F / G 的主链路和失败边界，而不是只跑一条 happy path。
- G4 默认离线 fixture 回放，不执行真实 `codex exec` / `codex exec resume`，不发送真实 prompt，不读写 `/Users/yoyi/.codex`。
- readback unavailable、diagnostics warning、runtime log 摘要不能被包装成真实成功结果。
- UI 回放只能验证中间版本应可理解，不新增产品能力，不修复 G3-B/G3-C 截图缺口。

## 1. 已知事实 / 未知 / 假设

已知事实：

- 阶段 C / C1-C6 已完成，接受为受控自动化工作流闭环。
- 阶段 D / M1-M13 已完成，M13 结论为 `accepted_with_deferred_items`。
- 阶段 E / E1-E7 已完成，结论为 `accepted_with_deferred_items`；E5 Level B 只接受为指定 mario test session 的最小真实 resume 健康探针。
- 阶段 F / F1-F5 已完成，结论为 `accepted_with_deferred_items`。
- G1 Runtime Log Boundary And Minimal Store 已完成。
- G2 Diagnostics Health And Degraded State 已完成。
- G3-A Real Tauri Acceptance Plan And Fixture Freeze 已完成。
- G3-B Real Tauri Manual Screenshot Acceptance 已回交但未完成，只接受为 10 / 13 真实 Tauri 部分截图证据。
- G3-C Screenshot Evidence Recovery And Gap Matrix 已完成，结论为 `accepted_with_deferred_items`。

未知：

- G5 最终冻结是否会把 G3-C 的 3 个 deferred 截图项放入后续真实 Tauri 安全 fixture，还是放入最终蓝图验收 backlog。

假设：

- G4 在 G3-B 回交且 G3-C 缺口矩阵完成后执行；G3-B 不必被包装成完成。
- G4 使用受控 fixture / 测试项目离线回放，不触发真实外部 agent、真实 provider 或真实 Codex。
- G4 可以引用 G1 `runtime_log_store` 和 G2 `diagnostic_summary` 的脱敏读模型作为回放输入。
- G4 不创建或修改产品 fixture 文件；只创建 evidence / handoff 文档。

## 2. 前置条件

派发前置：

- G3-B 已回交，并有截图目录、截图索引和未覆盖项记录。
- G3-C 已完成截图证据回收和缺口矩阵，明确哪些页面 / 状态通过、哪些 deferred。
- G1 / G2 / G3-A / G3-B / G3-C 的 task、evidence、handoff 可追溯。
- 全局主管确认 G4 只做离线 fixture 回放，不执行真实 Codex，不读写 `/Users/yoyi/.codex`。
- 本轮不新增或修改 fixture 文件。

## 3. 接受范围

接受为：

- G4 离线端到端回放的主链路 fixture 已准备或可由现有读模型组合。
- C / D / E / F / G 之间的用户可理解路径被逐项回放并形成验收矩阵。
- 回放覆盖成功、阻断、readback unavailable、planned adapter unavailable、diagnostics warning / degraded、runtime log / audit 分层等关键边界。
- 每个阶段只声明该阶段已完成的能力，不把 deferred 项包装成完成。
- 输出 G4 evidence / handoff 时能明确哪些是 accepted、哪些是 accepted_with_deferred_items、哪些需要 G5 冻结。

不接受为：

- G5 最终冻结或阶段 G 完成。
- 真实 Codex / worker 执行完成。
- 通用 send / resume 产品化完成。
- planned adapters 真实接入。
- provider credential / model verification 完成。
- 自动重试、自动修复、自动恢复完成。
- 普通浏览器 smoke 或单次演示替代中间版本端到端验收。

## 4. Fixture 矩阵建议

| Fixture | 覆盖阶段 | 目标 | 必须断言 | 禁止冒领 |
| --- | --- | --- | --- | --- |
| `g4-plan-confirmation-fixture` | C1-C3 | 用户确认方案、全局主管边界复核、授权生效 | proposal / authorization / guard / audit refs 可追溯；高风险仍需确认 | 不说成用户已批准所有未来操作 |
| `g4-project-director-dispatch-fixture` | C4 | 项目主管拆任务和 prepared dispatch | task package、memory packet、授权边界、prepared 状态可理解 | 不说成真实 worker 已执行 |
| `g4-worker-report-process-fact-fixture` | C5-C6 | worker 汇报、项目主管过程事实确认、全局最终结果复核 | worker report 与 process fact 区分；结果验收进入用户可理解摘要 | 不把 worker 汇报直接写正式事实 |
| `g4-memory-candidate-formal-fixture` | D / M1-M6 | observation -> candidate -> formal memory -> task memory packet | observation / candidate / formal memory / audit / source refs 分层清楚 | 不把 observation、candidate、knowledge hit 当正式记忆 |
| `g4-memory-lifecycle-governance-fixture` | D / M7-M13 | 记忆中心、lifecycle、lint、maintenance、entity relation、mature pattern gate | 正式记忆变更走提案 / 影响面 / 确认 / 审计；冲突进待办 | 不说成 GraphRAG、向量库、自动技能化完成 |
| `g4-agent-session-boundary-fixture` | E1-E7 | adapter、model / credential、send / resume preview / stub、runtime attention | `codex-local` 与 planned adapters 区分；E5 Level A/B 口径准确；readback unavailable 不是 0 条结果 | 不说成通用真实 send / resume 产品化 |
| `g4-project-workflow-canvas-fixture` | F1-F5 | 项目工作流画布、节点详情、编辑提案、项目 / 实验画布边界 | React Flow 只渲染；workflow 事实变更只生成 proposal；项目画布和实验画布不混 | 不说成画布编辑器或实验运行写项目事实 |
| `g4-runtime-log-diagnostics-fixture` | G1-G2 | runtime log、audit boundary、diagnostics、degraded state | runtime log 只引用 audit refs；diagnostics 只读解释，不修复 | 不说成自动修复、自动重试或 G3/G4/G5 完成 |
| `g4-tauri-evidence-reference-fixture` | G3 | 引用 G3-B/G3-C 截图和缺口矩阵 | 真实 Tauri 截图路径、步骤、缺口可追溯 | 不用普通浏览器 smoke 替代真实 Tauri |

## 4.1 回放验收矩阵

| 回放项 | 覆盖阶段 | 输入证据 | 回放判断 | deferred / 风险 |
| --- | --- | --- | --- | --- |
| 方案确认与授权 | C1-C3 | C1/C2/C3 task、evidence、handoff | accepted | 不代表用户批准所有未来操作 |
| 项目主管拆任务与 prepared dispatch | C4 | C4 task、evidence、handoff；M4/M6 任务记忆包证据 | accepted | 不代表真实 worker 已执行 |
| worker 汇报和过程事实确认 | C5-C6 | C5/C6 task、evidence、handoff；Observation / process fact 边界 | accepted | worker 汇报不是正式事实；observation 不是正式记忆 |
| 记忆候选到正式记忆闭环 | M1-M6 | M1-M6 task、evidence、handoff | accepted | 不代表完整记忆系统或真实 worker 执行 |
| 记忆中心 / 知识库 / lifecycle / entity / maintenance / mature pattern | M7-M13 | M7-M13 task、evidence、handoff | accepted_with_deferred_items | 真实窗口截图不全；GraphRAG / 向量库 / 自动技能化 deferred |
| adapter / provider / session operation / continuation | E1-E7 | E1-E7 task、evidence、handoff；E5 Level B 独立健康探针 | accepted_with_deferred_items | 通用真实 send / resume 产品化、planned adapters、provider credential 验证 deferred |
| 项目工作流画布产品化 | F1-F5 | F1-F5 task、evidence、handoff | accepted_with_deferred_items | 真实 Tauri 截图仍由 G3 部分覆盖；画布编辑器 deferred |
| runtime log / diagnostics | G1-G2 | G1/G2 task、evidence、handoff | accepted | 不代表自动修复、自动重试 |
| 真实 Tauri 截图证据 | G3-A/G3-B/G3-C | G3-A/B/C task、evidence、handoff；截图目录 | accepted_with_deferred_items | 05 智能体、06 send/resume、09 任务记忆包预览 deferred |

## 4.2 回放结论

中间版本主链路可以被用户理解并可回收：

1. 用户确认方案，进入授权对象和全局边界复核。
2. 项目主管在授权范围内拆任务、生成任务包和任务记忆包，并只准备 dispatch。
3. worker 汇报、项目主管过程事实确认和最终结果复核保持分层，不把 worker 汇报直接写正式事实。
4. observation、candidate、formal memory、lifecycle、lint、maintenance、mature pattern gate 的记忆链路可追溯。
5. adapter / provider / session continuation 均有只读边界；E5 Level B 只作为指定 mario test session 健康探针，不扩展为通用能力。
6. 项目工作流画布、节点详情、编辑边界和项目 / 实验画布边界可解释。
7. runtime log 和 diagnostics 进入管理入口的只读健康 / 最近错误 / degraded 摘要。
8. 真实 Tauri 截图证据已有 10 / 13，并由 G3-C 冻结缺口。

因此 G4 可接受为离线端到端回放完成，结论为 `accepted_with_deferred_items`。

## 5. 回放顺序建议

1. 方案入口：展示用户确认方案、全局主管边界复核和授权状态。
2. 项目执行：展示项目主管如何拆任务、准备任务包和任务记忆包。
3. Worker 汇报：展示 worker 汇报、项目主管过程事实确认、失败 / readback / 权限边界。
4. 记忆链路：展示 observation、candidate、formal memory、lifecycle、lint、maintenance、mature pattern gate。
5. 会话边界：展示 adapter、provider availability、session operation、send / resume preview、stub attempt、runtime attention。
6. 画布理解：展示项目工作流画布、节点详情、编辑提案和项目 / 实验画布边界。
7. 运维理解：展示 runtime log、diagnostics、degraded state、数据位置和最近错误。
8. G3 证据引用：引用真实 Tauri 截图和 G3-C 缺口矩阵，说明哪些已真实窗口验证。
9. 最终摘要：输出 accepted / deferred / blocked 的回放矩阵，供 G5 最终冻结使用。

## 6. UI 显示边界确认

本任务是否改前端：

- [x] 不改前端、不改读模型、不改 UI 文案。
- [ ] 改前端类型 / Tauri wrapper，但不新增可见 UI。
- [ ] 改读模型摘要或状态显示。
- [ ] 改已有页面局部 UI。
- [ ] 新增入口、面板、tab、按钮或确认动作。

已读取 / 必读：

- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `docs/plans/2026-06-06-stage-e-f-g-refinement-plan-v1.md`
- `docs/workbench-frontend-display-boundary-v1.md`
- G1 task / evidence / handoff
- G2 task / evidence / handoff
- G3-A task / evidence / handoff
- G3-B task / evidence / handoff，执行 G4 时必须补读
- G3-C task / evidence / handoff，执行 G4 时必须补读

回放必须检查：

- 项目页仍以项目工作流画布为主，不变成任务包管理器。
- 智能体页默认只显示主对话和必要摘要，不默认展示 raw transcript、internal key、sqlite / index 来源或原始 event log。
- 记忆和知识库不混成一套；知识库不能直接写正式记忆。
- 秘书不批准权限、不确认 worker 汇报、不写正式记忆、不派任务。
- 通知、待办、运行中分开。
- 管理入口显示 runtime log、diagnostics、健康和最近错误，raw log / internal id 仅限详情或开发者模式。
- 权限、错误、证据都用人话表达。
- 开发者模式默认关闭。

## 7. 验证清单建议

文档 / 矩阵验证：

- 每个 fixture 都有对应阶段、输入读模型、预期 UI 状态、预期边界和不接受范围。
- 每条回放路径都有可追溯 task / evidence / handoff 引用。
- G3-B/G3-C 的截图和缺口矩阵被引用，但不被 G4 修改。
- 所有 deferred 项保留 deferred，不包装为 accepted。

安全验证：

- 不执行真实 `codex exec`。
- 不执行真实 `codex exec resume`.
- 不发送真实 prompt。
- 不读写 `/Users/yoyi/.codex`。
- 不读取完整 transcript / rollout、auth、token、`.env`、secret、keychain、OAuth、provider credential。
- 不调用外部模型或 provider。
- 不写正式记忆、workflow state、observation、candidate、runtime log，除非正式 G4 任务包另行授权并列清路径。

口径扫描建议：

- 不出现 `G4 已完成`、`G5 已完成`、`阶段 G 已完成`。
- 不出现 `真实 send/resume 已产品化`、`planned adapter 已接入`、`provider credential 已验证`。
- 不出现 `readback unavailable = 0 条结果`。
- 不出现 `runtime log 替代 audit` 或 `audit 替代 runtime log`。
- 不出现 `普通浏览器 smoke 替代真实 Tauri`。

## 8. evidence / handoff 要求

正式执行 G4 时才允许创建：

- `evidence/2026-06-07-stage-g-g4-middle-version-end-to-end-acceptance-replay-v1.md`
- `handoffs/2026-06-07-stage-g-g4-middle-version-end-to-end-acceptance-replay-v1-result.md`

G4 evidence 必须包含：

- fixture 矩阵。
- 回放步骤。
- 每步 accepted / deferred / blocked 判断。
- G3-B/G3-C 截图证据引用。
- G1 runtime log 和 G2 diagnostics 引用。
- 未覆盖项和不能冒领项。

G4 handoff 必须包含：

- G4 是否可接受。
- 是否允许进入 G5。
- 哪些 deferred 项必须由 G5 freeze。
- 是否发生任何边界偏差。

## 9. 验收记录

已完成：

- 读取当前权威入口。
- 读取 G1 / G2 / G3-A / G3-B / G3-C 任务包和回收记录。
- 复核 G3-C 缺口矩阵。
- 建立 C / D / E / F / G 主链路回放验收矩阵。
- 创建 G4 evidence / handoff。

未执行：

- 未执行真实 `codex exec`。
- 未执行真实 `codex exec resume`。
- 未发送真实 prompt。
- 未读写 `/Users/yoyi/.codex`。
- 未读取完整 transcript / rollout、auth、token、`.env`、secret、keychain、OAuth、provider credential。
- 未调用外部模型或 provider。
- 未改产品功能代码。
- 未写 workflow state、formal memory、observation、candidate 或 runtime log。

## 10. 下一步

下一步进入 G5 Final Authoritative Acceptance And Deferred Freeze。

G5 必须冻结：

- G4 `accepted_with_deferred_items` 结论。
- G3-B/G3-C 的 3 个真实 Tauri 截图 deferred 项。
- E5 Level B 只作为指定 session 健康探针，不扩展为通用 send / resume。
- 阶段 G 未完成项和最终蓝图 / 后续 backlog 归属。
