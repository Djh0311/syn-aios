# 阶段2 Lite Agent 规则权威纠正

总计划：product-line 唯一基线与 Harness Lite 切换
目标：把 AGENTS/CLAUDE 恢复为 Lite 0.4.0 标准规则并纳入 ownership，使 agent 规则随 Lite 安全升级；项目检查、Git Hook 和产品资料各留在自己的载体中。

允许动：

- `AGENTS.md`
- `CLAUDE.md`
- `.claude/harness-lite/`
- `.codex/harness-lite/`
- `docs/harness/plan.md`
- `docs/harness/authorization.json`
- `docs/harness/stages/`
- `docs/harness/leaves/`
- `docs/harness/done/`
- `docs/harness/audit/`
- `docs/harness/reports/`
- `docs/harness/check-results.jsonl`
- `docs/harness/usage/`

只读：

- `/Users/yoyi/harness engineering/harness-lite@19be06b`
- `/Users/yoyi/workspace/product-line`
- `/Users/yoyi/workspace/product-line-syn-fnd-002`

不许动：

- `.githooks/`、`docs/harness/checks.json`、`docs/harness/policy.json`
- `prototypes/` 和其他产品代码、产品架构、decisions、数据与资产
- 其他 worktree 的 tracked/untracked/ignored 内容
- 网络、远端、push、部署、发布、provider、数据库、浏览器和真实消息

## 叶子

- [x] PLH201 Lite 标准 Agent 规则纠正与认领
