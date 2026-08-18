# OSS-01 阶段收口后的一次公开 push 与 Codex for OSS 申请

状态：`UNFINISHED / NOT CURRENT / WAITING_FOR_STAGE_14`

本文件不是 current leaf。唯一 current leaf 仍是 `M5R09-m1-enrollment-and-pre-closeout-hardening`。`authorization.json` 保持精确 closed 两字段。本文件不授权现在 push、不授权现在提交 OpenAI 表单、不关闭 stage-14、不宣布 M5 完成。

来源：用户 2026-08-18 明确「先补门面，然后等这个阶段做完了做一次 push，然后再做正式申请」。

## 已在本轮落地的门面（本地 working copy，未 push）

- 根目录 `LICENSE`：Apache-2.0，版权署名「呆头鹅」
- 根目录 `README.md`：对外英文预览 + 既有中文产品/开发入口；写明 pre-1.0、无安装包、不把 fixture 当发布
- `CONTRIBUTING.md`、`SECURITY.md`：短对外说明
- 桌面壳 `Cargo.toml` 填写 `license` / `repository`；npm 根包保持 `private: true`，只补 `license`

## 触发条件

同时满足后才进入下一步，缺一不可：

1. 当前 stage-14 / M5 按既有独立验收与 closeout 真正做完，或用户另用自然语言说这个阶段已经做完；
2. 用户看到并确认精确 remote、HEAD、refspec 和待推送提交范围后，明确要求这一次 push。

未满足时只保持本 unfinished，不 push，不申请。

## 阶段做完后按这个顺序做

1. 只读核对公开仓 `https://github.com/Djh0311/syn-aios` 仍为 Public，profile `Djh0311` 仍为 public。
2. 展示待 push 范围：remote `origin`、当前 HEAD、相对 `origin/main` 的 ahead/behind、门面文件是否在范围内。不 `git add -A`，不混入无关 dirty / 未归属 WIP。
3. 用户确认后只做一次精确 push。失败后停止并报告，不自动重试扩大范围。
4. push 成功后设置 GitHub About（仍须用户确认或当场授权 `gh`）：

```text
Description:
Local-first AI workbench for durable, auditable, and recoverable long-running agent workflows.

Topics:
local-first
ai-agents
agent-workflows
desktop-app
tauri
rust
typescript
react
sqlite
codex
```

5. 再用公开仓事实刷新申请三段英文，确认都不超过 500 字符，且不把未落地的 Harness 附录、Release、stars 或生产连接写成已发生。
6. 用户本人填写 First name、Last name、ChatGPT 邮箱、OpenAI Organization ID 后提交 https://openai.com/form/codex-for-oss/ 。

## 申请预填（push 当天按公开仓再核）

- GitHub username：`Djh0311`
- Repository URL：`https://github.com/Djh0311/syn-aios`
- Role：Primary maintainer
- Interest：Codex Security + API credits for my project

Why does this repository qualify?

```text
Syn is an actively maintained, local-first AI workbench (Tauri, Rust, React, TypeScript, SQLite) with 600+ public commits. It targets a real OSS gap: durable, auditable, recoverable long-running agent work, with implemented foundations for server-owned roles, scoped permissions, persistent sessions, handoffs, and source-backed daily coordination rather than stateless chat. The project is still pre-1.0 and in active development; we apply for ecosystem importance and maintenance depth, not stars or downloads.
```

How will you use API credits for your project?

```text
API credits will support Syn's open-source maintenance only: Codex-assisted issue triage, pull-request review, regression-test generation, release-readiness checks, documentation-drift detection, and security review. We will use them to produce reproducible evidence, shorten review cycles, and help contributors understand and safely change a large Rust/TypeScript desktop codebase.
```

Anything else we should know?

```text
Syn is still in active development and early in public adoption. This application is based on ecosystem importance and maintenance depth, not popularity. Roles, sessions, handoffs, and source-backed daily coordination have implemented foundations with local evidence; project-supervisor work is in progress and not a public 1.0 release. No production connectors or packaged release are claimed.
```

push 当天若 M5 已 closeout，只把「project-supervisor work is in progress」改成当时公开仓能核对的一句，不扩大其他声称。

## 不许动

- 现在 push / merge / rebase / release / tag
- 现在提交 OpenAI 表单
- 把本文件提升为 current leaf，或并进 M5R09 完成标准
- 购买或制造 stars
- 把 Harness Lite 写成已公开发布的独立组件
- 把 isolated fixture、本机 working copy 或未 push 提交冒充公开仓事实
