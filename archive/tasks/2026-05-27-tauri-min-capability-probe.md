# 任务包：Tauri 最小能力验证

## 所属开发线

桌面应用线。

这是阶段 2 桌面应用线的能力验证任务，不新增常设开发线。

## 背景

用户已确认接受“Tauri 最小能力验证”路线。

依据：

- `product-line/decisions/2026-05-27-desktop-container-route.md` 推荐先做 Tauri 最小能力验证。
- `product-line/README.md` 当前决策是保留静态网页壳，真桌面能力优先推荐 Tauri 最小能力验证。
- `product-line/STAGE_PLAN.md` 阶段 2 目标包括打开文件夹、复制路径、定位日志等低风险本机动作。
- `product-line/handoffs/2026-05-27-desktop-app-static-shell-validation-review.md` 已接受静态网页壳验证结果，但说明它不是真桌面应用。

## 目标

- 只做最小能力验证，不做完整产品化桌面应用。
- 检查本机 Tauri / Rust / Node 工具链是否可用。
- 如果工具链已可用，在 `product-line/prototypes/tauri-capability-probe/` 做最小 Tauri 原型。
- 原型应尽量复用或加载现有静态 UI，读取同一个 `codex-index.json`。
- 验证显式打开项目文件夹、复制路径、定位 rollout 日志所在文件这类低风险动作是否可实现。
- 输出 evidence 和 handoff。

## 允许读取

- `product-line/decisions/2026-05-27-desktop-container-route.md`
- `product-line/prototypes/desktop-app/`
- `product-line/prototypes/index-kernel/codex-index.json`
- `product-line/handoffs/2026-05-27-desktop-app-static-shell-validation-review.md`
- `product-line/STAGE_PLAN.md`
- `product-line/README.md`

## 允许写入

- `product-line/prototypes/tauri-capability-probe/`
- `product-line/evidence/`
- `product-line/handoffs/`

## 禁止事项

- 不写 `/Users/yoyi/.codex`。
- 不改真实 Codex 状态库。
- 不读取或展示 `auth.json`、`.env`、密钥、令牌、授权文件。
- 不展示 Codex 会话正文、工具输出、命令输出、输入历史或记忆正文。
- 不自动运行 harness。
- 不做完整产品化打包。
- 不做自动更新、系统托盘、通知、登录项、多 agent 接入。
- 不把能力验证写成“最终桌面应用已完成”。
- 不安装网络依赖；如果缺 Tauri / Rust / Node 依赖，只记录缺口和建议，不绕过限制。

## 建议实现边界

若工具链可用，最小原型只需要验证：

- 窗口能加载本地 UI。
- 能读取 `product-line/prototypes/index-kernel/codex-index.json`。
- 能对索引内已有项目路径执行“打开文件夹”。
- 能对索引内已有 rollout 路径执行“定位日志文件所在位置”。
- 能复制路径到剪贴板。
- 所有动作必须由用户点击触发。
- 所有动作只允许索引内已有路径，不接受任意用户输入路径。

如果工具链不可用：

- 不要安装。
- 输出环境检查结果。
- 说明缺什么依赖、建议如何安装、安装后第一条验证命令是什么。

## 验收标准

- 有工具链检查结果。
- 若工具链可用，有可运行最小原型和运行说明。
- 若工具链不可用，有清晰缺口说明，不伪造完成。
- 有安全边界说明：允许动作、禁止动作、路径限制。
- 不写 `.codex`，不展示密钥，不展示正文类内容。
- 输出 evidence 和 handoff。

## 必须回传

1. 做了什么
2. 工具链是否可用
3. 改了哪些文件
4. 验证了哪些桌面能力
5. 哪些能力未验证，原因是什么
6. 安全边界是否符合任务包
7. 风险和下一步建议
