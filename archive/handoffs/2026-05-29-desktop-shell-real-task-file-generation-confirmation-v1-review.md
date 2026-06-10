# 真实任务包文件生成确认 v1 总指导回收意见

## 回收对象

- 任务包：`product-line/tasks/2026-05-29-desktop-shell-real-task-file-generation-confirmation-v1.md`
- 开发线：桌面应用线
- Evidence：`product-line/evidence/2026-05-29-desktop-shell-real-task-file-generation-confirmation-v1.md`
- Handoff：`product-line/handoffs/2026-05-29-desktop-shell-real-task-file-generation-confirmation-v1-result.md`
- 真实生成任务包：`product-line/tasks/2026-05-29-generated-task-draft-smoke.md`
- 被回收产物：`product-line/prototypes/productized-desktop-shell/`

## 结论

接受为“真实任务包文件生成确认 v1 已完成”。

不接受为“生成的任务包内容已适合派发 Codex 执行”，不接受为“真实 Codex 会话派发完成”，不接受为“自动编排执行完成”，不接受为“真实 Tauri 窗口生成链路已验证”。

依据：

- 真实文件 `/Users/yoyi/workspace/product-line/tasks/2026-05-29-generated-task-draft-smoke.md` 存在，大小为 1610 bytes。
- 真实 `product-line/tasks/` 下只出现一个 `2026-05-29-generated-*.md` 文件。
- 任务文件包含标准段落：任务名、目标、禁止事项、验收标准、必须回传。
- 字段级校验显示真实工作台状态里有 1 个 artifact path 指向该文件。
- 字段级校验显示真实工作台状态里有 1 条 `task_package_file_generated` audit event。
- 备份文件 `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/backups/workflow-state.v0.1780043100407.json` 存在。
- 后端代码已调整为先备份状态，再写任务文件。
- 总指导线复跑常规验证通过。

## 先说薄弱点

- 第一次真实确认曾出现半成品风险：任务文件已经落盘，但状态备份失败，导致 artifact path 一度没有回填。
- 代码已修为先备份状态再写任务文件，并支持同内容孤儿文件恢复；这个修复已从当前 `lib.rs` 代码顺序读到。
- 本轮真实确认通过 ignored Rust 测试调用同一后端逻辑完成，不是通过真实 Tauri 窗口点击完成。
- 生成出来的任务包来自已有真实草稿，标题和目标有输入法污染，且多处仍是“待补充”；它只能证明真实落盘闭环，不能直接作为高质量开发任务派发。
- 生成任务包里仍保留“不生成真实 `product-line/tasks/*.md` 任务包文件，除非后续任务明确要求并再次确认”这类历史草稿禁令，后续必须修正任务内容模板或草稿字段。

## 接受内容

接受真实落盘闭环：

- 在真实 `product-line/tasks/` 目录下生成了一个任务包 Markdown 文件。
- 没有覆盖已有 generated-prefix 任务包。
- 回填了真实工作台状态里的 `artifacts[].path`。
- 清空了该 artifact 的 warnings。
- 追加了 `task_package_file_generated` audit event。
- 生成前后有备份路径记录。

接受后端修复：

- `generate_task_package_file_at` 当前先完成 workflow state 备份，再写任务包文件。
- 已存在同内容目标文件时，可以走同内容恢复路径，不生成第二个重复文件。
- 常规 `cargo test --offline` 默认忽略真实写入确认测试，避免普通测试再次写真实任务文件和真实状态。

## 真实生成文件

路径：

```text
/Users/yoyi/workspace/product-line/tasks/2026-05-29-generated-task-draft-smoke.md
```

只读校验：

- 文件存在。
- 文件大小：1610 bytes。
- 文件在 `/Users/yoyi/workspace/product-line/tasks/` 下。
- `find product-line/tasks -maxdepth 1 -type f -name '2026-05-29-generated-*.md'` 只返回该文件。
- 标准段落存在：`# 任务包：`、`## 任务名`、`## 目标`、`## 禁止事项`、`## 验收标准`、`## 必须回传`。

## 状态字段级校验

没有打印完整真实 `workflow-state.v0.json` 正文，只做字段级读取。

字段级结果：

```json
{
  "projects": 1,
  "workflows": 1,
  "work_items": 1,
  "artifacts": 1,
  "audit_events": 4,
  "matching_artifact_paths": 1,
  "artifact_warnings": [[]],
  "generated_audits": 1
}
```

匹配的 audit event：

- `event_type = task_package_file_generated`
- `target_ref = work-item:workflow:users-yoyi-gameai-agent-world:default:1780032043420`
- `before_state = draft`
- `after_state = draft`
- `created_at = 1780043100407`

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

- 37 个 Rust 单测通过。
- 1 个真实确认测试保持 ignored。

## 安全和范围判断

接受当前安全边界。

依据：

- 没有派发真实 Codex 会话。
- 没有启动 Codex CLI。
- 没有运行 harness。
- 没有写 `/Users/yoyi/.codex`。
- 没有改 Codex 状态库。
- 没有写项目业务目录。
- 没有打印完整真实 `workflow-state.v0.json` 正文。
- 没有读取或展示 auth、env、密钥、令牌、授权文件内容。
- 没有读取或展示 Codex 会话正文、工具输出、命令输出、输入历史或记忆正文。
- 没有接入非 Codex agent。
- 没有知识库、向量搜索、模型调度、release 打包或网络依赖拉取。

## 当前状态

这条任务从“进行中”改为“已回收”。

当前可以说：

- 桌面壳已经完成一次真实任务包文件落盘确认。
- 真实工作台状态里的 artifact path 已回填。
- 真实工作台状态里已有生成任务包文件的 audit 记录。
- 后端已修正先写文件后备份失败造成半成品的顺序风险。

仍不能说：

- 生成的任务包内容已适合派发给 Codex。
- 真实 Tauri 窗口生成链路已验证。
- 真实 Codex 会话派发完成。
- 自动编排执行完成。

## 下一步建议

下一步建议做“任务包内容修正与派发准备 v1”。

理由：

- 真实落盘闭环已经打通。
- 但当前真实生成文件内容质量不足，不能作为正式派发入口。
- 进入真实 Codex 派发前，需要让工作台能发现“已生成但内容不合格”的任务包，并支持修正字段后重新生成或生成新版文件。

建议下一条任务：

- `任务包内容修正与派发准备 v1`
- 目标是修正当前任务草稿字段、避免历史禁令进入新任务包、标记任务包是否 ready to dispatch。
- 仍不派发 Codex，不运行 harness。
