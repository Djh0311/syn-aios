# 桌面壳 harness resources 真实窗口验证总指导回收意见

## 回收对象

- 任务包：`product-line/tasks/2026-05-28-desktop-shell-harness-resources-validation.md`
- 开发线：验证线
- Evidence：`product-line/evidence/2026-05-28-desktop-shell-harness-resources-validation.md`
- Handoff：`product-line/handoffs/2026-05-28-desktop-shell-harness-resources-validation-result.md`
- 被验证产物：`product-line/prototypes/productized-desktop-shell/`

## 结论

接受为“真实 Tauri 窗口中已验证 harness_resources 只读展示”。

不接受为“harness 可运行”，不接受为“harness 已验证、已支持或管理完成”，也不接受为“完整 UI 自动化完成”。

依据：

- 验证线 evidence 记录真实 Tauri dev 窗口已启动，窗口标题为 `Codex 治理工作台`，尺寸为 `1280, 820`。
- 真实窗口显示 `已读取索引。所有本机动作仍需用户点击并确认。`。
- 真实窗口未出现 `当前页面不在 Tauri 窗口中运行`。
- Harness 管理页窗口文本包含 `文件夹级 harness resources`、`文件级 harness candidates`、`候选资源，未验证`。
- Harness 管理页窗口文本包含 `missing_manifest`、`missing_readme`、`missing_entrypoints`、`missing_version`、`weak_harness_signal`、`entrypoints_truncated`。
- 项目详情页切到 `workspace` 项目后，窗口文本包含 `文件夹 harness resources`、`文件 harness candidates`、`Resource warning`。
- 验证线记录未运行 harness，未点击或执行任何 harness 入口，未改前端、Rust 或索引代码。
- 验证线记录清理了 `cargo-tauri dev`、Vite、Tauri 进程，并复核 `5173` 无监听残留。
- 总指导线复跑 `npm run typecheck`、`npm run test:offline-interaction`、`npm run build`、`cargo test --offline` 均通过。
- 总指导线复核 `lsof -nP -iTCP:5173 -sTCP:LISTEN` 无监听输出。

## 先说薄弱点

- 这只是窗口文本级 smoke，不是完整 UI 自动化。依据：验证线通过窗口文本确认关键内容，没有做 DOM 级端到端点击、截图像素验收或跨项目全量巡检。
- 这不证明 harness 可用。依据：验证线没有运行 harness，没有点击或执行任何 harness 入口。
- `harness_resources` 仍是候选资源。依据：页面显示 `候选资源，未验证`，且 warning 很多。
- 这不是敏感内容红队测试。依据：验证范围是任务允许的索引元数据展示，不覆盖全面敏感内容扫描。
- 这没有验证 release。依据：任务禁止 release 打包，验证只覆盖 Tauri dev 窗口。

## 接受的验证结果

接受以下结论：

- 真实 Tauri 窗口能读取当前静态索引。
- 页面没有停留在普通浏览器保护性失败状态。
- Harness 管理页能区分文件夹级 `harness_resources` 和文件级 `harness_candidates`。
- Harness 管理页能展示 resource warning。
- 项目详情页能展示文件夹级 harness resource、文件级 harness candidate 和 resource warning。
- 页面没有新增 harness 运行按钮。
- 页面没有把 resource 标成可用、已验证或已支持。
- 验证后无 5173 监听残留。

## 总指导线复跑验证

在 `product-line/prototypes/productized-desktop-shell/` 复跑：

```bash
npm run typecheck
npm run test:offline-interaction
npm run build
```

结果：

- `npm run typecheck` 通过。
- `npm run test:offline-interaction` 通过，输出 `offline interaction tests passed: 3`。
- `npm run build` 通过。

在 `product-line/prototypes/productized-desktop-shell/src-tauri/` 复跑：

```bash
CARGO_HOME=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-home \
CARGO_TARGET_DIR=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-target \
cargo test --offline
```

结果：

- 3 个 Rust 单测通过。

端口复核：

```bash
lsof -nP -iTCP:5173 -sTCP:LISTEN
```

结果：

- 无监听输出。

说明：

- 总指导线没有重复启动 Tauri 窗口。真实窗口验证依据来自验证线 evidence 和 handoff；本轮只复核基础命令和清理状态，避免重复启动引入新的残留进程。

## 安全边界判断

接受当前安全边界。

依据：

- 未写 `/Users/yoyi/.codex`。
- 未改真实 Codex 状态库。
- 未读取或展示授权文件内容、密钥、令牌、`.env` 内容。
- 未读取或展示会话正文、工具输出、命令输出、输入历史或记忆正文。
- 未自动运行 harness。
- 未点击或执行任何 harness 入口。
- 未新增或修改前端、Rust、索引代码。
- 未读取剪贴板。
- 未执行 Finder 打开、定位 rollout 或复制路径。
- 未接入非 Codex agent。
- 未做知识库、向量搜索、模型调度或 release 打包。
- 未拉取外网依赖。

## 当前状态

这条验证任务从“待派发”改为“已回收”。

当前可以说：

- 桌面壳已只读展示 `harness_resources`。
- 真实 Tauri 窗口已验证 resources / candidates 区分和 warning 展示。

仍不能说：

- harness 已可运行。
- harness 已验证。
- harness 管理完成。
- 完整 UI 自动化完成。
- release 打包完成。

## 下一步

下一步进入阶段 3 工作流事实层 v0 的前置决策。

理由：

- UI 骨架和 harness resources 展示已经验证到真实窗口。
- 阶段 3 目标是项目级可视化编排，不是继续扩静态索引展示。
- 当前仍缺本地工作台事实层的存储位置、schema、迁移口径和写入边界。
- 在这些没有定之前，不应让桌面应用线直接写工作流状态。
