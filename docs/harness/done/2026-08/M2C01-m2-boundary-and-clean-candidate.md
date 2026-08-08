# M2C01 M2 边界冻结与干净候选提交

阶段：stage-03 M2 主线收口与交接
目标：把混合 M2 工作树中真正属于 M2 的产品实现、必要映射和证据精确提取到独立干净 worktree。
干完的标准：来源哈希与 WIP 哨兵复核无漂移；候选文件逐项可归因；候选分支形成独立本地提交；原混合工作树零写入。

允许动：

- /Users/yoyi/workspace/product-line-syn-m2-closeout/ [新增]
- refs/heads/codex/syn-m2-closeout [新增]
- docs/harness/audit/ [新增]
- docs/harness/reports/ [新增]

## 步骤

1. 复核 M2 worktree、main、receipt、7 个绑定源和 13 项战略 WIP 哨兵。
2. 依据权威 M2 退出条件、测试调用链和证据文档，把产品/测试/Code Map/证据分为纳入、保留 WIP、排除。
3. 从 main 建独立候选 worktree，只应用纳入清单；处理 Harness Lite 迁移后的文档形状差异。
4. 运行候选聚焦检查与 `git diff --check`，精确暂存并本地提交。
