# 产品任务补充模板

新工作先从 `docs/harness/templates/leaf.md` 创建 Harness Lite leaf；本文件只补充 product-line 的产品分析字段，不激活任务、不授予权限。

## 任务与目标

- 任务名：
- 所属产品域：
- 背景与直接证据：
- 具体交付：
- 不做什么：

## 范围

- 允许读取：
- 允许写入：
- 禁止事项：
- 受保护 dirty / untracked：

## 变更辐射面

写清：改变了什么假设 → 哪些现有组件依赖它 → 每项如何验收。涉及容器、布局、时序、状态机、Tauri command、sidecar 或存储格式时必须逐项列出。

## 五态旅程（UI 任务）

说 → 批 → 干 → 交货 → 卡住。每一相说明本变更的界面和行为；不涉及的相位明确写“不涉及”。

## 验收

- 使用 `hl tests <本次改动路径...>` 选择最小相关检查。
- 使用 `hl check task <本次改动路径...>` 只执行已登记且命中的 task 检查。
- Rust production 路径同时跑 non-test build 与相关 tests。
- `git diff --check`。
- full、真实 App、provider、数据库、浏览器、部署和发布只在显式授权/调用时执行并单独结算。

## 回传

1. 做了什么。
2. 改了哪些文件。
3. 跑了什么验证，原始结果是什么。
4. 哪些结论已证实，哪些仍未知。
5. 风险、遗留和下一入口。
6. start/end commit；没有提交时如实说明。
7. 是否碰到 hard gate 或 catch；没有就写“无”。
