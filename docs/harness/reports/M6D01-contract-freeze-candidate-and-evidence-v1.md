# M6D01 contract freeze candidate and evidence v1

状态：`SUPERVISOR_SELF_REVIEW_PASS / CP1_NOT_YET_REVIEWED / CONTRACT_AND_FIXTURES_ONLY / NOT_RELEASED`

## Harness

- stage：`stage-15`；leaf：`M6D01 跨项目与成员合同冻结（ORG-001，只写合同）`；执行来源 receipt：`u-bf5cbe4117b9c087c7f1`。
- 内容候选：`80ddebdf17889035bc7acde423e32ad6de6f17bb`；tree：`9b9ed64be8f8cf6f02c0436ec9883631fe55b56e`。Grok 写出初稿合同与 fixtures，Codex 独立审查后收紧 exact join、状态与 fixture 自洽性；内容提交按事实同时署名。
- 内容提交实际写域只有 3 路径：新增 M6 合同、新增 `m6-org-001` fixture JSON、M6D01 Grok 任务包。零 Rust/前端源码，零既有合同修改。
- `manifest.v1.json` 经核实是 M1 固定十合同注册表，旧 verifier 硬编码数量、顺序、依赖和 exports；M3–M5 增补合同也未登记。本叶按标准保持它 byte-identical，parent/candidate blob 均为 `ac2c08ae102bf9d276a70f1ace072242cf8d0fdb`。
- 本叶已归档，M6D02 进入唯一 current。这里签发的是主管逐叶自复核，不代替覆盖 M6D01+M6D02 的 CP1 独立 verdict，不关闭 stage-15。

## 产品

- 新合同明确标记 `STATIC_CONTRACT_AND_FIXTURES_ONLY`，没有实现 service、repository、projection、UI、provider、runtime、Tauri command、schema migration 或 store。
- ProjectSummary 只可经 M5 `ProjectSummaryQueryPort`，消费 gate 精确绑定 global RoleSession、scope、expiry、policy、project owner、summary id/schema/version/watermark/hash/source refs。fresh/stale/missing/denied/degraded 按确定性优先级判定，不允许把 denied/missing/degraded 降格展示或以 cache 替换。
- CrossProjectAdvisory 精确 join 同时要求 AdvisoryId、global RoleSession、ConsultHandoff 的 handoff id/revision/status/receipt、顶层 policy decision、generated_at/source links，以及每个 summary 的完整字段、owner、freshness=fresh 和 source refs；缺失或不匹配均 fail-closed/zero-write。水位或版本变化只把既有 advisory 标 stale，不静默重算或覆盖历史。
- 用户采纳只创建 source-owned `DecisionRequest`；各项目 owner 后续走彼此独立的 authoritative command/grant/receipt。M6 application projection 只以 append-only observation 投影 `applied/failed/rolled_back/unknown`，不拥有项目结果、不改变 advisory lifecycle；partial apply 和补偿/回滚只引用后续权威 receipt。
- StableMember/TemporaryAgent 严格分型；scope/role assignment、只读 capability/permission refs、availability TTL、contact no-grant、同名不合并、停用保留 refs 和人工 `promoted_from` 均已冻结。TemporaryAgent 身份只来自 M5 完整 12 字段 envelope，禁止 report 自报、缺字段兼容或 runtime trace 推导；ChildRunRef 只作引用。
- 多视角会诊固定同一最小有来源问题包、至少两个互相独立的 RoleSession/Workcell/context packet、提交前不可读 peer conclusion、显式 budget limit/deadline/budget/timeout/result state，只产出用户待决定项，不产出 command/grant/fact。
- 迁移矩阵禁止旧单项目 review、Agent Center session 自动升格；TemporaryAgent 仅从 immutable refs 重建且不复制报告正文；无法逐字段映射的 legacy 记录进入 ref-only quarantine。

## 证据

原始证据根：`/home/synadmin/workspace/.syn-gates/evidence/M6D01-80ddebd/`。候选验证在 detached worktree `/tmp/syn-verify-80ddebd` 上运行；干净父提交基线在 `/tmp/syn-m6d01-baseline.AAyu3q/repo` 上运行。本叶合同标准明确不需要 cargo、GUI 或真实 provider 证据。

- `node /tmp/m6d01-fixture-check.mjs /tmp/syn-verify-80ddebd`：exit 0；11 个 JSON fenced blocks、26 exports、14 actions、41 fixtures（13 positive / 28 negative）均解析并逐例命中 expected code 与 mutation targets。
- `git diff --check 80ddebd^ 80ddebd`：exit 0；Rust changed count 0；既有 M1–M5 contract changed count 0；detached worktree clean。
- 旧 `node docs/contracts/verify-syn-fnd-001.mjs`：候选 exit 1，仅 12 个 `SOURCE_WORKTREE_DRIFT`；干净父提交同样 exit 1、同样 12 项。两份完整日志 SHA-256 均为 `62c8b147630a34ec0f6067823879b56863c89d84d702a004a719a8a7f5b10853`，因此不是本叶引入；不把这个既有红灯伪装成绿色。

主管七项判据：

1. 写域：`git show --stat 80ddebd` 只有合同、M6 fixture 与本叶任务包 3 个允许路径；无源码。
2. 冻结物：manifest blob 前后相同，既有 M1–M5 合同变化数为 0；M5 execution envelope、receipt/audit/quarantine 与 guarded legacy 边界只被精确引用、未被放宽。
3. WIP 保全：6 个受保护 `m6_*.rs`、`.bak` 与 `linux-schema.json` 仍为未跟踪且 SHA-256/字节数逐项等于 M5R08 记录；暂存区为空。
4. 独立重跑：所有本叶要求的 contract/fixture/diff/manifest 检查均在 detached candidate 重跑；旧 verifier 用独立干净父提交复现相同红灯。
5. 实质：本叶要求是冻结唯一判据而非实现运行路径；合同 machine blocks 与 41 个正反 fixtures 逐项覆盖 ACL/freshness、exact join、采纳/应用、成员分型、执行 envelope、多视角和迁移，且 Codex 修正了会导致后续分歧的字段/规则不一致。
6. 不越级：证据只证明静态合同与离线 fixtures；没有声称 service/runtime/App/GUI/真实资料/provider/账号、部署或发布成立。
7. 欠账：本叶标准内没有未满足项。旧 M1 verifier 的 12 个 source drift 已是既存基线并沿用 ENG-01 记录；M6 产品实现从 M6D02 开始，M6D03 仍须先修 M6P00 verdict 点名的 canonical workflow owner-binding 前置再消费跨项目查询。

## 载体

- 产品载体只是候选 `80ddebd` 中的新合同、离线 fixtures 与任务包；不是运行服务或发布产物。
- 本报告、归档 leaf、stage/plan/current-state、audit 与 authorization closed 属独立 Harness 记账；M6D02 active authorization 在该记账提交后按同一真实 receipt 重新签发，不跨 leaf 继承。
- 当前结论为 `M6D01 SUPERVISOR SELF-REVIEW PASS / CP1 NOT YET REVIEWED`。CP1 还必须完成 M6D02，并由 Cursor Opus 独立验收后才可进入 M6D03。
