# S1B-H2 自然信息流离线验证 v1

日期：2026-07-19

状态：**代码与离线闸通过；真实 Tauri App 已启动并发送指定第一句，但 canonical 用户消息未记录，真实验收未通过。已止损：未发送第二句、未重试、未点卡。**

任务包：`tasks/2026-07-19-s1b-h2-supervisor-syn-natural-information-flow-package-v1.md`

案发预登记：`evidence/2026-07-19-s1b-h2-real-app-message-to-proposal-failure-preregistration-v1.md`

## 已验证的产品合同

1. resident 私有配置在 initial 与 resume 均只声明 `supervisor_orchestrator.submit_proposal`，且只对该工具使用 `approval_mode="approve"`；实际生产 argv 均带唯一 `--strict-config`，未知 config 字段 fail-closed，`--sandbox read-only` 未变。离线 fake-Codex 直接读取实际 `$CODEX_HOME/config.toml` 取证；既有共享配置不获该能力。
2. one-shot 将可恢复 `error` 与硬失败分开：`error → final → turn.completed → exit 0` 保留对话；`turn.failed`、非零退出、缺完成、缺真实 `thread.started` 仍关闭。invalid-resume 仍只轮转一次后 initial 重建。
3. canonical 用户消息、主管答复、proposal tool outcome、Pending 卡是可分别结算的事实。工具失败不会吞已完成对话；普通读模型只获得人话摘要，原始工具参数和运行诊断留在既有私有审计。审计写入本身失败时也只返回稳定人话，不把路径或底层错误上脸。
4. 前端只在 `message_not_recorded` 时显示“这句没送到主管”。已记录但主管未完成、主管完成但卡失败、仅刷新失败、transport 状态未知均有独立文案；前端不乐观伪造消息或卡。
5. 幂等键是服务端派生的 `sha256(project_id:workflow_id:canonical_message_id)`；UI `client_request_id` 只关联 transport 重试，不进入 MCP 或 card key。同一 turn 双工具调用及看门狗技术重试复用一张卡，之后用户新发的 canonical 消息可创建另一张卡。

## 离线测试结果

| 闸 | 命令 / 证据 | 结果 |
|---|---|---|
| H2 定向 | `cargo test --lib s1b_h2_ -- --nocapture` | 11 passed / 0 failed |
| S1B 聚合 | `cargo test --lib s1b_ -- --nocapture` | 27 passed / 0 failed / 1 ignored |
| S1 聚合 | `cargo test --lib s1_ -- --nocapture` | 11 passed / 0 failed / 1 ignored |
| M5B | `cargo test --lib m5b_ -- --nocapture` | 9 passed / 0 failed |
| M5C | `cargo test --lib m5c_ -- --nocapture` | 5 passed / 0 failed |
| 全库 | `cargo test --lib -q` | 1021 passed / 0 failed / 44 ignored |
| 编译 | `cargo check --offline -q` | passed（历史 warnings 未作本包处理） |
| 前端类型 | `pnpm typecheck` | passed |
| 前端离线交互 | `pnpm test:offline-interaction` | passed（15 组入口套件） |
| 格式 / diff | H2 Rust `rustfmt --edition 2021 --check`；scope `git diff --check` | passed |
| shape baseline | `node scripts/harness/workbench-shape-gate.js --mode baseline --json` | 13 errors / 5 warnings / 5 infos，pass |
| shape check | `node scripts/harness/workbench-shape-gate.js --mode check --json` | 同为 13 / 5 / 5；历史债务按设计 exit 1，无 H2 净增 |

H2 定向包含真实错误原文夹具（`no rollout found` / `(code -32600)`）的宽分类、回合内重复落卡幂等、transport 回包丢失重试，以及审计写失败时稳定错误门面；原始文本不进入用户可见投影。

## 真实 App 首句复核（未通过，已止损）

- 使用当前源码构建的 App 打开固定测试项目后，按任务包只发送第一句：`我想给这个游戏里的标题改成小马里奥`。UI 返回：`这句没送到主管——稍后再试一次。`
- 随即停止：未发送第二句 `按这个出方案`，未重试首句，未点击任何既有或新增卡，也未启动 chain。
- 只读对账未见本轮 `supervisor_resident_user_message_recorded`、主管回合或 MCP handler 到达证据；proposal 总数保持 `74 → 74`、Pending 保持 `17 → 17`、chain 保持 `40 → 40`。
- 固定业务项目没有改写；仅做 SHA-256 与 `git status` 只读对账，前后相同。进程登记表仍为空；系统中原有的 P1-A 进程作为基线保留，未声称全机无进程。
- 现场有既存的 DB-primary 对账冻结。代码与时序共同支持的高置信根因是：该进程的首个 blocked 写先落一笔降级审计并递增 workflow revision，随后同一调用中已按旧 revision 构造的 canonical 用户事件触发 CAS 冲突；入口回读不到 canonical 后，按合同返回 `message_not_recorded`。原始写入 `Result` 在该入口被收口，故这不是“已抓到原始错误串”的结论。
- 该冲突位于既有 M5 storage-mode fallback，H2 不修改存储模式、M5 对账或其 fallback。需在得到单独授权后修复或恢复该现场，再从两句完整流程重新验收。

## 明确未验证

- 未验证真实 Codex 对产品私有配置的实际消费、handler 到达、同 thread 回路、真实 Pending 卡或 live 刷新幂等。
- 未验证首句之后的自然答复，也未验证第二句“出方案”；二者均因首个 canonical 事实未写入而未触发。
- 没有批准任何卡、没有启动 chain、没有派 worker，也没有改写 `/Users/yoyi/codex-workflow-mario-test` 业务文件。
- 获得单独的 M5 现场修复 / 恢复授权后，真实 App 必须由用户在场从两句完整流程复跑；到一张目标 `PendingUserConfirmation` 卡、chain 与项目文件不动即停，绝不点击批准。
