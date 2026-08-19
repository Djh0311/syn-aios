# 阶段16 F2 壳—核心受控桥（syn 核心侧）

状态：`ACTIVE / F2_CORE_SIDE / WORKING_COPY_ONLY / NOT_RELEASED`。

来源收据：当前用户 2026-08-19 的“F2 核心侧 Kickoff（syn 仓库）”，receipt `u-675e71df2b9e60eb7baf`。本阶段只覆盖 `/home/synadmin/workspace/syn` 的核心侧合同、headless 桥、定向测试与必要状态记账；不进入 `syn-shell`，不做壳侧客户端、真实恢复取证、真实 provider/model、外部网络业务动作、部署、发布或公开 push。

与 stage-15 的关系：stage-15 最终独立 verdict 已为 PASS，但其终包明确要求由总指导处置关闭；本阶段不修改 stage-15 的开闭状态、不改其 verdict 或候选。stage-16 的源码写面串行开始，不与 stage-15 施工并发，也不把 stage-15 的域层候选当作本桥 v1 的依赖。

目标：冻结 Syn 治理核心与新桌面壳之间的 v1 受控接口合同，并在核心侧实现一个 `__syn_bridge` headless 子命令。壳只能读取核心提供的 RoleSession 状态与 typed read model，并通过核心提交一个受控写动作取得 receipt；核心仍是身份、权限、事实与完成判定的唯一权威。

完成边界：

1. 新增增补合同与逐 case fixture，明确稳定错误、幂等键、超时、Stop、崩溃恢复、no-model-invocation 与壳/核心双后端边界；不修改 `manifest.v1.json` 或 M1–M6 冻结合同正文。
2. `main.rs` 新增与既有 headless 子命令同形的 `__syn_bridge` 分支；新模块只派发 kickoff 点名的 5 个函数，不修改 `commands.rs`，不扩大既有函数或 `AppState` 构造器可见性。
3. 桥构造显式使用 ordinary product seeds，不新增 path-derived fallback，不设置 `SYN_R4_ACCEPTANCE_PROFILE`，不把壳提供的 id、路径、provider、权限或身份当作核心真值。
4. fixture 中 cfg(test) 真正可达的正常与错误 case 有定向单测；`cargo check` 与相关测试记录精确命令、退出码和 passed/failed。单测不得冒充 cfg(not(test)) 的生产构造链或真实进程取证。
5. `ACC-01` 只追加 F2 后续结算口径，不改第 1–4 条正文，不关闭它，并保持该文件未跟踪；不做真实数据、真实 provider 或真实恢复动作。

叶子：

- [ ] `F2C01-shell-core-bridge-v1.md`：合同冻结、核心侧 headless bridge、定向测试与记账。

硬停点：需要 push/merge/rebase/tag/发布、真实凭据、真实 provider/model、外部网络业务动作、`syn-shell` 写入、修改 `commands.rs` / AppState 可见性 / 既有冻结合同 / stage-15，或需要进入 kickoff “不许动”路径时，停止并交回总指导。
