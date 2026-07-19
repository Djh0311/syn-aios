# S1C 交办布局测试翻案预登记 v1

日期：2026-07-19  
任务包：`tasks/2026-07-19-s1c-jiaoban-layout-conversation-left-history-as-proposal-index-package-v1.md`

## 必须翻案

1. `tests/jiaoban-merged-layout.test.tsx`：旧断言锁「工作历史 → 交办主区 → 画布」DOM 顺序和左栏措辞；改锁「交办对话 → 历届方案索引 → 方案/交货实体」及 240px 中列、577px 同序纵排。
2. `tests/jiaoban-history-and-secretary-board.test.tsx`：旧断言锁“工作历史”和单行“状态点+目标+时间”；改锁“历届方案”、目标一句+九态人话状态+日期，以及「旧单·无方案记录」诚实兜底。
3. `tests/jiaoban-conversation-center.test.tsx`：旧历史行 `aria-controls` 指向对话短讯锚点；改为指向右区 proposal/delivery tabpanel，并保留交货单选择分支。

## 必须保持

- `JiaobanConversation` 消息内容、分组、常驻输入框与发送命令断言不改。
- 方案/交货四视图内容、批准动作、工作流入口、九态来源与筛选判据不改。
- 历史点击不得触发对话滚动；删除旧对话锚点耦合后，不用新副作用替代。
- 本包不删测试组；若断言变化超出上述三类，先停下说明。

## 收口

上述三类翻案已按预登记完成，测试组未删；`JiaobanConversation` 消息内容、方案/交货卡内部、批准动作与数据/九态来源保持。离线与浏览器实渲结果见 `evidence/2026-07-19-s1c-jiaoban-layout-render-verification-v1.md`。
