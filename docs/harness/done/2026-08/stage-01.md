# 阶段1 代码事实收敛、唯一权威与 Lite 切换

总计划：product-line 唯一基线与 Harness Lite 切换

目标：在独立迁移 worktree 中收敛代码与项目事实，演练并完成旧 Harness 退出和 Lite 切换；不丢失任何既有 WIP，不继承旧授权。

干完的标准：

- 所有 product-line worktree、分支、独有提交和脏改都有可复核去向。
- 迁移分支形成唯一、干净、可复现的候选代码基线；不把旧 Harness 修复冒充产品成果。
- 项目活动权威只有 `AGENTS.md -> docs/harness/plan.md -> current stage -> unique current leaf -> authorization.json`。
- Adaptive Harness v0.5 runtime、manifest、配置与活动入口退出；历史材料与产品资料保留。
- `AGENTS.md`、`CLAUDE.md` 保留项目规则并人工适配 Lite，不由安装器模板覆盖。
- Lite ownership、chain、progress、auth、quick/task、Stop 和重复安装通过；Stop 不跑产品测试。
- 不 push、不部署、不发布；两个既有脏 worktree 的 branch/HEAD/staged/内容哨兵不变。

允许动：

- `/Users/yoyi/workspace/product-line-lite-migration/` 中具体 leaf 列出的 Git、Harness 与权威入口
- `/private/tmp/product-line-harness-lite-*` [新增]
- `docs/harness/audit/` [新增]
- `docs/harness/reports/` [新增]

只读范围：

- `/Users/yoyi/workspace/product-line`
- `/Users/yoyi/workspace/product-line-harness-i5-repair`
- `/Users/yoyi/workspace/product-line-syn-fnd-001`
- `/Users/yoyi/workspace/product-line-syn-fnd-002`
- `/Users/yoyi/workspace/product-line-syn-integration-main`
- `/Users/yoyi/workspace/product-line-syn-m1-baseline`
- product-line 共享 Git 对象与本地 refs

不许动：

- `/Users/yoyi/workspace/product-line` 的既有 tracked/untracked WIP
- `/Users/yoyi/workspace/product-line-syn-fnd-002` 的既有 tracked/untracked WIP
- 未经 leaf 精确列出的产品实现
- 远端、push、部署、发布、provider、数据库、浏览器、真实账号和真实消息

## 叶子

- [x] PLH001 workspace、代码与 Harness 事实冻结
- [x] PLH002 隔离代码收敛演练
- [x] PLH003 唯一候选代码基线落迁移分支
- [x] PLH004 唯一项目权威与旧 Harness 卸载演练
- [x] PLH005 真实卸载旧 Harness并完成 Lite 适配
- [x] PLH006 分区验收、回滚证据与阶段收口

<!-- 当前 authorization.json 不存在；旧 Harness 的 READY、任务包、授权和
     Harness Lite 源仓 Stage-07 授权均不进入本项目。 -->
