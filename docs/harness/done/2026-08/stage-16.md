# 阶段16 F2 壳—核心受控桥（syn 核心侧）

状态：`CLOSED / F2_CORE_SIDE_SCOPED_PASS / NOT_RELEASED`。总指导 2026-08-20 02:23 决定关闭，主管代执行记账。

关闭依据链：首轮（合同 `57f0830`、桥 `629e4b2`、记账 `86dd29e`）获整 F2 独立验收 **FAIL**（唯一阻断项：合同规定 register 的 `capability_permission_refs` 与 `contact_bindings` v1 必须为空，实现接受并落库且无稳定错误码）；`F2C01R01` 返修（v1 缩为三方法、先真进程证明后冻结合同、修 unclassified 泄漏与 fixture 覆盖）；`F2C01R02`（提交 `e511726b`）补执行 v1 四数组必须为空，非空 fail-closed 返回 `F2_FORBIDDEN_AUTHORITY_INPUT`、被拒写零落库零幂等残留；2026-08-20 02:18 独立复核 verdict **PASS**（范围限阻断项修复 + 回归 + 两仓合同钉扎；结论绑定 syn `e511726b627aa1207d6d1128e626a34ecd6bbfaa`、syn-shell `16f01df388e979deeff3246896641e681a6e1c86`）。

放行只到两份 verdict 实际范围。以下均未成立：核心 mid-write 崩溃恢复、SIGKILL 核心作为专门崩溃恢复测试、F3 界面重建、F4 执行治理对接、真实 provider/model/账号/凭据、外部网络业务写、打包、部署、发布、push。

来源收据：当前用户 2026-08-19 的“F2 核心侧 Kickoff（syn 仓库）”，receipt `u-675e71df2b9e60eb7baf`。本阶段只覆盖 `/home/synadmin/workspace/syn` 的核心侧合同、headless 桥、定向测试与必要状态记账；不进入 `syn-shell`，不做壳侧客户端、真实恢复取证、真实 provider/model、外部网络业务动作、部署、发布或公开 push。

与 stage-15 的关系：stage-15 最终独立 verdict 已为 PASS，但其终包明确要求由总指导处置关闭；本阶段不修改 stage-15 的开闭状态、不改其 verdict 或候选。stage-16 的源码写面串行开始，不与 stage-15 施工并发，也不把 stage-15 的域层候选当作本桥 v1 的依赖。

目标：冻结 Syn 治理核心与新桌面壳之间的 v1 受控接口合同，并在核心侧实现一个 `__syn_bridge` headless 子命令。壳只能读取核心提供的 RoleSession 状态与 typed read model，并通过核心提交一个受控写动作取得 receipt；核心仍是身份、权限、事实与完成判定的唯一权威。

完成边界：

1. 新增增补合同与逐 case fixture，明确稳定错误、幂等键、超时、Stop、崩溃恢复、no-model-invocation 与壳/核心双后端边界；不修改 `manifest.v1.json` 或 M1–M6 冻结合同正文。
2. `main.rs` 新增与既有 headless 子命令同形的 `__syn_bridge` 分支；新模块只派发合同点名的方法集（首轮 kickoff 曾点名 5 个函数；`F2C01R01` 起为三方法注册表：`secretary_status`、`global_supervisor_status`、`register_stable_member`，directory/detail 移交 F3。本行原写“5 个函数”，2026-08-20 复核 verdict 点名为过期表述，关闭处置时更正，历史见 F2C01/F2C01R01 叶），不修改 `commands.rs`，不扩大既有函数或 `AppState` 构造器可见性。
3. 桥构造显式使用 ordinary product seeds，不新增 path-derived fallback，不设置 `SYN_R4_ACCEPTANCE_PROFILE`，不把壳提供的 id、路径、provider、权限或身份当作核心真值。
4. fixture 中 cfg(test) 真正可达的正常与错误 case 有定向单测；`cargo check` 与相关测试记录精确命令、退出码和 passed/failed。单测不得冒充 cfg(not(test)) 的生产构造链或真实进程取证。
5. `ACC-01` 只追加 F2 后续结算口径，不改第 1–4 条正文，不关闭它，并保持该文件未跟踪；不做真实数据、真实 provider 或真实恢复动作。

叶子：

- [x] `F2C01-shell-core-bridge-v1.md`：首轮合同冻结、核心侧 headless bridge、定向测试与记账（内容 `629e4b2`，主管本地自复核 PASS 后已归档；独立验收 FAIL；历史结论不悄悄改写）。
- [x] `F2C01R01-shell-core-bridge-v1-repair.md`：stage-16 第二轮返修。先接线三项方法并用真进程证明，再按真实结果冻结合同；修 unclassified/路径泄漏与 fixture 覆盖；更正首轮过满表述。主管自复核 PASS 已归档。
- [x] `F2C01R02-v1-empty-array-enforcement.md`：stage-16 第三轮。只补执行合同已写明的四个 v1 数组必须为空；非空返回 `F2_FORBIDDEN_AUTHORITY_INPUT`；被拒写不留幂等记录。2026-08-20 02:18 独立复核 PASS。

关闭处置（2026-08-20）：本文件由 `docs/harness/stages/` 原子移动至 `docs/harness/done/2026-08/`。遗留欠账不随关闭消失，归属照复核 verdict：ENG-01（coverage-audit 仅名字解析、CLI basename 未预拦且需落账、收据二进制 sha 路径绑定、fmt 17 文件盲区、/tmp 既有载体）、F3（directory/detail 生产安装路径）、OSS-01（推送决定）、syn-shell 工具链债（默认 node v22、launcher 无超时 fetch）。

硬停点（存档）：需要 push/merge/rebase/tag/发布、真实凭据、真实 provider/model、外部网络业务动作、`syn-shell` 写入、修改 `commands.rs` / AppState 可见性 / 既有冻结合同 / stage-15，或需要进入 kickoff “不许动”路径时，停止并交回总指导。
