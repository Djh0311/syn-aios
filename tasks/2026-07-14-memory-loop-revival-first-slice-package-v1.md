# 任务包:记忆环通血(L1 第一刀·零新机器)v1

日期:2026-07-14 · 档位:**轻档** · 拍板依据:`docs/plans/2026-07-14-memory-layer-design-talk-materials-v1.md`(用户按推荐拍:板 2a+3a+5a+6①③;板 1b 第二刀;板 4a 锁+注记)· 勘察正本:同日记忆层全景勘察(坐标全录,照抄别重数)。

## 一句话

让已建好的记忆机器**通血**:转正入口复活+采纳可达+token 估算修真+假警示清理+canon 指针——做完真机能走通「干活→候选→inbox[属实,沉淀]→转正→下次召回带上」全环。

## 交付项(五件,全是接线/一行修)

1. **inbox 复位**:`FE/components/DailyMemoryCandidateInbox.tsx`(121 行现成组件)挂载进 `FE/views/MemoryCenterView.tsx`(候选区上方或替换纯展示列表,布局最小动);App.tsx:357 分发器已存在、PermissionDialog 采纳文案已备(:1062-1063)——纯接线。
2. **候选详情采纳按钮**:候选详情面板补[采纳为正式记忆]动作(走既有 `adopt-memory-candidate-to-formal-memory` 命令+既有确认对话),含状态机合法性(needs_review→confirmed 白名单已在 control_core:677-704)。
3. **token 估算 CJK 修**:`task_memory_packet_builder.rs:502-504` `chars/4` → CJK-aware(CJK 字符≈1 token、ASCII≈4 字符/token 的分段估算);默认预算 8 条/2000 **不动**,回传里报修后典型 packet 实际条数变化(供后续重定预算参考)。
4. **假警示清理**:`formal_memory_store.rs:173`(无条件输出的 `m1_no_candidate_adoption_or_task_injection`)与 `:197`(display_text「M1 不包含候选采纳和任务包注入」)——采纳与注入均已存在,删除或改真话。
5. **文档三小件**:①`docs/memory-layer-consolidated-canon-v1.md:103` 老待办了结=CURRENT/AUTHORITY 补记忆层 canon 入口指针;②注入面边界重申一段(Syn 记忆入 worker 唯一合法面=任务包 prompt block·不触 `~/.codex/memories`)写进 consolidated-canon;③板 4 注记(「SQLite 内记忆表=07-13/14 历史导入快照·非镜像·记忆店仍 JSON 主写」)同档落。

**可选第六件(时间富余才做,单独回传项)**:删除零 import 孤儿 `FE/views/projects/ProjectWorkflowMemoryPanels.tsx`(522 行)——删前必须 grep 测试目录+离线测试引用确认零 fixture 依赖(RunningWorkflowsView 是 3 个离线测试的 fixture,**不许碰**)。

## 红线

1. 零新机器/零新 sidecar/零新命令;**capture bus/source_type 词表零碰**(板 2b/6② 明确留给后刀);召回两通道结构零碰(板 1b 第二刀);
2. 不碰 `~/.codex/memories`、不动 worker prompt 注入面;
3. 布局不重做(板 5a):inbox 挂载=区域内插入,不改导航/分栏;
4. 前端棘轮:offline-permission-dialog.test.tsx 已超水线**零加行**,新离线测试落新文件(<2000);styles.css 已超水线,样式尽量复用既有类;
5. live 根零写(采纳动作的真实执行留给用户真机验收那一下);不 commit;回传 10 项第 7 项 shape gate 必报。

## 验收(预写死)

- 机器面:离线测试覆盖 inbox 挂载渲染+采纳分发;token 估算单测(中英混合样例·修前后对比断言);`cargo test --lib` 基线只增不减;typecheck+offline-interaction 过;fmt 仅历史三;gate 14 零净增;
- **真机半边(完成≠收口)**:回传后由用户真机走一遍「候选出现→inbox 采纳→记忆中心见正式记忆→下一单召回带上」——**UI 铁律:真机过才算完成**;回传里写「已实现,待真机」,不许写「做好了」。

## 回传

10 项模板;附:inbox 挂载位置截图说明(文字描述挂哪)+修后 token 估算的典型 packet 对比数。
