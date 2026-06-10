# Handoff：final-skeleton-12 适配器能力声明骨架 v1

日期：2026-06-03

## 本轮完成

`final-skeleton-12-adapter-capability-registry-v1` 已完成。

完成内容：

1. 新增前端只读适配器能力声明模型。
2. Codex adapter 现在有 `codex-local` descriptor。
3. 智能体页展示 Codex 已有能力和边界。
4. 未实现的 Claude Code / OpenClaw / OpenCode 不显示能力按钮。

本轮没有进入 `final-skeleton-11`，没有实现黑板候选写入。

## 改动文件

- `prototypes/productized-desktop-shell/src/lib/adapterCapabilities.ts`
- `prototypes/productized-desktop-shell/src/views/AgentView.tsx`
- `prototypes/productized-desktop-shell/src/App.tsx`
- `prototypes/productized-desktop-shell/src/styles.css`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`

## 验证结果

通过：

```text
npm run typecheck
npm run test:offline-interaction
npm run build
```

`npm run build` 有 chunk size 提醒，但构建成功。

## 没有做

- 没有接 Claude / OpenClaw / OpenCode。
- 没有改真实 Codex 执行语义。
- 没有执行真实 Codex。
- 没有改 workflow state JSON。
- 没有读写 `/Users/yoyi/.codex`。
- 没有显示未实现能力按钮。
- 没有实现黑板候选写入。

## 后续建议

下一步可以按总包继续 `final-skeleton-13-memory-governance-schema-design-v1`。

如果要把 adapter descriptor 从前端读模型升级为后端 read model，需要单独切片处理，避免误改 workflow state JSON 或真实 Codex 执行路径。
