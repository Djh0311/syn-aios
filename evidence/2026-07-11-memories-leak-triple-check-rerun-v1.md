# memories 渗出三查巡检复跑 · 2026-07-11(总指导)

口径 = C1 包 §2.2 + 七拍决策 :28(测试项目 / 工作台 store / 记忆池反向 + 池内工作台条目计数);基线 = 07-08/09 三面零渗出、池内 0 工作台条目。补欠账:C6、修包群、合一改造多个切片收口未巡。

## 结果

| 面 | 方法 | 结果 |
|---|---|---|
| ① 测试项目(`/Users/yoyi/codex-workflow-mario-test`) | 池内它项目特征词(kt_erp/美团/otome/sku_upc/molian/crazytown/活棋局/gameai)全目录反查 + 近 4 天 mtime | **净**。零命中;近 4 天仅 `index.html`(派发任务的正当工作产物) |
| ② 工作台 store(`~/Library/Application Support/CodexGovernanceWorkbench/workflow-state`) | 同词表反查全 store(口供/账本/授权/提案) | **净**。零命中 |
| ③ 记忆池反向(`~/.codex/memories/`) | 工作台词表(CodexGovernanceWorkbench/product-line/productized-desktop-shell/manual_relay/工作台/交办/jiaoban/mario/任务包/director_agent)计数 + 命中原文分类 | **判据触发**:池内现工作台相关条目(product-line raw=7·summary=6;director_agent raw=6;任务包 raw=3;jiaoban raw=2;rollout 摘要 18 份中 ≥4 份是 product-line 开发会话) |

## ③ 的定性(读原文分类,非猜)

- 命中条目 **全部来自执行线/研究线自己的开发会话**(`cwd: /Users/yoyi/workspace/product-line`,内容 = 任务包 v2 落地心得、代码地图、原型盘点)。
- **工作台派发的 worker 会话零渗出**:mario 全池 0 命中;rollout 摘要无一以派发任务命名。C1 原威胁模型(管发环泄进池)仍干净。
- 新风险面(与基线的实质差异):池内现在装着**工作台内部实现知识**(allowed_write 语义、director 内部结构等),而 memories 默认注入每个新会话(+约 3.3k tok/次)——**下一个派发的 worker 会话将带着这些开发记忆开工**,破 C1「worker 干净上下文只吃任务包」的本意;worker 口供若回声池内容,②面会开始变脏。

## 按规程上报

七拍 :28:「池内现工作台条目 → 回加旗/开关议题重拍」。判据字面成立 → 议题回用户桌面。选项:
a) 维持观察(实害仍零,下轮真派发后立即复巡②面);
b) 派发 worker argv 加 `--disable memories`(C1 已验证可关死[探测 019f424f];**动 runner=重档,要用户授权那一下**);
c) 执行线开发会话侧关 memories(不动 runner,代价=执行线丢跨会话记忆);
d) 写 `~/.codex/config.toml` 全局关(07-09 终拍明确不做,列出仅为完整)。

总指导建议:**b**,在下一个自然窗口做(非急修,当前无成品渗出)。

验证方式:全程只读 grep/ls/find,命令与计数如上;未写 `~/.codex` 任何文件。

## 重拍结果

用户 2026-07-11 拍 **a·维持观察**:下轮真派发后立即复巡②面;b(worker argv `--disable memories`)留作后手,②面一出现池内容回声即升级 b。
