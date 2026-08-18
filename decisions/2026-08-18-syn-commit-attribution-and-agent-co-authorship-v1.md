# 提交归属与 agent 署名（2026-08-18）

## 事实

公开 push（`01d107b..b900bb6`）之后，公开仓贡献者面板只显示 claude 与 cursor，既没有维护者本人，也没有 Codex。原因不是贡献多少，而是邮箱能否被 GitHub 认领：

- 897 个既有提交的 author 与 committer 全为 `Codex <codex@local>`。该地址不可验证，GitHub 无法解析到任何账号，因此这些提交不计入贡献者面板。
- 其中 330 个提交带 `Co-Authored-By` trailer，指向 `noreply@anthropic.com`（162 + 110 + 49 + 3）与 `cursoragent@cursor.com`（6）。这两个地址能解析到官方账号，于是面板里只剩它们。
- GitHub 官方 Codex 账号确认存在：`login=codex`、`id=267193182`、`company=OpenAI`、bio 为 “OpenAI's coding agent”，可认领地址 `267193174+codex@users.noreply.github.com` 形式的 noreply。
- 维护者账号 `Djh0311` 的数字 id 为 `277674664`，可认领且不暴露真实邮箱的地址为 `277674664+Djh0311@users.noreply.github.com`。

## 决定

1. 仓库 local git 身份改为 `Djh0311 <277674664+Djh0311@users.noreply.github.com>`。往后所有提交的 author/committer 都是维护者本人，agent 的参与用 trailer 如实记录。
2. 提交信息按事实加 co-author trailer：Codex 参与的加 `Co-Authored-By: Codex <267193182+codex@users.noreply.github.com>`；产品内容由 Grok 写的再加 `Co-Authored-By: Grok (xAI grok-4.6) <noreply@x.ai>`。
3. **不改写历史。** 897 个既有提交保持原样。改写会改变全部 SHA，而五份独立验收结论、`docs/harness/plan.md` 的内容锚（如 `c91d8fc`）、各候选报告绑定的 SHA/tree 都以这些 SHA 为审计锚点；为了贡献者面板的显示而作废整套证据链不成立。已公开的历史也不做 force 覆盖。

## 明确不做

- 不使用 `grok`（`id=495761`，真人 Sterling Hamilton）或 `xai`（`id=1155551`，真人 Olaf Lessenich）的账号邮箱署名。xAI 官方只有组织 `xai-org`，组织不能作为提交 co-author，因此 Grok 无法出现在贡献者面板；它的 trailer 只作如实记录，不可认领。
- 不给未参与某提交的身份署名。
- 不声称 OpenAI 或 xAI 对本仓库有背书。co-author trailer 只是归功，GitHub 对 trailer 不做签名校验。

## 边界

Anthropic 公开说明了 `noreply@anthropic.com` 作为署名地址；OpenAI 是否公布同等口径未查到公开依据。使用 `codex` 账号的 noreply 地址属于对其真实工作的如实归功；若日后 OpenAI 另有明确口径，按其口径改。
