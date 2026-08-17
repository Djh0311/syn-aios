# M1I01R03R01 共享 AppState 验收 profile 隔离

阶段：stage-14 仍开。本文件只是独立组合边界纠正投影，**不是** current leaf。唯一 current leaf 保持 `M5R07-project-ui-isolated-app-and-stage-candidate`。`authorization.json` 保持精确 closed 两字段。

目标：让隔离验收 `AppState` 与遗留组合保持 M1 / M3 未安装；普通 Tauri 产品继续安装。未安装 accessor 返回稳定不可用码。不改变权威行为或登记语义。不声称 M1 / M3 已解阻。

来源：独立验收拒绝 `061eefee9291dbeddf792af6d78dc48bb5b0f8e5`。

产品：`docs/contracts/m1-m3-shared-appstate-acceptance-profile-isolation-v1.md`，显式 `SharedProductAuthorityProfile`，真实隔离验收构造测试。

既有 `M1I01R03` / `M3O01R01` 报告与 unfinished note 保留为历史 candidate 证据。
