# S1B-H2-R2 修后 binary 重冻结与真实 App Pending 卡验收 v1

日期：2026-07-22（+0800，用户在场）

状态：**Gate 0/1 通过；Gate 2 按失败矩阵止损；S1B-H2 真实 App 验收未通过。未发送第二句，未落 Pending 卡，未触发 chain。**

任务包：`tasks/2026-07-22-s1b-h2-real-app-refreeze-and-pending-card-verification-package-v1.md`

raw 证据：`evidence/raw/2026-07-22-s1b-h2-r2-real-app/`

## 结论

1. 本次以新鲜 Gate 0 基线重新执行，随后按任务包命令重建并冻结当前裸 debug binary；构建 exit `0`，冻结源未漂移。
2. 用户事后确认首句共发送三次。三次均被持久记录，但没有任何一次进入主管注入/自然回复链：计数从 `recorded/injected/replied = 8/3/3` 变为 `11/3/3`。`+3` 对应用户三次发送，不是产品重复落账；真正的阻断是 injected 与 supervisor reply 均未增长。
3. 这是任务包的明确停机条件。执行者没有自动重发首句；没有发送“按这个出方案”、没有尝试补卡或点击批准；因此不具备继续 Gate 3/4 的前提。
4. proposal/Pending/chain 保持 `74/17/40`，没有 MCP `submit_proposal` handler 成功证据；固定测试项目、DB-primary 健康、registry 与进程卫生均保持安全。

## Gate -1：授权

用户已给出本包唯一现场开工令，并在发现错误启动的 `.app` 附着后，明确授权正常关闭两个实例并从 Gate 0 重启。错误附着发生在任何验收对话前；关闭后以新的真实 store 基线重新开始，未把那次 preflight 计入本次验收回合。

## Gate 0：现场与脏基线

- Workbench、驻留 Codex、Tauri/dev、Vite、cargo-tauri 均无残留；workflow-state、DB、WAL/SHM（存在时）无 holder；registry entries 为 `0`。
- HEAD 为 `e9ad7f3a204a1ebb11ce26c1e8c05b19c04c0991`，staged 集为空。任务包 §3 的七个 SHA-256 均匹配，未触发 `BLOCKED_DIRTY_OVERLAP`；既有脏项未清理、未覆盖。
- SQLite `integrity_check=ok`；DB 与 JSON 的核心投影、proposal、supervisor 与 chain 计数一致。基线为 proposal `74`、Pending `17`、chain `40`，canonical `8/3/3`。
- DB-primary 现场健康为 initialized `35`、degraded `11`。固定测试项目 HEAD 与 manifest 已冻结，详见 raw Gate 0。

## Gate 1：重建与裸 binary 冻结

指定命令：

```text
../tauri-capability-probe/.tauri-cli/bin/cargo-tauri build --debug
```

exit `0`。构建前后七个源码 hash 都未变。实际启动对象是裸 executable：

```text
/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src-tauri/target/debug/codex-governance-workbench
SHA-256 5669ee86e7edcad3273efdf1f8ee8b7e3fcb5307fbc9767d019d7421b51cccbc
size 66503272 bytes; mtime_epoch 1784666565
```

未启动历史 `.app`。启动仅新增一条健康 initialized 审计（`35 -> 36`），degraded 保持 `11`。

## Gate 2：首句与止损

用户事后确认在固定项目的交办页共发送规定首句三次。产品用户面说明消息已送达但主管未完成回复。只读 canonical 账本确认三次第一层记录均持久存在，但第二、三层没有完成：

| 账本计数 | Gate 0 | 止损时 | 增量 |
| --- | ---: | ---: | ---: |
| `supervisor_resident_user_message_recorded` | 8 | 11 | +3 |
| `supervisor_resident_user_message_injected` | 3 | 3 | 0 |
| `supervisor_resident_supervisor_message_recorded` | 3 | 3 | 0 |

`+3` 与用户三次发送相符，并非产品重复落账；但三次均无 injected/reply，仍不满足“一个新鲜首句 → 主管自然回复”的 Gate 2 条件。按合同，执行者未自动重发；未发送第二句、未打开工具批准、未尝试另一工具或 chain。

## Gate 3 / Gate 4

未执行。因为 Gate 2 缺主管回复，第二句没有合法前提。没有 Pending 卡，故也没有 refresh 幂等观察；这不是失败后通过重发补做的替代证明。

## Gate 5：收尾对账

- proposal/Pending/chain 从 Gate 0 到最终始终为 `74/17/40`；dispatches/bindings/attempts/controls 也保持 `404/76/164/164`。
- 无 `submit_proposal` handler acceptance，未生成卡、未批准、未启动 worker。
- 以 immutable 只读查询复核的 DB/JSON 计数仍一致，`integrity_check=ok`；initialized `36`、degraded `11`，无新增降级。正常关闭后常规 SQLite read-only 客户端未能立即重新打开该无 WAL/SHM 的 DB，但文件可读、无 holder，immutable 只读核验完成；未对 store 作任何补写或恢复。
- 固定测试项目 HEAD、porcelain 状态和全文件 manifest 均与 Gate 0 相同。
- 正常关闭裸 App 后，registry entries 为 `0`，相关进程和 holder 均为空。

## Gate 6：后续边界

本次不是 H2 真实通过，也不能用离线或副本店证据补齐。下一步必须先以单独诊断/修复授权解释“canonical recorded 但未 injected/replied”的真实产品路径；用户已确认 `+3` 对应三次发送，不构成额外的重复写入缺陷。在新的任务包和新 Gate 0 前，不应对当前 App 自动重发首句或继续发送第二句。

## 变更面

- 未修改 Rust、TypeScript、配置、schema、审批/沙箱策略、测试项目、真实 store 的直接写入面。
- 未 stage、未 commit。
- 本包新增的仓库文件仅为本 evidence、其 metadata-only raw 证据、`CURRENT.md` 最小状态回写和 catch log EOF 记录。
