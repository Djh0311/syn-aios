# OSS-01 阶段收口后的一次公开 push 与 Codex for OSS 申请

状态：`UNFINISHED / NOT CURRENT / WAITING_FOR_STAGE_14`

本文件不是 current leaf。唯一 current leaf 仍是 `M5R09-m1-enrollment-and-pre-closeout-hardening`。`authorization.json` 保持精确 closed 两字段。本文件不授权现在 push、不授权现在提交 OpenAI 表单、不关闭 stage-14、不宣布 M5 完成。

来源：用户 2026-08-18 明确「先补门面，然后等这个阶段做完了做一次 push，然后再做正式申请」。

## 已在本轮落地的门面（本地 working copy，未 push）

- 根目录 `LICENSE`：Apache-2.0，版权署名「呆头鹅」
- 根目录 `README.md`：对外英文预览 + 既有中文产品/开发入口；写明 pre-1.0、无安装包、不把 fixture 当发布
- `CONTRIBUTING.md`、`SECURITY.md`：短对外说明
- 桌面壳 `Cargo.toml` 填写 `license` / `repository`；npm 根包保持 `private: true`，只补 `license`

## 已发生：一次公开 push（2026-08-18 20:34，总指导执行）

- 触发：stage-14 / M5 已 closeout 并获独立验收（`M5C01-20260818-1939.verdict.md`，`M5 SCOPED PRODUCT-CHAIN PASS / STAGE-14 CLOSED / NOT RELEASED`）；用户本人 2026-08-18 20:03 明确「push」。
- 载体：`git push origin main`，`01d107b..b900bb6`，fast-forward，未 force、未 merge、未 rebase、未打 tag、未 release。
- 事前只读核对：远端 main 真实为 `01d107b`（本地 `origin/main` ref 已过期，实际差 228 个提交而非 111）；`FETCH_HEAD..HEAD` 远端独有 0 条，确认纯 fast-forward；228 提交范围内凭据样式命中 9 处，全部为本仓安全扫描测试的占位假值且位于删除行；敏感文件名命中全部为 `secretary` 误配。
- 事后核对：远端 `main` = 本地 `HEAD` = `b900bb66e4f3034c4fc237b8d0829541d691785a`；公开历史 895 个提交。
- 认证载体：本机原先无任何 GitHub 凭据。先试 fine-grained token（HTTPS）被 403 拒（身份为 `Djh0311`，缺 Contents 写权限），改为新建本机 ed25519 密钥 `~/.ssh/id_ed25519`（`SHA256:41eq84EaFCCOU5mNFwwMeS/tQpnwAKgdX69MdvUjSd0`），由用户加进 GitHub Authentication Keys。`github.com` 三个主机公钥取自 `https://api.github.com/meta`（TLS 通道），指纹与官方公布值逐一相符后写入 `known_hosts`；未使用 `StrictHostKeyChecking=no`。临时 token 文件已 `shred -u` 删除，**仍需用户本人在 GitHub 撤销该 token**。
- GitHub About / Topics 已设置（2026-08-18 20:40）：`gh` 2.97.0 官方二进制装在 `~/.local/bin/gh`（sha256 与官方 `checksums.txt` 相符，未用 sudo、未加系统源），由用户以设备码流程授权为 `Djh0311`（scopes `gist, read:org, repo`）。`gh repo edit` 写入描述与 10 个 topics，回读确认：可见性 `PUBLIC`、默认分支 `main`、描述与 topics 与本文件文案一致。
- 未做：Codex for OSS 申请表提交（须用户本人填写姓名、ChatGPT 邮箱、OpenAI Organization ID）。申请文案里「600+ public commits」按当前公开事实为 895，可照填或改准，不得往上夸大。

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

## 2026-08-18 21:10 定稿的四段申请文案（每段 ≤500 字符，字符数经实测）

表单实际有四问，比本文件早前预设的三段多一问「为何需要 Codex Security」。下列文案基于当日实测事实：公开仓 898 个提交、首个提交 2026-06-11、53 个活跃开发日、219 个 Rust 文件、129 个 TS/TSX 文件、2114 个 Rust `#[test]`、50 份合同、0 star、0 下载、无安装包、**无 CI（`.github/workflows` 不存在）**。旧版「600+ public commits」作废。

1. Why qualify（483 字符）：见下方 `applications/` 引用或直接使用本节文案；如实以生态位与维护深度立论，先自陈无 star 无下载。
2. Why Codex Security（499 字符）：如实列举提示注入、凭据处理、路径逃逸、校验-读取 TOCTOU（一处已修、一处记为欠账）、Tauri IPC 命令面、SQLite 完整性、Rust/npm 供应链，并主动承认无 CI、无自动化安全审查。
3. API credits（485 字符）：只用于本仓维护——issue 分诊、PR 审查、回归测试生成、发布就绪检查、合同漂移检测、安全审查；具体到自动化现由人工执行的独立验收、检测 50 份合同与实现的漂移、补跨模块攻击链用例。明确不用于终端用户功能或制造指标。
4. Anything else（500 字符）：自陈 pre-1.0、0 star、无发布物、无外部连接器、未接真实 provider/账号/凭据、无生产运行；申请依据是生态位与维护深度；并如实披露本仓由多 agent 在一套治理规则下开发（Codex 主管、Grok 写产品码、另起无上下文继承的 agent 做独立验收）。

为压到 500 字符被砍掉、可在字数允许时补回的内容：失败重试建立新 lineage 而旧链不可改写；每个里程碑经独立验收并绑定确切 SHA/tree；希望把 fail-closed 约定变成机械化检查并在 1.0 之前补上。

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
