# 任务包:audit event_id 生成器去截断(止血·单源收编)v1

日期:2026-07-13 · 档位:**轻档**(活写路径代码改造·不碰安全闸/迁移面/真实数据)· 基线 commit `0064aba` · 基线测试 904/0/44。
上承:法证 v1 §C(生成器全清单+活跃性)·撞号史已由 a 修复(d96f88e·live 现零撞号)——**本包管未来:生成器不修,新撞号还会产生**(近 30 条 20 条 96 顶格)。

## 一句话

把全部 audit event_id 构造位点收编到**单一 helper**,新格式去截断保唯一;历史 id 不动(读取侧视 id 为不透明串,法证 D2 已证无解析依赖)。

## 新 id 格式(拍死)

`audit:{kind}:{stable_id 完整版(不 take(96))}:{sha256(实体串)前12hex}:{timestamp_ms}`

- 完整 slug 保可读;sha12 兜 slug 折叠碰撞(如 `a-b` vs `a_b` 同 slug 不同实体);同毫秒批次天然唯一(实体不同→sha 不同)。
- 与 a 修复后的历史格式不同无妨:id 是不透明串,店内混格式合法;禁写任何"按 id 结构解析"的新代码。

## 实现要求

1. **单一 helper**(落 `workflow_audit.rs` 或 `utils/`):`audit_event_identity(kind_slug, entity, timestamp_ms) -> String`;**全库唯一真源**,谁都不许再手拼 `format!("audit:...")`。
2. **收编法证 C1 全部位点**(≈44 处,逐个改为调 helper;清单=法证 §C1 表,file:line 全给了)。本就不经 `stable_id` 截断的位点(如 `stable_fragment+nanos`、纯 timestamp 的 init/migrated)可豁免,**豁免逐条列明+理由**。
3. **全局 `stable_id` 一个字不动**(它还撑 workflow/work-item/node 等既有 id——动它=全库 id 漂移)。
4. 案发测试:①同毫秒 N 实体批次(C4 prepare 场景)→ N 个唯一 id;②slug 折叠碰撞对(`a-b`/`a_b`)→ sha12 区分;③helper 输出格式断言。
5. 既有测试若断言旧 id 形状 → 随包更新,**逐条列进回传**(不许静默放松其它断言)。

## 禁止事项(红线)

1. 不碰:安全闸/沙箱/审批/敏感谓词;迁移面六处(importer/apply/schema/exporter/production_apply/repository);`workflow_state_store.rs` 活路径;`stable_id` 本体。
2. live 根/生产 DB 零碰(测试全走 temp/fixture)。
3. 零新 tauri command;不进 lib.rs(mod 声明除外);零新 sidecar;棘轮不碰;`read_cut.rs` 零加行。
4. 不 commit;回传 10 项(第 10 项无也写「无」)。

## 变更辐射面

- 改了什么假设:「audit id=各处手拼+可能截断」→「单源 helper+保唯一」。
- 依赖旧假设的:断言 id 形状的测试(逐条列);`audit_refs` 前缀构造(memory_daily_loop 等)**构造的是 refs 非 event_id,不在本包**——若实现中发现 refs 与 event_id 有格式耦合,停下报。

## 验收(预写死)

- grep 证明:仓内 `format!("audit:` 手拼位点=0(全走 helper;豁免清单除外);
- 案发测试三件全绿;`cargo test --lib` 基线 **904/0/44 只增不减**(id 形状断言更新逐条列明);
- `git diff --check` 过;fmt 仅历史三漂移;shape gate 零净增;真实根 hash 前后一致(`03f8bebe…` 起点)。

## 总指导回收动作

亲跑全库+案发测试;grep 手拼残留;扫 diff 核红线(迁移面/谓词/stable_id/活 store 零碰);真实根 hash 亲核 → 接受/需改 → commit 问一次。收口后 M5 前置只剩 preflight v2(待用户授权)。
