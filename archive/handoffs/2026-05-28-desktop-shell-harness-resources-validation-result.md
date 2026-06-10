# 桌面壳 harness resources 真实窗口验证 handoff

任务包：`product-line/tasks/2026-05-28-desktop-shell-harness-resources-validation.md`

回收时间：2026-05-28

## 回收结论

验证线任务已完成。可以接受为“真实 Tauri 窗口中已验证 harness_resources 只读展示”。

边界：

- 不接受为 harness 可运行。
- 不接受为 harness 已验证、已支持或管理完成。
- 不接受为完整 UI 自动化。

## 做了什么

1. 复跑基础验证：
   - `npm run typecheck`
   - `npm run test:offline-interaction`
   - `npm run build`
   - `cargo test --offline`
2. 启动真实 Tauri dev 窗口：
   - 窗口标题 `Codex 治理工作台`
   - 窗口尺寸 `1280, 820`
   - 页面显示 `已读取索引。所有本机动作仍需用户点击并确认。`
3. 验证 Harness 管理页：
   - 能看到文件夹级 `harness_resources`。
   - 能看到文件级 `harness_candidates`。
   - 能看到 `候选资源，未验证`。
   - 能看到 warning。
   - 能看到不运行、不代表可运行或已验证的边界说明。
4. 验证项目详情页：
   - 切到有 resource warning 的 `workspace` 项目。
   - 能看到文件夹 harness resources。
   - 能看到文件 harness candidates。
   - 能看到 Resource warning。
5. 做负向检查：
   - 没发现 harness 运行按钮。
   - 没发现把 resource 标为可用、已验证、已支持的正向能力文案。
6. 清理进程和端口。

## 改了哪些文件

没有修改产品源码、Rust 或索引代码。

新增：

- `product-line/evidence/2026-05-28-desktop-shell-harness-resources-validation.md`
- `product-line/handoffs/2026-05-28-desktop-shell-harness-resources-validation-result.md`

## 验证结果

通过：

- `npm run typecheck`
- `npm run test:offline-interaction`，输出 `offline interaction tests passed: 3`
- `npm run build`
- `cargo test --offline`，3 个 Rust 单测通过

真实 Tauri 窗口：

- 已启动。
- 已读取索引。
- 未出现普通浏览器保护失败文案。

## 关键窗口文本

Harness 管理页：

- `资源和候选看板`
- `文件夹级 resource 是候选资源；文件级 candidate 是候选入口。两者都不代表可运行或已验证。`
- `文件夹 resources`
- `来自 projects[].harness_resources[]`
- `文件 candidates`
- `来自 projects[].harness_candidates[]`
- `文件夹级 harness resources`
- `候选资源，未验证`
- `文件级 harness candidates`
- `文件候选`
- `missing_manifest`
- `missing_readme`
- `missing_entrypoints`
- `missing_version`
- `weak_harness_signal`
- `entrypoints_truncated`
- `不新增运行按钮，不自动运行 harness，不把资源显示为可用或已验证。`

项目详情页：

- `Harness 资源`
- `文件夹级 resource，候选未验证`
- `文件夹 harness resources`
- `文件 harness candidates`
- `Resource warning`
- `missing_manifest`
- `missing_readme`
- `missing_entrypoints`
- `missing_version`

## 禁止事项状态

未触碰：

- 没写 `/Users/yoyi/.codex`。
- 没改真实 Codex 状态库。
- 没读取或展示授权文件内容、密钥、令牌、`.env` 内容。
- 没读取或展示会话正文、工具输出、命令输出、输入历史或记忆正文。
- 没自动运行 harness。
- 没点击或执行任何 harness 入口。
- 没新增或修改前端、Rust、索引代码。
- 没读取剪贴板。
- 没执行 Finder 打开、定位 rollout 或复制路径。
- 没接入非 Codex agent。
- 没做知识库、向量搜索、模型调度或 release 打包。
- 没拉取外网依赖。

## 清理状态

已清理本轮启动的：

- `cargo-tauri dev`
- `vite --host 127.0.0.1`
- `codex-governance-workbench`

最终复核：

- `5173` 无监听残留。
- 无同名 Tauri / Vite / cargo-tauri 进程残留。
- 未留下临时验证文件。

## 风险和下一步

风险：

- 这只是窗口文本级 smoke。
- `harness_resources` 仍是候选资源，且 warning 很多。
- 页面展示路径和候选元数据，不展示正文，但也不是敏感内容红队测试。

下一步建议：

- 不要把 resource 数量当作可用能力。
- 若要进入可运行阶段，先定义 manifest 规范、权限确认、执行审计和验证结果存储。
