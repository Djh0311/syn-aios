# 任务包：桌面应用静态索引壳

## 所属开发线

桌面应用线。

这是原型阶段固定常设线里的桌面应用线，不新增开发线。

## 背景

索引内核线已经提供可供界面读取的静态索引样例。

依据：

- `product-line/handoffs/2026-05-27-index-kernel-project-context-review.md` 接受项目上下文补齐结果。
- `product-line/prototypes/index-kernel/codex-index.json` 当前包含项目、会话、skills、plugins、memories、authority / handoff / evidence / harness 候选和 warning。
- `product-line/handoffs/2026-05-27-v1-information-architecture-review.md` 已接受第一版信息架构。
- `product-line/STAGE_PLAN.md` 阶段 2 目标是桌面应用壳，展示项目列表、会话列表、skills 列表、任务线状态。

## 目标

- 做一个零依赖或低依赖的本地只读应用壳原型。
- 读取 `product-line/prototypes/index-kernel/codex-index.json` 作为静态输入。
- 实现第一版页面骨架：首页、项目页、会话页、skills 页、harness 页、任务线页。
- 展示 warning 和“不确定候选”状态，不把候选写成事实。
- 输出 evidence 和 handoff。

## 范围建议

优先做静态本地网页壳，路径建议：

- `product-line/prototypes/desktop-app/`

建议文件：

- `index.html`
- `styles.css`
- `app.js`
- `README.md`

如果实现线认为需要 Vite、Electron、Tauri 或其他框架，必须先在 handoff 里说明理由、成本和新增依赖，不要直接安装网络依赖。

## 允许读取

- `product-line/prototypes/index-kernel/codex-index.json`
- `product-line/handoffs/2026-05-27-v1-information-architecture-result.md`
- `product-line/handoffs/2026-05-27-v1-information-architecture-review.md`
- `product-line/handoffs/2026-05-27-index-kernel-project-context-review.md`
- `product-line/STAGE_PLAN.md`
- `product-line/README.md`
- `product-line/tasks/README.md`
- `product-line/tasks/*.md`

## 允许写入

- `product-line/prototypes/desktop-app/`
- `product-line/evidence/`
- `product-line/handoffs/`

## 禁止事项

- 不写 `/Users/yoyi/.codex`。
- 不读或展示 `auth.json`、`.env`、密钥、令牌、授权文件。
- 不展示 Codex 会话正文、工具输出、命令输出、输入历史或记忆正文。
- 不展示 README / AGENTS / handoff / evidence 正文，只展示候选路径和元数据。
- 不自动运行 harness。
- 不自动判定项目类型。
- 不自动判定 authority 文件就是当前权威。
- 不自动安装依赖。
- 不嵌入 Codex 聊天窗口。
- 不做移动端。

## 页面验收标准

- 首页能显示项目数、线程数、skills 数、plugins 数、warning 摘要、数据生成时间。
- 项目页能按项目展示线程数、候选 authority / handoff / evidence / harness、`context_warnings`。
- 会话页能展示会话标题、编号、项目路径、更新时间、rollout 是否存在、会话 warning；标题过长要保持可读。
- skills 页能区分本地 skill 和插件 skill，展示路径、插件名、warning。
- harness 页只展示候选入口，不展示命令正文，不提供运行按钮。
- 任务线页能从 `tasks/README.md` 或任务包元数据展示待派发、进行中、已回收、暂停状态；如果只做静态解析困难，可以先展示任务队列文档入口和当前待派发任务。
- 页面必须清楚显示“候选”“未知”“warning”，不能空白吞掉。

## 技术验收标准

- 不依赖网络。
- 有本地运行方式，例如 `python3 -m http.server` 或打开静态 HTML 的说明。
- 有基础 smoke 验证：至少截图或命令证明页面可打开、核心数据可见。
- 不新增大型框架依赖，除非 handoff 说明并获得后续确认。
- 输出 evidence 和 handoff。

## 必须回传

1. 做了什么
2. 改了哪些文件
3. 页面现在能展示哪些数据
4. 哪些交互只是占位或不能在静态壳里实现
5. 是否读取或展示了任何正文类内容
6. 验证方式和结果
7. 风险和下一步建议
