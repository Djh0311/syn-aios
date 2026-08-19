# Syn M6 → M7 域事件与 source-ref 输入交接 v1

日期：2026-08-19

状态：`INPUT_CONTRACT_HANDOFF / M7_NOT_ACTIVE / NO_EXECUTION_AUTHORITY`

来源：stage-15 M6D01 冻结合同与 M6D02–M6D08 域层候选；最终域层锚 `a3d575975033f7eb5ec92dab18c24fe97ddb8001` / tree `d710e6f38be216e813dbb66482d87e8bc80ce923`，尚待 stage-15 独立 verdict。

目的：给未来 M7 一份可执行的输入边界；本文件不建立 current leaf、不签 authorization、不激活 M7，也不证明 UI、真实 provider/project 或发布。

## 1. 上游真值与读取方式

- 唯一 schema 判据是 `docs/contracts/m6-cross-project-and-organization-v1.md`。M6D01 的合同与 fixtures 是字段/状态判据；运行事实仍由各原 owner 持有。
- M6 只拥有 Global Supervisor advisory/consultation、organization directory、temporary history projection 与相关 scrubbed audit/read model。M3 继续拥有 `RoleSession` / `Handoff`，M4/SOURCE_OWNER_REF 继续拥有 `DecisionRequest`，M5/各项目 owner 继续拥有 `ProjectSummary`、ExecutionEnvelope、command/grant/receipt 与项目结果。
- M7 只能从未来明确批准的 typed event/read port 读取 M6 事件 envelope、opaque source refs、typed ids、revision、watermark、hash、freshness/status 与 owner receipt refs。不得直读 M6 SQLite、项目 store/root、raw summary/transcript、prompt/provider output、secret 或未裁剪 memory。
- source ref 解引用必须重新经过该 source owner 的 policy gateway；deep link 是导航，不是把原文注入 M7。

## 2. 允许交给 M7 的事件类别

| 事件 | 可消费的最小事实 | M7 必须保留的边界 |
|---|---|---|
| `ProjectSummaryFreshnessJudged` | subject ref、owner、freshness、watermark/hash refs、consumer gate ref | denied/missing/degraded/stale 不得被重写为 fresh，不缓存替代 |
| `CrossProjectAdvisoryRecorded` / `CrossProjectAdvisoryMarkedStale` | AdvisoryId/revision/status、global RoleSession ref、exact consumed-summary refs、source links、policy/Handoff receipt refs | advisory 是意见，不是项目事实或个人记忆；stale 保留历史 |
| `AdvisoryAdoptionDecisionRequested` | advisory ref、pending DecisionRequest ref、actor/owner refs | 只代表待决定；不得自动创建 Memory、PersonalFact、project command 或 grant |
| `AdvisoryApplicationObserved` | advisory/decision refs、per-project authoritative command/grant/receipt ref、outcome | M6/M7 都不拥有项目结果，不得改变 advisory lifecycle 或伪造补偿 |
| `StableMemberRegistered` / lifecycle、`AvailabilityObserved`、`MemberContactRecorded` | MemberId/revision、typed role/scope/capability/permission/memory refs、TTL/source、contact/Handoff receipt | 目录不是授权真源；stale availability=unknown；provider/runtime/thread/process 不得成为身份 |
| `TemporaryAgentProjected` / `TemporaryAgentPromoted` | TemporaryAgentId、完整 M5 envelope refs、projection/quarantine status、explicit promotion binding | temporary 不等于 stable；ChildRunRef 不是成员；promotion 保留原 temporary 历史 |
| `MultiViewConsultationStarted` / `ConsultationViewSubmitted` / `MultiViewConsultationAssembled` | QuestionPacket hash/source refs、distinct RoleSession/workcell/context refs、budget/timeout/result state、consensus/disagreement/evidence refs、pending DecisionRequest ref | 提交前保持独立；结果只是意见和待决定项，不生成 command/grant/fact |
| `LegacyRecordQuarantined` | typed record ref、reason code、scrubbed audit ref | payload 只 REF_ONLY；不得 guess-fill 或升级为事实 |

事件 envelope 必须携带稳定 event/idempotency key、schema/version、owner、subject typed id、revision/status、occurred_at、correlation/audit ref，以及该事件类别要求的 source/receipt/hash 字段。缺失、owner 不匹配、revision 回退、未知 schema、hash/watermark 不一致或同 key 异载荷，一律在 M7 写入前 fail-closed/quarantine。

## 3. M7 允许形成什么

- 在未来 M7 合同明确授权后，可建立**来源可追溯的候选**：例如待用户确认的 memory candidate、knowledge candidate 或 personal-fact proposal，只保存最小 typed refs、摘要 hash、owner/policy refs 与 provenance。
- 任何候选必须保持 `CANDIDATE/PENDING`，直到用户或 M7 原 owner 的独立确认动作。M6 advisory、成员资料、availability、咨询共识或运行结果都不能因为被读取而自动成为正式 Memory / PersonalFact。
- M7 可按 `(event_id, schema_version, revision, payload_hash)` 幂等消费；重复同载荷不新增记录，异载荷同 key quarantine。回放不得重放 contact、provider、project command 或其他 effect。
- M7 删除/遗忘策略不得反向删除 M6/M3/M4/M5 的 authoritative history；最多撤销 M7 自有投影并保留审计/ref。

## 4. 明确禁止继承

- 不继承 Global Supervisor 的项目写权限，因为它本来就没有；不从 adoption、consultation 或 contact 推导 ExecutionGrant/CapabilityGrant。
- 不继承 provider/model/thread/process/session 名称作为成员、人物或身份真值。
- 不继承 raw project data、raw report body、transcript、prompt、provider response、stdout/stderr/tool output 或 secret/credential material。
- 不把 `NoExecutionHistory` 当 schema/carrier mismatch，也不把缺 `m5_durable_operations` 静默解释为没有 ChildRunRef。
- 不以本地 synthetic stage-15 证据声称真实资料、真实 provider/account、真实项目、GUI、新壳、部署或发布通过。

## 5. 激活前硬前置

1. stage-15 获独立 PASS；若 verdict 点名 M7 输入欠账，先据实合入未来 M7 叶的完成标准。
2. 用户明确开始 M7，并建立唯一 current leaf 与新鲜 authorization；本 handoff 不能替代。
3. M7 自己冻结消费合同、owner/read port、candidate/confirmation 状态机、retention/rollback 与 fail-closed fixtures。
4. 先用 isolated app-data、synthetic refs 与 fake provider 验证；真实个人资料、真实项目/provider/account/connector 继续逐项另批。

## 6. 回滚

关闭 M7 consumer/read projection 即可停止新消费；保留 M6 event/ref 与既有 M7 audit，不重放 effect、不修改 M6 advisory/member/temporary/consultation lifecycle。回滚不得恢复 raw cross-project read，也不得把已 quarantine 的输入重新解释为合法事实。
