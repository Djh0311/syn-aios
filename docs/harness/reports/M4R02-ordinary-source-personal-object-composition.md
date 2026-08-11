# M4R02 普通产品来源与个人对象组合验收报告

日期：2026-08-11

阶段：`stage-07`

任务包：`M4R02`

## 1. 结论

M4R02 已通过。WorkItem 与 project consultation proposal（项目咨询方案）两个真实内部 owner 都从既有普通产品命令进入各自原子 UoW，经同库脱敏 publication/outbox、生产 dispatcher、注册 mapper 和 M4 原子 ingress 形成闭环；fixture 没有直调 adapter、dispatcher 或 repository 写入口。

PersonalAction、Reminder、来源型 Notification 与 typed Decision 均已有普通产品组合入口、持久生命周期、幂等 receipt/event/audit、重启恢复与 owner 非反写证据。全新隔离 profile 的三次真实 debug App 启动最终得到 `PASS`，证据等级为 `ISOLATED_PRODUCT_APP`；这仍不等于真实个人资料、真实 provider 或长期日常试点。

## 2. 普通产品调用链

### WorkItem 主来源

```text
普通 renderer/tauri.ts
  -> update_work_item_state（registered command）
  -> server-sealed client_request_ref identity + authoritative revision
  -> M2 WorkItem immediate UoW
  -> state/receipt/event/audit/snapshot + M4 source publication 同事务
  -> production owner-outbox dispatcher
  -> RegisteredWorkItemSourceOwnerMapper
  -> M4 registered-source ingress
  -> provenance + Inbox/OpenLoop + delivered Notification 同事务
```

`source_revision` 来自 owner receipt/event 的精确 revision，native watermark 在 owner UoW 内与 event id 复核后再分域 seal；不同 aggregate 不用 revision 充当全局 cursor，消费按 publication sequence、每 adapter checkpoint 和 CAS lease 推进。同一 command/client ref 的 exact replay 返回同 receipt，owner outbox 与 M4 effect 都零新增。

### proposal typed Decision

普通 `create_project_consultation_proposal`、`record_project_consultation_proposal_decision` 与 server-clock expiry 由 proposal owner 在同一 immediate UoW 写 owner fact、audit 和 publication。新 proposal id 为固定长度 opaque SHA-256 identity，不把项目路径、标题、目标或敏感词带进 outbox。同 scope 新方案会按稳定顺序把旧 Draft/Pending 标为 Superseded，并以 event-local revision 形成 WITHDRAWN；普通 owner 到 M4 的真实测试覆盖 OPEN、ANSWERED、EXPIRED、WITHDRAWN，M4 本地 read/dismiss 不改 owner tuple、hash 或 revision。

## 3. 存储、恢复与失败边界

- 普通 Tauri app-data root 通过同一个 `ProductDataPaths` resolver 得到隔离的 index、tasks、workflow、owner DB、M3 和 M4 路径；隔离 profile 只替换基础 root，不安装历史 acceptance AppState。
- JSON -> DB-primary 冷迁移在共同 workflow lock 内完成 confirmed import、两次 reconcile、DB create-new 发布，最后才发布不可变 storage config；旧 JsonOnly writer 在取得同锁后重查 config 并 fail closed。
- owner publication 具备 exact overlay catalog、global sequence、per-adapter checkpoint、claim lease、retry/attempt、durable quarantine 与 candidate rejection。M4 commit 后、owner checkpoint 前重放由 M4 event/hash 去重并只推进一次 checkpoint。
- same event / different payload、native provenance 漂移、敏感或路径型 candidate 均 fail closed；candidate rejection 只保存 seal/hash/机械原因，不保存原始正文或路径。
- PersonalAction 只能由显式用户命令创建；Reminder 只能绑定已授权本地 owner；Notification 由来源事件创建，不开放 renderer create；Decision owner 状态和本地可见状态保持双轴。

## 4. 隔离 App 行为回执

可携带 composite receipt 保存于：

- `docs/harness/reports/M4R02-source-and-personal-object-behavior-receipt.json`
- SHA-256：`a1d8bf4b5bff0b71bd7f95955b5b3a4ca5cc1358817c3db81fecbc231a3dd392`

关键直接事实：

- 三个互异 App `process_id` 的绑定 receipt 分别完成 initialize / mutate / readback，均为 `PASS`；三次 LaunchServices waiter 均 `exit_code=0`、`timed_out=false`、`signal=null`。三次使用同一 profile fingerprint，nonce 与 App `process_id_sha256` 均互异。
- `ordinary_composition=true`、`acceptance_wrapper_calls=0`、`direct_repository_seed_calls=0`；普通 registry、普通 AppState 和 production outbox tail 均由 receipt 精确声明并由 runner/源码静态门交叉校验。
- WorkItem owner publication 恰好 1 条，terminal=`DELIVERED`；M4 admitted=1、Notification=1；checkpoint sequence=1、status=`CAUGHT_UP`。
- duplicate replay：同 receipt，owner outbox delta=0、M4 effect delta=0。
- PersonalAction create/replay 与 Reminder create/replay 各自只有 1 receipt + 1 event；Notification source delivery 后 READ、DISMISS 的 receipt/event 都恰好一条，revision 精确为 `2 -> 3 -> 4`。
- 重启 readback：subject outbox delta=0、M4 effect delta=0；PersonalAction、Reminder、DISMISSED Notification 和 owner invariant 均与 mutate launch 一致。
- owner local-action 前后 tuple SHA-256 完全相同，source revision 均为 `2`。
- `isolation_boundary` 五项 receipt 字段均为 0，且只读原始 M4 DB 的 model invocation 为 0；这些字段与静态调用链共同限定本次隔离运行，不是 OS 级全网流量审计。`environment_unchanged=true` 只表示 launcher 进程的 `HOME` 与 `CODEX_HOME` 前后未变。

首个真实运行曾在 Home 读取处快速失败。只读核证定位为普通 M4 adapter 把 M3 sealed binding 误当 typed identity，随后 Home unavailable 使用中文展示句而前端只接受机械 code。修正后，普通 adapter 先逐字段校验 M3 sealed binding，再投影固定 M4 role/scope/object/DAILY；Home 使用 ASCII machine code，Explain 中文提示保持不变。失败 profile 保留为诊断证据，没有改写成 PASS。

## 5. 验证

本包最新运行并通过：

- `node scripts/run-m4-remediation-probes.mjs --only=source --expect=green`：12/12 static markers GREEN；六份冻结合同 SHA-256 exact。该探针只证明 marker，不替代行为证据。
- `RUSTFLAGS=-Awarnings cargo test --lib --offline m4r02_ -- --nocapture`：18/18。
- `RUSTFLAGS=-Awarnings cargo check --tests --offline`：exit 0。
- `RUSTFLAGS=-Awarnings cargo test --lib --offline -- --skip s1b_h2_real_initial_and_resume_consume_only_the_private_submit_proposal_config`：exit 0；唯一 skip 是受限环境下读取 `ps -o lstart` 的既有主机权限差异，不属于 M4R02 产品链。
- `npm run typecheck`：exit 0。
- `npm run test:offline-interaction`：15/15 entrypoint groups 通过，包含 M4C06/C08/C09 与 M4R02。
- `node --check scripts/run-r4-isolated-app-preflight.mjs`：exit 0。
- `node scripts/run-r4-isolated-app-preflight.mjs --m4r02-ordinary-composition`：实际 debug App 三启动 `PASS`。
- `git diff --check`：exit 0。

独立审查在行为运行前后均未发现 blocker、P1 或 P2；实际 PASS receipt 另由独立任务逐字段和只读 DB 复核。

## 6. 证据上限与下一入口

本包只证明普通内部 owner、个人对象及隔离 App composition。它没有实现或验收 M4R03 到期时钟、M4R04 精确回源、M4R05 持续对话、M4R06 五类旧读面或 M4R07 最终总验收；也没有进入真实资料、真实模型/provider、真实消息、账号/凭据/connector、外部网络写入、远端、发布或 M5–M10。

下一唯一入口是 `M4R03`。本包完成后只激活该 leaf，不提前施工 M4R04–M4R07。
