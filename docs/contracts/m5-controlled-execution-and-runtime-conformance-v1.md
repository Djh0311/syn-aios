# M5 受控执行与 Runtime Conformance 合同 v1

- 版本：v1（2026-08-16）
- 状态：**FROZEN（M5R05 冻结）**
- 关系：补充 M5R02 Grant 链与 M5R03 RuntimeReceipt；**不改 M1–M4 正文与 hash**。

## 1. 持久控制

DurableOperation / lease / checkpoint / dead letter 由 Syn store 持有。`stop` / `retry` / `resume` 必须改变持久状态，不得只返回文字。未知外部结果先按 effect id readback/reconcile，禁止盲重跑。

## 2. Runtime

- `AgentRuntimeAdapter` 是 vendor-neutral 合同。
- Syn-native 默认实现进入普通产品 composition（非 `#[cfg(test)]`）。
- 第二实现必须状态语义独立，禁止复制一份 fake。
- runtime event/trace 不能自动成为项目事实。
- DSH Approval/Sandbox 只是第二道防线；dynamic package 默认关闭。
- Child run 不得扩大 parent Grant。

## 3. 产出

每次受控执行产出 `RuntimeReceipt`（receipt_id、grant_id、attempt_id、dispatch_id、effect_id、trace_hash、actor_binding、enforcement_status、outcome）供 M5R03 独立验证。
