# Syn Harness Lite official-04→0.8 迁移收口

日期：2026-08-16
范围：closeout-only。此记录不改 runtime、Hook、authorization、产品代码，不执行 Syn App、产品构建或产品全测。

## Attempt6 SOURCE_BYTES_MATCH 绑定

- 收据绝对路径：`/home/synadmin/workspace/.syn-source-migration-attempt6-20260816T023204/remote-0003/SOURCE_BYTES_MATCH.txt`
- 收据 SHA256：`afea7630244341b662ae26802ccf6b7637f8dd27e06ead2afabd0c75a071d562`
- HEAD：`9103c3b26b060e854be119a8cedaa856a2a900ce`
- TREE：`4080362684d53e6ccc845de87827ec722490efae`
- main 可达提交：`784`
- WIP：tracked `5`，untracked `15`
- 总文件数：`3596`；worktree manifest SHA256：`c6b084f48834d4347aa1b312a7c870e79c0a047f20f3d19bd290f542850280af`；receipt status SHA256：`d33e548fce74e6714892faaf97290a086b052d17ec8508dcd969ef8916023fa9`
- 收据结论：`INDEX_CLEAN=yes`、`IGNORED_FILES=0`、`SYMLINKS_OUTSIDE_GIT=0`、`GIT_FSCK=PASS`。

## D0D01 lifecycle

- 原 D0D01 leaf 已由 pre-D0D 路径按原字节移至 `docs/harness/done/2026-08/D0D01-syn-full-source-migration-to-5600x-wsl.md`；本次 closeout 前的 tar 比对为 `byte_identity=PASS`，原件未改写。
- 该结算不关闭 stage-12，不创建 current leaf，不移动或提升 D0C04/D0C05；两项仍在 `docs/harness/unfinished/`。
- 本报告、stage-12 checkbox 与计划优先级只让 Attempt6 收据、D0D 结算及当前状态可追溯；没有追加 synthetic Hook 或用户事件 audit。

## 0.8 后态

- runtime：`runtimeVersion=0.8.0`，`profile=project`，`packageDigest=sha256:7c1727e4fbf91a12b020cbea121abd788dad213ab84ad6aac94635615aee7b83`。
- Codex：`PreToolUse, SessionStart, Stop, UserPromptSubmit` 四事件均由单 dispatcher 配置；Claude 旧 Mac Harness Hook 为零；两份 live Hook 配置的旧 Mac 路径计数为 `0`。
- authorization：精确 closed 两字段（`schemaVersion=1`，`authorized=false`）。
- rollback：项目内受控 carrier 为 `/home/synadmin/workspace/syn/.claude/.harness-lite.rollback-Go9ehz`；迁移外部 pre-D0D 与 R1 tar 仍可读，SHA256 分别为 `sha256:2c7b2b1cb8c99033d67f491ca6a110af07723ad3f652d736b73f40a07bf2fb17` 和 `sha256:b1274e077985527acd61dda98835d20d64d17801eb5ab8c76d132d4748ff5b24`。
- 宿主边界：`NOT_TRUSTED`，`NOT_OBSERVED`（四事件均为 false）。离线/直接 dispatcher 不能改写该结论。

## 产品边界

迁移写入的后验路径比较无非 Harness 预期 delta。本 closeout 只新增本报告并修改 `docs/harness/plan.md`、`docs/harness/stages/stage-12.md`；不对产品行为作额外推断。
