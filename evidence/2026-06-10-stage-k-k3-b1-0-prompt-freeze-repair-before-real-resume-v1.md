# Stage K / K3-B1.0 Prompt Freeze Repair Before Real Resume Evidence v1

日期：2026-06-10

状态：已完成，结论为 `accepted`。

## 结论

K3-B1.0 接受为 K3-B1 真实 `resume` 执行前的 prompt freeze 修补完成。原 K3-B1 `prompt_hash` `e963aa00f7ba0cb94d973996794e98db9d019bf8d9e568c330eb272d7ddd9fbf` 没有可审计正文，已标记为 `superseded_prompt_body_unrecoverable`。新的 runtime-only prompt 已写入受控 `tmp` 路径，并同步到 K3-B1 执行任务包、K3-Level-B 字段冻结任务包和 Rust K3-B1 常量。

本轮没有执行真实 `codex exec` / `codex exec resume`，没有发送 prompt，没有读写 `/Users/yoyi/.codex`，没有读取 secret、token、`.env`、auth、keychain、OAuth、provider credential、full transcript 或 rollout。

过程说明：收尾扫描时有一条 `rg` 命令的搜索串误混入 Markdown 反引号，shell 尝试执行了一个中文片段并返回 `command not found`。该偏差没有触发 Codex CLI，没有发送 prompt，也没有读写 `/Users/yoyi/.codex`。

## 修改内容

- 新增任务包：`tasks/2026-06-10-stage-k-k3-b1-0-prompt-freeze-repair-before-real-resume-v1.md`
- 新增 runtime-only prompt：`tmp/k3-b-real-workflow-automation/prompts/k3-b1-mario-test-read-only.prompt.txt`
- 更新 K3-B1 执行任务包：`tasks/2026-06-10-stage-k-k3-b1-mario-test-workflow-read-only-real-resume-run-v1.md`
- 更新 K3-Level-B 字段冻结任务包：`tasks/2026-06-10-stage-k-k3-level-b-real-workflow-execution-field-freeze-v1.md`
- 更新 Rust K3-B1 常量：`prototypes/productized-desktop-shell/src-tauri/src/project_workflow_automation.rs`

## Prompt Freeze

当前 runtime prompt path：

```text
/Users/yoyi/workspace/product-line/tmp/k3-b-real-workflow-automation/prompts/k3-b1-mario-test-read-only.prompt.txt
```

当前 hash：

```text
ab0442e86e75900ab47b293328e4a2b46512ae68868799b94e8608ffedd57039
```

旧 hash：

```text
e963aa00f7ba0cb94d973996794e98db9d019bf8d9e568c330eb272d7ddd9fbf
```

旧 hash 不再作为 K3-B1 执行 gate。

## 验证记录

已通过：

- `shasum -a 256 tmp/k3-b-real-workflow-automation/prompts/k3-b1-mario-test-read-only.prompt.txt`：`ab0442e86e75900ab47b293328e4a2b46512ae68868799b94e8608ffedd57039`
- `cargo test --lib project_workflow_automation`：15 passed / 4 ignored；K3-B1 real entry 仍为 ignored，未执行真实 Codex。
- `cargo test --lib real_execution_command`：36 passed / 7 ignored。
- `cargo fmt -- --check`：通过。

扫描结果：

- 旧 hash `e963aa00f7ba0cb94d973996794e98db9d019bf8d9e568c330eb272d7ddd9fbf` 只出现在 superseded 说明和历史字段冻结 evidence / handoff 注释中；K3-B1 执行任务包和 Rust 常量已改为新 hash。
- 新 hash `ab0442e86e75900ab47b293328e4a2b46512ae68868799b94e8608ffedd57039` 已出现在 runtime prompt、K3-B1 执行任务包、K3-Level-B 字段冻结任务包、Rust 常量和当前入口文档中。
- `Command::new("codex")|mod codex_runner|spawn_director|spawn_subagent|RealCodexResumeRunner` 扫描无命中。
- 敏感词扫描命中均为既有 guard、类型、测试、viewer / readback 边界、UI 警示和 secret lint，不是 K3-B1.0 新增真实读取。
- readback / formal memory 边界扫描命中均为既有 unknown-result、candidate / observation 非正式记忆边界、正式采纳命令和测试，不是 K3-B1.0 新增误导口径。

## 边界

K3-B1.0 不接受为：

- K3-B1 已真实执行。
- K3-B2 已准备或已执行。
- K3-Level-B 已完成。
- K3 或 Stage K 已完成。
- 通用任意项目自由执行已完成。
- prompt body 可持久化到 sidecar / runtime log / audit / memory。

## 下一步

K3-B1 可以进入执行前 gate：

1. 重新跑 K3-B1 任务包要求的非真实前置测试。
2. 重新核对 `/Users/yoyi/Documents/mario test` 核心文件 hash。
3. 确认 runtime prompt 文件 hash 为 `ab0442e86e75900ab47b293328e4a2b46512ae68868799b94e8608ffedd57039`。
4. 只有在执行 gate 全部通过后，才运行 K3-B1 ignored env-gated real execution entry。
