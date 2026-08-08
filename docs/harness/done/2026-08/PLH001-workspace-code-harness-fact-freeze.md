# PLH001 workspace、代码与 Harness 事实冻结

阶段：stage-01 代码事实收敛、唯一权威与 Lite 切换

目标：用只读证据冻结全部 product-line worktree、分支、提交、dirty/staged、旧 Harness 归属和活动 consumer，给出唯一代码基线候选及逐项去向。

干完的标准：代码拓扑、脏改分区、旧 Harness 四类清单、项目权威冲突和 PLH002 固定输入均有可重放证据；目标项目零写入。

允许动：

- `docs/harness/audit/` [新增]
- `docs/harness/reports/` [新增]

只读范围：

- stage-01 列出的全部 worktree、分支和共享 Git 对象

## 步骤

1. 冻结 worktree/branch/HEAD/ahead-behind/ancestor/unique commits/upstream。
2. 冻结 tracked/untracked/staged，并按 Harness、项目文档、产品源码、测试/证据分区。
3. 按 manifest/hash/consumer 将旧 Harness 分为删除、迁移、保留、历史。
4. 对照 main HEAD、源码与当前 authority，列出事实冲突和未知项。
5. 产出 PLH002 的提交顺序、排除项、保护哨兵和停止条件。

## 验证

- 六个既有 worktree 的 branch/HEAD/staged 和 dirty 指纹前后相同。
- 所有计数可由报告里的 Git/manifest 命令重放。
- 未运行产品测试、服务或外部动作。
