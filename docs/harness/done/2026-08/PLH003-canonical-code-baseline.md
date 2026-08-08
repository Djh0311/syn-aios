# PLH003 唯一候选代码基线落迁移分支

阶段：stage-01 代码事实收敛、唯一权威与 Lite 切换

目标：按 PLH002 的精确结果，把仍有效的既有代码成果收敛到 `codex/product-line-lite-migration`；不吞入脏 worktree 的未确认内容。

干完的标准：迁移分支对应唯一可复现代码树；来源提交和冲突处理可追溯；两个脏现场哨兵不变；相关小检查通过。

允许动：

- 本 worktree 中 PLH002 最终清单列出的 Git 路径
- `docs/harness/audit/` [新增]
- `docs/harness/reports/` [新增]

不许动：

- PLH002 未列出的产品文件
- 两个既有脏 worktree
- 旧 Harness 迁移文件（留给 PLH004/PLH005）
- main、push、远端和发布

## 停止条件

- PLH002 输入与真实 HEAD、索引或工作树发生漂移。
- 冲突需要创造新产品行为，而不是收敛已有成果。
- 需要改写或丢弃用户 WIP。
