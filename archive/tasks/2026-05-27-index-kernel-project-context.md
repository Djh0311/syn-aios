# 任务包：索引内核项目上下文补齐

## 所属开发线

索引内核线。

这是 `2026-05-27-codex-index-kernel.md` 同一条开发线的后续任务，不新增常设开发线。

## 背景

索引内核原型、hardening 和边界异常补测已经回收，但阶段 1 仍有缺口。

依据：

- `product-line/STAGE_PLAN.md` 阶段 1 通过标准要求把项目、会话、skills、harness 候选信息关联起来，并读取项目内候选 handoff / evidence / README / AGENTS.md。
- `product-line/prototypes/index-kernel/codex-index.json` 当前没有 `harness` 字段，也没有项目内 README / AGENTS / handoff / evidence 候选字段。
- `product-line/handoffs/2026-05-27-v1-information-architecture-review.md` 已接受第一版页面结构，项目页和 harness 页需要这些只读候选数据。
- `product-line/handoffs/2026-05-27-index-kernel-edge-validation-review.md` 已接受边界异常补测，并把这个缺口列为派生任务。

## 目标

- 在索引内核里增加项目上下文的只读元数据扫描。
- 把项目根路径、会话、权威入口候选、handoff / evidence 候选、harness 候选关联起来。
- 输出更新后的 `codex-index.json`。
- 增加离线夹具测试，不依赖真实 `/Users/yoyi/.codex`。
- 输出 evidence 和 handoff。

## 建议字段

字段名可以按实现调整，但必须能表达这些信息：

- 每个项目的 `authority_files` 候选：README、AGENTS、CLAUDE、阶段计划、任务队列等入口文件，只记录路径和类型，不读取全文。
- 每个项目的 `handoff_files` 候选：只记录路径、类型、更新时间、大小等元数据。
- 每个项目的 `evidence_files` 候选：只记录路径、类型、更新时间、大小等元数据。
- 每个项目的 `harness_candidates`：记录入口类型和来源，例如 package script 名、Makefile target 名、脚本路径、Godot / Vite / Python / Node 等配置入口；默认不记录脚本命令正文。
- 每个项目的 `context_warnings`：目录不存在、权限拒绝、候选过多被截断、符号链接越界、文件名异常等。
- 顶层或 `source_stats` 中增加项目上下文扫描统计。

## 允许读取

- `product-line/prototypes/index-kernel/build_index.py`
- `product-line/prototypes/index-kernel/tests/test_build_index_failures.py`
- `product-line/prototypes/index-kernel/tests/test_build_index_edge_cases.py`
- `product-line/prototypes/index-kernel/codex-index.json`
- `product-line/handoffs/2026-05-27-v1-information-architecture-result.md`
- `product-line/handoffs/2026-05-27-v1-information-architecture-review.md`
- `product-line/handoffs/2026-05-27-index-kernel-edge-validation-review.md`
- 临时测试目录内的夹具文件。

运行真实索引生成时，可以只读检查索引中项目根路径下的候选文件存在性和安全元数据；不要读取任意项目正文。

## 允许写入

- `product-line/prototypes/index-kernel/build_index.py`
- `product-line/prototypes/index-kernel/tests/`
- `product-line/prototypes/index-kernel/codex-index.json`
- `product-line/evidence/`
- `product-line/handoffs/`

## 禁止事项

- 不写 `/Users/yoyi/.codex`。
- 不改真实 `state_5.sqlite`。
- 不读取或打印 `auth.json`、`.env`、密钥、令牌。
- 不读取或输出 Codex 会话正文、工具输出、命令输出、输入历史或记忆正文。
- 不把 README、AGENTS、handoff、evidence 的正文加入索引；第一版只要候选路径和元数据。
- 不遍历 `node_modules`、`.git`、构建产物、大型缓存目录。
- 不自动运行 harness。
- 不自动判定 harness “有用”或“没用”。
- 不自动判定项目是 ERP 或游戏；没有明确文件依据时显示未知或候选。

## 验收标准

- 有可运行测试命令。
- 原 17 个索引内核测试仍通过。
- 新增项目上下文夹具测试通过。
- `codex-index.json` 里能看到项目上下文候选和 harness 候选统计。
- 候选扫描遇到缺目录、权限拒绝、符号链接越界、候选过多时能降级并给 warning。
- 不依赖网络。
- 不写真实 `/Users/yoyi/.codex`。
- 输出一份 evidence 和 handoff。

## 必须回传

1. 做了什么
2. 改了哪些文件
3. 新增了哪些字段和测试
4. 哪些项目上下文候选能稳定读取
5. 哪些字段只是不确定候选，不能当事实
6. 是否建议桌面应用线开始接静态索引
7. 风险和下一步建议
