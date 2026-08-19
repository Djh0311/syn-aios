# M6D05 stable member directory candidate and evidence v1

状态：`SUPERVISOR_SELF_REVIEW_PASS / CP3_SEGMENT_CONTINUES / DOMAIN_RUNTIME_ONLY / NOT_RELEASED`

## Harness

- stage：`stage-15`；leaf：`M6D05 稳定成员目录（ORG-005，域层）`；执行来源 receipt：`u-bf5cbe4117b9c087c7f1`。
- 内容候选为 `a58815ff02b912003de8abcf84507c43ad7245dc` / tree `bdd45a1ca82eb85c9ce242f7b493ce72e339365f`；parent 为 `f920dff115f7b9daeac814a51f2ae6c1597c30ec` / tree `1b119c30525505b903eaf5b5e559d768dbd0f2a8`。
- 用户已明确改定：Grok 是优先实现者而非唯一写者，无人值守完成整个 stage-15 是主目标。对 Grok 的本叶产品派发因私有仓库代码出站策略被系统拒绝，未产生 Grok 修改；同一长驻 Codex 随即在 current leaf 精确写域内接管，全程只有一个产品源码写者，内容提交按实际贡献只署 Codex。
- 候选精确包含 6 个允许产品路径：新增 `m6_org_member_directory.rs`；修改 `m6_org_schema.rs`、`m6_org_consult_handoff.rs`、`commands.rs`、`command_registry.rs`、`lib.rs`。没有修改 M3/M5 冻结实现、合同正文、前端或 manifest。
- 本叶已按七项判据自复核放行并归档。同属 CP3 段的 M6D06 随后成为唯一 current leaf；M6D06 收口前不交 CP3，M6D06 收口后必须关闭 authorization、交 M6D05+M6D06 包并前台阻塞运行独立 Cursor Opus 验收，PASS 前不得进入 M6D07。

## 产品

- 新增持久化稳定成员目录，以 `MemberId` 和唯一 explicit identity contract 建立身份；display name 仅为标签，同名注册产生两个独立成员。成员记录不含 provider、model、thread、process 身份字段，且 `temporary_agent_*`、provider/model/thread/process/session/child-run 命名空间不能注册为 stable member。
- `AGENT_CENTER_SESSION`、provider thread、runtime child、session count 与 display-name match 五类 heuristic 输入只生成 `REF_ONLY` quarantine，`mapped_to=null`，不创建 identity/history 行。请求 DTO 使用 `deny_unknown_fields`，伪装附带 provider/model/thread 字段的输入在反序列化阶段拒绝。
- membership lifecycle 以 append-only revision 实现 establish/update/activate/deactivate；scope、role、memory、contact 与 capability/permission refs 只追加或保留，停用不物理删除。revision 在事务内再次 CAS，避免并发陈旧写覆盖。
- capability/permission 只接受 `source + revision + observed_at` 的只读 ref，并强制 `directory_is_authority=false`、`read_only=true`。目录写前后既有 Global Supervisor 项目写授权判定保持相同 fail-closed 结果。
- availability 保存 source、source revision、observed time 与有上限 TTL；`now > observed_at + ttl` 时 effective state 强制为 `unknown`，所有响应 `authorizes=false`。按 capability 查询时只有同时具备只读 capability ref 与新鲜 `available` 观察才返回；陈旧观察不会参与能力查找，更不会变成 permission。
- contact 只从已有显式 contact binding 创建 M3-owned Handoff，并在 M6 保存 `capability_granted=false`、`project_writeback=false` 的引用回执。它复用普通 Secretary RoleSession 与 M3 Handoff repository，不调用 provider/model/message/connector，不接触项目 store、projection、root、outbox 或文件写面。
- export 包含成员 revision history、availability、contact refs 与 quarantine；普通生产导出路径会先在内存 M6 store 中真实 restore/rebuild，并核对每个成员最新 revision，重建后 `MemberId`、identity contract、memory refs、permission refs 与历史保持不变。
- 普通生产调用链为 Tauri `generate_handler!` → `commands.rs` 七个真实 command → `m6_org_member_directory` → M6-owned SQLite；contact 分支再进入 `m6_org_consult_handoff::start_member_contact_handoff_for_state` → M3 Handoff owner。七个 command 均进入普通 registry，不是 `#[cfg(test)]` 或 fixture-only 路径。
- `m6_org_schema` 从 v2 迁移到 v3，新增 stable identity/history、identity quarantine、availability history、contact receipt 与 command receipt 表；既有 advisory/decision/Handoff 表和语义未改。`lib.rs` 只增加模块声明；`command_registry.rs` 只增加叶子点名的七个精确 command。

## 证据

原始证据根：`/home/synadmin/workspace/.syn-gates/evidence/M6D05-a58815f/`。全部命令在绑定候选 `a58815f` 的 detached disposable checkout 上执行，只使用 fake Secretary/provider/runtime、隔离 app-data 与合成 identity/availability/contact 输入。`exit-codes.tsv` 保存每条命令的真实退出码。

- `cargo test --lib --offline m6d05_ -- --test-threads=1`：exit 0；7 passed / 0 failed / 0 ignored / 2142 filtered。覆盖 explicit identity、同名不合并、五类 heuristic quarantine、temporary/stable 分型、append-only lifecycle、目录非授权源、TTL stale、伪 provider/runtime 替换、export/rebuild、M3 contact 与普通 command registry。
- `cargo test --lib --offline m6d04_ -- --test-threads=1`：exit 0；4 passed / 0 failed / 0 ignored / 2145 filtered。
- `cargo test --lib --offline m6d03_ -- --test-threads=1`：exit 0；13 passed / 0 failed / 0 ignored / 2136 filtered。
- `cargo test --lib --offline m6d02_ -- --test-threads=1`：exit 0；15 passed / 0 failed / 0 ignored / 2134 filtered。
- `cargo test --lib --offline m4c05_ -- --test-threads=1`：exit 0；9 passed / 0 failed / 0 ignored / 2140 filtered。
- `cargo test --lib --offline m3c05_ -- --test-threads=1`：exit 0；43 passed / 0 failed / 0 ignored / 2106 filtered。五组相邻回归共 84/84，本叶连同相邻回归共 91/91。
- `cargo check --lib --offline`：exit 0；rustc 汇总 888 个既有 warnings，日志 `warning:` 文本行 889，与 CP2 接受基线一致；本叶没有把 warning debt 扩成清理任务。
- `git diff --check HEAD^ HEAD`：exit 0；`git-name-status.log` 精确列出上述 6 个产品路径；`frozen-contract-diff.log` 为空；验证后 `git diff --exit-code` 仍为 0，说明测试没有改写任何 tracked 候选文件。
- detached checkout 验证后唯一 delta 是 Tauri build 生成的 untracked `gen/schemas/linux-schema.json`，SHA-256 为 `7e51a7ed92547e6c96f8d37d0ff7de836e9ee5b6102b1c6ba06ae075207c2a15`，与主工作树受保护载体相同。临时 worktree 已用精确 `git worktree remove --force` 清理，仓外原始证据保留。
- `protected-wip-sha256.log` 固定 6 个受保护 `m6_*.rs`（含 `.bak`）和主工作树 `linux-schema.json` 的 7 个 SHA-256，逐项等于 CP2 基线；它们未暂存、提交、修改、清理、恢复或用作实现输入。
- 自复核期间发现对 `lib.rs` 运行 rustfmt 会递归触碰四个既有 tracked WIP 的 mtime。只读比对确认这四个文件的 blob 指纹与本叶开始前既有 WIP 指纹完全一致，字节未变，故没有执行覆盖式恢复；四者未暂存、未进候选。`commands.rs`/`lib.rs` 中由同次格式化产生的非本叶排版差异已逐块还原，最终候选只剩精确 command/module 接线。

主管七项判据：

1. 写域：候选只有 6 个 current leaf 明示路径；`command_registry.rs` 恰为七个点名 command，`lib.rs` 恰为一个 module 声明，contact helper 只接既有 M3 owner，没有前端、manifest、用户载体、后续 M6 叶或禁止域写入。
2. 冻结物：冻结合同 diff 为空；M3 Handoff、M4 Secretary、M5 ExecutionGrant/WorkerReport/receipt/audit/quarantine 与 guarded-legacy 语义未放宽，M3C05 43/43、M4C05 9/9、M6D02–D04 32/32 回归通过。
3. WIP 保全：7 个受保护载体哈希与 CP2 基线一致且未入候选；四个既有 tracked WIP 内容指纹不变且未暂存；其他 runtime usage/untracked 报告同样未归责、未清理、未进入候选。
4. 独立重跑：M6D05 7、M6D04 4、M6D03 13、M6D02 15、M4C05 9、M3C05 43 个测试和 cargo check/diff-check 均在 SHA 绑定的 disposable checkout 退出 0，原始日志留在证据根。
5. 实质：七个真实 Tauri command 进入普通 handler；CRUD/list/export 真实落 M6-owned SQLite，contact 真实落 M3 Handoff 与 M6 non-grant receipt，export 生产路径真实执行 restore/rebuild 校验，不是测试专用空转。
6. 不越级：证据只证明 WSL local/offline/synthetic 的 M6D05 域层与普通 Tauri composition；没有证明 GUI、新壳、真实人员资料、真实 provider/model/message/account、项目写、部署、发布、CP3 或 M6 完成。
7. 欠账：本叶标准内没有未满足项。888/889 warnings 继续归 ENG-01；renderer/new-shell consumption 归 M6S01；M6D06、CP3 与后续 M6D07/D08 继续按既定叶序执行，不由本叶提前承担。

## 载体

- 产品载体是候选 `a58815f` 的 Rust 域层、M3/M6 ordinary composition 与七个 Tauri command，不是正在运行的 GUI、新壳、真实 provider 集成或发布产物。
- 本报告、归档 leaf、stage/plan/current-state、audit 与下一叶 authorization 属 Harness 记账；它们不改变候选 tree，也不代替 M6D05+M6D06 的 CP3 独立 verdict。
- 当前结论为 `M6D05 SUPERVISOR SELF-REVIEW PASS / CP3 SEGMENT CONTINUES / NOT RELEASED`。
