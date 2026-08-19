# F2C01 壳—核心受控桥 v1（syn 核心侧）

阶段：stage-16 F2 壳—核心受控桥（syn 核心侧）

状态：`SUPERVISOR_SELF_REVIEW_PASS / ARCHIVED / F2_CORE_SIDE_LOCAL_ONLY / AUTHORIZATION_FILE_CLOSED / NOT_RELEASED`。合同/生命周期提交 `57f0830`；桥与定向测试内容候选 `629e4b2`。

来源收据：当前用户 2026-08-19 的“F2 核心侧 Kickoff（syn 仓库）”，receipt `u-675e71df2b9e60eb7baf`。当前指令明确开始 F2 syn 侧；`docs/harness/authorization.json` 不在预声明写面内，保持精确 closed，本叶不借 Stop 续跑扩大范围。

## 目标与方法集

冻结一个 newline-delimited JSON stdin/stdout 的 v1 headless 合同，并实现 `__syn_bridge`。首批方法集固定为：

1. `load_secretary_role_session_status_for_state(&AppState)`；
2. `load_global_supervisor_role_session_status_for_state(&AppState)`；
3. `load_role_session_directory_for_host(&AppState, host, &request)`；
4. `load_role_session_detail_for_host(&AppState, host, &request)`；
5. `operation_control::record_operation_control_decision_at(&state.workflow_state_path, &request, &timestamp)`。

host 由桥固定；renderer/壳只可提供合同定义的 opaque selection/cursor 与动作 payload，不得提供或覆盖身份、角色、项目路径、provider、权限、核心会话 id。v1 不包含任何 provider/model 调用方法，不包含 cfg(not(test)) + `spawn_blocking` 的秘书对话/source-route 方法，不包含需要第二个 Tauri State 的方法，也不消费未签发 M6 候选的 `m6_org_*` 方法。

## 建叶时的受保护工作树基线

本段投影 kickoff 已核实的 22 条基线，施工后的新鲜核验另记，不用本段反向覆盖并发 WIP 事实：

- HEAD 基线：`dca7229`，仅为四个 `include!` 隐藏模块的 rustfmt 1.9.0 formatting-only 提交；Rust 源码面零未提交改动，cargo test 基线干净。
- `git status --porcelain=v1 -uall` 共 22 条：3 条 tracked 修改——`docs/harness/unfinished/ENG-01-*.md`、`docs/harness/usage/.observed.json`、`docs/harness/usage/.observed.jsonl`；19 条未跟踪。
- 19 条未跟踪由 kickoff 点名为既有未归属 WIP/生成载体：6 个 `m6_*.rs`（含 1 个 `.bak`）、`gen/schemas/linux-schema.json`、日期 Harness 报告与 Harness usage/host 运行观察文件；它们不在 index、不进入本叶提交，受保护文件内容 hash 不得变化。
- origin 为 PUBLIC，本地领先 40 个提交且均未公开；本叶不执行任何远端 transport。
- stage-15 在 kickoff 基线中仍 active、等待 M6D08 候选 `a3d5759` 最终独立 verdict；施工前读取到的更新事实是该 verdict 已 PASS，但仍须总指导处置，本叶不触碰 stage-15 开闭状态。
- 建叶前 syn 侧无 current leaf，`authorization.json` 为 `{"schemaVersion":1,"authorized":false}`，源码写面为空，无并发施工者。
- 已存在 stage 文件仅 stage-12 与 stage-15；本轮新增 stage-16。

## 预声明写面

本叶只允许以下路径发生 F2 归属变化：

- 新增 `docs/contracts/f2-shell-core-bridge-v1.md`
- 新增 `docs/contracts/fixtures/f2-bridge-001/contract-cases-v1.json`
- 新增 `prototypes/productized-desktop-shell/src-tauri/src/f2_shell_core_bridge.rs`
- 新增 `docs/harness/stages/stage-16.md`
- 新增并在收口时原子归档本 leaf 文件
- 修改 `prototypes/productized-desktop-shell/src-tauri/src/main.rs`（仅 `+1` 子命令分支）
- 修改 `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`（仅 `+1 mod` 声明与 `+1 pub fn run_syn_bridge_cli`）
- 修改 `docs/harness/plan.md`（F2 未激活表述）
- 修改 `docs/current-state.md`（F2 未激活同类表述）
- 追加 `docs/harness/unfinished/ACC-01-real-data-and-real-provider-stage-acceptance.md`（仍保持未跟踪）

除 leaf 生命周期的原子位置变化外，实际写面不得扩大；非 push 偏差只报告。stage-15 最终收口可据此逐路径归责，不得把这些 F2 路径读成 M6 漂移。

## 做完的标准

1. 先沿用 M6D01 合同与 fixture 的形状冻结 v1 合同。每个方法逐 case 列出正常与错误路径，并把稳定错误码、不静默降级、幂等键、超时、Stop、进程崩溃恢复写成可判别约束。
2. 合同含硬约束 `no-model-invocation`：v1 method registry 的每个方法必须被判定为不会触发 provider/model；任何可能触发者注册即为合同不合格。F2 的真实模型验证因此明确为不适用，而不是缺失的通过项。
3. 合同逐类划定壳/核心后端：thread/desktop/pairing id 只能作为 receipt external refs，不能成为 Syn RoleSession 或 owner 身份；better-sqlite3/drizzle 只存壳布局、面板、线程 UI 等壳状态，不得成为 Syn 事实存储；`view.home` / `view.schedules` 与 Syn Secretary/Schedule 点名为同名不同物；壳可驱动 agent，但 ExecutionGrant 与完成判定留在 Syn 核心，壳不得自报完成。
4. 不动 `manifest.v1.json`：本合同作为 M1–M6 同惯例的增补合同并存；缺少增补合同独立机检索引继续归 ENG-01 第 11 条，本叶不解决。
5. `main.rs` 只加 `__syn_bridge`；`f2_shell_core_bridge.rs` 解析合同请求、调用点名函数并回写 typed response；`lib.rs` 只加普通 `mod` 与跨 crate 的 `pub fn run_syn_bridge_cli`。不修改 `commands.rs`，不改变任何 AppState/构造器/既有函数可见性。
6. 桥调用 `try_new_with_tauri_ordinary_product_seeds` 并显式传入合同启动参数中的 app-data/index/tasks seeds；不得调用 app-data-root 便利构造器，不新增 path-derived 兜底，不读取或设置 `SYN_R4_ACCEPTANCE_PROFILE`。
7. host 常量由桥固定；请求字段拒绝身份、角色、项目路径、provider、权限或会话 id。请求/响应按行隔离；畸形 JSON、未知方法、非法参数、EOF/Stop、timeout、幂等冲突、核心错误与内部 panic/进程崩溃恢复都有合同稳定结果或明确的客户端恢复规则，绝不静默改走另一权威槽位。
8. 定向单测消费合同 fixture，覆盖 cfg(test) 下真正可达的正常与错误 case 及全部桥错误码。任何只落到 test branch 的 case 明确标注；单测不证明 cfg(not(test)) 的 `AppState` production construction，真实 `__syn_bridge` 进程构造与崩溃恢复取证留给后续。
9. 执行 `cargo check` 与相关定向测试，记录精确命令、exit code、passed/failed；执行定向 rustfmt/check 与 `git diff --check`，不运行全仓 fmt、不触碰 17 个 fmt 盲区文件。
10. `ACC-01` 仅追加：“F2 阶段末结算口径已在本叶定义；ACC-01 第 1、2 条的可判别标准仍未做。”不改第 1–4 条正文、不关闭该叶、不改变它的未跟踪事实。
11. 独立检查实际 diff、产品调用图、冻结物与受保护 WIP；完成标准全部有直接证据后收口，结论只到 working-copy/local/offline fixture 与定向 Rust 证据，不冒充壳客户端、真实进程恢复、真实系统、部署、发布或公开历史。

## 不许动与停点

完整边界以当前用户 kickoff 为准，尤其禁止 push/merge/rebase/tag/release、真实凭据/provider/model/外部网络业务写、`commands.rs`、AppState 或构造器可见性、stage-15、任何 PASS verdict/候选、6 个受保护 `m6_*.rs`/`.bak`、`linux-schema.json`、17 个 fmt 盲区文件、F0 prune、远程 transport/Primary/authority epoch、F3/M6 renderer/双项目 App、stage-12/D0C04/D0C05、M1–M5 冻结合同正文、`manifest.v1.json`、`syn-shell` 与壳兼容标识。需要越过任一项时立即停点交总指导。

## 证据边界（收口时填写）

- cfg(test) 可达 case：5 个精确方法的 ready/unavailable、Jiaoban fixed-host directory/detail 与 opaque selector、operation receipt/同键 authoritative-audit replay/分歧冲突/执行自报拒绝，以及 11 个稳定桥错误、Stop、显式路径、external refs receipt-only、no-model registry/source 约束均由 `f2c01` 过滤覆盖。方法目标读取的三个槽位本身均无 cfg 门。
- test fixture 边界：Secretary/Global positive 使用 `try_new_with_tauri_ordinary_product_seeds` 的 cfg(test) 组合；directory/detail positive 手工安装 M3C07 isolated read-runtime fixture；operation positive 使用本地临时 workflow-state。它们验证真实函数体与 dispatch，但不证明 ordinary cfg(not(test) 启动组合或真实子进程。
- 落在 cfg 门后的 production case：本 v1 五个 dispatch 目标没有 cfg-gated 方法体；但 `AppState` production construction 含四个 cfg(not(test) 字段和真实 scheduler/composition，所以本轮单测只证明 test 构造分支，不把它记成 production startup PASS。
- 未证明的 production/真实进程路径：cfg(not(test) `try_new_with_tauri_ordinary_product_seeds` 组合、真实二进制 `__syn_bridge` 子进程启动、stdin/stdout 跨进程交互、SIGKILL/崩溃后对端重连、壳销毁后的恢复与新壳窗口。合同定义了恢复语义，真实取证另派。
- 主工作树最终证据（cwd `prototypes/productized-desktop-shell/src-tauri`）：
  - `cargo test --lib f2c01 --offline`：exit 0，10 passed / 0 failed / 0 ignored / 2171 filtered out；
  - `cargo test --lib operation_control::tests --offline`：exit 0，5 passed / 0 failed / 0 ignored / 2176 filtered out；
  - `cargo check --offline`：exit 0，rustc 汇总 888 warnings，与既有候选基线一致，F2 新增 warning 为 0；
  - `node -e "JSON.parse(require('fs').readFileSync('docs/contracts/fixtures/f2-bridge-001/contract-cases-v1.json','utf8'))"`：exit 0；
  - `rustfmt --edition 2021 --check prototypes/productized-desktop-shell/src-tauri/src/f2_shell_core_bridge.rs prototypes/productized-desktop-shell/src-tauri/src/main.rs`：exit 0；
  - `git diff --check`：exit 0。
- detached 证据：`/tmp/syn-f2-verify-629e4b2` 精确 checkout `629e4b2`；同三条 Cargo 命令分别 exit 0，10/10、5/5、cargo check 888 warnings；验证后该精确 worktree 与其 target 已移除，未 prune 其他 worktree。
- 返修轨迹：最初 F2 过滤 8/10（两处测试尺子偏差），随后 9/10（源码切片自引用），消除两条新增 warning 时又暴露一次 9/10 的测试切片锚点偏差；均只修 fixture/测试判据，最终 10/10 并把 warning 890 收回既有 888。失败轮次不冒充通过证据。
- 写面核账：F2 归属精确等于预声明 10 路径（leaf 最终只作原子归档位置变化），`commands.rs`、`manifest.v1.json`、stage-15 与 17 个 fmt 盲区文件零 diff；kickoff 后多出的 5 个 `.turns/*.json` 是 Hook 运行时，不归 F2。7 个受保护未跟踪载体 hash 仍为 stage-15 verdict 记录的 `620faa27…`、`2c576d9b…`、`6cd604b4…`、`147bd08e…`、`6155c26a…`、`7db42ba1…`、`7e51a7ed…`，均未进 index/提交。
