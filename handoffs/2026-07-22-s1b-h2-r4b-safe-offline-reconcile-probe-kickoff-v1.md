# Kickoff：S1B-H2-R4B 安全离线 reconcile probe v1

执行 `S1B-H2-R4B 安全离线 reconcile probe v1` 前，先完整阅读：

`tasks/2026-07-22-s1b-h2-r4b-safe-offline-reconcile-probe-package-v1.md`

R4A 已确认 `project_proposals` 的 74 个 natural key 完全相同，排除了 count/key/default-field presence 的简单解释；但当前 production surface 没有可安全调用的 Rust canonical reconciliation 命令。

本包需要新的精确授权，才可在仓外 0700 私有临时目录创建最小离线 copy，并以 test-only redacted probe 直接运行 `reconcile_db_vs_json`。不得启动 App、写真实 store、重 seed、发送 H2 消息、扩大工具审批、输出原文或完整 ID。未获该授权时不得复制、build 或运行 probe。
