# 桌面壳接入 harness_resources 总指导回收意见

## 回收对象

- 任务包：`product-line/tasks/2026-05-28-desktop-shell-harness-resources.md`
- 开发线：桌面应用线
- Evidence：`product-line/evidence/2026-05-28-desktop-shell-harness-resources.md`
- Handoff：`product-line/handoffs/2026-05-28-desktop-shell-harness-resources-result.md`
- 被回收产物：`product-line/prototypes/productized-desktop-shell/`

## 结论

接受为“桌面壳已只读展示文件夹式 harness resources”。

不接受为“harness 可运行、已验证、已支持或管理完成”。依据：本轮没有运行 harness，没有新增运行按钮，没有写入验证状态，也没有定义 harness manifest 规范。

## 先说薄弱点

- 这仍是展示层接入，不是能力验证。依据：任务包禁止自动运行 harness，交接也明确未新增任何运行能力。
- 真实索引里的 `harness_resources` warning 很多。依据：上游索引内核回收意见记录 `missing_manifest=12`、`missing_readme=12`、`missing_entrypoints=6`、`missing_version=14`。
- 这次总指导复核没有启动真实 Tauri 窗口检查页面文本。依据：本次复核只重跑了 typecheck、离线交互测试、构建、Rust 单测和端口检查；真实窗口 smoke 应交给下一条验证线。
- 源码里仍有之前已有的 `pbcopy` 和 `open` 本机动作。依据：`src-tauri/src/lib.rs` 中存在复制路径和打开路径实现；这不是本轮新增 harness 运行能力，但后续验证仍要确认不会从 harness 页面绕过确认弹层。

## 接受内容

接受以下实现结果：

- 前端类型新增 `HarnessResource` 和 `HarnessEntrypoint`。
- Rust 数据映射新增 `HarnessResource`、`HarnessEntrypoint`、`parse_harness_resources`、`parse_harness_entrypoints`。
- 桌面壳读取 `projects[].harness_resources[]`。
- Harness 管理页区分“文件夹级 harness resources”和“文件级 harness candidates”。
- Harness 管理页把 resources 标为候选资源、未验证。
- 项目详情显示文件夹级 harness resource 数量和 warning 数量。
- 缺 manifest、README、version、entrypoints 的字段和 warning 会显示。
- 原有 `harness_candidates` 文件级候选展示保留。

接受展示字段：

- `display_name`
- `root_path`
- `harness_kind`
- `agent_type`
- `adapter_id`
- `source_kind`
- `capabilities`
- `manifest_path`
- `readme_path`
- `version`
- `entrypoints`
- `permission_level`
- `warnings`

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

源码抽查：

- `HarnessBoardView.tsx` 显示“文件夹级 harness resources”“文件级 harness candidates”“候选资源，未验证”。
- `ProjectsView.tsx` 显示“文件夹 harness resources”和“Resource warning”。
- `types.ts`、`lib.rs` 均包含 `harness_resources` 映射。
- 未发现新增 harness 运行按钮或 harness 执行入口。

## 安全和范围判断

接受当前安全边界。

依据：

- 未写 `/Users/yoyi/.codex`。
- 未改真实 Codex 状态库。
- 未读取或展示 auth、env、密钥、令牌、授权文件内容。
- 未读取或展示 Codex 会话正文、工具输出、命令输出、输入历史或记忆正文。
- 未自动运行 harness。
- 未新增“运行 harness”按钮。
- 未把 `harness_resources` 标成已可用、已验证或已支持。
- 未接入非 Codex agent。
- 未做知识库、向量搜索、模型调度。
- 未做 release 打包。

## 当前状态

这条任务从“待派发”改为“已回收”。

当前可以说：

- 桌面壳已能读取并展示 `harness_resources`。
- Harness 管理页已能区分文件夹级 resource 和文件级 candidate。
- 项目详情已能显示 resource 数量和 warning。

仍不能说：

- harness 已可运行。
- harness 已验证。
- harness manifest 规范已定。
- harness 管理完成。
- 真实 Tauri 窗口 harness resources UI smoke 已完成。

## 下一步

下一步派给验证线：做真实 Tauri 窗口 smoke 验证，重点检查：

- Harness 管理页是否在 Tauri 窗口里显示文件夹级 resources 和文件级 candidates 的区别。
- warning 是否可见。
- 页面是否没有运行按钮。
- 页面是否没有把候选资源标成可用、已验证或已支持。
- 验证后是否清理 Tauri / Vite 进程，并复核 5173 无监听。
