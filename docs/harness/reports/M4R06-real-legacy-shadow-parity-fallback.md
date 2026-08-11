# M4R06 五类旧读面的实际 shadow/parity/fallback 验收报告

日期：2026-08-11

阶段：`stage-07`

任务包：`M4R06`

## 1. 结论

M4R06 已通过。普通产品 AppState 现安装独立、只读的 `M4LegacyReadRegistry`；零参数 registered command 从五个固定 server-owned legacy surface 读取安全投影，再由 M4 repository 对 canonical source 做只读 reread、parity/quarantine 比较与 WorkItem owner cut 复核。renderer 不提交 legacy tuple、owner、scope、revision、route 或 callback 作为权威，也不把 React 临时态或新 M4 canonical read model 倒灌成旧面真源。

同一 synthetic fresh profile 中，受控 launcher 先用 3 个独立 App process 完成 R02 普通产品准备，再用第 4 个独立 App process 执行 R06 的真实 report、可见 fallback 和 exact replay。4 次启动均 `exit_code=0`、`signal=null`、`timed_out=false`，没有额外重启。最终 portable composite 为 `PASS`，证据等级为 `ISOLATED_PRODUCT_APP`。

本包使用 synthetic fixture 数据。为进入既有 guarded Home fallback，第 4 个 debug App 只消费一次既有 `HOME_UNAVAILABLE` envelope，receipt 将范围明确记录为 `HOME_UNAVAILABLE_ONE_SHOT`。该 trigger 不是自然生产故障，不生成 reader 结果，也不替代普通 command/report；server-owned reader、M4 canonical reread、Board DOM 与零写入证据仍走普通产品链。

可携带 receipt：

- `docs/harness/reports/M4R06-real-legacy-shadow-parity-fallback-behavior-receipt.json`
- SHA-256：`0ddc6353aab39703656d11486d303bfbf6cab8ce82f7a6df7336c3fd4b32fdee`
- 权限：`0600`
- profile 内 composite 与 portable receipt 逐字节相同。

## 2. 生产调用链、读边界与 O2 cut

```text
普通 Secretary Home 初始读取
  -> [仅 R06 debug 一次] 既有 Home UNAVAILABLE envelope
  -> 普通 App guarded legacy fallback
  -> load_secretary_legacy_read_compatibility_report()（零参数）
  -> AppState.m4_legacy_read_registry
  -> 五个固定 server-owned reader / 固定无 owner 分类
  -> M4 repository canonical snapshot 只读 reread
  -> exact parity / quarantine comparator
  -> WorkItem owner O2 reread 与 O1 exact 比较
  -> READY compatibility report
  -> Board 仅展示 PARITY + PRIMARY 的只读来源项
```

五类固定读面为：旧 Secretary deterministic-summary primitives、WorkItem right-rail notification/todo projection、server runtime-attention projection、React pending-action visibility 和 memory daily candidate store。React 没有 server-owned exact tuple 真源，因此 registry 按冻结合同固定返回 `UNJOINABLE`，不读取 renderer、localStorage 或 React state。旧 Secretary/runtime/memory 只读真实 server primitives；缺失 workflow/optional sidecar 是合法 `EMPTY`，存在但不可读是 `QUARANTINED/M4R06_READER_UNAVAILABLE`，存在但 schema、必需数组、store version、storage kind 或 revision 无效是 `QUARANTINED/M4R06_READER_REJECTED`。

WorkItem 是唯一 exact-tuple reader。它先按每个 canonical object 取得最新 owner publication，验证 `DELIVERED`、`M4_INGESTION` terminal receipt、current owner record 与 native rebuild，再调用与 R02 相同的 registered mapper 和共享 open-loop predicate；`ready_to_dispatch` 因此可正确映射为需要关注，而不是由 R06 手写一套状态枚举。

WorkItem 跨库读边界以 O2 线性化点表述：

1. registry 取得 preliminary M4 scope watermark，并读取 WorkItem owner（O1）；
2. M4 repository 在只读 transaction 内 reread canonical source 并生成 report；
3. registry 用同一 preliminary watermark 再读 WorkItem owner（O2），要求 O1/O2 的 receipt 与 candidates 完全相等。

O1 到 O2 间 owner 前进但尚未进入 M4 时，整份 report fail closed 为 `UNAVAILABLE`，旧 candidate 不会继续成为 `PARITY` fallback。该保证不是跨 SQLite 原子事务，也不覆盖 O2 之后发生的外部更新；runtime 与 memory 的时间相关读面不属于此 WorkItem owner cut。

## 3. 实际隔离 App 行为

有效 profile root 为 `syn-r4-acceptance-YJ0YZJ`，profile SHA-256 为：

`509ab551df94d47e5c35fba215653fb50cb223aa39aa2d13dd5cf45f3e7d5b47`

实际启动严格为 4 次，`same_profile=true`、`distinct_app_processes=true`：

| 序号 | phase | App PID SHA-256 | nonce SHA-256 | phase receipt SHA-256 |
| --- | --- | --- | --- | --- |
| 1 | R02 `initialize` | `376efbf400bed3a5c5b289ecd2118926bc19b632223e8fcbb187e8a2d915a601` | `48d9fa18a69be82b053a2e57c574afcc6a650f9bdcefa2300d504ca2aa2660d6` | `4c888b101c60dbe752bf6de3f74207cf8aac4cbc04f2111a1cfa3fe7eca40e12` |
| 2 | R02 `mutate` | `f3b79631f21a489e363181ed9b6a30b109f214f9a13370382169d1947e25867c` | `56f7d79b0661f2be125bee436b4aa0fdda5a38190409750ddb62f51288bc204a` | `99fbd525046e9a2e66156e996457e13178086afbc78274a8ef07b376d91f13ce` |
| 3 | R02 `readback` | `c1ddb3ab3686ccad81ae3889e9b6b561bd33bfe19a0b88a3c62dd64a1072a43b` | `70f073873dfab690ad7f7049e1b1fb919d5e1d34174e3071cd0f1a0733262524` | `bb314df163d366481c06919f89598052f3f346137f56e3690d15f64a71124b49` |
| 4 | R06 `read_and_replay` | `fcf98959f8dea2e5b24f68d741958934cf848a600520b0b01bdd996a30311e97` | `25c56347b1263108495a5b4b46d16c8b3b6417bdff3bad239ce1690986c75f7c` | `54dfb868da9b774208753d95290868c5c14b1b662ce65a1c866d4bbddb63ac66` |

R02 readback receipt SHA 与 R06 的前置引用 exact 相等；R02 ingestion adapter 与 R06 WorkItem reader adapter 的 SHA-256 均为 `28711f4fc978421b70530a689df9cc68a646a7600259405606426c7441d3d114`。first report 与 exact replay report SHA-256 均为 `9de2a559d226c733dda8e3701252d35b1bc30a160bf6323853734e954104fa32`，五项 receipt 完全一致。

本次实际 receipt 出现 `OBSERVED`、`EMPTY`、`UNJOINABLE` 三种状态；`QUARANTINED` 由聚焦反例覆盖，没有伪称在这次 App 现场出现：

| 旧读面 | 实际 state | candidate / complete tuple | 说明 |
| --- | --- | --- | --- |
| `SECRETARY_READ_MODEL_DETERMINISTIC_SUMMARY` | `UNJOINABLE` | 1 / 0 | `M4R06_UNJOINABLE_NO_EXACT_TUPLE` |
| `RIGHT_RAIL_NOTIFICATION_AND_TODO_PROJECTION` | `OBSERVED` | 1 / 1 | registered WorkItem adapter |
| `RUNTIME_ATTENTION_PROJECTION` | `EMPTY` | 0 / 0 | `M4R06_EMPTY_SERVER_SURFACE` |
| `REACT_PENDING_ACTION_VISIBILITY` | `UNJOINABLE` | 0 / 0 | renderer 临时态不升格 |
| `MEMORY_DAILY_INBOX_CANDIDATE` | `EMPTY` | 0 / 0 | `M4R06_EMPTY_SERVER_SURFACE` |

唯一实际 `PARITY` 是 revision `2` 的 Right Rail WorkItem。receipt 与独立 DB 审计均确认 owner publication、M4 current、M4 provenance、parity-primary 各 1 行；owner/object/revision 和 route hash 与 DOM tuple 精确绑定。fallback eligible row 只有 1 条，且全部为 `PARITY + PRIMARY`。

真实 DOM 观察到：打开 Board 1 次、compatibility fallback root 1 个、PARITY primary attention row 1 条、非 PARITY row 0 条、可操作来源控件 1 枚、嵌套摘要来源控件 0 枚，Board 内 coordination/personal action control 均为 0。`source_route_clicks=0`，因此本包只证明精确来源入口可见和 tuple 绑定，不主张已完成来源页导航。Board 内零写控件也不扩大为“整个 App 没有其他可用控件”。

此前调试 profile `syn-r4-acceptance-s1IOpU` 在发现 schema gate P1 后立即中止；现场只完成 prelaunch layout（profile、fixture、workflow-state，以及空的 `app-data`、`codex-db`、`logs` 目录），未产生 ordinary app-data runtime artifact、phase receipt、composite 或 App process 启动证据，因此不计入上述有效 4 次启动，也未生成 portable PASS。

## 4. 零副作用：runner 断言与 DB 指纹

runner/driver 直接断言：

- `ordinary_composition=true`，`acceptance_wrapper_calls=0`，`direct_repository_seed_calls=0`，`manual_legacy_candidate_calls=0`；
- synthetic Home trigger 只消费一次，ordinary reader report 与 DOM fallback 均实际观察到；
- bridge 的 first/replay 各调用零参数 report command 一次，`zero_arg_load_calls=2`；server command boundary 实际记录 3 次 load，即自动 fallback 1 次加 first/replay 2 次；
- profile、4 个 nonce、4 个 PID、R02 readback receipt 与 R06 receipt 相互绑定；
- launcher 记录 real model/provider、external connector/network write、real Codex message attempt 均为 0。

最后一组是受控 launcher/fixture 计数，不是 OS 抓包或系统全局网络审计。

driver 用 10 个 `READ_ONLY + PRAGMA query_only=ON` 连接，在 pre-render baseline、`after_ui_fallback`、`after_first_read`、`after_exact_replay` 四个采样点读取指纹。每个点均为 `integrity_check=ok`、foreign-key violation 0，各组 hash 全程不变；本轮独立现场审计另以 SQLite `mode=ro&immutable=1` 重算并精确匹配，且观察到 DB/WAL/SHM 的尺寸与 mtime 前后不变。该文件元数据观察未写入 portable receipt，不能由 portable 单独复验：

| 读库范围 | table / record count | canonical hash |
| --- | --- | --- |
| owner | 84 / 43 | `5afba1a934ce7fad370787ef18e09b51b4ce4ce6d2dae0c59ce66e3067b19ca7` |
| M4 reader-related scope | 22 / 27 | `8f31773b9e698d28973303a79393cbca1e3a9c93d5d24d4d378f4744320be150` |
| coordination | 3 / 15 | `375595a0227716d16c975225d129b4d6f34598bab010bafc491b3435c364b350` |
| effects | 10 / 4 | `cedcd38a1473e42697b307eb190a6a6d14b27bbd27ab8efdcfa55c770731f40f` |
| writeback | 2 / 0 | `234c52ce86c8db09b7a7389762ba95c0868027f89eb4cf382bbd4c19749bbb53` |

因此在四个采样点，UI fallback、first read 与 exact replay 对上述 owner/M4/coordination/effect/writeback scope 的 delta 均为零。M4 scope 明确是 `READER_RELATED_M4_EXCLUDING_INDEPENDENT_DAILY_SCHEDULER`；本报告不把它扩大为整个 M4 所有表无变化，也不把采样点相同扩大为采样间绝无瞬态写后恢复。

## 5. 聚焦反例与验证

聚焦反例覆盖：

- 五类 fixed receipt 的 match/empty/unavailable/rejected wire matrix；
- Secretary/runtime/memory 的真实 empty、unjoinable（适用者）、I/O unavailable、malformed/validator rejected；
- existing but uninitialized workflow 的 wrong schema 与缺必需数组，Secretary/runtime 均 fail closed；missing workflow/optional sidecar 仍保持合法 `EMPTY`；
- memory `store_version/revision` 与 continuation `schema/store/storage/revision` validator 拒绝；路径名含 `invalid/schema/.json` 的 I/O 错误仍归 `UNAVAILABLE`；
- React 固定 `UNJOINABLE`，不读取 renderer；
- WorkItem `ready_to_dispatch` PARITY、terminal empty、undelivered/malformed rejected、owner unavailable、scope watermark mismatch 与 O1/O2 间 owner advance；
- raw evidence scanner 对 Unix slash、Windows 单反斜杠、raw route/owner/object 等字段 fail closed；
- fallback 仅显示 `PARITY + PRIMARY`，不显示 quarantined/non-parity row；portable 仅在 PASS 全门成立后原子发布，并与 root composite 做 read-back byte/SHA exact 校验。

本包最终运行并通过：

- `RUSTFLAGS=-Awarnings cargo test --offline --lib m4r06 --quiet --no-fail-fast`：25/25。
- `RUSTFLAGS=-Awarnings cargo check --tests --offline --quiet`：exit 0。
- `pnpm typecheck`：exit 0。
- `node scripts/run-offline-interaction-test.mjs`：exit 0，`offline interaction tests passed: 15`，包含 M4C08、R06 driver 与 R06 isolated runner。
- `node --check scripts/run-r4-isolated-app-preflight.mjs`：exit 0。
- `node tests/m4r06-isolated-app-preflight-runner.test.mjs`：exit 0。
- `node scripts/run-m4-remediation-probes.mjs --only=legacy --expect=green`：8 项 fixed static marker GREEN；该 probe 只证明 literal marker，不替代行为证据。
- `node scripts/run-r4-isolated-app-preflight.mjs --m4r06-ordinary-legacy-read`：严格 4 次受控 App launch，portable composite=`PASS`。
- `git diff --check`：exit 0。

最终语义、false-PASS、实际 receipt/SQLite 与 Harness scope 均由 Terra Ultra 子代理独立复核；静态与行为审计结论为 0 blocker / 0 P1 / 0 P2。

## 6. 证据上限与下一入口

本包证明的是 synthetic fresh profile 下普通 product constructor、registered zero-argument command、server-owned readers、M4 canonical shadow parity、一次受守卫的可见 fallback、exact replay 和指定读库 scope 的零增量。它没有证明真实用户资料、真实旧面数据、自然发生的 Home outage、真实 provider/model/connector、OS 网络抓包、来源页导航、independent daily scheduler 零变化、旧面退役或生产迁移。

实际 root 没有保存被执行 App 的独立副本，portable receipt 也没有记录 executable SHA、mtime、CDHash、bundle path 或 codesign 结果。独立审计观察到当前 debug executable SHA-256 为 `31d7cd4391fe52db99b49827fd4a5fd839b85526e9b74e5a1f94296228f2de3b` 且 `codesign --verify` 通过，但这只是当前构建产物与 runner build 记录的组合观察，不能追溯性地升级为 portable receipt 已封存的 binary identity。

本包没有实现或验收 M4R07 最终总验收，也没有进入远端、发布或 M5–M10。下一唯一入口是 `M4R07`。
