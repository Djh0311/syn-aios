# Kickoff：S1B-H2-R4A DB/JSON 全语义预检诊断 v1

执行 `S1B-H2-R4A DB/JSON 全语义预检诊断 v1`。

完整阅读并严格遵守：

`tasks/2026-07-22-s1b-h2-r4a-db-json-reconciliation-preflight-diagnosis-package-v1.md`

这是只读诊断授权。先确认 R4-R2 的裸 binary、App/dev/Codex/MCP、registry 和所有 store holder 都为空；任何残留时停止，不 kill。

只定位 project-proposals 的 DB/JSON natural-key/hash mismatch：解释为何 count-level parity 绿而启动期 full reconciliation 不绿。不得启动 App、build、发送 H2 文本、写真实 store、重 seed、改代码或安全闸；不得输出用户正文、proposal 正文、原始错误、auth/token 或私有路径正文。

若现有证据仍不足，只能裁决 `NEEDS_SAFE_OFFLINE_RECONCILE_PROBE` 并另出包；不得猜根因。完成 evidence/CURRENT 最小回写后停止，真实 R4 重验需另授权。
