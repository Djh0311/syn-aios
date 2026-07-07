# Kickoff:product-line 源码异地备份(push 私有 GitHub)· 用户另一对话执行 v1

日期:2026-07-07 · 拍板:用户(**信任 GitHub·只 push 软件源码**——不含记忆库/工作台 store/~/.codex)· 出单:主导线。

## 0. 自包含背景(冷启即读)

- 仓 = `/Users/yoyi/workspace/product-line`,目前 **零 remote**(`git remote -v` 空),全部资产只在这台 Mac 上。本单 = 异地备份的**代码半边**。
- 上位依据:`backlog.md`「关键数据异地备份」(2026-06-13 用户确认,"git 私有 remote" 为其列明形态之一);判据原话:**没演练过恢复的备份等于没有备份**。

## 1. 安全线(必守)

- push = `AGENTS.md` 高危清单 #5,**用户在场明确那一下**才推;
- 目标仓必须 **private**——推之前核实可见性(`gh repo view <repo> --json visibility` 或让用户网页确认),公开仓一律停;
- **不 force push**;只动这一个仓;不碰 `~/.codex`;不装新工具(没有 gh 就走 https remote + 用户浏览器核实)。

## 2. 步骤

1. **预检**:`git status` 树应净(不净先问用户);`du -sh .git`(知道要推多大——工作树 36G 是构建产物,**不会**被推);`git branch -a` 列全分支(main + 若干历史分支 + `ui-wip-snapshot-2026-07-01` 备份枝);顺手扫一眼防意外秘密:`git grep -ilE "(api[_-]?key|token|secret|password)[\"']?\s*[:=]" -- '*.rs' '*.ts' '*.tsx' '*.json'`,命中的逐个人眼判(预期没有真秘密:auth 从未入库,workbench 只存路径不存凭据);
2. 用户在 GitHub 建**私有空仓**(不初始化 README/license,防首推冲突),把 URL 给你;
3. `git remote add origin <url>` → `git push --all origin` → `git push --tags origin`;
4. **核实**:`git ls-remote origin main` 的 hash == 本地 `git rev-parse main`;远端分支清单 == 本地;
5. **恢复演练(不做 = 没备份)**:`git clone <url> <临时目录>` → `git log --oneline -3` 与本地一致 → 打开 `CURRENT.md` 首屏核对是最新版 → 删临时目录;
6. 交待习惯:此后每次收口 commit 顺手 `git push`(远端不落后);要不要把这条写进 `AGENTS.md` 由用户拍;
7. 完成后**向用户报一句**(推了哪些分支/远端 hash/演练结论);**不要改 `CURRENT.md`/`backlog.md`**——单一写者纪律,回写由主导线对话做。

## 3. 明确不含(用户已拍,勿扩面)

3 个 Claude 记忆库(`~/.claude/projects/*/memory/`,合计 <200K)、workbench 线上 store(`~/Library/Application Support/CodexGovernanceWorkbench/workflow-state/`,146M,正式记忆/口供/审计)、`~/.codex` 会话史、codexbridge 仓(已有 origin)。这些**仍零副本**,由主导线挂账,用户知情。
