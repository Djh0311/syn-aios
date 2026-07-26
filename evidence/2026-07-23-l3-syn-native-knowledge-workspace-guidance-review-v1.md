# L3 Syn 原生知识工作区指导线验收 v1

- 日期：2026-07-23
- 验收对象：`tasks/2026-07-23-l3-syn-native-knowledge-workspace-development-package-v2.md`
- 指导线裁决：**N0-N5 离线验收通过；N6 只读闭锁通过，但真实 App 与 `knowledge_open` 原生打开未验收。**

## 1. 本次核对范围

指导线没有把执行线回报直接当作验收结论。本次只读核对了：

- 当前 `CURRENT.md`、`AUTHORITY.md`、v2 decision/plan/task；
- N0-N6 两份执行线 evidence；
- Tauri command 注册、知识页生产路由、原生工作区组件与 typed client；
- MCP capability registry、可信 conversation binding 与 `knowledge_search/read/open/cite` handler；
- `knowledge_open` 从 MCP stdio 子进程到 Syn 主进程的实际数据流。

未启动 Syn App、Obsidian、Codex CLI 或 MCP server，未访问真实 vault，也未生成截图或真实运行日志。

## 2. 独立复跑结果

| 检查 | 指导线结果 |
| --- | --- |
| `cargo test knowledge_ --lib` | 42 passed、0 failed、1 ignored；ignored 项正是 host dispatch 红合同。 |
| `cargo check --lib` | 通过；598 条项目既有 warning。 |
| `npm run typecheck` | 通过。 |
| `node scripts/run-offline-interaction-test.mjs` | 通过，15 项。 |
| 目标 Rust `rustfmt --check` | 通过。 |
| `git diff --check` | 通过。 |
| `git diff --cached --name-only` | 空。 |
| `workbench-shape-gate --mode check` | 失败：16 error / 5 warning / 5 info；与执行线报告一致，仍是聚合历史债，不写成绿色。 |

## 3. 生产接线核对

- `command_registry.rs` 已注册固定的 snapshot/search/graph/read、Markdown、Canvas、附件与恢复命令；前端不能传 command 名、vault root、shell 或任意文件系统路径。
- `ActiveWorkbenchView.tsx` 的 `knowledge` 生产路由进入 `KnowledgeBaseView.tsx`，原生 Markdown、图谱、Canvas 和维护区不是测试专用孤岛。
- `knowledge_capabilities.rs` 的 search/read/cite 只使用固定 Syn vault 与已验证 `relative_path`；`knowledge_open` 也先读取已验证 Markdown 投影。
- capability registry 的共享主管 allowlist 是 `submit_proposal + knowledge_search/read/open/cite`；写类知识工具保持拒绝。

以上足以支持 N0-N5 的**离线代码与合同验收**，但不替代桌面 App 的实际布局、键盘、焦点、文件选择、重启恢复和 MCP 会话验收。

## 4. N6 精确停点

当前 `knowledge_open` 返回：

```json
{
  "target": "syn_native_view",
  "dispatch_status": "trusted_host_dispatch_required",
  "opened": false,
  "external_open_requested": false
}
```

这是诚实的 fail-closed 结果，但它仍被普通 `Ok` 结果包裹；在真正的 relay 落地前，不得把该 tool result 解释为原生视图已打开。

缺口不是 Markdown 路径校验，而是：

1. MCP stdio 子进程把已验证目标交给 Syn 主进程；
2. 主进程复核当前 Active binding 和短期 grant；
3. App 切到知识页并让原生工作区真正读取、选中、聚焦目标；
4. UI 把精确 intent 的完成/拒绝回给主进程；
5. 主进程再把有界 ack 交还 MCP handler。

路径不能写进 conversation binding、静态全局、DB/JSON schema 或新 sidecar；否则只是用第二真相源绕过缺口。

## 5. 验收裁决

- **通过**：N0-N5 的离线实现、生产注册、路径/冲突/只读边界和独立离线门。
- **通过但不等于产品完成**：N6 的 capability allowlist 与 fail-closed 安全停点。
- **未通过**：`knowledge_open opened=true`、N6 十二项真实 App 场景、视觉与桌面交互验收。

下一步以 `tasks/2026-07-23-l3-knowledge-open-host-owned-relay-and-real-app-acceptance-package-v1.md` 为唯一执行包。

