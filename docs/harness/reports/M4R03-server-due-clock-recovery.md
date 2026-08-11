# M4R03 服务端到期时钟与恢复验收报告

日期：2026-08-11

阶段：`stage-07`

任务包：`M4R03`

## 1. 结论

M4R03 已通过。普通产品 AppState 在启动时同步执行 `StartupRecovery`，随后由同一个 production scheduler helper 每 60 秒执行 `TimerTick`；两条路径都以一次 server-now capture 驱动 snoozed OpenLoop 与 Reminder 的原子到期批次。renderer 只提交普通 snooze/create 用户意图和未来 schedule marker，不拥有 clock/fire/SERVER_CLOCK authority。

全新隔离 profile 的真实 debug App 组合验收完成了到期前强退、到期后重启恢复、真实 TimerTick、再次重启零增量，最终 composite 为 `PASS`、证据等级为 `ISOLATED_PRODUCT_APP`。该证据仍不等于真实个人资料、真实 provider 或长期日常运行。

## 2. 生产调用链与事务边界

```text
普通 isolated product AppState
  -> start_m4_secretary_scheduler
  -> 同步 StartupRecovery
  -> 同一 production cycle helper
  -> repository daily scheduler cycle
  -> Active scheduler + 单次 captured server-now
  -> stable due candidate batch
  -> OpenLoop CLOCK / Reminder FIRE
  -> state + receipt + event + SERVER_CLOCK audit 同一 IMMEDIATE transaction
  -> 有状态变化时同事务机械重投影 brief 一次

同一 helper
  -> 60s production worker
  -> TimerTick
  -> 同一 due batch
```

due candidate 以纳秒级 RFC3339 UTC key 比较，稳定按 due instant、aggregate kind 和 aggregate id 排序。OpenLoop 只接受 `SNOOZED && snoozed_until <= now`；Reminder 只接受 `SCHEDULED && scheduled_for <= now` 或 `SNOOZED && snoozed_until <= now`。内部 idempotency scope 与普通用户 scope 分离，key 绑定 aggregate、revision 和 exact due marker；同批任一写入失败会整批回滚。

Disabled/无效 timezone 只更新 degraded scheduler checkpoint，不推进 due 对象。普通 user command 不能占用 server-clock 内部 idempotency namespace；renderer 也没有 `OPEN_LOOP_CLOCK`、`REMINDER_FIRE` 或 `SERVER_CLOCK` 命令面。

## 3. 真实隔离 App 行为

可携带 composite receipt 保存于：

- `docs/harness/reports/M4R03-server-due-clock-behavior-receipt.json`
- SHA-256：`b62b421ffed110e137f340c4f5298a63c3c5f5724a17e5c9bf194ba110230d65`

fresh profile：`syn-r4-acceptance-IWE6rr`。profile 内 composite 与可携带 receipt 逐字相同；三份 phase receipt SHA-256 分别为：

- arm：`4f2a150f45289c950987b9217cf11f20328e1ffb3a6b96109d9ba7a512d2827f`
- recovery_timer：`25111cd80080b01653515f157124a42101cf52962c71c67b60b11a133f515c20`
- repeat：`73f550ea7d14780319f3ff0b3b4cbc944d135e7884b389ada273906cfea47462`

关键直接事实：

- `ordinary_composition=true`、`acceptance_wrapper_calls=0`、`direct_repository_seed_calls=0`、`direct_transition_calls=0`；三次 App process、nonce 与 phase receipt 均互异并由同一 profile fingerprint 和前序 receipt SHA 链精确绑定。
- arm 只经普通 command registry 执行一次 `OPEN_LOOP_SNOOZE` 与一次 `REMINDER_CREATE`，写命令数为 2；OpenLoop 为 `SNOOZED/rev2`，Reminder 为 `SCHEDULED/rev1`。
- arm receipt 可见后，runner 对直接启动的真实 bundle PID 请求并确认 `SIGKILL`；请求时间 `05:44:38.188Z`、关闭确认 `05:44:38.193Z`，均早于 due marker `05:45:23.072Z`。
- 同 profile 重启后，constructor 的 `StartupRecovery` 在 `05:45:24.691Z` 同批推进两对象：OpenLoop `OPEN/rev3`、Reminder `FIRED/rev2`；receipt/event/SERVER_CLOCK audit 各 2，两个 key、一个 batch timestamp。
- 同一 recovery launch 再以两条普通 snooze 命令把两对象设为 `SNOOZED/rev4`、`SNOOZED/rev3`，timer due marker 为 `05:45:55.205Z`。
- 真实 production `TimerTick` 在 `05:46:24.716Z` 同批再次推进：OpenLoop `OPEN/rev5`、Reminder `FIRED/rev4`；累计 receipt/event/SERVER_CLOCK audit 各 4、四个不同 due key、两个 batch timestamp。该次两条 due receipt 的 `recorded_at_utc` 与同一 `TimerFired.occurred_at_utc` 精确相等。
- 第三次 repeat 启动读取到完全相同的对象、revision、receipt/event/audit 计数，`repeat_zero_delta=true`、`evidence_exact_match=true`。
- 原始 M4 SQLite `PRAGMA integrity_check=ok`、foreign-key violation=0；model invocation=0、source-owner writeback=0。receipt 的 real model/provider/message、external connector/network write 五项也均为 0；这些字段与静态调用链共同限定本次隔离运行，不是 OS 级全网流量审计。

首个 actual-App 运行在 arm 阶段诚实返回 `REJECTED`。只读现场显示两条普通命令均已成功，OpenLoop snooze receipt 为 `APPLIED`，Reminder create receipt 为正式的 `CREATED`；renderer 错把后者要求为 `APPLIED`，因此自身拒绝。修正为 arm 精确绑定 `APPLIED/CREATED`、timer re-snooze 精确绑定 `APPLIED/APPLIED` 后，使用全新 profile 完整重跑得到上述 PASS；失败 profile 没有被改写成 PASS。

## 4. 并发、回滚与反例

repository 行为测试直接覆盖：

- 到期前 tick 零变化、到期后 StartupRecovery 单次推进；
- 同时到期的 snoozed OpenLoop 与 scheduled/snoozed Reminder 使用同一 captured now；
- 批内 failpoint 使 state/receipt/event/audit/brief 全部回滚，重试只提交一次；
- 两线程并发 TimerTick 由 IMMEDIATE transaction/CAS 串行，总计只产生一组 due evidence；
- scheduler 与 stale user CAS 的两种先后顺序都只允许一方成功且不漏不重；
- Reminder 第二次 snooze 使用新 due marker 只再 fire 一次；
- Disabled/无效 timezone 不推进，恢复 Active 后补偿一次；
- 用户预占相似 key 不影响内部 server-clock scope。

## 5. 验证

本包最新运行并通过：

- 改动前基线 `ac53281f5b806d9d36f9f765a4bbe6ca6a45ada9` 上执行 `node scripts/run-m4-remediation-probes.mjs --only=clock --expect=red`：exit 0、状态 RED；5 个 marker 中只有 scheduler entry 存在，due batch、production caller 和两类 SERVER_CLOCK reason 均缺失。
- `node scripts/run-m4-remediation-probes.mjs --only=clock --expect=green`：5/5 static markers GREEN；六份冻结合同 SHA-256 exact。该探针只证明 marker，不替代行为证据。
- `RUSTFLAGS=-Awarnings cargo test --lib --offline m4r03_ -- --nocapture`：7/7。
- `RUSTFLAGS=-Awarnings cargo test --lib --offline m4c04_ -- --nocapture`：21/21。
- `RUSTFLAGS=-Awarnings cargo test --lib --offline m4c07_ -- --nocapture`：24/24。
- `RUSTFLAGS=-Awarnings cargo check --tests --offline`：exit 0。
- `npm run typecheck`：exit 0。
- `npm run test:offline-interaction`：15/15 entrypoint groups 通过，包含 M4C06/C07/C09、M4R02 与 M4R03 静态组合门。
- `node --check scripts/run-r4-isolated-app-preflight.mjs`：exit 0。
- `node scripts/run-r4-isolated-app-preflight.mjs --m4r03-server-clock`：实际 debug App 三阶段 `PASS`。
- `git diff --check`：exit 0。

代码冻结前的独立静态复审为 0 blocker / 0 P1 / 0 P2；actual PASS 后另由独立任务逐字段核验 composite、phase receipt SHA 链和只读 SQLite。

## 6. 证据上限与下一入口

本包只证明普通产品 scheduler 的到期时钟、StartupRecovery、TimerTick 和重启幂等。它没有实现或验收 M4R04 精确回源、M4R05 持续对话、M4R06 五类旧读面或 M4R07 最终总验收；也没有进入真实资料、真实模型/provider、真实消息、账号/凭据/connector、外部网络写入、远端、发布或 M5–M10。

下一唯一入口是 `M4R04`。本包完成后只激活该 leaf，不提前施工 M4R05–M4R07。
