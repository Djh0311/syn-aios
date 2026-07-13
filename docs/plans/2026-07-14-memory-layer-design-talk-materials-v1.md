# 记忆层设计谈话材料 v1(勘察实据+六块板·总指导附推荐·决定权在用户)

日期:2026-07-14 · 实据来源:同日记忆层全景勘察(只读·file:line 全核)。

## 〇、定调事实

机器面完成度**远超 canon 印象**:九种生命周期操作全建(`formal_memory_lifecycle.rs:345-387`)+UI 可达(`MemoryDetailPanels.tsx:34-88`)+审计三件套全覆盖;双召回通道(刀B top5+任务包 packet)、8 店数据面、迁移六处一致均在。**真空洞=使用面**:live 正式记忆 1 条(06-17 残留)、九操作零真实执行、**转正入口死码**(`DailyMemoryCandidateInbox.tsx` 121 行无挂载=「转正加餐」挂账实体)、lint 仅手动按钮、捕获钩子 3/4(operation_control 没接线)。

## 一、六块板(利弊详见勘察原文·此处按拍板格式)

**板1·双召回要不要归一**(P2:top5 绕过 lint/过期/冲突,`consultant_agent.rs:233-261`)
a) 刀B 改走 packet builder=单一真源但坏店可挡咨询,违"召回是增益"哲学 / **b) 抽最小共享 filter(lint blocking+valid_until+conflict)两面各调=改动小保哲学,双轨仍在** / c) lint 升级为写回记忆状态=召回零改但 lint 从建议变权力,违 canon。
**推荐 b,排第二刀**(第一刀不动召回结构)。

**板2·转正加餐落法**
**a) inbox 复位进记忆中心(挂现成 121 行组件+后端命令+确认对话全存在)=几乎零新码清挂账** / b) 捕获侧密度闸(相似合并/冷却)=治淤积根因,但碰 capture bus=06-25 决策"另议"件,需一并重拍 source_type 双词表 / c) 秘书周报拉人清=零机制但吞吐不变。
**推荐 a 进第一刀;b 等真淤积(拉动式)。**

**板3·召回质量**
**a) token 估算 CJK 修(`task_memory_packet_builder.rs:502-504`,中文低估≈4 倍)=一行改,预算 8 条/2000 同步复核** / b) 加 recency+词法排序=新设计面 / c) 召回 v2 一刀=包大,违"疼一件做一件"。
**推荐 a 进第一刀;b 等使用数据。**

**板4·记忆切库(17 表)**
**a) 照旧锁+落注记「DB 记忆快照=历史导入·非镜像」(坑13)** / b) 记忆店纳入 db_primary(M4 引擎现成)=口径统一但 M6 前风险叠加+CURRENT 明锁需重拍 / c) reconcile 扩记忆表=巡检面变宽。
**推荐 a;M6 后重议 b。**

**板5·记忆中心 UI 范围**
**a) 小刀:inbox 复位+候选详情采纳按钮,布局不动** / b) 按 Phase D 整页重做=live 1 条记录即重做=设计过度 / c) 先清孤儿(522 行零 import 面板;RunningWorkflowsView=测试 fixture 慎动)。
**推荐 a 进第一刀;c 的清孤儿随第一刀顺手(死码删除+fixture 迁移另列)。**

**板6·边界与词表**
①注入面边界重申进新 canon(Syn 记忆入 worker 唯一合法面=任务包 prompt block;不触 `~/.codex/memories`)——零成本随档落;③「蓝图能力层」命名落档+consolidated-canon:103 老待办(CURRENT 入口指针)一并了结;**②source_type 双词表(capture 8 vs canon 11-14)第一刀不碰**,留给板2b 那刀(06-25 决策原话"真做精确化时一并理清")。

## 二、第一刀切片包(若按推荐拍)=「记忆环通血包」

inbox 复位挂载(记忆中心)+候选详情采纳按钮+token 估算 CJK 修+M1 假警示串清理(`formal_memory_store.rs:173/:197`"不含候选采纳"已是假话)+canon 入口指针补。**零新机器、零新 sidecar、零布局改**;验收=真机走通「干活→候选→inbox[属实,沉淀]→转正→下次召回带上」全环。

## 三、遗留债索引(不进第一刀,已记坐标待拉动)

P2 候选无密度闸(板2b)/P2 top5 绕 lint(板1b)/P3 排序无 recency/坑7 operation_control 没接线/坑9 成熟模式三名并立+黑板信封异构/坑10 source_type 双词表/坑11 schema 超前空表/坑12 lint 无自动触发/坑13 记忆 DB 快照漂移注记/坑14 物理共库靠读侧过滤。
