# M3C01 RoleSession / Turn / Handoff 实施合同与迁移矩阵

阶段：stage-05 阶段5 M3 角色会话与显式交接
目标：保留 M1 冻结合同不变，用一份增量实施补充合同冻结 M3 的稳定 join、provider natural key、碰撞/孤儿、权限漂移、重启、最小知识上下文、Handoff 完整状态和 legacy 迁移矩阵。
干完的标准：新增合同结构块可机械解析；M1 合同 hash 不变；所有 M3 HOLD 都有 fail-closed 实施规则或明确后置 owner；下一个 owner/scope 守卫叶不再需要重问产品方向。

允许动：

- docs/contracts/m3-role-session-turn-handoff-resolution-v1.md [新增]

## 步骤

1. 固定 M1 role-session-v1 / handoff-v1 和 M3 计划输入 hash，确认新增目标文件不存在。
2. 冻结 RoleSession / Turn / ProviderHandle / ConversationContext / Handoff 的 key、状态、不变量、幂等与敏感字段边界。
3. 冻结 permission drift、collision/quarantine、restart orphan、timeout/cancel/retry/result-return 行为。
4. 写明 Codex index/rollout、durable supervisor binding、valid continuation、前端 cache 和 raw transcript 的迁移分类。
5. 机械解析结构化 JSON，跑合同边界检查、链接检查与 `git diff --check`。
6. 独立审查通过后精确暂存该文件、本地提交并归档本叶。
