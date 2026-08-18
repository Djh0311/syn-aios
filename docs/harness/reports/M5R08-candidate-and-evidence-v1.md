# M5R08 内容候选与证据报告 v1

日期：2026-08-18

状态：`CANDIDATE_READY / AWAITING_INDEPENDENT_ACCEPTANCE / NOT_CLOSEOUT / NOT_M5_COMPLETE`

## Harness

- 唯一 current leaf 仍是 `M5R08-m1-consumption-runtime-idempotency-and-acceptance-debts`；本报告不归档 leaf、不关闭 stage-14、不激活 M6、stage-15 或 syn-shell。
- M5R07 已接受载体完整序列：产品 `ab5c46e` → `7cab37203fe70fe69f696e45fc6a12b314d1fd84`，记账 `0b7b5e1` → `a85278a`。本叶不反写或扩大该 scoped PASS。
- M5R08 生命周期入口：`a7bd4914ea1a611304794ed258e750c88ae8a819`；随后按 Grok 单写者窄包串行形成：
  - `311114c` 任务包 → `4f7b153` M1 canonical memory/mature governance 消费；
  - `34d0b4f` 任务包 → `6ad0e5c` attempt/grant-scoped runtime carrier；
  - `7a079be` 任务包 → `75019e1` ordinary identity source 同句柄读取；
  - `e1a25cc` 任务包 → `30f1c7b` M5R07 acceptance driver 默认 bundle gate；
  - `cf0ad36` seed/下游/WIP 文档；
  - `720a6f2` 红灯返修任务包 → `09e9b32` production-prefix 静态守卫修复。
- 最终内容候选：`09e9b323c26046b750209424aa7aca77e9c7aadb`，tree `657f7db696d3004eb3f6c5921e365df468ce617a`。本报告所在最终记账提交的 SHA/tree 因 Git 对提交内容的自指限制不嵌入自身；仓外节点请求必须精确列出并作为最终载体绑定。
- `authorization.json` 在本轮始终保持精确 closed 两字段。节点请求写入前仍须再次逐字节核对。

## 产品

1. 六个已注册 memory/mature governance command 在读写前经 AppState 安装的 M1 read port `resolve_exact_alias` 取得 canonical ProjectId；无生产 path-derived fallback。旧 top-level path-derived id 只在 exact legacy 边界迁移，foreign id fail-closed，nested legacy carrier 保留兼容读取。
2. 普通 runtime 的 workcell、durable operation 与 receipt 由 admitted attempt/grant 派生。同一项目两个合法 attempt 的 workcell/operation/receipt/effect 均不同，旧 lineage 不改写；同一 attempt/effect 的持久化重入在 adapter 前以 `duplicate_effect` 拒绝，零第二 effect。
3. 任一 scoped M5 candidate 节点前跑完整 `m5_` 是 leaf 级候选流程规则；这里不声称此前每一个产品任务包都实际包含或执行了该矩阵。本候选实际执行 188/188；前驱 `cf0ad36` 的 187/188 红灯没有被隐去，并由窄返修后全量重跑关闭。
4. M5R07 acceptance driver 只有 `VITE_SYN_M5R07_ACCEPTANCE_DRIVER` 精确为 `1` 才进入 production bundle；默认 build 无 `m5r07_`、`m5r07Ordinary`、`m5r07Isolated`、`syn-m5r07` 标记。后端 `status.active` / `status.isolated` gate 保留。
5. `try_new_with_tauri_app_data_root` 的 tasks seed 已发生路径修正，已补记进 `docs/contracts/m5-r07-product-path-correction-v1.md`；修正事实绑定原提交 `99a5afc`，不宣称由 M5R08 新实现。
6. ordinary identity source 以 `O_NOFOLLOW` 打开最终组件，regular-file metadata 与 bytes read 绑定同一已打开 handle；symlink fail-closed，路径替换后仍消费原 handle bytes。证据边界不扩大为 parent-component 或 in-place mutation 全面防御。
7. `handoffs/2026-08-18-syn-m5-to-m6-and-shell-deferred-debts-v1.md` 已记录新壳 F3 不继承 acceptance driver、F5 承担真实窗口像素证据；本轮没有进入 syn-shell，也没有像素证据。
8. verdict 的 34 项 WIP 已在 `M5R08-protected-wip-attribution-v1.md` 逐项记录 path/status/hash/source-semantic/disposition。23 modified + 11 untracked 均原位保全并排除于候选；6 个 `m6_*.rs` 继续未跟踪。当前第 35 项 `.turns/` 是本轮新生成的 Harness runtime 载体，已单列而未伪装成 opening WIP。

## 证据

最终 evidence 根：`/home/synadmin/workspace/.syn-gates/evidence/M5R08-09e9b32/`

所有命令在 `09e9b32` detached disposable checkout 上执行；原始 stdout/stderr 与每项 `.exit` 分开保留，`commands.txt`、`exit-summary.txt`、`sha256sum.txt` 和 `summary.md` 提供索引。

| 命令 | 结果 | exit |
|---|---|---:|
| `cargo check --lib --offline` | finished dev；保留既有 warning debt | 0 |
| `cargo test --lib m5r08_m1_ --offline -- --test-threads=1` | 21 passed / 0 failed | 0 |
| `cargo test --lib memory_entity_relation_ --offline -- --test-threads=1` | 11 / 0 | 0 |
| `cargo test --lib mature_pattern_ --offline -- --test-threads=1` | 11 / 0 | 0 |
| `cargo test --lib m5r08_runtime_ --offline -- --test-threads=1` | 8 / 0 | 0 |
| `cargo test --lib m1_ordinary_identity_source_ --offline -- --test-threads=1` | 5 / 0 | 0 |
| `cargo test --lib m5_runner_entry_registry::tests --offline -- --test-threads=1` | 10 / 0 | 0 |
| `cargo test --lib --offline m5_ -- --test-threads=1` | 188 / 0，1873 filtered | 0 |
| `npm run typecheck` | passed | 0 |
| 默认 `npm run build` | passed；既有 >500 kB chunk warning | 0 |
| 默认 bundle marker `rg -l` | 无匹配 | 1（预期） |
| `VITE_SYN_M5R07_ACCEPTANCE_DRIVER=1 npm run build` | passed | 0 |
| 显式 bundle marker `rg -l` | 命中 acceptance asset | 0（预期） |
| `git diff --check a7bd491..09e9b32` | 无输出 | 0 |
| `git diff --name-status a7bd491..09e9b32` | 仅 leaf 允许产品/任务/合同补记/报告/交接路径 | 0 |

前驱红灯：`/home/synadmin/workspace/.syn-gates/evidence/M5R08-cf0ad36/07-full-m5.log` 为 187 passed / 1 failed、exit 101；失败是旧 guard 对 `m5_product_commands.rs` 全文件误扫 `#[cfg(test)]` 的 `run_authorized_workcell`。`09e9b32` 将三个 bypass 断言改为扫描既有 `production_prefix(product)`，未改 runtime 行为；最终完整矩阵 188/188。

detached worktree 验证前状态为空。验证后只有离线依赖 `node_modules` symlink 与 Tauri build 生成的 untracked `linux-schema.json`；二者不是候选内容，也不作为 clean-tree 证据。

## 载体

- 产品/内容候选：`09e9b323c26046b750209424aa7aca77e9c7aadb` / tree `657f7db696d3004eb3f6c5921e365df468ce617a`。
- 最终记账载体：由包含本报告、leaf/plan/stage/current-state/audit 更新的提交形成，精确 SHA/tree 写入仓外节点请求。
- 证据载体：`/home/synadmin/workspace/.syn-gates/evidence/M5R08-09e9b32/`；前驱失败证据保留在 `M5R08-cf0ad36/`。
- 证据只到 detached checkout 的 Rust/TypeScript/build/静态 bundle 与合成测试；不是 GUI/Tauri 进程、真实窗口、真实项目/个人资料、真实 provider/账号/凭据、外部业务写、部署或发布。
