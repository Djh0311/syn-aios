# PLH201 Lite 标准 Agent 规则纠正与认领

阶段：stage-02 Lite Agent 规则权威纠正
目标：用 Lite 0.4.0 标准 AGENTS/CLAUDE 替换迁移期厚适配，显式认领并证明未来升级安全。
干完的标准：两份规则逐字匹配 Lite 源仓；ownership 0.4.0 认领它们；普通重复升级 payload 零写入；项目检查与 Hook 仍各自有效；产品代码和两个脏 worktree 不变。

允许动：

- `AGENTS.md`
- `CLAUDE.md`
- `.claude/harness-lite/`
- `.codex/harness-lite/`
- `docs/harness/plan.md`
- `docs/harness/authorization.json`
- `docs/harness/stages/stage-02.md`
- `docs/harness/leaves/PLH201-lite-agent-rule-authority.md`
- `docs/harness/done/` [新增]
- `docs/harness/audit/` [新增]
- `docs/harness/reports/` [新增]
- `docs/harness/check-results.jsonl`
- `docs/harness/usage/`

## 步骤

1. 冻结当前规则 hash、ownership、产品 diff、staged 和两个脏 worktree 哨兵。
2. 先把 AGENTS/CLAUDE 精确替换为 Lite 0.4.0 标准，再运行显式安全认领。
3. 验证 ownership、普通重复升级零 payload 写入、chain/progress/auth、quick/task、Hook 和产品零 diff。
4. 归档 Stage 2，精确暂存并本地提交，不 push。
