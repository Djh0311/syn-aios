# 任务包：桌面应用静态壳验证

## 所属开发线

验证线。

这是对桌面应用线静态索引壳的验证任务，不新增常设开发线。

## 背景

桌面应用线已交付静态网页壳，但回收线没有完成 Playwright 浏览器复核。

依据：

- `product-line/handoffs/2026-05-27-desktop-app-static-index-shell-review.md` 接受静态壳原型，同时记录 Playwright 因网络受限未完成。
- `product-line/handoffs/2026-05-27-desktop-app-static-index-shell-result.md` 声称浏览器 smoke 通过，需要验证线复核或补充证据。
- `product-line/STAGE_PLAN.md` 阶段 2 要求桌面应用能运行、不依赖网络、不展示密钥、不写 Codex 状态库。

## 目标

- 复核静态网页壳能在本地打开并读取静态索引。
- 复核 6 个页面可切换。
- 复核核心数据数量和 warning 展示。
- 复核页面没有展示正文类内容和敏感文件内容。
- 输出 evidence 和 handoff。

## 允许读取

- `product-line/prototypes/desktop-app/`
- `product-line/prototypes/index-kernel/codex-index.json`
- `product-line/tasks/README.md`
- `product-line/handoffs/2026-05-27-desktop-app-static-index-shell-review.md`
- `product-line/handoffs/2026-05-27-desktop-app-static-index-shell-result.md`

## 允许写入

- `product-line/evidence/`
- `product-line/handoffs/`

## 禁止事项

- 不写 `/Users/yoyi/.codex`。
- 不读或展示 `auth.json`、`.env`、密钥、令牌、授权文件。
- 不读取或展示 Codex 会话正文、工具输出、命令输出、输入历史或记忆正文。
- 不自动运行 harness。
- 不安装网络依赖。
- 不改桌面应用源码；如果发现问题，只在 handoff 里说明并退回。

## 验收标准

- `node --check product-line/prototypes/desktop-app/app.js` 通过。
- 本地 server 能稳定返回 `index.html`、`app.js`、`codex-index.json`、`tasks/README.md`。
- 如果浏览器自动化可用，完成页面 smoke：6 个导航页可切换，控制台错误为 0。
- 如果浏览器自动化不可用，必须说明原因，并用 HTTP 响应、DOM 静态检查和索引数据解析给出替代证据。
- 核心数量和当前索引一致：项目、会话、skills、plugins、harness 候选。
- 明确是否存在正文类内容或敏感内容展示风险。

## 必须回传

1. 做了什么
2. 验证了哪些文件和页面
3. 使用了哪些命令或浏览器工具
4. 哪些验证通过
5. 哪些验证无法完成，原因是什么
6. 是否建议退回桌面应用线修改
7. 风险和下一步建议
