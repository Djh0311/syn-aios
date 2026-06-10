# 桌面壳 harness resources 真实窗口验证 evidence

任务包：`product-line/tasks/2026-05-28-desktop-shell-harness-resources-validation.md`

验证时间：2026-05-28

## 结论

真实 Tauri 窗口验证通过：产品化桌面壳能读取当前索引，并在 Harness 管理页展示文件夹级 `harness_resources` 与文件级 `harness_candidates` 的区别，也能展示 resource warning。

边界先说清楚：

- 这不是 harness 可运行、已验证或已支持的证明。
- 本轮没有运行 harness，没有点击或执行任何 harness 入口。
- 本轮没有新增或修改前端、Rust、索引代码。
- 这不是普通浏览器验证，是 Tauri dev 窗口验证。

## 读取范围

本轮读取：

- `product-line/tasks/2026-05-28-desktop-shell-harness-resources-validation.md`
- `product-line/tasks/README.md`
- `product-line/evidence/2026-05-28-desktop-shell-harness-resources.md`
- `product-line/handoffs/2026-05-28-desktop-shell-harness-resources-result.md`
- `product-line/prototypes/productized-desktop-shell/`
- `product-line/prototypes/index-kernel/codex-index.json`

没有读取或展示：

- `auth.json`
- `.env`
- 密钥、令牌、授权文件内容
- Codex 会话正文、工具输出、命令输出、输入历史、记忆正文

## 验证命令

在 `product-line/prototypes/productized-desktop-shell/` 执行：

```bash
npm run typecheck
npm run test:offline-interaction
npm run build
```

结果：

- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：通过，输出 `offline interaction tests passed: 3`。
- `npm run build`：通过。

在 `product-line/prototypes/productized-desktop-shell/src-tauri/` 执行：

```bash
CARGO_HOME=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-home CARGO_TARGET_DIR=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-target cargo test --offline
```

结果：

- `path_whitelist_accepts_only_index_projects_and_rollouts`：通过。
- `snapshot_keeps_metadata_without_session_body`：通过。
- `reads_real_static_index_summary`：通过。

启动 Tauri dev：

```bash
CARGO_HOME=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-home CARGO_TARGET_DIR=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-target npm run tauri:dev
```

结果：

- Vite ready：`http://127.0.0.1:5173/`
- Tauri 进程：`codex-governance-workbench`
- 窗口标题：`Codex 治理工作台`
- 窗口尺寸：`1280, 820`

## Tauri 窗口读取索引

真实 Tauri 窗口中读取到：

- `Codex 工作台`
- `已读取索引。所有本机动作仍需用户点击并确认。`
- 主导航包含 `Harness 管理`

未停留在普通浏览器保护失败状态：

- 未出现 `当前页面不在 Tauri 窗口中运行`

结论：真实 Tauri 窗口成功读取当前静态索引。

## Harness 管理页验证

切换到 `Harness 管理` 后，真实窗口读取到：

- `资源和候选看板`
- `文件夹级 resource 是候选资源；文件级 candidate 是候选入口。两者都不代表可运行或已验证。`
- `文件夹 resources`
- `来自 projects[].harness_resources[]`
- `文件 candidates`
- `来自 projects[].harness_candidates[]`
- `Resource warning`
- `缺 manifest / README / version / entrypoints 必须显示`
- `文件夹级 harness resources`
- `候选资源，未验证`
- `文件级 harness candidates`
- `文件候选`
- `这仍只是候选，不代表已验证。`

字段可见：

- `manifest_path`
- `readme_path`
- `version`
- `entrypoints`
- `permission_level`

warning 可见：

- `missing_manifest`
- `missing_readme`
- `missing_entrypoints`
- `missing_version`
- `weak_harness_signal`
- `entrypoints_truncated`

边界说明可见：

- `harness_resources 是文件夹级候选资源；harness_candidates 是文件级候选入口。`
- `missing_manifest、missing_readme、missing_version、missing_entrypoints 等 warning 直接展示，不自动降噪。`
- `不新增运行按钮，不自动运行 harness，不把资源显示为可用或已验证。`

结论：

- resources / candidates 区分可见。
- warning 可见。
- 页面没有把资源标成可用、已验证或已支持。
- 页面没有新增运行 harness 按钮。

## 项目详情页验证

项目页默认项目没有 resource warning，因此先根据索引查找有 warning 的项目。

索引中 `workspace` 项目有：

- `resourceCount: 1`
- `missing_manifest`
- `missing_readme`
- `missing_entrypoints`
- `missing_version`

在真实 Tauri 窗口中切换到 `workspace` 项目后，项目详情页读取到：

- `项目详情`
- `workspace`
- `Harness 资源`
- `文件夹级 resource，候选未验证`
- `详情面板`
- `有 warning`
- `文件夹 harness resources`
- `文件 harness candidates`
- `Resource warning`
- `missing_manifest`
- `missing_readme`
- `missing_entrypoints`
- `missing_version`

结论：

- 项目详情页显示文件夹级 harness resource 数量入口。
- 项目详情页显示文件级 harness candidate 数量入口。
- 项目详情页能显示 resource warning。

## 负向检查

源码扫描：

```bash
rg -n '运行 harness|运行|可用|已验证|已支持|auth\.json|\.env|secret|token|authorization|first_user_message|payload\.content|stdout|stderr|raw_memories|MEMORY\.md|writeFile|child_process|exec\(|spawn\(' product-line/prototypes/productized-desktop-shell/src product-line/prototypes/productized-desktop-shell/src-tauri/src product-line/prototypes/productized-desktop-shell/src-tauri/tauri.conf.json
```

命中说明：

- `当前页面不在 Tauri 窗口中运行`：保护性错误文案。
- `运行边界`：旧诊断视图文案。
- `可用`：Agent / Skill 语境，不是 harness resource 状态。
- `已验证`：Harness 页边界文案里用于否定，例如“不代表可运行或已验证”。
- `没有运行能力`、`不新增运行按钮，不自动运行 harness`：Harness 页边界文案。
- `spawn()`：后端 `pbcopy` 实现存在，但本轮未触发复制。
- `auth.json`：Rust 单测里作为非白名单拒绝样本字符串。

未发现：

- harness 运行按钮。
- 把 harness resource 标为 `可用`、`已验证`、`已支持` 的正向能力文案。
- 读取或展示敏感内容的代码路径。

## 禁止事项核对

本轮未做：

- 未写 `/Users/yoyi/.codex`。
- 未改真实 Codex 状态库。
- 未读取或展示 `auth.json`、`.env`、密钥、令牌、授权文件内容。
- 未读取或展示 Codex 会话正文、工具输出、命令输出、输入历史或记忆正文。
- 未自动运行 harness。
- 未点击或执行任何 harness 入口。
- 未新增或修改前端、Rust、索引代码。
- 未读取剪贴板内容。
- 未执行 Finder 打开、定位 rollout 或复制路径动作。
- 未接入非 Codex agent。
- 未做知识库、向量搜索、模型调度。
- 未做 release 打包。
- 未拉取外网依赖。

## 进程和端口清理

清理前本轮相关进程：

- `cargo-tauri dev`：PID 30688
- `vite --host 127.0.0.1`：PID 30864
- `codex-governance-workbench`：PID 30903

已定向清理上述 PID。

清理后复核：

- `lsof -nP -iTCP:5173 -sTCP:LISTEN` 无输出。
- `ps` 只剩查询命令和 `rg` 自身，没有 `codex-governance-workbench`、`vite --host 127.0.0.1`、`cargo-tauri dev` 残留。

未留下临时验证文件。

## 风险

- 本轮是窗口文本级 smoke，不是完整 UI 自动化。
- 页面显示索引内路径和候选元数据；按任务允许验证展示，但不等于敏感内容红队测试。
- `harness_resources` 仍有大量 warning，不能按数量推断可用能力。
- 下一步若要进入“可运行”，需要先定义 manifest 规范、权限确认、执行审计和验证结果存储。
