# F2C01R01 壳—核心受控桥 v1 返修（syn 核心侧）

阶段：stage-16 F2 壳—核心受控桥（syn 核心侧）

状态：`SUPERVISOR_SELF_REVIEW_PASS / ARCHIVING / F2_CORE_SIDE_REPAIR_LOCAL_ONLY / AUTHORIZATION_FILE_CLOSED / NOT_RELEASED`。本叶是 stage-16 第二轮返修，不是重做。首轮合同 `57f0830`、桥 `629e4b2`、记账 `86dd29e` 已获独立验收 FAIL；本叶只修 kickoff 点名的缺陷，不推翻首轮已通过项。本轮报告不构成 stage-16 关闭，须重新送独立验收。工作副本尚未形成新的 git 候选 SHA。

来源收据：当前用户 2026-08-19 的“F2 核心侧返修 Kickoff v2（syn 仓库，stage-16 第二轮）”。该 kickoff 构成 stage-16 返修的明确开始。`docs/harness/authorization.json` 保持精确 closed 两字段；本叶不借 Stop 续跑扩大范围。

## 目标与方法集

先接线并用真进程证明，再按真实结果冻结合同文字。v1 方法集改为三项：

1. `role_session.secretary_status` → `load_secretary_role_session_status_for_state(&AppState)`；
2. `role_session.global_supervisor_status` → `load_global_supervisor_role_session_status_for_state(&AppState)`；
3. `organization.register_stable_member` → `m6_org_member_directory::register_for_state(&AppState, &M6OrgRegisterStableMemberRequest, now_ms)`。

`role_session.directory` / `role_session.detail` 放进 F3，本轮不为它们新增任何生产安装路径。首轮写动作 `operation_control.record_decision` 换成上述 M6 登记；首轮 kickoff 中“不得使用 m6_org_* 函数”已作废。

## 建叶时的受保护工作树基线

本段投影 kickoff 与建叶时新鲜核验，不用本段反向覆盖并发 WIP 事实：

- HEAD：`86dd29e`（首轮记账 close）。`origin/main` 仍为 `4791654`。
- `docs/harness/leaves/` 在建叶前为空（仅 `.keep`）；无 current leaf。
- `authorization.json` 为 `{"schemaVersion":1,"authorized":false}`。
- porcelain 含既有未归属 WIP：3 条 tracked 修改（`ENG-01`、Harness usage 观察文件）与 19+ 条未跟踪（6 个受保护 `m6_*.rs`/`.bak`、`linux-schema.json`、日期报告、usage/host/turns、`ACC-01`）。它们不进本叶提交，受保护载体 hash 不得变化。
- 首轮已通过、不得推翻：写面恰为预声明范围；`commands.rs` 零差异；AppState 与两个构造器仍非 pub；`main.rs` 只加一个同形 `__syn_bridge` 分支；`manifest.v1.json` / 其余冻结合同 / stage-15 候选与 verdict 零差异；`METHOD_REGISTRY` 为编译期定长数组；无 provider/model/外部网络可达；请求中的 thread/desktop/pairing id 只在 receipt 回显；生产段无 env/cwd/home/`CARGO_MANIFEST_DIR`；不读不设 `SYN_R4_ACCEPTANCE_PROFILE`；888 warnings 基线、F2 新增 0；`run_syn_bridge_cli` 未被单测伪装执行；7 个受保护未跟踪载体 hash 不变；无 push/merge/rebase/tag；ACC-01 前 23 行逐字未变、仍未跟踪、未关闭。

## 预声明写面

本叶只允许以下路径发生 F2 返修归属变化：

- 修改 `prototypes/productized-desktop-shell/src-tauri/src/f2_shell_core_bridge.rs`
- 修改 `docs/contracts/f2-shell-core-bridge-v1.md`（只在真进程证明之后按真实结果改写）
- 修改 `docs/contracts/fixtures/f2-bridge-001/contract-cases-v1.json`
- 修改 `docs/harness/stages/stage-16.md`
- 新增并在收口时原子归档本 leaf 文件
- 修改 `docs/harness/plan.md`（stage-16 返修状态表述）
- 修改 `docs/current-state.md`（stage-16 返修状态表述）
- 追加 `docs/harness/audit/2026-08.jsonl`
- 新增 `docs/harness/reports/F2C01-archived-leaf-coverage-correction-v1.md`（更正首轮过满表述，不改写已归档 F2C01 历史结论正文）
- 必要的 fixture 覆盖统计入口（只服务本叶第 4 步机械审计）

不改 `main.rs` / `lib.rs` / `commands.rs` / AppState 可见性 / `manifest.v1.json` / stage-15 / ACC-01 第 1–4 条正文。除 leaf 生命周期的原子位置变化外，实际写面不得扩大；非 push 偏差只报告。

## 做完的标准

1. **先接线、后冻结合同。** `METHOD_REGISTRY` 改为三项。移除 `role_session.directory` / `role_session.detail` 及随之失效的代码（含 `FIXED_READ_HOST` 与仅为这两方法存在的 M3C07 手工安装测试脚手架），不得留 dead code 或用 blanket allow 掩盖。接上 `register_for_state` 的派发与参数结构。
2. **真进程证明。** 用真实 `__syn_bridge` 子进程、在全新空 `app_data_root`（末段目录名恰为 `local.codex.governance.workbench`）上对三个方法各成功调用一次，并对写动作用同一 `idempotency_key` 再调一次以证明 `replayed=true`。记录每次完整请求与响应 JSON。任一方法在全新根上无法成功，立即停下回总指导，不自行改选方法或补前置能力。
3. **按第一步真实结果冻结合同。** 合同只写已被真进程证明可用的东西。必须包含：(a) v1 完整输入域，逐条可判别，每条配稳定错误码与合规示例，至少覆盖 app_data_root 末段目录名、domain 方法必须带 `deadline_unix_ms`、deadline 距今上限 30_000ms、写动作 `idempotency_key` 格式、写动作请求各字段取值约束；(b) 幂等键、deadline、Stop、崩溃后同键恢复、稳定错误码；(c) no-model-invocation 硬条款；(d) 四条双后端分界线条款；(e) 写动作语义边界：只产生 M6 组织目录的成员登记与其收据，不构成执行 / ExecutionGrant / 完成判定；(f) 一段由第一步真进程产出的成功请求/响应样例。禁止写入未经真进程验证的方法或输入域。如实记明首次写入惰性创建 `<root>/m6/organization.sqlite` 是核心自建存储，不是桥造路径。
4. **unclassified 与路径泄漏。** 所有可达的核心失败都映射到合同登记的稳定错误码；边界响应不得含绝对文件系统路径、原始 OS 错误串、stderr 或任何宿主机路径片段；确实无法分类的残余必须有一个登记在合同里的显式错误码并且不带原始消息。补定向测试覆盖至少一条“核心返回未登记错误”的路径，断言响应不泄漏路径。
5. **fixture 覆盖 100%。** 移除 directory/detail 相关 case 后重新统计。CF-F2-POS-008 的 external refs 断言必须挂在其精确 case 上；CF-F2-POS-010 崩溃恢复必须有真实断言；CF-F2-NEG-015 / 016 显式重分类为文档级条款并配合同正文文本断言；CF-F2-NEG-017 写成可测形式（例如断言 v1 方法集中无任何方法接受完成声明字段）。fixture schema 新增可判别的 `case-class` 字段区分行为级与文档级。只查 required keys 的检查不计入覆盖。剩余 case 100% 有精确断言，覆盖统计可由一条命令核出。
6. **更正首轮过满表述。** F2C01 已归档 leaf 第 70 行把 external-refs 与 fixture 覆盖写得过满。按既有更正惯例另写更正报告，不悄悄改写已归档 leaf 的历史结论。
7. **证据与记账。** 重跑并记录精确命令、exit code、passed/failed：`cargo check`（888 基线、F2 新增 0）、F2 定向测试、fixture 覆盖统计。叶内分层如实写明：哪些由真实进程证明、哪些只由 cfg(test) 单测覆盖、哪些完全未证明（Stop 与幂等重放之外的真实进程场景、SIGKILL/崩溃恢复、壳侧客户端、真实新壳窗口）。
8. 不动 kickoff“不许动”路径。需要 push、真实凭据、外部网络业务动作或越过不许动路径时，立即停点回总指导。本轮报告不构成 stage-16 关闭。

## 不许动与停点

完整边界以当前用户 kickoff 为准。尤其禁止：为 Jiaoban binding 新增生产安装路径；动 M3 authority 组装；新增创建 `workflow-state.v0.json` 的路径；让桥自行 bootstrap 核心状态文件；push / merge / rebase / tag / 发布；改 `commands.rs`；改 AppState 或其构造器可见性；给既有函数加 pub/pub(crate)；把会调 provider/model 的方法放进 v1；接真实 provider、落凭据、发外部网络业务写；设 `SYN_R4_ACCEPTANCE_PROFILE`；改 stage-15 开闭/候选/已 PASS verdict；处置既有未归属 WIP；reset / stash / clean / 批量 prune worktree；动 17 个 cargo fmt 盲区文件；动 syn-shell；做壳侧客户端或真实窗口取证；改写 ACC-01 第 1–4 条正文或关闭该叶；碰 stage-12、D0C04/D0C05、M1–M5 冻结合同正文与 `manifest.v1.json`；替总指导决定 stage-16 开闭。`/tmp` 下首轮验收/探测载体归 ENG-01 第 4 条，可复用构建产物，不得擅自 broad prune 或删除 Git 注册项。

## 证据边界（收口时填写）

- 真实 `__syn_bridge` 子进程（cwd 无关；二进制 `/tmp/f2-accept-target-629e4b2/debug/codex-governance-workbench`；探针 `/tmp/f2c01r01-probe-1675070`）：
  - 全新空根 `/tmp/f2c01r01-probe-1675070/local.codex.governance.workbench`；
  - `role_session.secretary_status` → exit 过程中 `ok=true` / `F2_OK` / `host=SECRETARY` / `session_state=ACTIVE`；
  - `role_session.global_supervisor_status` → `ok=true` / `F2_OK` / `availability=ready` / `state=ACTIVE` / `scope_kind=GLOBAL` / `read_only=true` / `project_write_capability=false` / `provider_handle_authorizes=false`；
  - `organization.register_stable_member` 首次 → `ok=true` / `F2_OK` / `disposition=REGISTERED` / `receipt.replayed=false`；
  - 同键再调 → `ok=true` / `F2_OK` / `receipt.replayed=true`；
  - `bridge.stop` → `F2_STOP_ACKNOWLEDGED`；进程 exit 0；stderr 空；
  - bootstrap：`m5/orchestration.sqlite`、`conversation/m3-role-session-v1.sqlite3`、`secretary/m4-secretary-v1.sqlite3`、`<root>/m6/organization.sqlite` 均为 true。
  - 完整请求/响应 JSON：`/tmp/f2c01r01-probe-1675070/pairs.json`。
- cfg(test) 定向单测：`cargo test --lib f2c01 --offline -- --test-threads=1` exit 0，17 passed / 0 failed / 0 ignored / 2171 filtered out。覆盖协议错误、idempotency 冲突、路径泄漏、POS-008 精确 case、文档级条款与 app-data-root 末段名。这些不是生产子进程证据。
- 完全未证明：Stop 与同进程幂等重放之外的真实进程场景、SIGKILL/崩溃后替换进程恢复、壳侧客户端、真实新壳窗口。没有 git 提交，因此也没有可点名的新候选 SHA。
- 主工作树最终证据（cwd `prototypes/productized-desktop-shell/src-tauri`，`CARGO_TARGET_DIR=/tmp/f2-accept-target-629e4b2`）：
  - `cargo test --lib f2c01 --offline -- --test-threads=1`：exit 0，17 passed / 0 failed；
  - `cargo check --offline`：exit 0，rustc 汇总 888 warnings，F2 新增 0；
  - `node docs/contracts/fixtures/f2-bridge-001/coverage-audit.cjs`：exit 0，26/26 = 100.0%（23 BEHAVIOR / 3 DOCUMENT）；
  - `rustfmt --edition 2021 --check prototypes/productized-desktop-shell/src-tauri/src/f2_shell_core_bridge.rs`：exit 0；
  - `git diff --check`：exit 0。
- 首轮已通过项保持：`commands.rs` 与 `manifest.v1.json` 零 diff；`origin/main` 仍为 `4791654`；7 个受保护未跟踪载体 hash 仍为 `620faa27…`、`2c576d9b…`、`6cd604b4…`、`147bd08e…`、`6155c26a…`、`7db42ba1…`、`7e51a7ed…`；ACC-01 仍未跟踪，本叶未改其第 1–4 条正文。
- 更正：`docs/harness/reports/F2C01-archived-leaf-coverage-correction-v1.md` 指出已归档 F2C01 第 70 行过满，不改写该历史正文。
