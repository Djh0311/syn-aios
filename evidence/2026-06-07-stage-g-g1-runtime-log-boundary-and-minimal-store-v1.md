# Evidence: Stage G / G1 Runtime Log Boundary And Minimal Store v1

日期：2026-06-07

## 结论

G1 已完成，接受为：

```text
runtime_log_boundary_and_minimal_store_completed
```

只接受为：

- runtime log 与 audit event 边界定义完成。
- `runtime_log_store.v1` 最小结构完成。
- 六类运行记录最小支持完成：app session、workflow run、dispatch attempt、readback、permission wait、diagnostic event。
- `WorkbenchSnapshot.runtime_log_store` 后端读模型完成。
- 管理入口 runtime log 摘要和分类过滤展示完成。
- 运行日志展示脱敏，不展示 token、secret、完整 transcript、raw provider credential、auth、`.env`、keychain、OAuth 内容。

不接受为：

- G2 diagnostics 完成。
- G3 真实 Tauri / 截图验收完成。
- G4 端到端回放完成。
- G5 最终验收或阶段 G 完成。
- 自动重试、真实执行、真实 prompt 发送、真实 readback 完成。
- GEPA eval export。

## 改动文件

代码：

- `prototypes/productized-desktop-shell/src-tauri/src/runtime_log_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/types.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src/lib/types.ts`
- `prototypes/productized-desktop-shell/src/App.tsx`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`

文档：

- `tasks/2026-06-06-stage-g-g1-runtime-log-boundary-and-minimal-store-v1.md`
- `evidence/2026-06-07-stage-g-g1-runtime-log-boundary-and-minimal-store-v1.md`
- `handoffs/2026-06-07-stage-g-g1-runtime-log-boundary-and-minimal-store-v1-result.md`
- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `README.md`
- `docs/plans/2026-06-06-stage-e-f-g-refinement-plan-v1.md`
- `docs/plans/middleware-version-stage-plan-v1.md`
- `docs/agent-mistake-ledger.md`

## 实现证据

- 后端新增 `runtime_log_store.rs`，支持 `runtime-logs.v1.json` 读取、schema 校验和展示前脱敏。
- 无 sidecar 时从工作台自有 `session_continuation_store` 与 `runtime_session_attention` 派生安全摘要。
- runtime log entry 只保存脱敏摘要、source refs 和 audit refs，不保存 audit event 本体。
- 前端右侧入口保留为 `管理`，没有新增一级入口或额外右侧顶级日志入口。
- 管理入口显示 `日志 / 审计边界`、分类 chip 和 summary counts。

## 验证结果

通过：

```text
cargo test --lib
```

结果：通过，231 passed / 0 failed / 1 ignored。保留既有 warning：`JsonRpcError::invalid_params` dead code。

```text
rustfmt --check src/runtime_log_store.rs src/types.rs src/lib.rs
```

结果：通过。

```text
npm run typecheck
```

结果：通过。

```text
npm run test:offline-interaction
```

结果：通过，输出 `offline interaction tests passed: 12`。

```text
npm run build
```

结果：通过；保留既有 Vite chunk size warning。

普通浏览器检查：

- `npm run dev -- --port 4179` 在沙箱内启动失败：`listen EPERM`。
- 提升权限后 Vite server 成功启动：`http://127.0.0.1:4179/`。
- bundled Playwright 缺 Chromium：`Executable doesn't exist ... chromium_headless_shell`。
- 本机 Chrome headless 尝试失败：`Target page, context or browser has been closed`，并出现 `kill EPERM`。
- 普通浏览器检查未完成。
- 这不等于 G3；真实 Tauri / 截图验收仍未开始。

进程收尾：

- 发现残留 Vite pid `21552` 和 Playwright / Chrome 相关 pid `78430`、`78431`、`78441`、`78442`、`78443`、`78447`、`78448`、`78450`。
- 已执行 `kill 21552 78430 78431 78441 78442 78443 78447 78448 78450`。
- 复查时仅剩本次 `ps` / `rg` 命令自身命中，未见上述 Vite / Playwright / Chrome 残留。

## 脱敏与禁止项

Rust 测试覆盖：

- 输入中包含 token / secret / auth / raw provider credential / full transcript 等敏感片段时，序列化后的 runtime log store 不泄露这些片段。
- 六类运行记录均存在。
- `audit_refs` 只作为引用出现。
- `runtime_log_store.boundary.separation_rule` 明确二者不能互相替代。

前端离线测试覆盖：

- 管理入口显示 `日志 / 审计边界`。
- 管理入口显示六类 runtime log 分类。
- 管理入口不显示敏感片段。
- 不新增右侧顶级日志入口。

## 边界偏差

发生：

- 误读 `/Users/yoyi/.codex/skills/playwright/SKILL.md`。

影响：

- 违反本轮“不读写 `/Users/yoyi/.codex`”边界，因此不能声称“本轮完全未读 `.codex`”。

未发生：

- 未读取用户 Codex 会话数据。
- 未读取完整 transcript / rollout。
- 未读取 auth、token、`.env`、secret、keychain、OAuth、provider credential。
- 未写 `/Users/yoyi/.codex`。
- 未执行真实 `codex exec`。
- 未执行真实 `codex exec resume`。
- 未发送真实 prompt。
- 未调用外部模型或 provider。
- 未做 GEPA eval export。

## 当前结论

G1 已完成。下一步是 G2 Diagnostics / Health / Degraded State 待开始 / 待拆。G2-G5 和阶段 G 不得标记完成。
