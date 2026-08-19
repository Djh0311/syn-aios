# F2C01R02 四个 v1 空数组约束补执行（syn 核心侧）

阶段：stage-16 F2 壳—核心受控桥（syn 核心侧）

状态：`SUPERVISOR_SELF_REVIEW_PASS / ARCHIVING / F2_CORE_SIDE_EMPTY_ARRAY_ENFORCEMENT / AUTHORIZATION_FILE_CLOSED / NOT_RELEASED`。本叶是 stage-16 第三轮返修，只修整 F2 独立验收的唯一阻断项。不重做、不推翻其余已过验收项。本轮报告不构成 stage-16 关闭。

来源收据：当前用户 2026-08-19 的 “F2C01R02 返修 Kickoff（syn 仓库，stage-16 第三轮，单阻断项）”。该 kickoff 构成本返修的明确开始。`docs/harness/authorization.json` 保持精确 closed 两字段。

## 阻断项

合同已写明 `organization.register_stable_member` 的 `capability_permission_refs` 与 `contact_bindings` 在 v1 必须为 `[]`，但桥对这两个字段零校验；非空值可穿透到 `register_for_state` 被接受并持久化，无稳定错误码。

## 预声明写面

本叶只允许以下路径发生 F2 返修归属变化：

- 修改 `prototypes/productized-desktop-shell/src-tauri/src/f2_shell_core_bridge.rs`
- 修改 `docs/contracts/f2-shell-core-bridge-v1.md`（只改两个错误码单元格与样例注记）
- 修改 `docs/contracts/fixtures/f2-bridge-001/contract-cases-v1.json`（+2 行为级 case）
- 修改 `docs/harness/stages/stage-16.md`
- 新增并在收口时原子归档本 leaf 文件
- 修改 `docs/harness/plan.md`（stage-16 第三轮返修状态表述）
- 修改 `docs/current-state.md`（stage-16 第三轮返修状态表述）
- 追加 `docs/harness/audit/2026-08.jsonl`
- 新增 `docs/harness/reports/F2C01R02-*`（本阻断项真进程 JSON 与四段报告）

不改 `register_for_state` 与任何 `m6_org_*` / `commands.rs` / AppState 可见性；不改 `manifest.v1.json` 与其余冻结合同；不动 stage-15、受保护 WIP、syn-shell。

## 做完的标准

1. 派发 register 前显式校验 v1 四个数组全部为空；非空返回 `F2_FORBIDDEN_AUTHORITY_INPUT`；被拒写不留幂等记录。
2. 合同两行错误列改为该稳定码；不改其余条款。
3. fixture 加两个 BEHAVIOR case，各配独立具名 `#[test]`，测试体含 case id；coverage-audit 更新。
4. `cargo check` 888 基线、F2 新增 0；`f2c01` 定向测试全绿。
5. 真进程：全新空根上两个非空反例各一次得该错误码且未落库；三个成功路径 + 同键重放回归一次。记录完整请求/响应 JSON。
6. 精确路径提交，回报 SHA。本报告不构成关闭。

## 不许动

完整边界以当前用户 kickoff 为准。尤其禁止改 `register_for_state`、m6_org_*、commands.rs、AppState 可见性、manifest 与其余冻结合同、stage-15、受保护 WIP、syn-shell；禁止 push；禁止设 `SYN_R4_ACCEPTANCE_PROFILE`；禁止 provider/model/外部网络。

## 证据边界（收口）

- 修复前真进程（未改码二进制 `/tmp/f2-shell-core-b33038e/debug/codex-governance-workbench` sha256 `39d544ca8dccf0c8c8c6e5101a5e76a790d01d1d6ec560ce72cd48a41f30c351`；探针 `/tmp/f2c01r02-before-1787154585`）：两个非空反例均 `F2_OK` / `REGISTERED`，并写入 `m6/organization.sqlite`（两条 member + 两条幂等收据）。
- 修复后真进程（新建二进制 `/tmp/f2c01r02-target-b33038e/debug/codex-governance-workbench` sha256 `93c50d7fc454190d2ac90ebc7116dae5385a3818a780ab876e2846c8e5efed25`；探针 `/tmp/f2c01r02-after-1787156542`）：两反例均 `F2_FORBIDDEN_AUTHORITY_INPUT`，拒绝后 `organization.sqlite` 仍不存在；随后三方法成功 + 同键重放 `receipt.replayed=true`；库中只有合法成员 `member_f2c01r02_after_legal`。
- cfg(test)：`cargo test --lib f2c01 --offline -- --test-threads=1` exit 0，19 passed / 0 failed。
- `cargo check --offline` exit 0，rustc 汇总 888 warnings，F2 新增 0。
- `node docs/contracts/fixtures/f2-bridge-001/coverage-audit.cjs` exit 0，28/28 = 100.0%（25 BEHAVIOR / 3 DOCUMENT）。
- 完整 JSON：`docs/harness/reports/F2C01R02-before-pairs.json` 与 `docs/harness/reports/F2C01R02-after-pairs.json`。
- 未证明：SIGKILL/崩溃后换进程恢复、壳侧客户端、真实新壳窗口。本报告不构成 stage-16 关闭。
- 未改：`register_for_state`、`m6_org_*`、`commands.rs`、AppState 可见性、`manifest.v1.json`、stage-15、syn-shell。authorization 保持 closed。无 push。
