# M4R04 注册 owner 精确回源验收报告

日期：2026-08-11

阶段：`stage-07`

任务包：`M4R04`

## 1. 结论

M4R04 已通过。renderer 只把 server-minted `source-route:sha256:*` 作为唯一请求字段交给服务端 closed owner registry；renderer 不提交 owner、object type、canonical id、revision、项目路径、URL 或 callback 作为路由权威。registry 先以 M4 current/provenance/ingestion 精确确定 owner-native identity，再按已注册 WorkItem 或 consultation Proposal owner 只读重建当前目标，最后只返回两种有限 typed navigation target。

全新隔离 profile 的真实 debug App 组合验收完成了 WorkItem、Proposal、重启后当前 WorkItem 的真实 DOM route 点击与目标页消费，并覆盖旧 WorkItem capability 的 `STALE` 与 route tamper 失败。最终 composite 为 `PASS`、证据等级为 `ISOLATED_PRODUCT_APP`。该证据仍不等于真实个人资料、真实 provider 或 OS 级网络审计。

## 2. 生产调用链与权限边界

```text
Secretary Home / Board 的 server-minted route action
  -> 普通 tauri.ts resolveSecretarySourceRoute({ source_route_ref })
  -> registered Tauri command
  -> AppState.m4_source_route_registry
  -> M4 current + provenance + ADMITTED ingestion receipt
  -> closed owner/type registry
  -> owner publication triple (adapter, sequence, publication id)
  -> DELIVERED + M4_INGESTION terminal receipt
  -> owner DB current record/full envelope rebuild
  -> finite target:
       WORK_ITEM(project_id, workflow_id, work_item_id, revision)
       CONSULTATION_PROPOSAL(project_id, workflow_id, proposal_id, revision)
  -> renderer 二次 exact resolve 包围普通 owner read cut
  -> Projects/Task 或 Projects/Jiaoban 精确记录消费
  -> CONSUMED / fixed FAILED outcome
```

route seal 绑定 `source_owner_ref`、owner-native `source_object_type`、canonical source id、source revision 与 native scope seal。Proposal 的 owner-native type 来自 M4 provenance（`proposal_decision`），不会误用 Home 展示层统一的 `workflow_attention`。owner publication 只按 provenance 三元读取，不按 route ref 全库猜测；必须为 `DELIVERED`，terminal kind 必须为 `M4_INGESTION`，terminal receipt 必须与 M4 ingestion receipt 相同。

成功 response 固定为 7 个顶层字段，target 只含 typed ids 和 canonical decimal revision，不含 project root、文件路径、URL、view name 或 callback。unknown owner/type、scope mismatch、revision mismatch、stale、tamper、missing target 与 terminal integrity failure 都返回固定 machine code；失败不导航，也不写成功 notice 或消费 marker。

前端先 resolve sealed route，再读取完整 page/workflow/proposal owner read cut，随后以同一 capability 二次 resolve；两次 response 必须逐字段 exact，才发布 attempt-bound focus。旧的全局异步 reload 不能覆盖该 read cut。显式刷新或 owner action 只在 phase/ref/attempt 全部匹配的 `CONSUMED` 状态释放 read cut；Proposal 若来源页本身是 Projects，则回 Home，避免清 focus 后挂载完整 Jiaoban Browser 及其 provider/preview effects。非 rollout Proposal 的 source focus 在 Browser 挂载前进入专用只读卡。

## 3. 真实隔离 App 行为

可携带 composite receipt 保存于：

- `docs/harness/reports/M4R04-registered-owner-exact-source-return-behavior-receipt.json`
- SHA-256：`e86e7767ad30894d42eb6ed388b340533a46d6232524459d2c2c6eadde6f939b`

fresh profile：`syn-r4-acceptance-YTy0xp`。profile 内 composite 与可携带 receipt 逐字相同；三份 phase receipt SHA-256 分别为：

- work_item：`5059ff90092394d2b90a18e3f0543d2a2546b0923ad25481e6f13928648bb915`
- proposal：`14bbb7bcd5ba1f8321c07ec9bc04f4dcaf43e60b405cbc3454a2b516824fb24c`
- restart_negative：`1a3b15c2c20c331c504afe9ad56c61ed5fb4e742274c996cfd8bc6d76c968d19`

actual bundle executable SHA-256 为 `3c7448bcab8988fb2b5ef8110ac113989494e942b08d6e87a6146486b9145b6d`；相关源码最大 mtime 为 17:38:00，bundle executable 为 17:39:11，首份 phase receipt 为 17:39:49。`codesign --verify --deep --strict` exit 0，因此这组行为收据绑定的是最终 frozen source 构建，而不是此前失败或中止的 bundle。

关键直接事实：

- `ordinary_composition=true`、`acceptance_wrapper_calls=0`、`direct_repository_seed_calls=0`、`direct_resolver_calls=0`。三次 R04 App launch 的 PID hash、nonce hash 均互异，并由同一 profile fingerprint `1b48ad8643d8aa8478487919fcfcb56afcbda03ab7c7667c3a09929aba6703da` 和前序 receipt SHA 链绑定。
- WorkItem revision 2 经真实 Home DOM action 点击，服务端 resolve 两次，Projects/Task 精确消费一条 marker，route phase=`CONSUMED`、success notice=1；M4 event/current/provenance/ingestion 与 owner publication/target 都各 1 行，`owner_publication_status=DELIVERED`，且 owner terminal receipt 存在（kind=`M4_INGESTION`）。
- 普通 `create_project_consultation_proposal` 创建 Proposal revision 1；Home refresh 后真实 Proposal action 点击，Projects/Jiaoban 专用只读卡精确消费，typed owner=`owner:project-consultation-proposal:v1`、native type=`proposal_decision`，所有 route/owner cardinality 同样 exact。
- 第三次重启先重新点击并消费旧 WorkItem revision 2 与 Proposal revision 1；随后普通 `update_work_item_state` 把同一 WorkItem 推进至 revision 3，旧 route 变历史、current route 改变。其中旧 route 还经真实 DOM 点击进入 `FAILED/M4_SOURCE_ROUTE_STALE`；该 stale UI negative 前后 active view 均为 Home，navigation、consume marker 与 success notice 都零增量。tampered current route 由同一真实 App 内的普通 resolver 固定返回 `M4_SOURCE_ROUTE_TAMPERED`，不主张额外 GUI 导航观察。current WorkItem revision 3 的新 route 随后真实 DOM 点击并精确消费。
- 三个 phase 都 `exit_code=0`、`timed_out=false`、`signal=null`；portable receipt 只在所有 exact 门通过后原子更新。失败回执不填写成功性的零值声明。
- owner/M4 SQLite 的 publication、event、provenance、ingestion、terminal receipt 与 target record 由 driver 以 read-only 连接逐项核对；独立 immutable 审计另行确认 Workbench/M4 `integrity_check=ok`、foreign-key violation=0。M4 model invocation 与 source-owner writeback 为 0。

## 4. 反例、碰撞与失败边界

两条 exact repository integration test 由 launcher 逐条执行，并要求 `running 1 test`、完整 test identity 与 `1 passed` sentinel；零匹配不能冒充通过：

- `full_registry_resolves_real_delivered_work_item_and_proposal_owner_collision`：真实 owner native fact/outbox -> production dispatcher -> M4 ingestion -> registry resolve；同 canonical object id 在 WorkItem/Proposal 两个 owner 下仍得到不同 typed target，不串 owner。
- `full_registry_returns_fixed_failures_for_stale_revision_missing_and_tamper`：覆盖 unknown owner/type、missing target、revision mismatch、scope mismatch、route tamper、route stale 与 terminal receipt mismatch 的固定 code。

额外聚焦反例覆盖 WorkItem workflow revision 被其他 item 推进、owner scope seal 不一致、publication terminal receipt 漂移、owner target 缺失、M4 current 已前进与旧 historical route。resolver 只使用 DB-primary Ready 的 read accessor；DB commit 后 JSON projection 失败会先把 storage health 置为 Blocked，因此不会把 DB-leading/JSON-lag 误报成已消费。

产品页消费也 fail closed：相同 project root 下必须同时匹配 project id、workflow id 和 object id；目标 read 尚未完成时保持 PENDING，不抢先报 missing；失败恢复原 view、清 focus 且不显示成功。非 rollout Proposal 只显示 exact 只读方案卡，不挂 composer、授权、执行或 supervisor review 状态机。

## 5. 验证

本包最终运行并通过：

- red baseline commit `7f9c6da717f0ec49c22fcd76327431fcfff0cb4e` 上的 route 探针为 RED；AppState registry、closed owner registry、finite target、server command、renderer client 与 owner consumer 等 marker 尚未齐备。
- `node scripts/run-m4-remediation-probes.mjs --only=route --expect=green`：11/11 static markers GREEN；六份冻结合同 SHA-256 exact。该探针只证明固定 marker，不替代行为证据。
- `RUSTFLAGS=-Awarnings cargo test --offline m4_source_route -- --nocapture`：5/5。
- `RUSTFLAGS=-Awarnings cargo test --offline m4r04_ -- --nocapture`：7/7。
- `RUSTFLAGS=-Awarnings cargo check --tests --offline`：exit 0。
- `pnpm run typecheck`：exit 0。
- `node scripts/run-offline-interaction-test.mjs`：15/15 entrypoint groups 通过，包含 M4C06/C09、M4R02、M4R03 与 M4R04 UI/runner 门。
- `node --test tests/m4r04-isolated-app-preflight-runner.test.mjs`：1/1。
- `node --check scripts/run-r4-isolated-app-preflight.mjs`：exit 0。
- `node scripts/run-r4-isolated-app-preflight.mjs --m4r04-ordinary-route`：最终 frozen source 的实际 debug App 六次受控启动（3 次普通产品准备 + 3 个 R04 phase）为 `PASS`。
- `git diff --check`：exit 0。

最终静态复审为 0 blocker / 0 P1 / 0 P2；actual PASS 后另由独立任务以 immutable SQLite 逐字段核验 composite、phase receipt SHA 链、owner/M4 双库与 build/source 新鲜度。

## 6. 证据上限与下一入口

本包证明的是 synthetic fresh profile 中普通产品 command registry、真实 owner publication、生产 M4 ingress、closed resolver、真实 DOM action 与目标页消费的组合。DB 可直接证明 M4 model invocation/source-owner writeback 为 0；launcher 记录 external capability/provider/connector/network write attempts 为 0，但这不是 OS 级抓包或系统全局网络审计。raw PID 与 nonce 按设计只携带 hash。

本包没有实现或验收 M4R05 持续对话、M4R06 五类旧读面或 M4R07 最终总验收，也没有进入真实资料、真实模型/provider、真实消息、账号/凭据/connector、远端、发布或 M5–M10。

下一唯一入口是 `M4R05`。本包完成后只激活该 leaf，不提前施工 M4R06–M4R07。
