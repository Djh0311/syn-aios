# 决策:交办「开个新的」= 先生后绑(birth-before-bind),execute 的 resume-only 不反转 v1

日期:2026-07-05 · 用户开工令:「开a,卫生包并行」· 主导线拟(路线选择,守全部旧闸)。

> **过时点(2026-07-09·C1 架构收官)**:本决策的**先生后绑机制沿用**(C1 每任务各建会话正走它);但「合流-new 开头建**单条 S0** 引导会话 → 整链跑在这一条上」这个**用法已退役**——合流-new 现每任务各建(C1)。prepare 改 C1-aware 后不再需要 S0 那条预绑(见 `decisions/2026-07-09-session-mode-drives-per-task-creation-v1.md` + `tasks/2026-07-09-prepare-c1-aware-close-c1-v1.md`)。机制在、单 S0 用法退。

## 背景

- 交办授权卡的「用哪个对话干:开个新的」自 fix2 起置灰「下一阶段支持」;刀1 合流对 `session_choice=new` 清错拒。
- 根因不是没能力:runner 层 `new_session` 操作**存在且 06-21 验过通**;卡点是**链的执行路径** `execute_project_workflow_node_at` 写死 `operation_id="resume"`(commands.rs·死线文件),让链直接 new_session = 改执行闸所在文件 = 高危#3 边缘。

## 拍板

**先生后绑**:合流绑会话步遇 `session_choice=new` 时,先经**现成的 manual_relay GUI 直发 new_session 单次路径**(`run_manual_relay_gui_direct_new_session_once`——工作台"唯一真能指挥 codex 的路径"、自带闸与回执)在**固定测试项目**里真建一条会话(带一条初始化消息),从回执取 thread_id,**随后走现有绑定机器绑到节点**;链照旧逐任务 **resume**。

## 为什么是这条路(不是别条)

- **execute/commands.rs 0-diff**:resume-only 的老语义(P3 C)**不反转、不绕**——我们没让链 new_session,是"先建好再 resume"。
- **不造第二套会话创建**:复用 relay 已验机器(沙箱限项目/拒审批绕过/回执),不新开闸。
- 代价:多一次真 codex 调用(初始化那一下,~15-60s)——换来的是零死线改动,值。

## 边界

- 仅交办合流路径、仅固定测试项目(path-lock 在)、仅用户点[允许并开始]的直接效果(人闸)。
- relay 路径若有本设计无法诚实满足的门(如在场闸字段),执行线**停、回主导线**,不许伪造。
- 非测试项目/自由会话(无 work_item)/自动批准——全不放开。
