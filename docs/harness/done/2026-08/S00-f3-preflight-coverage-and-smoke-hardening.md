# S00 F3 前置：coverage audit 与 Electron smoke 稳定性

阶段：F3 第一批 kickoff 的前置小叶（跨 `syn` / `syn-shell` 两仓）

状态：`DONE / CONSTRUCTION_CLOSED / CODEX_REVIEW_PASS`

来源收据：当前用户 2026-08-20 的 F3 第一批 kickoff 与 S00 前置指令，
receipt `u-e93138631d97a9d16bcd`。用户明确授权 S01-S05 连续推进；本叶只
先处理两个共用前置件。既有 `docs/harness/authorization.json` 保持 closed，
本叶不以它扩大范围。

## 目标

### A. Syn fixture coverage audit

在 `syn` 中修改 `docs/contracts/fixtures/f2-bridge-001/coverage-audit.cjs`：

1. `precise_assertion` 必须解析到 Rust `#[test]` 函数，而不是只做名字包含检查；
2. 该测试函数体必须包含对应 fixture `case id` 字符串；
3. 输出每个 `case id -> function` 映射以及 precise assertion 的多对一统计。

不放宽审计器。现有 28 个 fixture case 中缺失的测试体字符串补到
`prototypes/productized-desktop-shell/src-tauri/src/f2_shell_core_bridge.rs`，
不得改变行为或把 case 合并成新的测试替身。审计命令必须 exit 0。

### B. Syn-shell isolated Electron smoke

在 `syn-shell/scripts/f2-isolated-electron-smoke.mjs` 中为 `waitForPage` 使用的
CDP `fetch` 增加显式请求超时与有限重试；耗尽或其它失败路径必须写 failure
receipt，然后以非零状态退出。保留现有两代 smoke 语义，不改兼容标识、桥合同、
客户端事实边界或 renderer 进程职责。修后使用现有构建完整跑一次 smoke，保留
两代窗口、NDJSON/launcher receipt 与失败/成功状态证据。

## 允许写面

产品实现者 Grok 只可写：

- `syn`: `docs/contracts/fixtures/f2-bridge-001/coverage-audit.cjs`；
  `prototypes/productized-desktop-shell/src-tauri/src/f2_shell_core_bridge.rs`
- `syn-shell`: `scripts/f2-isolated-electron-smoke.mjs`

Codex 负责本叶生命周期文件、复核记录和最终报告；生成的 smoke 证据可落在
`syn-shell/docs/harness/reports/F3S00-*` 或既有隔离证据目录，但不得把它写成
真实账号、发布或完整 F3 验收。

## 完成标准

1. coverage audit 逐条证明 28/28：`precise_assertion` 是 `#[test]` 函数、函数体
   含 case id；stdout 有完整 case→函数映射与多对一统计；命令 exit 0。
2. Syn F2 定向测试与必要 Rust 检查全绿；既有行为和合同正文不变。
3. `waitForPage` 的 fetch 有显式 timeout、重试上限和可判别失败 receipt；正常
   两代 smoke 不因新超时逻辑改变成功路径。
4. 现有构建上的完整 smoke 成功跑完，证据包含两代截图、launcher receipt、
   NDJSON/identity 对比和脚本 exit 0；证据载体明确为 isolated/local。
5. A、B 各形成一个不使用 `--no-verify` 的完整钩子提交；提交只含各自允许写面，
   不吸收既有 dirty WIP。

## 禁止与路由

- 不动 `commands.rs`、AppState 可见性、M1-M6 冻结正文/manifest、既有 F2 合同
  语义、`src/supervisor`、受保护 `m6_*.rs`、stage-12、ACC-01、OSS-01、真实
  provider/model/凭据、外部业务写、push/merge/rebase/release。
- 不做 S01-S05 的 UI/桥扩面实现；S01 独立验收与 S05 独立验收停点保持。
- S02 动合同时必须顺带加入 CLI basename 预拦：错误码
  `F2_CLI_APP_DATA_ROOT_IDENTITY`、合同 CLI 表对应行与 1 个具名 fixture case；
  这属于 S02，不另立叶，本叶不提前实现。

## 角色

Grok 负责上述产品实现、定向测试、现有构建 smoke 与两个内容提交；Codex 已独立
检查 staged diff、提交钩子、测试计数、receipt 与范围，A/B 均通过并归档。产品
载体：syn `7e009d8`，syn-shell `c6aae130`；coverage audit 退出 0，现有完整
两代 isolated smoke 退出 0。S01-S05 后续片已在各自叶内完成，不改变本叶范围。
