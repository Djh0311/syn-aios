# mario test 四角色工作流绑定 Evidence

## 薄弱点

- 本轮只准备了工作台真实 workflow state，不代表真实多 Codex 会话编排已经跑通。
- 本轮没有执行 `codex exec` / `codex exec resume`，没有向四个会话发送任何消息。
- `/Users/yoyi/Documents/mario test` 当前只读复核显示只有 `.git` 元数据，还没有马里奥 demo 文件。
- 首个 work item 只是进入 `ready_to_dispatch`，下一轮真实派发仍需要用户再次明确批准，因为会写 `/Users/yoyi/.codex`。

## 用户目标

用户指定继续用马里奥 demo 作为测试项目，并给出项目目录：

- `/Users/yoyi/Documents/mario test`

用户说明四个 Codex 会话名：

- 总指导
- 开发线
- 验证线
- 回收线

本轮目标是把这四个已有 Codex 会话登记到工作台真实 workflow state，形成可派发的四角色测试工作流。

## 写入结果

已写真实 workflow state：

- 状态文件：`/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json`
- 备份文件：`/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/backups/workflow-state.v0.1780156724812.json`

新增 project：

- `project:users-yoyi-documents-mario-test`
- root path：`/Users/yoyi/Documents/mario test`

新增 workflow：

- `workflow:users-yoyi-documents-mario-test:default`
- title：`mario test 四角色编排测试工作流`
- entry node：`workflow:users-yoyi-documents-mario-test:default:node:director`

新增 work item：

- `workflow:users-yoyi-documents-mario-test:default:create-mario-demo-v1`
- title：`创建马里奥 demo 的总指导计划`
- state：`ready_to_dispatch`
- current node：`workflow:users-yoyi-documents-mario-test:default:node:director`
- assigned role：`director`

新增节点数量：

- nodes：7
- edges：7

四个 active bindings：

| 角色 | node id | native thread id | session title | rollout |
| --- | --- | --- | --- | --- |
| 总指导 | `workflow:users-yoyi-documents-mario-test:default:node:director` | `019e798a-6ce5-76c3-b8ee-33bd0fda841f` | `总指导` | exists |
| 开发线 | `workflow:users-yoyi-documents-mario-test:default:node:codex-dev` | `019e798a-ac37-7771-b982-e38084fcd22e` | `开发线` | exists |
| 验证线 | `workflow:users-yoyi-documents-mario-test:default:node:validation` | `019e798a-f9d4-7922-8da8-1b6a8dbd6769` | `验证线` | exists |
| 回收线 | `workflow:users-yoyi-documents-mario-test:default:node:review` | `019e798b-2ee7-7f90-beb2-9031f6ad3e05` | `回收线` | exists |

新增审计事件：

- `audit:mario-four-role-project-registered:1780156724812`
- `audit:mario-four-role-workflow-created:1780156724812`
- `audit:mario-four-role-work-item-ready:1780156724812`
- `audit:mario-four-role-sessions-bound:1780156724812`

## 只读复核

复核结果：

- project count：1
- workflow count：1
- nodes：7
- edges：7
- work items：1
- active bindings：4
- 四个 binding 的 `rollout_exists` 均为 `true`
- 四个 binding 的 `warnings` 均为空数组
- `codex-index.json` 校验：`validation_ok`

项目目录只读复核：

- `/Users/yoyi/Documents/mario test/.git/HEAD`
- `/Users/yoyi/Documents/mario test/.git/config`
- `/Users/yoyi/Documents/mario test/.git/description`

## 边界

- 是否写真实 workflow state：是。
- 是否写 `/Users/yoyi/.codex`：否。
- 是否执行 `codex exec`：否。
- 是否执行 `codex exec resume`：否。
- 是否向四个会话发送消息：否。
- 是否修改 `/Users/yoyi/Documents/mario test`：否。
- 是否读取完整 transcript：否。
- 是否读取 `auth.json`、`.env`、密钥、token、授权文件：否。

## 下一步

下一步不是直接做 demo 文件，而是派发给“总指导”会话，让总指导制定创建马里奥 demo 的阶段计划，并输出可解析的派发块。

该下一步会执行真实 `codex exec resume`，会写 `/Users/yoyi/.codex`，所以需要用户再次明确批准。
