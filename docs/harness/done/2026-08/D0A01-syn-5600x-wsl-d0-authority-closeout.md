# D0A01 Primary/Edge 架构候选与 5600X/WSL 开发迁移权威收口

阶段：stage-08 阶段8 Syn Primary/Edge D0 文档与迁移权威收口
目标：把外部草案修订为不冒充现行架构或执行授权的仓内候选，并建立一份把开发迁移、Headless 实现和正式 Primary 切换严格分开的可执行计划。
干完的标准：三份业务文档互相引用且状态一致；四项已知冲突全部修正；每个迁移阶段都有成功门、停止门和回滚；新鲜机械检查通过；未发生联网、设备连接、源码迁移、产品实现或 Git add/commit/push。

允许动：

- docs/product/syn-primary-edge-core-distributed-runtime-architecture-candidate-v2.md [新增]
- docs/product/candidate-register-v1.md
- docs/plans/2026-08-13-syn-5600x-wsl-development-environment-migration-plan-v1.md [新增]
- docs/harness/authorization.json
- docs/harness/plan.md
- docs/harness/stages/stage-08.md
- docs/harness/leaves/D0A01-syn-5600x-wsl-d0-authority-closeout.md
- docs/harness/done/2026-08/D0A01-syn-5600x-wsl-d0-authority-closeout.md [新增]
- docs/harness/done/2026-08/stage-08.md [新增]
- docs/harness/audit/2026-08.jsonl
- docs/harness/usage/.turn

## 步骤

1. 固定外部草案 SHA-256、当前 HEAD、工作树现有改动、M1–M10 和 Harness 生命周期事实。
2. 新建架构候选，修正时间线、Mac UI 过渡、权威引用和阶段状态，并把设备操作移出架构候选。
3. 在唯一候选登记新增 Primary/Edge 转正门，明确当前未实现和重新评估条件。
4. 新建 A–G 开发迁移计划，逐阶段写清操作者、动作、成功标准、停止门、回滚和权限边界。
5. 机械检查链接、状态词、禁区、外部来源 hash、Git diff 和 Harness 链；完成后归档本叶与 stage-08，并停止等待 B 阶段授权。
