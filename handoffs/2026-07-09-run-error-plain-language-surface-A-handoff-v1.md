# 回交:A·运行错误上脸(人话诊断层·C6 观测补强)· 执行线 → 主导线 v1

日期:2026-07-10 · 包:`tasks/2026-07-09-run-error-plain-language-surface-A-v1.md`。**子线未 commit。** 轻档。

## 一句话结论

fix8「只认供给类」的翻译层推广成**七族全谱分类器**(新模块单一真源·结构化 `{family, human, raw_snippet}`),接到 run-history 详情位,前端失败单出**人话摘要+族标+下钻原文**两层脸——替掉原来「跑挂了(去工作流看详情)」死胡同。收编散在四处的翻译器成一套(判据单一真源·humanize×2 也顺手消)。**冻结核 byte-0-diff、director retry 两个承重信号不断、成败判定一字未动、前端只碰允许文件。** 全量 764/0/43。

## 1. 落点清单

**新模块 `run_error_translation.rs`**(核心·纯函数零副作用):`classify_run_error(raw) -> RunErrorHuman{family, human, raw_snippet}` 七族+unknown(判据具体优先·大小写不敏感·**下划线归一**识别 dispatch.failure_reason 的压缩形);`classify_provider_failure_human`(fix8 供给类判据**整段搬来·字面一字未改**=单一真源);`humanize_error_for_display`(收编 humanize×2·unknown 回退原文保老语义)。七族:①provider_unavailable ②network ③timeout ④sandbox_denied ⑤command_failed ⑥codex_subsystem ⑦readback_failed。

**收编(单一真源·grep 自证零残留散翻译器)**:
- `codex_local_runner.rs::classify_codex_provider_failure`(:386)→ **薄委托**到新模块(签名/前缀/调用点 :364/:373 + resume 分类器 :226 全不动·供给类前缀照旧产出);
- `append_stderr_tail`(:409)→ 原来吞真相(只贴裸 stderr)改经 `classify_run_error` 翻人话,**base 打头的承重前缀不碰**(`consult_last_message_read_failed:` 保留·director retry 照旧读到);
- `secretary_agent.rs:187` + `global_supervisor_agent.rs:411` 两处逐字节重复的 `humanize_consult_error` → 薄委托新模块(**不动 director retry 读法**·§2.1(b) 边界内);
- **残留自证**:`grep 'strip_prefix("codex_provider_unavailable:")'` 生产码去掉新模块 = **0 命中**(前缀剥离逻辑单一真源);`classify_codex_provider_failure`/`humanize_consult_error` 剩下的全是「委托/取 human」调用,无散的第二套。

**run-history 详情位**(`run_history_read_model.rs`·纯只读投影):`ChainLite` 加 `failure_raw`(失败节点 `message` + 按 `dispatch_id` 关联 dispatch 的 `failure_reason` 合并·**只对失败/中断态抠·completed/running 不抠**);`RunHistoryEntry` 加 `error: Option<RunErrorHuman>`(仅 `state=="blocked"` 时 `classify_run_error` 翻译)。**成败字段 state/state_note 一字未动**(单测逐条断言)。

**前端两层脸**(`ProjectJiaobanPanel.tsx` + css + types):`JiaobanHistoryDetail` 失败单默认显 `historyErrorFamilyLabel(family)` 人话族标 + human 摘要;`<details><summary>查看原文</summary>` 下钻 raw_snippet(**原生 details·零 hooks·避 harness 平铺坑**);成功单不渲染错误区。呈现不阻断(黄牌哲学)·不露 family 机器键。

## 2. §4 证据

- **分类器单测**(run_error_translation·7):七族各一命中样本 + 结构化三段齐 + 原文必带 + 大小写不敏感;unknown 兜底(带原文·不装假人话·空输入不炸);下划线压缩形识别;子系统优先于命令失败(不被泛族盖);state-db 只读归 sandbox;humanize 收编语义(前缀取内嵌人话/unknown 回退原文/可识别错误翻人话)。
- **retry 不回归**:`fix8_classify_codex_provider_failure_hits_and_misses`(供给类分类器薄委托后行为不变)✓ + `jiaoban_director_plan_no_retry_on_provider_unavailable`(供给类仍不 retry)✓;secretary/global humanize 老测试(前缀剥离+额度用完人话)逐条绿。
- **族④一致性**(§4 硬要求):`a_sandbox_family_agrees_with_existing_state_error_detector`——同一 state-db 只读样本,现成 `classify_phase_b_stderr_for_codex_state_error` 判 true、A 分类器归 `sandbox_denied`,一致(证没造矛盾探测)。
- **接线测**(run_history·3):失败链→`entry.error` 投影 {人话/原文/族} 且 state/state_note 不变(07-08 memories 活证据形态·归 codex_subsystem·**默认脸不灌裸错误·原文藏 raw_snippet**);完成/跑中单 error=None;`project_chains` 只对失败态抠 failure_raw(含节点 message + dispatch failure_reason 更丰富原文)。
- **离线 DOM**(jiaoban-history 第 6 组):失败单显族标人话(不露 `codex_subsystem` 英文键)+人话摘要+「查看原文」下钻带原文+默认脸主体不灌裸 stderr;成功单不显错误区。offline 全套 15 passed。
- **三闸**:tsc 绿 / offline 绿 / build ✓。**全量 `cargo test --lib` = 764/0/43**(基线 753 + A 新增 11:分类器 7 + run_history 3 + 一致性 1;计数不降·0 失败)。
- **真跑**:timeout/子系统错难真造,如实标注 **stub 级为主**——分类器对 07-08 memories 原文「no such table: jobs」的翻译由单测钉死(族⑥·`#[ignore]` 端到端真跑「顺带核」按包 §4 口径不硬造);真机=用户日常遇失败自然看人话上脸。

## 3. 0-diff 自证(§3 精确重划)

- **冻结核 byte-0-diff**:`git diff codex_local_runner.rs` 的 +/- 行里 `command_plan_for`/`run_phase_b`/`run_real_codex_process`/`RealWorkflowNodeCodexRunner` **零命中**;diff 只两个 hunk——`readonly_codex_consult` 内的报告层两函数(`classify_codex_provider_failure` 委托 + `append_stderr_tail` 路由)+ 我的测试。沙箱/闸/进程本体/成败判定一字未动。
- **前端范围**:`git diff --name-only src` = 只 `workflow.ts`/`ProjectJiaobanPanel.tsx`/`projectWorkflowSidePanel.css`(全在允许清单)。
- **改动面**:新 `run_error_translation.rs` + 6 后端(runner 报告层/registry+1/run_history/secretary/global 各薄委托)+ 3 前端 + 1 测试。

## 4. ⚠️ fmt 报备(重要·踩了 cargo fmt 全库坑)

- **A 的所有新文件/新行 cargo-fmt-clean**:权威 `cargo fmt --check` 对 `run_error_translation.rs`(新)/`run_history`/`secretary`/`global`/`command_registry` + codex_local_runner **我改的行**——**零 flag**。
- **踩坑并已纠正**:我先跑了裸 `cargo fmt`(全库),它顺手重排了 **A 完全没碰的预存 fmt 债**(`codex_db.rs` 9 处 / `mcp/storage.rs` 1 处 / `codex_local_runner.rs` 的 164/216/336/1790 四处·后者是 RealWorkflowNodeCodexRunner 签名等 runner 执行路径)。**已 `git checkout` 撤回这些越界重排**——保 A 的 diff 只碰报告层、守「runner 冻结核零命中」。(记忆 `rustfmt-递归` 坑坐实:全库 fmt 会碰 mod 子文件预存债。)
- **权威 `cargo fmt --check` 现仍 flag 那些预存债行**(codex_db/mcp/storage/codex_local_runner 的 4 处 164/216/336/1790)——**全是 A 不碰的区**,A **故意不 reformat**(reformat=越界 diff 碰 runner 适配器+两个无关文件·违 §3 冻结核与 §2.4 边界)。这些 fmt 债该另起卫生包清,不该混进 A。
- **另注**:`rustfmt --config skip_children` 的 ad-hoc 检查对 run_error_translation 报 9 处「diff」= 配置与项目 cargo fmt 不同的**假阳性**(包 §4 正为此警告「别 ad-hoc rustfmt·用权威 cargo fmt」)——以 cargo fmt 为准,A 文件全净。

## 5. §8 归后续(按包边界·本片没做)

彻底删 `codex_provider_unavailable:` 前缀 + director 1384/1396 改读结构化 `family` 的 retry 契约重构——**归 §8 单独一步**(本片保前缀=承重信号不断);本片只把「前缀判据/剥离」做成单一真源。

## 6. 回交动作

§4 证据 + 七族+unknown 覆盖 + 冻结核 0-diff 自证 + 收编落点(runner 委托/humanize×2 已消/grep 残留 0)+ §4 fmt 报备 → 主导线核实物。**子线不 commit。**
