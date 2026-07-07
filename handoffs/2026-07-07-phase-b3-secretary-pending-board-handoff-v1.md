# 回交:B3·秘书「待你拍板」汇总面(确定性秒出+按需解释)· 执行线(单线双面)→ 主导线 v1

日期:2026-07-07 · 包:`tasks/2026-07-07-phase-b3-secretary-pending-board-v1.md` · 决策第 2 条 + 架构 §7。**子线未 commit。** Phase B 收官片。

## 一句话结论

秘书面接上「待你拍板」清单(待批方案/全局主管提醒/记忆候选三组·全确定性读盘零 LM·秒出)+ 按需 [让 AI 解释现状](唯一烧额度处·会话内缓存);**接的是现成骨架**(derive 加法两输入、Brief 扩块),秘书全程零写入(连自己的 store 都没有)。App.tsx 窄口 **+10/-2**(±20 达标)。三闸绿、704/0/41、死线全 0-diff。剩真机(§5)。

## 1. 落点清单

**后端(薄·两条命令)**:
- `global_supervisor_review_store.rs` **只加** `load_global_supervisor_review_store`(照 load_formal_memory_store 家族先例·soft 语义损坏空店不炸·store 本体语义 0-diff——diff 里仅此一个函数);
- **新** `secretary_agent.rs`:档案常量(**§7 职责+禁区逐条原文**:不绕确认改事实/不绕主管操作项目/不绕权限读私密/不写长期记忆/不替代审计中心+「你整理和解释,不判断不裁决不派活」)| `run_secretary_explain`——**输入全后端盘读**(pending 方案=proposal store、主管两类意见=review store soft、记忆候选计数=memory candidate store,全现成 loader;每路 best-effort 读不到就在 prompt 里老实注明)→ `readonly_codex_consult`(cwd=固定测试根·形式锚)→ **返回纯文本**(无 json 契约无解析步);**零持久化**(解释即抛·不落盘不写审计——秘书不写为原则);失败照 fix8 剥前缀人话、不 Err 断面板;
- registry:`mod secretary_agent` + 2 注册(借道先例·lib.rs 0-diff)。

**前端**:
- `secretaryReadModel.ts`:derive 输入**加法**两可选参(`proposalStore`/`supervisorReviewStore`·旧调用不炸);新派生块 `pending_board`——①待批方案(status=pending_user_confirmation·非今天生成标「N 天前生成的旧方案」·口径照批卡 stale 日历日判据)②主管提醒(结果复核 `needs_human_check`/`human_verify` 带 human_note **首句** + 批前 `mismatch` 带 summary 首句;**caution 刻意不入**[批卡提醒过·进秘书面=噪音]、unavailable 不入)③记忆候选**引用现有 pending 计数**(不重复算·>0 才一条聚合条目);每条带文字去处提示(「在交办页批」「在记忆中心处理」——不做跳转接线);**现有字段(risk_signals/suggestions/global_summary/context_id)语义 0-diff 只加不改**;
- `SecretaryBrief.tsx`:顶部「需要你确认」计数并入两组新路(记忆候选原计数已含·不重复加);`SecretaryPendingBoardSection`(export·无 hooks)三组渲染/空组不渲染/全空「桌面干净,没有等你的事」+「这些是提醒,不是命令」边界话;`SecretaryExplainSection` 四态(idle/loading「约 1-2 分钟」/文本+[重新解释]/失败人话+[重试])·**模块级会话缓存**(面板关了重开不重烧·再点才重跑);
- `App.tsx` 窄口(**+10/-2 行**·git diff --stat 自证):import×2、新 state 1(带注释 2 行)、`reloadCandidateStores` 加第 11 店(Promise.all/解构/setState 各 1)、derive 喂 2 参+deps 2 项——**别处零碰**;
- `RightDetailPanel.tsx` **最终零改**(Brief 内自含解释按钮,包允许碰但没必要);css 进 styles.css secretary 族(带 min-width:0——fix9 溢出教训预防);types/tauri.ts 加法。

## 2. ⚠️ 过程要事(2 条·透明报备)

1. **既有离线 harness 撞 hooks(修法=产线侧最小规避·非 harness 手术)**:`offline-permission-dialog.test.tsx` 的 `findElement/renderComposite` 把一切 function 组件**当普通函数平铺调用**(ProjectJiaobanPanel 注释点名过的限制)——我把带 useState 的解释区嵌进 Brief 后,既有右栏场景遍历树时炸 `Invalid hook call`。修:**`React.memo` 包 `SecretaryExplainSection`**——memo 元素 type 是 object 非 function → harness 当叶子跳过;真渲染(Tauri/renderToStaticMarkup)完全不受影响。组件注释写明 memo 是**必要而非性能优化**。既有测试文件一行未动(§2.5 边界守住),既有场景回归全过。
2. 我自己 derive 测试的词表断言首版误伤(JSON.stringify 连内部 entry_id 一起查,fixture 名含 mismatch)——已修为只查呈现字段(title/detail/where_hint),语义更准。

## 3. §4 机器证据

- **后端单测**(secretary mod·2/2 绿):`explain_prompt_grounded_from_disk_and_caution_excluded`——**真 store API 造盘上事实**(bootstrap+create_proposal+upsert_review/upsert_boundary_review),stub consult 收到的 prompt 逐项断言含盘上方案标题/human_note/mismatch 摘要 + **§7 禁区原文在 prompt 里** + caution 不进提醒;`provider_failure_humanized_and_empty_stores_soft`——供给类剥前缀人话/空店零炸/空回包→unavailable 可重试。**load 命令往返**由 store 既有 roundtrip+legacy 测试覆盖(soft loader 同一条路)。
- **前端 derive 单测**(新 `secretary-pending-board.test.ts`·4 组):三组入选/排除判据(pending 入 confirmed 不入·needs_human_check/human_verify 入 pass/unavailable 不入·**mismatch 入 caution 不入**)·旧方案标注·首句截断·缺参/显式 null 零炸·**现有字段回归断言**(source_kind/数组/只读警示)·呈现词表无枚举原文;
- **离线 DOM**(新 `secretary-pending-board-face.test.tsx`·4 组):三组渲染+去处提示+边界话/空组不渲染/全空文案/词表(无「审批」无枚举原文无黑话)。offline 全套 **15 passed**(含既有右栏秘书场景回归=memo 方案实证)。
- **全量**:`cargo test --lib` = **704/0/41**(基线 702+2;计数不降)。三闸:tsc 绿/offline 绿/build ✓。fmt:3 个 rs 文件 skip_children check CLEAN。

## 4. 0-diff 自证(§2.5 全名单)

改动面=允许名单精确吻合(git status 全清单在此):新 `secretary_agent.rs`+2 测试文件;M = readModel/Brief/App(窄口)/tauri.ts/types/styles.css/registry/store(仅 load)/跑器(+2 行)。死线逐一 `git diff --stat` 空:**global_supervisor_agent(B1/B2 本体)/ director / consultant / c4_c6 / controller / commands / runner / control_core / worker_report / manual_relay / lib.rs / ProjectJiaobanPanel 全 0-diff**。安全死线:秘书零写入(后端 grep 自证:secretary_agent 无任何 store 写调用·唯一 IO=四路只读 load)/意见零驱动/RightDetailPanel 零改。

## 5. 真机待验(§4·用户)

1. 打开秘书面板 → 「待你拍板」秒出,与交办页实况对得上(盘上 pending 方案/主管提醒条目一致);
2. 点 [让 AI 解释现状] → 「秘书正在整理解释…(约 1-2 分钟)」→ 人话解释上脸;关面板重开 → 解释还在(缓存)·[重新解释] 才重跑;
3. 全空时显「桌面干净,没有等你的事」;
4. 断供一次 → 失败一行人话+[重试],别的区不受影响。

## 6. 回交动作

§4 证据 + App.tsx 窄口 diff 行数自证(+10/-2)+ 落点清单如上 → 主导线核实物。**子线不 commit。**
