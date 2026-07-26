# 任务包：S1B-H2-R4F `preflight_home` 受控私有 home fail-closed 闭环修复 v2

- 日期：2026-07-22
- 状态：已原位修订为 v2，未执行；关闭现场元数据只读归因 + 代码/离线修复授权
- 前置证据：`evidence/2026-07-22-s1b-h2-r4f-one-pass-real-app-tool-attribution-and-pending-card-verification-v1.md`
- 唯一 kickoff：`handoffs/2026-07-22-s1b-h2-r4f-preflight-home-repair-kickoff-v1.md`

## 0. 唯一目标与已证边界

R4F 的一条新首句只新增 canonical `recorded=1` 与同 message 的一条安全 diagnostic：`stage=preflight`、`stable_error_family=preflight_home`；`injected/reply=0`，没有 resident binding、R4E 工具事实、proposal/Pending/chain 增量。最早已证边界是 controlled resident-home preflight，发生在 runner 启动前。

`preflight_home` 是聚合稳定族：它可来自 private-home 创建、既有 active-home 校验、owner-only/认证符号链接、精确 MCP 配置或 generation/run/state 身份校验等多个 fail-closed 分支。现有安全留痕故意不保留原始 detail；不得把它猜写为 auth、配置、权限或任一具体子因。

本包的唯一修复目标是：先在**关闭现场**以固定、脱敏的只读元数据分类锁定 R4F 当前 controlled home 命中的具体 leaf，再在同一个包内为该 leaf 建立离线红灯、实施最小修复并跑完离线闸。R4G 只能做最终 live 验收，不再承担 leaf 诊断。

不得为了减少轮次而猜根因。若当前现场已经漂移到无法复现 R4F leaf，只能裁决 `BLOCKED_PREFLIGHT_HOME_STATE_NOT_REPRODUCIBLE` 并停止；不得退回“先补分类、R4G 再看”的循环，也不得另出中间诊断包。

## 1. 冻结、写入面与禁止项

开工先冻结 HEAD、staged、porcelain、既有 dirty ownership，以及以下最小源码面 SHA-256：

- `prototypes/productized-desktop-shell/src-tauri/src/supervisor_resident_oneshot_session.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/supervisor_resident_oneshot_tests.rs`

允许的代码写面仅为上述两文件；永久文档写面仅为本包 evidence、`CURRENT.md`，以及确有新拦截时 catch log EOF。若证明必须触及 shared launcher、approval/allowlist、MCP transport、sidecar、command、watchdog、invalid-resume、进程组清理、M5 或任何其他源码，立即停止并报告扩范围，不得顺手扩大。

绝对不得：启动/构建真实 App、运行 Codex CLI/MCP server、读取/复制/写入真实 Workbench store、runner 产物或认证资料；除下一节列出的 fixed-output 元数据检查外，不得读取、复制或写入 controlled private home。不得改 `submit_proposal` 的唯一预批准、read-only/approval/sandbox/reviewer/path-lock/写根，不得点卡、批准卡、启动 chain/worker，亦不得修改固定测试项目。

## 2. 关闭现场的唯一元数据归因

在 App/Workbench/dev/Codex/MCP、registry、lock、store/DB/WAL/SHM holder 全空后，允许对**当前 workflow/run 派生出的单一 controlled resident home**做一次本机、流式、只读检查。不得遍历其他 home，不得复制或导出文件。

允许读取内部结构以完成比较，但终端、evidence 和回传只能出现以下固定结果：

| 检查面 | 唯一允许输出 |
| --- | --- |
| base / active | `present/absent`、`directory/not_directory`、`owner_only/not_owner_only` |
| config / metadata | `present/absent`、`regular/not_regular`、`owner_only/not_owner_only` |
| MCP config | `expected` / `exact_legacy` / `drift` / `malformed` / `unreadable` |
| home metadata | `run_match`、`workflow_match`、`generation_match` 三个 boolean，及 `valid/malformed/unreadable` |
| auth entry | `present`、`is_symlink`、`targets_default_auth` 三个 boolean；只允许对默认 auth 源做 `regular_file` boolean 检查 |
| 创建/迁移条件 | `base_creatable`、`legacy_migration_required`、`replacement_target_safe` 固定 boolean/`not_observed` |

严禁输出或入仓：任何文件内容、TOML/JSON 字段值、路径、run/workflow 完整 identity、generation 以外的元数据、symlink target、auth 内容、token、原始错误或 errno 文本。不得读取默认 auth 文件正文；只可 `lstat/metadata` 并输出 `regular_file` boolean。

必须用源码分支表与上述现场分类两类证据唯一锁定一个 leaf，例如 `active_permission`、`config_drift`、`metadata_generation`、`auth_link_target`、`legacy_migration` 等固定枚举。多个 leaf 同时异常时，以源码执行顺序裁决最早 leaf；无法唯一锁定则按 `BLOCKED_PREFLIGHT_HOME_STATE_NOT_REPRODUCIBLE` 停止。

## 3. 最小修复语义

1. 保持现有 parent family `preflight_home` 的 fail-closed 含义；未知或不可信 home 仍拒绝复用，绝不以“现场能继续”为由删除检查、覆盖 active home、复制认证资料或泛化 config migration。
2. 只修第 2 节已由现场分类与源码执行顺序共同锁定的最早 leaf；不得顺手处理其他 home 状态。
3. 如需保留安全分类，只能在既有 message-scoped delivery diagnostic 中增加一个固定 leaf 枚举，复用 Batch 2 canonical 写路；不得增加路径、raw error、认证状态、argv、环境或 token。
4. 对 owner-only、身份/generation、auth symlink、未知 config drift 等安全拒绝面，默认保持 fail-closed。只有 fixture 能证明输入属于本产品自己生成、可无歧义恢复且不覆盖未知状态时，才允许归档/重建或原子迁移。
5. 诊断写失败必须保持 recorded 事实和既有用户面结果，不触发 retry、rebase、降级或新 DB 写路。

## 4. 先红后绿与不变量

先以最小 fixture 建红灯，再实现最小修复。至少覆盖：

- 第 2 节锁定的真实 leaf 必须有对应最小 fixture；修前复现相同固定分类和失败行为，修后满足预期受控行为；
- recorded 保留、diagnostic 单条幂等，且没有伪造 injected/reply、runner 启动、registry 或工具事实；
- 未知 config drift、非 owner-only、身份/代际不符、认证符号链接异常等既有拒绝面仍 fail-closed，不能被本包“修好”；
- delivery diagnostic Batch 2 写失败不影响既有 recorded/incomplete 业务结果；
- H2 只有 `supervisor_orchestrator.submit_proposal` 可预批准，R4E tools/list→call→handler→audit 行为不变；invalid-resume 单次轮转、watchdog/进程清理与 M5 DB-primary/CAS/降级不退步。

新增 facts 只可含固定枚举/布尔、既有 message identity、generation 与短摘要；测试、canonical/read model/evidence 均不得含正文、arguments、raw error/stderr、argv、环境、token/auth、完整 identity 或 private path。

## 5. 离线闸与停止条件

运行最小定向 red/green、相关 S1B/H2 与 M5 离线闸、`cargo check --lib`、既有 shape gate、脱敏扫描与 `git diff --check`。历史 shape 债与本包净增分开报告；任何离线绿都不得称为 H2 live 通过。

代码和离线闸通过即停：不得在本包启动 App、发送首句或第二句。成功回传必须同时给出：现场固定 leaf、两类锁定证据、修复文件/语义、红灯与绿灯结果、安全拒绝面不变量。

最终现场只允许另出 R4G 最终验收包、另获用户在场授权，并从全新 Gate 0、新裸 binary、新 client/message identity 开始；不得复用 R4F 的失败 message。R4G 不得再承担 home leaf 或工具线诊断；若仍失败，按实际产品失败处理，不再继续拆观察包。
