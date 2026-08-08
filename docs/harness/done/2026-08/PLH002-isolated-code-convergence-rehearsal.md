# PLH002 隔离代码收敛演练

阶段：stage-01 代码事实收敛、唯一权威与 Lite 切换

目标：只在临时 worktree/副本演练 main 与仍有效提交链的收敛，分离产品成果、历史 WIP 和即将退出的旧 Harness 修复。

干完的标准：候选提交顺序、冲突文件、保留/排除决定、目标 tree、相关小测试和回滚步骤完整；真实既有 worktree 零写入。

允许动：

- `/private/tmp/product-line-harness-lite-code-rehearsal-*` [新增]
- `docs/harness/audit/` [新增]
- `docs/harness/reports/` [新增]

## 步骤

1. 以 PLH001 冻结 OID 建隔离候选树。
2. 演练仍有效提交；旧 Harness I5 repair 单独拆出，不当作产品完成证明。
3. 冲突逐文件核对直接事实，归属不唯一时停止。
4. 只跑与候选集成直接相关的小检查。
5. 产出 PLH003 精确 Git 动作和文件清单。
