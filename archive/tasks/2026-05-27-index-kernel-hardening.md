# 任务包：索引内核路径注入与语义校验修复

## 所属开发线

索引内核线。

## 背景

验证线已经证明索引内核在 9 个异常夹具下能降级，但也暴露两个问题：测试只能覆盖全局路径常量，`--check` 只能做结构校验。要进入桌面应用线前，内核需要正式的数据源注入入口和更清楚的校验模式。

依据：

- `product-line/handoffs/2026-05-27-index-kernel-validation-review.md`
- `product-line/evidence/2026-05-27-index-kernel-validation.md`
- `product-line/handoffs/2026-05-27-index-kernel-validation-result.md`

## 目标

- 给 `build_index.py` 增加正式数据源注入入口。
- 支持 CLI 指定 Codex home，例如 `--codex-home <path>`。
- 避免测试通过覆盖全局常量来注入路径。
- 将 `sqlite_thread_count_differs_from_inventory` 改成真实环境回归检查，避免夹具模式固定出现无意义 warning。
- 扩展 `--check`，支持可选 warning 语义校验。
- 更新现有异常夹具测试，让测试走正式入口。

## 允许读取

- `/Users/yoyi/workspace/product-line/prototypes/index-kernel/build_index.py`
- `/Users/yoyi/workspace/product-line/prototypes/index-kernel/codex-index.json`
- `/Users/yoyi/workspace/product-line/prototypes/index-kernel/tests/test_build_index_failures.py`
- `/Users/yoyi/workspace/product-line/evidence/2026-05-27-index-kernel-validation.md`
- `/Users/yoyi/workspace/product-line/handoffs/2026-05-27-index-kernel-validation-result.md`
- `/Users/yoyi/workspace/product-line/handoffs/2026-05-27-index-kernel-validation-review.md`

## 允许写入

- `/Users/yoyi/workspace/product-line/prototypes/index-kernel/build_index.py`
- `/Users/yoyi/workspace/product-line/prototypes/index-kernel/tests/`
- `/Users/yoyi/workspace/product-line/prototypes/index-kernel/codex-index.json`
- `/Users/yoyi/workspace/product-line/evidence/`
- `/Users/yoyi/workspace/product-line/handoffs/`

## 禁止事项

- 不写 `/Users/yoyi/.codex`。
- 不改真实 `state_5.sqlite`。
- 不读取或打印 `auth.json`、`.env`、密钥、令牌。
- 不把测试夹具放进 `/Users/yoyi/.codex`。
- 不把会话正文、命令输出、工具输出、输入历史或记忆正文加入索引。
- 不把 `.codex-global-state.json.thread-workspace-root-hints` 覆盖到 `threads.cwd`。

## 建议实现

- 增加 `IndexSources` 或同等配置对象，集中保存 SQLite、session_index、global_state、sessions、archived_sessions、skills、plugins、memories 路径。
- `build_index()` 接收配置对象；CLI 默认使用 `/Users/yoyi/.codex`，测试传入临时目录。
- `--check` 增加可选参数，例如检查必须包含某些 warning，或输出 warning 统计。
- 将真实环境线程数回归检查做成可配置项，不在夹具模式默认启用。

## 验收标准

- 真实环境生成索引仍通过。
- 现有 9 个异常夹具测试仍通过。
- 测试不再通过覆盖模块全局路径常量来注入假 Codex home。
- `--codex-home` 或等效入口可用。
- `--check` 能检查结构，并能选择性验证 warning 语义。
- 不读取或打印授权文件。
- 不写真实 `/Users/yoyi/.codex`。

## 必须回传

1. 做了什么
2. 改了哪些文件
3. 新增或修改了哪些测试
4. 新入口怎么使用
5. 哪些验证通过
6. 是否仍有未修问题
7. 风险和下一步建议
