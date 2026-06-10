# 任务包：桌面壳 harness resources 真实窗口验证

## 任务名

验证产品化桌面壳在真实 Tauri 窗口里展示 `harness_resources`。

## 所属开发线

验证线。

这是现有验证线任务，不新增常设开发线。

## 背景

桌面应用线已完成 `product-line/tasks/2026-05-28-desktop-shell-harness-resources.md`，把索引内核新增的 `projects[].harness_resources[]` 接入产品化桌面壳展示层。

总指导回收结论：

- 接受为“桌面壳已只读展示文件夹式 harness resources”。
- 不接受为“harness 可运行、已验证、已支持或管理完成”。
- 还缺真实 Tauri 窗口 smoke 验证。

依据：

- `product-line/handoffs/2026-05-28-desktop-shell-harness-resources-review.md`
- `product-line/evidence/2026-05-28-desktop-shell-harness-resources.md`
- `product-line/handoffs/2026-05-28-desktop-shell-harness-resources-result.md`
- `product-line/handoffs/2026-05-28-index-kernel-folder-harness-review.md`
- `product-line/prototypes/index-kernel/codex-index.json`

## 目标

- 在真实 Tauri dev 窗口中启动 `product-line/prototypes/productized-desktop-shell/`。
- 验证窗口能读取当前索引，不停留在普通浏览器保护失败状态。
- 验证 Harness 管理页显示：
  - 文件夹级 `harness_resources`。
  - 文件级 `harness_candidates`。
  - `候选资源，未验证` 或等价边界说明。
  - `missing_manifest`、`missing_readme`、`missing_version`、`missing_entrypoints` 等 warning。
- 验证项目详情页显示：
  - 文件夹级 harness resource 数量。
  - resource warning。
- 验证页面没有新增“运行 harness”按钮。
- 验证页面没有把资源显示为“可用”“已验证”“已支持”。
- 验证后清理 Tauri / Vite / cargo-tauri 相关进程，并复核 5173 无监听残留。

## 允许读取

- `product-line/STAGE_PLAN.md`
- `product-line/README.md`
- `product-line/DEV_LINES.md`
- `product-line/tasks/README.md`
- `product-line/tasks/2026-05-28-desktop-shell-harness-resources.md`
- `product-line/handoffs/2026-05-28-desktop-shell-harness-resources-review.md`
- `product-line/evidence/2026-05-28-desktop-shell-harness-resources.md`
- `product-line/handoffs/2026-05-28-desktop-shell-harness-resources-result.md`
- `product-line/prototypes/index-kernel/codex-index.json`
- `product-line/prototypes/productized-desktop-shell/`

## 允许写入

- `product-line/evidence/`
- `product-line/handoffs/`

## 禁止事项

- 不写 `/Users/yoyi/.codex`。
- 不改真实 Codex 状态库。
- 不读取或展示 `auth.json`、`.env`、密钥、令牌、授权文件内容。
- 不读取或展示 Codex 会话正文、工具输出、命令输出、输入历史或记忆正文。
- 不自动运行 harness。
- 不点击或执行任何 harness 入口。
- 不新增或修改前端、Rust、索引代码，除非发现验证阻塞且先回报总指导。
- 不读取剪贴板内容。
- 不执行 Finder 打开、定位 rollout 或复制路径动作。
- 不接入非 Codex agent。
- 不做知识库、向量搜索、模型调度。
- 不做 release 打包。
- 不拉取外网依赖。

## 验收标准

- 有 evidence 和 handoff。
- `npm run typecheck` 通过。
- `npm run test:offline-interaction` 通过。
- `npm run build` 通过。
- `cargo test --offline` 通过。
- 真实 Tauri 窗口 smoke 通过，能读取索引。
- Tauri 窗口中能看到文件夹级 resources 与文件级 candidates 的区别。
- Tauri 窗口中能看到 resource warning。
- Tauri 窗口中看不到 harness 运行按钮。
- Tauri 窗口中不把候选资源标成可用、已验证或已支持。
- 未展示敏感内容。
- 未写 Codex 状态库。
- 未运行 harness。
- 验证后 5173 无监听残留。

## 必须回传

1. 做了什么
2. 读了哪些文件或目录
3. 新增了哪些 evidence / handoff
4. 验证命令和结果
5. 真实 Tauri 窗口看到的关键文本
6. 是否看到了 resources / candidates 区分
7. warning 是否可见
8. 是否发现运行按钮或误标可用状态
9. 是否触碰禁止事项
10. 清理了哪些进程，5173 是否无监听
11. 风险和下一步建议

## 总指导回收重点

回收时必须判断：

- 是否是真实 Tauri 窗口验证，不是普通浏览器验证。
- 是否看到 `harness_resources` 与 `harness_candidates` 的区别。
- 是否看到 warning。
- 是否没有运行 harness。
- 是否没有把候选资源显示成可用事实。
- 是否没有展示正文或敏感内容。
- 是否清理进程。
