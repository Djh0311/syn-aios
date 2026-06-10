# 任务包：桌面容器路线决策

## 所属开发线

总指导线。

这是阶段 2 静态壳验收后的路线决策任务，不新增常设开发线。

## 背景

静态网页壳已经通过验证，但它不具备真正桌面能力。

依据：

- `product-line/handoffs/2026-05-27-desktop-app-static-index-shell-review.md` 接受静态网页壳原型，但不接受为最终桌面应用。
- `product-line/handoffs/2026-05-27-desktop-app-static-shell-validation-review.md` 接受静态壳验证结果，并指出真桌面能力必须先做路线决策。
- `product-line/STAGE_PLAN.md` 阶段 2 的目标包括打开文件夹、复制路径、定位日志等低风险操作；静态网页壳不能可靠提供这些能力。

## 目标

- 比较继续纯网页、Electron、Tauri 三条路线。
- 明确第一版是否需要真正桌面容器。
- 明确如果进入桌面容器，允许的能力和禁止的能力。
- 输出路线决策文档。

## 允许读取

- `product-line/README.md`
- `product-line/STAGE_PLAN.md`
- `product-line/PROTOTYPE_WORK_LINES.md`
- `product-line/DEV_LINES.md`
- `product-line/handoffs/2026-05-27-desktop-app-static-index-shell-review.md`
- `product-line/handoffs/2026-05-27-desktop-app-static-shell-validation-review.md`
- `product-line/prototypes/desktop-app/README.md`

## 允许写入

- `product-line/decisions/`
- `product-line/evidence/`
- `product-line/handoffs/`
- `product-line/tasks/README.md`
- `product-line/README.md`

## 禁止事项

- 不安装 Electron / Tauri 或任何网络依赖。
- 不改桌面应用源码。
- 不写 `/Users/yoyi/.codex`。
- 不读或展示密钥、`.env`、授权文件。
- 不把“想要桌面能力”直接写成“已经实现桌面能力”。

## 验收标准

- 有一份路线决策文档。
- 决策必须先列薄弱点和风险。
- 决策必须说明继续纯网页、Electron、Tauri 的取舍。
- 决策必须明确推荐路线和不推荐路线。
- 决策必须列出下一步任务包建议，但不直接新增实现任务，除非用户确认。

## 必须回传

1. 做了什么
2. 读取了哪些依据
3. 推荐路线是什么
4. 不推荐路线是什么
5. 仍然未知的问题
6. 下一步建议
