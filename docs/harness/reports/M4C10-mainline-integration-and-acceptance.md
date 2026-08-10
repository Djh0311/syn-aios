# M4C10 主线集成与验收报告

日期：2026-08-10<br>
状态：`COMPLETED / MAINLINE / STAGE-06 CLOSED`<br>
已关闭叶：`stage-06 / M4C10-integration-regression-closeout`

结论：M4C01–M4C10 的具名范围已经完成。M1/M3/M4 冻结输入 exact；M4 聚焦 Rust、最终完整主机权限 Rust、non-test check、前端 typecheck/offline/build、launcher 契约、定向格式和 C09 脱敏证据复核通过。受限 sandbox 完整 Rust 首跑的 6 个失败、其中 5 个 launcher 静态碰撞的修复和 1 个 PID 环境差异均保留。本结论只覆盖主线机械证据和 C09 synthetic isolated App，不包含真实日常使用、真实数据/模型/provider/connector、M5–M10 实现或发布。

## 1. 范围、来源与结论口径

| 项目 | 记录 |
| --- | --- |
| C10 审计候选 HEAD | `9e971202310a105bf2a9995c116c90f5287ff089`（launcher 回归修复提交） |
| 阶段授权 | `USER-SYN-M4-AUTONOMOUS-STAGE-06-20260810`；只作为已完成 stage-06 的历史授权记录 |
| 读取范围 | 根/仓库 `AGENTS.md`、Harness chain/progress/auth、stage-06、C01–C10 leaf、M1/M3/M4 合同、M3/M4 计划、current-state、C08/C09 事实与提交、C09 report/JSON receipt、C01–C09 Git 记录 |
| 文档清理范围 | 按 neat-freak 收尾清单枚举 Markdown 与关键文档体量；只修改 C10 leaf 允许的 current-state、master/M4/plan index/task queue、C10 report、下游 handoff 与 Harness 记录 |
| 证据层级 | 合同/static/unit/temp integration/non-test build/isolated debug App 分层；任何一层都不升级为真实日常使用或 release |
| 终态边界 | C10 内容与文档精确提交后执行 `hl done`、`hl close-stage --write` 并提交全部 Harness 记录；不 push、merge、rebase、部署或发布 |

## 2. M4C01–M4C10 提交清单

| 叶 | 内容提交 | 已进入的具名范围 |
| --- | --- | --- |
| M4C01 | `530ab41` | 冻结 M4 Secretary/Attention/Daily 实施合同与 source/owner/rollback 参数 |
| M4C02 | `d6d5293` | 普通产品 Secretary RoleSession、PersonalScope 与 AppState 接线 |
| M4C03 | `fe75e70` | M4 自有持久 source-first Inbox/Attention projection |
| M4C04 | `a5e8062` | 协调生命周期、Reminder/Notification、显式 PersonalAction 与 owner receipt |
| M4C05 | `64cb655` | 持久 Secretary application service、确定性 brief、模型 ledger 与 M3 Handoff 状态 |
| M4C06 | `48b7afd` | 首页 typed context、持续 Secretary 对话、deep link 与协调动作 |
| M4C07 | `3892d19` | DailyReport、scheduler、catch-up、失败恢复和空事件零模型 |
| M4C08 | `a786365` | 五类 legacy read shadow/parity/compatibility read-only 与 quarantine |
| M4C09 | `c823986c` | synthetic isolated debug App 首启/强退/重启与分层证据 |
| M4C10 | `9e97120` | 将 C06/C07/C09 测试挂入离线总入口，并等价消歧共享 launcher 的旧 R4/M3 静态契约 |

该表只说明本地主线提交和具名范围；不代表远端已核验或下游阶段已激活。`origin/main` 本地 tracking ref 未被当作实时远端事实。

## 3. 冻结输入

2026-08-10 重新计算的 SHA-256：

| 冻结输入 | SHA-256 | diff 依据 |
| --- | --- | --- |
| `docs/contracts/role-session-v1.md` | `77c82932e728d4982ebb501b167f274cc31d2076957602771904d96dc399b2ca` | 相对 `29085cc` exact |
| `docs/contracts/handoff-v1.md` | `3378f02f5dfb06e4db39125b5828eeda9440fc2c25ddbee3fe4e951fa6c386bf` | 相对 `29085cc` exact |
| `docs/contracts/identity-scope-v1.md` | `3cb0073c0fffc2423e3450ce9d9e3c683065cdd075bf618e0d406cc1475e3ea4` | 相对 `29085cc` exact |
| `docs/contracts/event-audit-outbox-v1.md` | `15a24d8040da054794e340fe7839b273dce0f60a2c1708513d1b998c8e968e99` | 相对 `29085cc` exact |
| `docs/contracts/m3-role-session-turn-handoff-resolution-v1.md` | `946c756b30a8e73aaad441e49ba39a5c9cbd7c7d47241ed97fa19d02783bac48` | 相对 `530ab41` exact |
| `docs/contracts/m4-secretary-attention-daily-resolution-v1.md` | `4e4d6251d53e1b9b156fb2fd1266d73d6beace38be2086e83e0f05694dec4e51` | 相对 `530ab41` exact |

计划/current-state 是状态型文档，会随阶段收口变化，不纳入上述 frozen contract exact 声明。

## 4. M4 退出矩阵

| 退出项 | 直接证据 | 结论 | 边界 |
| --- | --- | --- | --- |
| Secretary / PersonalScope 后端身份 | C02 commit + M4 Rust tests | 主线已实现 | 不来自 renderer/fixed cwd |
| source-first Inbox/OpenLoop/Decision | C03 repository/schema/read model + tests | 持久、重启、dedupe、quarantine 通过 | 不接 M5/M8 未提供来源 |
| attention/Todo/Reminder/Notification 生命周期 | C04 tests + C09 restart receipt | CAS、幂等、carry-over 与跨重启通过 | coordination 不等于 owner completion |
| Secretary service/context/Handoff | C05 tests | deterministic brief、model failure、UNAVAILABLE/PENDING/RETURNED 恢复通过 | M6 success 未实现 |
| 首页 typed context 与 deep link | C06 offline tests + C09 UI receipt | visible/clickable synthetic evidence 通过 | 不证明所有真实来源 |
| Daily scheduler/report | C07 Rust/TS tests + C09 runtime receipt | timezone/window/catch-up/idempotency/rebuild 通过 | 没有真实长期时钟观察 |
| 空事件零模型 | C07 tests + C09 receipt | material/agent/model count 均为 0 | fake model/isolated scope |
| legacy compatibility/rollback | C08 Rust/TS tests | shadow/parity/read-only/quarantine/rollback 通过 | 无 tuple adapter，inventory 当前全部 quarantine；未物理退役 |
| isolated product App | C09 report + 3 JSON receipt | 首启/SIGKILL/重启/deep link/lifecycle/model failure 通过 | synthetic + fake model + debug App |
| 完整主线回归 | §5 | 最终全绿 | 保留初次红灯、warning debt 与 chunk warning |
| M5/M6/M7/M8/M9/M10 | 合同 downstream boundaries + 源码 | `HOLD / NOT_ENTERED` | M4 只交 typed ref/event |

## 5. C10 命令与结果

| 层 | 实际命令 | 结果 |
| --- | --- | --- |
| frozen SHA | `shasum -a 256` 六份合同；`git diff --exit-code 29085cc` 四份 M1；`git diff --exit-code 530ab41` M3/M4 | 全部 exit 0，§3 exact |
| C09 Rust | `cargo test --lib m4c09_` | exit 0；3 passed / 0 failed |
| M4 Rust | `cargo test --lib m4c0` | exit 0；98 passed / 0 failed |
| 完整 Rust 首次受限 sandbox | `cargo test --lib` | exit 101；1633 passed / 6 failed / 45 ignored；85.90s |
| R4/M3/C09 定向修复回归 | `cargo test --lib acceptance_runtime_profile_`；M3 launcher exact；`cargo test --lib m4c09_` | 26/26、1/1、3/3，均 exit 0 |
| PID exact 主机权限 | `cargo test --lib supervisor_session_launcher::resident_session_tests::s1b_h2_real_initial_and_resume_consume_only_the_private_submit_proposal_config` | exit 0；1/1 |
| 最终完整 Rust 主机权限 | `cargo test --lib` | exit 0；1639 passed / 0 failed / 45 ignored；80.44s |
| non-test Rust | `cargo check --lib` | exit 0；0.42s；仍有 917 条仓库既有 warning，不声称零 warning |
| 定向格式 | `rustfmt --edition 2021 --check` 全部 7 个 M4 新增 Rust 文件 | exit 0；未递归格式化 `commands.rs` / `lib.rs` |
| TypeScript | `npm run typecheck` | exit 0 |
| 离线交互 | `npm run test:offline-interaction` | exit 0；44 个 entrypoint；包含 C06 UI/read model、C07、C08、C09；脚本内既有摘要 15 |
| production build | `npm run build` | exit 0；306 modules；最终 Vite 927ms；仅既有 `>500k` chunk warning |
| launcher syntax | `node --check` offline runner、R4 launcher、M4 fixed wrapper | 三项 exit 0 |
| C09 evidence | `shasum -a 256` + `jq -e` launcher/runtime/UI 语义 + redaction scan | 全部 exit 0；§6 hash exact |
| diff hygiene | `git diff --check` 与精确 staging 检查 | exit 0 |

npm 命令使用仓库既有 ignored `node_modules`；本轮没有执行或声称 clean install 成功。

### 5.1 完整 Rust 首跑红灯与修复

首跑 6 个失败全部有直接分类：

1. 5 个是 C09 新增共享 launcher 源码后，既有 R4/M3 测试用 `contains` / first-occurrence 解析命中了等价的新 token：`environment:`、bundle launch 局部变量、bundle identifier key、`logs` 路径片段和 M2/M3 互斥表达式。
2. C10 leaf 精确补列唯一共享 launcher，只做运行时等价消歧：computed JSON key、C09 局部变量改名、单行路径表达式和等价布尔分解。旧 R4 26/26、M3 exact 1/1、C09 3/3 与 offline C09 static 均通过。
3. 第 6 个是受限 sandbox 读取子进程 PID `lstart` 返回 EPERM。相同 exact test 在主机权限环境 1/1 通过；最终完整主机权限套件 1639/0/45。

初次 1633/6/45 不从报告中删除；最终绿灯也不把 PID 环境差异写成产品缺陷。

## 6. C09 隔离证据复核

| 证据 | SHA-256 |
| --- | --- |
| `M4C09-isolated-product-app-evidence/launcher-receipt.json` | `036d00250e68df7e057877dc628c46eb8be4ac421869d96397bce066e838d1eb` |
| `M4C09-isolated-product-app-evidence/runtime-receipt.json` | `53717738a24396644dd915f65b0d7a87e0594b531f901841598bffcdcf34e210` |
| `M4C09-isolated-product-app-evidence/ui-inspection.json` | `669f4b1902935c1856b86e9780c34172dd3531b783b297645f24646b62bb23c6` |
| 首启截图（不入仓） | `99b4af3384629c48e1cdb25b3611f8eba1bba2408640064454491bf4a6a93f6e` |
| 重启/deep-link 截图（不入仓） | `8a98afd316b6e90da016a373b400569eb3261d9fc3f314bfe9ed5a691659d322` |

语义复核确认：同 profile 3 次 launch；第 0 次 SIGKILL、第 1 次 exit 0、第 2 次 UI 完成后 SIGTERM；同 RoleSession 恢复；两个 owner 不合并；exact duplicate/restart replay 不增行；alpha `ACKNOWLEDGED`、beta `SNOOZED`、carry receipt 1；日报重跑稳定；空事件 material/agent/model 为 0；fake model 失败只调用 1 次且重启不重放；UI 观察完成并保存截图 hash。真实 model/provider/connector/network write/Codex message attempts 均为 0，receipt 明示 `MECHANICAL_AND_ISOLATED_PRODUCT_APP_ONLY_NOT_REAL_DAILY_USE`。

## 7. 迁移与回切

- 普通 M4 READY 始终 primary；只有后端显式 UNAVAILABLE 才尝试 C08 compatibility report。renderer 不提供 legacy inventory request，后端固定五类候选。
- 当前没有 backend legacy tuple adapter，五类 inventory 全部 quarantine；不同 kind 可 canonical dedupe，同一 legacy identity 映射多个 canonical source/key 时整组 quarantine。
- compatibility 面无协调写，只显示同一只读事务 canonical reread 得到的 source/status/priority reason/link/双 watermark。
- rollback 只可选择受守卫 legacy read-only display，或关闭 M4 ingestion/scheduler/read projection；保留 M1/M3 guards、M4 committed coordination/event/audit/receipt/quarantine/report versions。不得反写 owner、重放 effect、恢复固定 cwd 权限或物理删除旧面。

## 8. 下游交接与未进入

- M5：唯一拥有 ProjectSummary；当前 source 为 `HOLD / UNAVAILABLE`，M4 只消费未来可用 typed ref。
- M6：唯一拥有 Global Supervisor consult success；普通产品 M4 recipient 当前显式 unavailable，M4 只创建/消费 M3 Handoff receipt。
- M7：消费 `DailyWindowClosed` / `DailyReportVersioned` 并创建自己的 artifact/annotation；M4 不写 FormalMemory/PersonalFact/个人模型/Skill，M7 不原位修改 M4。
- M8：真实 connector、credential 与 external source facts 未进入；M9 command unregister/物理退役、M10 真实全日试点/发布也未进入。
- 未进入的操作还包括真实个人资料、真实用户项目写入、真实模型/provider、真实 Codex 消息、真实账号/凭据、网络外部写入、真实迁移、push、merge、rebase、部署与发布。

具体 typed fields、idempotency/join key 和接手顺序见 `handoffs/2026-08-10-syn-m4-to-m5-m6-m7-handoff-v1.md`。

## 9. Sol Ultra 最终审计与证据强度

本次 stage-level 审计由新接任的 GPT-5.6 Sol / ultra 主管在 C10 候选冻结后执行，独立于 M4C01–M4C08 的原施工线；逐项核对 stage diff、C01–C10 提交、合同 hash、完整红绿测试、C09 JSON 与文档边界。对本任务自己完成的 C09/C10 代码，它是分离审计回合，但不是另一位 reviewer 或第二模型。

另行第二会话在生成前命中账号用量限制，Oracle 当前 CLI/浏览器路径也没有形成可归因的 GPT-5.6 Sol / ultra 输出；这些尝试不计入审查通过证据。总线主管后续复核是额外独立证据，不改变本报告对已运行命令和 receipt 的事实记录。

| 级别 | 结论 | 说明 |
| --- | --- | --- |
| P0 | 0 | 未发现数据破坏、权限放宽、owner 反写或真实外部 effect |
| P1 | 0 | 5 个 launcher 静态回归已在 C10 修复并由旧/新聚焦与完整套件关闭 |
| P2 | 0 | 当前 M4 变更未留新 P2；既有 Rust warning debt、Vite chunk warning 与未实现下游均作为边界/HOLD 记录，不伪装成完成 |

## 10. 收口与停止点

current-state、master、M4 计划、plan index、task queue、C10 report 与 M4→M5/M6/M7 handoff 已按实际结果同步。C10 内容提交后，Harness 执行 M4C10 `done` 和 stage-06 `close-stage --write`，并把全部 audit/usage/archive 记录精确提交；最终只用普通 Git 核对洁净。

M4 收口后停止等待总线主管复核。当前没有自动下一包，也没有 push、merge、rebase、部署或发布。
