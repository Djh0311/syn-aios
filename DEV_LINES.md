# Development Lines

## 当前开发线

当前核心开发线是：

- 总指导线
- 索引内核线
- Codex 会话线
- 桌面应用线
- 信息架构线
- 验证线

当前主线是 Codex 会话管理和 Codex 工作流编排。当前阶段状态以 `CURRENT.md` 和 `tasks/README.md` 为准。任务包能力保留为内部协议、审计、导出和交接物，不作为主界面中心。

依据：`decisions/2026-05-29-codex-session-workflow-route-correction.md`。

## 总指导线

职责：

- 维护 `CURRENT.md`、`README.md`、`STAGE_PLAN.md`、`tasks/README.md`。
- 派发任务包。
- 回收任务结果。
- 判断任务是否接受、暂停、归档或废弃。
- 维护当前权威和归档索引。

禁止：

- 不把历史流水账继续堆进当前入口。
- 不把 dry-run 写成真实自动编排完成。

## 索引内核线

职责：

- 只读扫描本机 Codex 元数据。
- 生成 `prototypes/index-kernel/codex-index.json`。
- 维护 transcript reader。
- 做 schema 降级、坏数据、缺文件、越界和 warning 测试。

当前状态：

- 只读索引内核已完成原型和多轮 hardening。
- 单会话 transcript reader 已完成。
- 默认索引不包含会话全文正文。

禁止：

- 不写 `/Users/yoyi/.codex`。
- 不读取授权、密钥、`.env`。
- 不把完整 transcript 写入默认索引。

## Codex 会话线

职责：

- 读取用户选择的 Codex 会话完整正文、工具调用、工具结果和时间线。
- 探针验证 Codex CLI 的新建、resume、发送、等待、读回能力。
- 维护会话能力矩阵。
- 为工作流节点派发提供受控能力依据。

当前状态：

- 单会话 transcript 读取 v1 已完成。
- `codex exec` 新建无业务测试会话并读回已通过。
- `codex exec resume` 绑定无业务测试会话派发并读回已通过。
- 用户审核业务派发极小真实写入闭环已跑通一次。
- 四角色工作流机器已完成 stub 闭环和测试项目真实闭环。
- 工作流能力仍不等于复杂业务自动编排完成。

禁止：

- 不向真实业务会话发送测试消息。
- 不运行 `codex fork`。
- 不删除、移动、归档 Codex 会话。
- 不在未获明确批准时写 `/Users/yoyi/.codex`。

## 桌面应用线

职责：

- 维护 Tauri + Rust + React + TypeScript + Vite 桌面壳。
- 展示 Agent、项目、Skill 管理、Harness 管理。
- 提供 Agent 会话中心、项目内 Agent 会话入口、项目工作流视图。
- 实现工作流状态流转、节点绑定和后续受控派发。

当前状态：

- 产品化桌面壳可继续扩展，但不是完整发布版。
- Agent 会话中心只读 UI 已完成。
- 项目内 Agent 会话入口已完成。
- 工作流最小状态流转已完成。
- 工作流节点绑定已有 Codex 会话已完成。
- 工作流节点派发 Codex 指令 v1 代码路径已完成。
- 真实工作流节点 safe probe 派发闭环一次已完成。
- 派发结果 UI 读回与总指导 review 记录入口已完成。
- 总指导 `accepted` review 已写入真实 workflow state。
- Codex 角色编排离线入口、离线状态账本和复核修复已完成。
- 四角色工作流机器 v1、mario demo 真实闭环、真实总指导自然闭环已完成。
- uiwork 水墨工作台界面替换 v1 已接入真实工作台。
- workflow task package design v1 Task 0-12 已完成保守闭环读模型和只读 UI 草案。

下一步：

- 阶段性总结当前工作流闭环能力。
- 之后由总指导线决定转向工作台秘书型 AI 产品设计，还是继续加固失败重试、权限队列、长任务稳定性和真实 Tauri 窗口验收。

禁止：

- 不伪装成已完成真实业务自动执行。
- 不在未获明确批准时执行会写 `/Users/yoyi/.codex` 的真实派发。
- 不保存完整 transcript 到工作台状态。
- 不直接读取密钥、授权文件或 `.env`。

## 信息架构线

职责：

- 维护首页四入口。
- 维护项目级可视化工作流的信息结构。
- 维护 Agent 会话中心、项目内会话入口、Skill 管理、Harness 管理的边界。
- 明确字段来源和敏感信息展示规则。

当前状态：

- UI 与信息架构方向已定：首页四入口，项目打开后进入项目级工作流。
- 会话中心与项目内 Agent 会话架构已定。
- 水墨工作台第一版已接入，但不接受为截图级原模原样已证明。
- 当前 UI 仍在继续治理，真实 Tauri 窗口完整验收按任务单独安排。

禁止：

- 不把未接入 agent 写成可用能力。
- 不把参考源功能复刻写进当前阶段承诺。

## 验证线

职责：

- 单元测试。
- 离线前端交互测试。
- 构建检查。
- 真实 Tauri 窗口 smoke。
- 坏数据、缺文件、权限失败验证。

当前状态：

- 验证线按需另派。
- 真实窗口验证不再作为每个实现包的共同执行线。

禁止：

- 不绕过权限限制。
- 不用真实密钥或真实敏感文件做测试夹具。

## 已封存或后置的线

Codex 数据盘点线：

- 已完成并封存为上游证据线。
- 后续补盘点归索引内核线，不复用为常设线。

Skills 与 Harness 盘点线：

- 当前并入索引内核线和信息架构线。
- Skill 自动安装、仓库化、Harness 自动运行后置。

多 agent 接入线：

- 当前不开放。
- OpenClaw / OpenCode / Claude Code / VS Code 真接入后置。
