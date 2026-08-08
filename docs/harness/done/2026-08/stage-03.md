# 阶段3 M2 主线收口与交接

总计划：product-line 唯一基线与 Harness Lite 切换
目标：在保全全部既有 WIP 的前提下，把已通过 PASS_BOUNDED 验收的 M2 reference slice 精确提取、提交并集成到 main，用当前 Harness Lite 完成项目级收口，并把下一入口交给新的指导对话。

干完的标准：

- M2 产品实现、必要 Code Map 与价值无关证据从混合工作树中得到可审查的独立提交；13 项战略 WIP 和其他相邻候选不被吞入。
- M2 提交集成到 main，直接相关聚焦测试、完整 Rust 库测、R4 隔离 App、Harness task/quick 和 Git 检查通过。
- M2 在项目与 Harness Lite 层正式记为完成；M3 保持未激活，live Workbench、provider 和迁移边界不被扩张。
- main 和收口分支的 OID、tree、验证材料、遗留边界与下一入口可复核；不 push。

允许动：

- /Users/yoyi/workspace/product-line-syn-integration-main
- /Users/yoyi/workspace/product-line-syn-m2-closeout/
- refs/heads/codex/syn-m2-closeout
- refs/heads/main
- /private/tmp/product-line-syn-m2-closeout-
- docs/harness/

只读：

- /Users/yoyi/workspace/product-line-syn-fnd-002 的 64 tracked + 14 untracked 混合工作树及其 Git 历史
- /Users/yoyi/workspace/product-line 与其他既有 worktree
- M2 的隔离 R4 receipt、历史任务包、合同与验收证据

不许动：

- /Users/yoyi/workspace/product-line-syn-fnd-002 的 index、tracked/untracked 内容和分支头
- 13 项既有战略 WIP、M3/M5 相邻候选、旧 Adaptive Harness runtime/authority
- live Workbench、~/.codex、provider、真实账号和真实消息
- reset、clean、stash、rebase、远端、push、部署、发布

## 叶子

- [x] M2C01 M2 边界冻结与干净候选提交
- [x] M2C02 main 集成与完整验收
- [x] M2C03 Lite 收口、知识清理与新对话交接
