# 阶段4 M0 产品与文档正本干净基线收口

总计划：product-line 唯一基线与 Harness Lite 切换
目标：保全共享主线既有文档整理，把已经确认方向的 M0 产品正本、权威分层和历史降级材料收成可复现的本地干净基线，再把开发入口交给 M3。

干完的标准：

- 接管时冻结的 66 个 tracked Markdown 修改与 11 个 untracked Markdown 新增经过权威链、相对链接、格式和 Git 路径核验。
- 本轮只读核验给 `docs/harness/usage/.turn` 追加的四行已精确恢复，不覆盖该文件在接管前已有内容。
- 77 个 M0 文档与本阶段必要 Harness 控制记录形成边界清楚、可审查的本地提交集；没有产品源码、M3 实现、merge 或 push。
- 阶段收口提交后共享主线 staged、unstaged、untracked 均为空，HEAD 与实际提交路径可复核。

允许动：

- DEV_LINES.md
- PROTOTYPE_WORK_LINES.md
- README.md
- RESULT_REVIEW.md
- archive/
- backlog.md
- codex-multi-agent-safe-collaboration.md
- decisions/
- docs/
- evidence/
- handoffs/
- principles.md
- tasks/
- docs/harness/
- refs/heads/main

只读：

- /Users/yoyi/workspace/product-line-syn-fnd-002
- /Users/yoyi/workspace/product-line-syn-m2-closeout
- 当前 Git 历史、M1 冻结合同、M2 收口证据与产品源码

不许动：

- 产品源码、测试、配置、数据库、迁移和 M3 新文件
- 两个只读保全工作树的 index、tracked/untracked 内容与分支头
- reset、clean、stash、rebase、merge、push、远端、部署和发布
- 真实 provider、真实账号、真实消息、凭据、在线工作台和产品桌面应用

## 叶子

- [x] M0C01 文档正本权威链与干净本地基线
