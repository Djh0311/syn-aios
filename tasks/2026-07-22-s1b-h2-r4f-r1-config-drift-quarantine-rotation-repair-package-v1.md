# 任务包：S1B-H2-R4F-R1 `config_drift` 隔离换代修复 v1

- 日期：2026-07-22
- 状态：已获用户精确授权，待执行；仅代码与离线验证
- 前置现场：R4F 首句止于 `preflight/preflight_home`
- 前置归因：关闭现场 fixed-output 检查锁定最早 leaf=`config_drift`
- 前置止损：未知配置没有有限历史信任锚；泛化改写候选已撤回，两份 Rust 源码恢复开工 SHA
- 唯一 kickoff：`handoffs/2026-07-22-s1b-h2-r4f-r1-config-drift-quarantine-rotation-repair-kickoff-v1.md`

## 1. 唯一目标

当且仅当既有 resident active home 的目录、私有文件、项目身份、generation 与 auth 链接全部仍受信，唯一异常为**可解析但既不等于当前精确配置、也不等于既有精确 legacy 配置的 `config_drift`**时：

1. 不执行、不迁移、不覆盖未知配置；
2. 将整个旧 active home 原样、原子地隔离进既有 archive；
3. generation 精确 `+1`，以当前受控配置在 staging 中创建全新 active home；
4. 重建项目事实，将当前 canonical 消息作为新 generation 的 initial 回合仅执行一次；
5. 后续回合续接新 thread。

这是“拒绝复用未知 home 后换代”，不是“信任或修复未知配置”。R4G 只做最后一次真实两句验收，不再承担该 leaf 的诊断。

## 2. 授权与写入面

用户已在 2026-07-22 明确同意上述隔离换代设计；本授权只覆盖代码和离线 fixture，不覆盖真实 App、真实 store、真实 controlled home、真实 Codex/MCP 或消息发送。

开工冻结 HEAD、staged、porcelain、dirty ownership 与下列两文件 SHA-256。代码只准修改：

- `prototypes/productized-desktop-shell/src-tauri/src/supervisor_resident_oneshot_session.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/supervisor_resident_oneshot_tests.rs`

永久回写只准新增本包 evidence、最小更新 `CURRENT.md`，以及确有新拦截时向 catch log EOF 追加。两份 Rust 文件虽含既有 H2/R4E 未提交改动，但已由本包明确接管当前冻结 SHA；SHA 或内容与开工冻结不符则 `BLOCKED_DIRTY_OVERLAP`，不得覆盖。

## 3. 必须保持的安全条件

隔离换代前必须在同一受控方法内重新验证，不能依赖错误人话或一次旧分类：

- active 是真实目录、不是 symlink，且 owner-only；
- config 与 metadata 都是真实 regular file、不是 symlink，且 owner-only；
- metadata 可解析，并与当前 run、workflow state 路径和旧 generation 精确一致；
- auth entry 是 symlink，精确指向默认 auth source；默认 auth source 仍是 regular file；
- 当前 config 可解析，且重新分类仍精确为 `config_drift`；
- archive 根受控且 owner-only，归档目标不存在；新 generation 无溢出。

任一条件不满足，保持原有 fail-closed：不 rename、不建 staging、不跑 runner、不递增 generation。缺失、malformed、unreadable、类型异常、权限异常、metadata 身份/代际不符、auth 异常、expected config 与 exact legacy config 均不得误入本分支。

不得用错误字符串匹配驱动换代；使用内部 typed leaf/enum。不得输出或持久化未知配置内容、argv、路径、完整 identity、symlink target、raw error、auth/token。

## 4. 最小实现合同

1. 保留普通 `ensure_active` 的严格复用语义，以及 exact legacy 的既有原子迁移。
2. 为 `config_drift` 建立 typed 内部结果；只在持久 resident session 的 preflight、runner 尚未启动时接住该结果。
3. 构建 `generation+1` 的 initial plan；不得先尝试旧 resume。
4. 复用既有 archive → staging create → atomic promote → create 失败恢复旧 active 的机制，但隔离入口执行第 3 节独立安全验证，不要求未知 config 等于受信模板。
5. 重建事实并仅调用一次 runner；不得递归、自动重发或再次接住同类失败。
6. 复用既有 `SupervisorResidentLaunch::Replaced`，reason 固定为 `config_drift`；不得新增 DB 写路。既有 canonical recorded、delivery diagnostic、R4E tools/list→call→handler→audit 与 proposal 幂等语义不变。
7. 新 active 创建或 initial runner 失败时，用户面继续使用既有人话；不得泄露旧配置或私有路径。home 创建失败必须恢复旧 active；runner 已开始后的失败不得伪造 injected/reply/tool/card。

不得修改 H2 唯一 `submit_proposal` 预批准、sandbox/read-only/reviewer/path-lock、MCP command/sidecar、watchdog、进程组清理、invalid-resume 判定、M5 CAS/降级、Tauri command 或固定测试项目。

## 5. 先红后绿

至少覆盖以下离线 fixture：

1. 有效旧 resident + 唯一 `config_drift`：修前拒绝，修后旧 active 整体进入 archive、未知 config 字节原样保留，新 active 使用当前精确配置，generation `+1`。
2. 当前 canonical 消息只走一次新 initial，不执行旧 resume；opening 含重建事实。下一回合续同一新 thread。
3. 换代后的 H2 单工具配置仍精确，模拟第二句最多落一张 Pending 卡；proposal 幂等、chain 与固定项目不变。
4. expected config 正常复用、exact legacy 仍走原迁移，均不归档换代。
5. malformed/unreadable/missing config、config/metadata symlink、非 owner-only、metadata run/workflow/generation mismatch、auth mismatch 均 fail-closed；archive、active、runner 计数不变。
6. staging/create 失败恢复旧 active，未知 config 字节不变；不得留下 active 缺位或双 active。
7. typed leaf 驱动而非中文/英文错误字符串；同一回合无递归、无第二次换代。

## 6. 离线闸与停机条件

运行：新增 red/green、相关 S1B/H2、S1 submit、M5-B/C/F1、`cargo check --offline --lib`、TypeScript typecheck、既有离线 interaction、shape baseline/check、目标 Rust 文件 `rustfmt --check`、脱敏扫描与 scoped `git diff --check`。历史 shape 债与本包净增分开报告。

代码和离线闸通过即停。不得 build/start App，不得运行真实 Codex CLI/MCP，不得读取或操作真实 store/private home/auth，不得发送 H2 消息、刷新/点卡或启动 chain。

若实现必须扩大到第三个源码文件、放宽第 3 节任一安全条件、读取真实配置正文或改变审批/沙箱，裁决 `BLOCKED_SCOPE_EXPANSION` 并停止。完成后只可另出 R4G 最终 live 验收包。

## 7. 回传

回传十项：冻结/dirty；typed leaf；隔离前置条件；归档与回滚语义；generation/initial/事实注入；拒绝面；H2 工具与幂等不变量；定向/聚合闸；shape/diff/rustfmt；未执行的 live 动作与 R4G 状态。
