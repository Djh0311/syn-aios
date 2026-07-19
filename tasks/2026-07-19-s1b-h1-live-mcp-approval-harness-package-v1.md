# 任务包：S1B-H1 真 CLI 单工具批准 harness 收口 v1

日期：2026-07-19
状态：**已完成**
上位任务：`tasks/2026-07-18-s1b-supervisor-transport-oneshot-resume-package-v1.md` A5
证据：`evidence/2026-07-19-s1b-h1-live-mcp-approval-verification-v1.md`

## 一、授权与边界

- 用户在本对话确认继续 S1B-H1；该授权只覆盖**测试夹具修复、真实 Codex 副本店复跑和只读对账**。
- 本包不授权批准任何方案卡、不授权启动 workflow chain、不授权修改固定测试项目业务文件。
- 产品 `codex exec` / `exec resume` argv、`--sandbox read-only`、默认批准策略、项目私有 `CODEX_HOME` 白名单和 MCP handler 语义必须保持不变。
- 不得使用 `--full-auto`、`--dangerously-bypass-approvals-and-sandbox`、server-wide 默认批准或通配工具批准。

## 二、已知事实、未知项与实施假设

### 已知事实

1. 修前真实模型已经生成合法 `submit_proposal` 调用，但旧 thread 的 `exec resume` 以 `approval_policy=never` 运行，客户端在 handler 前返回 `user cancelled MCP tool call`。
2. 对新 thread 的独立探针证明，Codex 0.144.1 支持：
   - `mcp_servers.supervisor_orchestrator.enabled_tools=["submit_proposal"]`
   - `mcp_servers.supervisor_orchestrator.tools.submit_proposal.approval_mode="approve"`
3. `codex exec resume --help` 明确显示 `resume` 子命令拥有自己的 `-c/--config`；只在外层 `exec` 前传覆盖不能证明旧 thread 的 resume 层收到该配置。

### 未知项

- 后续 Codex CLI 版本是否继续保留相同的嵌套参数解析与配置字段。为此 wrapper 必须带 `--strict-config`，字段漂移时失败关闭。

### 实施假设

- 产品 command plan 的受控形状保持 `exec -C <root> --sandbox read-only [resume ...]`；测试 wrapper 只在该既有形状上分流 initial / resume。

## 三、最小改动

唯一代码改动面：

- `prototypes/productized-desktop-shell/src-tauri/src/supervisor_resident_oneshot_tests.rs`

实现：

1. ignored live 只有在精确确认变量存在时才安装临时 `PATH` wrapper，并要求真实 Codex 为绝对、存在、可执行文件。
2. wrapper 只接受首参数 `exec`：
   - initial：在 `exec` 外层注入 `enabled_tools=["submit_proposal"]` 与该工具 `approval_mode="approve"`；
   - resume：在字面 `resume` 后注入同一组覆盖，使配置进入 resume 子命令自己的解析层。
3. wrapper shell 作为父进程等待真实 Codex 并透传退出码，不以 `exec` 替换自身，保留既有 PID / argv / 进程组登记身份。
4. RAII guard 在测试结束恢复原 `PATH` 并删除临时 wrapper 目录。
5. 离线测试锁死：仅 `submit_proposal`、initial/resume 两支都注入、无 server-wide default、无 approval policy/reviewer/sandbox 放宽、无 full-auto/bypass。

## 四、验收条件

1. `s1b_h1_live_wrapper_preapproves_only_submit_proposal` 通过。
2. ignored live 在真实旧 thread 上完成：
   - 真实 `submit_proposal` 到达既有 handler；
   - 新增一张带唯一 marker 的 `PendingUserConfirmation` 卡；
   - workflow chain 数量不变；
   - 后两轮续同一旧 thread；
   - 人工置入无效 thread 后，真实 CLI 非零且无 stdout `thread.started`；
   - generation 换代、旧家归档、事实注入、新 thread 建立。
3. 测试相关 wrapper / Codex / MCP 进程无残留。
4. S1B、S1、M5B、M5C、全量 Rust、cargo check、typecheck、离线交互、shape gate 零净增、diff/rustfmt 均满足当前基线。

## 五、完成结果

- ignored live：**1 passed / 0 failed**，71.89s。
- proposal store revision 132；唯一 marker 卡状态 `pending_user_confirmation`。
- workflow chain 保持 40 条，无新增链。
- generation 6 归档，active generation 7；新 thread `019f771a-ac76-7bc1-b92e-a8204cf92f9f` 成功回引持久事实。
- 聚合闸：Rust **1009 passed / 0 failed / 44 ignored**；S1B **16/0/1**；S1 **11/0/1**；M5B **9/0**；M5C **5/0**。

## 六、下一步

S1B-H1 已关门。下一施工段是用户在场的底1真机首单：在 S1C 新布局完成“聊 → 工具落卡 → 用户点批 → 跑”。**本包授权到落 Pending 卡为止，不自动批准或起链。**
