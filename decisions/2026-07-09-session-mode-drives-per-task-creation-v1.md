# 决策:会话创建策略由「会话模式」驱动(C1 canon 补充)v1

日期:2026-07-09 · 拍板:用户口述 canon + 主导线核代码落地 · 上承:C1 定稿 `decisions/2026-07-08-phase-b2-execution-loop-final-v1.md`(会话跟任务走)。

## 拍板(用户 2026-07-09 原话意)

> 「如果选用现有的对话,就需要给每个任务人工指定对话;如果开新对话就不用管。」

落成 canon:

- **「开新对话」模式 = 全自动 = C1 每任务各开新会话**(免管;这是自动推进/[接着跑]这条免管路的天然模式)。
- **「用现有对话」模式 = 全手动 = 每任务由用户人工指定会话**(不掺进自动推进;是手动挡)。
- **两模式彻底分家**:自动推进这条免管路 = 只在「新对话」模式存在;「用现有」不是「批一次自动跑」,是逐任务手动指定。

## 代码推论(主导线核过·2026-07-09)

- **独立自动推进命令** `AutoAdvanceAuthorizedRoleLoopRequest`(director_agent.rs:1538)**无 `session_choice` 字段** → 天然=新对话免管路 → 应 C1 每任务新会话;现状跑在预绑会话(拐杖)= 待接。
- **合流命令**(run_confirm_and_start_authorized_run_inner)**带 `session_choice`**:`new`=先生后绑(现状建一条)、`existing`=手动挡。按 canon:`new` 分支走 C1 每任务、`existing` 分支保持手动挡不自动建。
- **护栏含义(防一锅端)**:
  - **留(不改)**:合流 5 处 `PanicJiaobanSessionCreator`(existing 手动挡 / 拒绝路径 / path-lock,lib.rs:7009/7066/7100/7311/8647)+ auth 授权闸(auto_advance_rejects_without_active_authorization)+ path-lock(auto_advance_blocks_non_test_project)——这些测的是手动挡/拒绝/安全闸,C1 后照样成立;**放开=误伤(7311/6689 是高危#1 path-lock)**;
  - **更新(随 C1)**:只有断言「自动路跑在预绑会话(拐杖)上」的测试(auto_advance_runs_chain_when_authorized_and_bound / stops_at_needs_binding_when_unbound 等)随 C1 改;
  - **逐个判**:每处 lib.rs 测试改前判「它钉的是拐杖 还是 守卫」——拿不准=停手报回,不许按"退役语义"一锅端(执行线首版误诊教训)。

## 生效

即日;auto_advance 接 C1 的外科手术包据此拆(`tasks/2026-07-09-auto-advance-connect-c1-surgical-v1.md`·独立[接着跑]已落地核过 commit 8de5a7a)。

## 补拍(2026-07-09·合流-new)

用户拍 **A**:合流命令 `session_choice=="new"` 也走 C1 每任务(退掉「合流开头建单条 S0」用法·**先生后绑机制保留改每任务用**);`existing` 手动挡不动。收官包 `tasks/2026-07-09-jiaoban-new-retire-s0-to-c1-per-task-v1.md`。**注**:此举退的是 2026-07-05 先生后绑决策里「合流一次性建 S0」这个用法(单 S0 → 每任务),先生后绑机制本身沿用;那份决策相应过时点在本包收口时回写。
