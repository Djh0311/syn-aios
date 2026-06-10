# 任务包：Tauri CLI 后最小原型验证

## 所属开发线

桌面应用线。

这是阶段 2 桌面应用线的后续能力验证任务，不新增常设开发线。

## 背景

上一次 Tauri 最小能力验证只完成了工具链缺口确认，没有做出可运行 Tauri 原型。

依据：

- `product-line/handoffs/2026-05-27-tauri-min-capability-probe-review.md`：接受为 Tauri 前置工具链检查结果，不接受为桌面能力验证完成。
- `product-line/decisions/2026-05-27-desktop-container-route.md`：推荐 Tauri 作为真桌面能力的优先验证路线。
- `product-line/STAGE_PLAN.md`：阶段 2 目标包括打开文件夹、复制路径、定位日志。
- 用户已确认允许安装或提供 Tauri CLI。

## 目标

- 先获取或验证 Tauri CLI。
- 验证 `cargo tauri --version` 或等价 Tauri CLI 命令可用。
- 在 `product-line/prototypes/tauri-capability-probe/` 做最小可运行 Tauri 原型。
- 原型只加载或复用现有静态 UI。
- 原型读取同一个 `product-line/prototypes/index-kernel/codex-index.json`。
- 验证三个低风险本机动作：
  - 复制索引内路径。
  - 打开索引内项目文件夹。
  - 定位索引内 rollout 日志所在文件。
- 输出 evidence 和 handoff。

## 允许读取

- `product-line/decisions/2026-05-27-desktop-container-route.md`
- `product-line/prototypes/desktop-app/`
- `product-line/prototypes/index-kernel/codex-index.json`
- `product-line/prototypes/tauri-capability-probe/`
- `product-line/handoffs/2026-05-27-tauri-min-capability-probe-review.md`
- `product-line/STAGE_PLAN.md`
- `product-line/README.md`

## 允许写入

- `product-line/prototypes/tauri-capability-probe/`
- `product-line/evidence/`
- `product-line/handoffs/`

## 允许安装或获取

- 只允许为 Tauri 最小验证安装或获取缺失的 Tauri CLI / Tauri 项目依赖。
- 如果安装需要联网，必须记录实际命令、版本和失败原因。
- 不允许为了这个任务安装无关桌面框架或产品化打包依赖。

## 禁止事项

- 不写 `/Users/yoyi/.codex`。
- 不改真实 Codex 状态库。
- 不读取或展示 `auth.json`、`.env`、密钥、令牌、授权文件。
- 不展示 Codex 会话正文、工具输出、命令输出、输入历史或记忆正文。
- 不自动运行 harness。
- 不做完整产品化打包。
- 不做自动更新、系统托盘、通知、登录项、多 agent 接入。
- 不把能力验证写成“最终桌面应用已完成”。
- 不接受任意用户输入路径执行本机动作；动作路径必须来自索引内已有路径。

## 验收标准

- 有 Tauri CLI 可用性结果。
- 有最小原型文件或明确失败原因。
- 如果原型可运行，至少验证窗口加载和索引读取。
- 如果本机动作可验证，必须记录触发方式和结果。
- 如果本机动作不能验证，必须写明是工具链、权限、实现还是环境原因。
- 有安全边界说明：允许动作、禁止动作、路径限制。
- 不写 `.codex`，不展示密钥，不展示正文类内容。
- 输出 evidence 和 handoff。

## 必须回传

1. 做了什么
2. 工具链是否可用
3. 改了哪些文件
4. 安装或获取了什么依赖
5. 验证了哪些桌面能力
6. 哪些能力未验证，原因是什么
7. 安全边界是否符合任务包
8. 风险和下一步建议
