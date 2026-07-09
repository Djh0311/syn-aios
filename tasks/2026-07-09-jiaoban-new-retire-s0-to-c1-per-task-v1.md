# 实现任务包:合流-new 退 S0·走 C1 每任务(C1 收官)· 主导线 → 执行线 v1

> **⛔ 已作废(2026-07-09)——本包前提错误,勿派。** 执行线实测:退 S0 撞 needs_binding 死结(prepare 要节点先绑会话才产 prepared·S0 是那条引导绑定·非多余会话)。主导线核实认账(是本包设计错·非执行线)。用户改拍 **3**:prepare 改 C1-aware。**取代包 = `tasks/2026-07-09-prepare-c1-aware-close-c1-v1.md`**(prepare 一 C1-aware·合流-new 退 S0 就干净了)。保留本文作教训存档(measure-before-guessing 又一例)。

日期:2026-07-09　性质:**轻档**(后端·测试项目圈内)。这是 **C1 收官最后一片**(用户 2026-07-09 拍「A」:合流「新对话」也每任务各开)。正本:canon 决策 `decisions/2026-07-09-session-mode-drives-per-task-creation-v1.md` + C1 定稿。

## 0. 接手须知(冷启即读·本包自包含)

- 你是**执行线**(后端)。**子线不 commit。** 全程中文。
- **背景**:C1 三条主生产路,直起链 + 独立[接着跑] 已 C1 每任务;**合流命令的 `session_choice=="new"` 分支仍是「先建 1 条 S0 → 绑 codex-dev 节点 → 在这一条上推进整链」(非 C1 每任务)**。用户拍 A:合流-new 也走 C1 每任务。
- **关键澄清(别理解偏)**:**退的是「合流开头建单条 S0」这个用法,不是退「先生后绑」机制**——先生后绑机制保留,只是从「合流一次性建 S0」改成「每任务各建」(跟直起链/[接着跑] 同源)。**existing 手动挡分支一字不动。**
- **主导线已勘的确切代码位(直接用)**:
  1. 合流 inner = `run_confirm_and_start_authorized_run_inner`(director_agent.rs:2185);`session_creator` 参已在(现用于建 S0);
  2. **S0 建+绑**在 `"new" =>` 分支(director_agent.rs:2311-2352):`create_initialized_session` 一次(2316)+ 绑到 `{workflow_id}:node:codex-dev`(2319-2348)+ new_session_notice(2349);
  3. **推进调用**(2373)现走 `run_auto_advance_authorized_role_loop`(**None 公有壳**·跑预绑);Some 变体 = `run_auto_advance_authorized_role_loop_with_session_creator`(1649·已由[接着跑]验证落地);
  4. `"existing" =>` 分支(2294-2309)= 手动挡绑现有会话,**神圣不动**;
  5. 那 4 个 `texts.len()==1` 测(lib.rs:7151/7240/7299/7394 区)断言「S0 出生口恰好一次」。

## 1. 拍板摘要

- **做什么**:合流 `new` 分支退掉单条 S0 建绑,推进改走 Some 变体(每任务先生后绑);`existing` 分支不动。
- **canon**:新对话=每任务各开(合流选「新对话」也是「开新对话」·三条路统一)。
- **为什么**:消掉「新对话」两义(合流一条 vs [接着跑] 每任务);C2 转发不用分叉判断。

## 一句话判据

**「是不是只:合流 `new` 分支删单条 S0 建绑、推进改走 `_with_session_creator`(每任务 C1);existing 手动挡/path-lock/reject/auth 守卫**一字不动**;那 4 个 `texts.len()==1` 测**逐个判**(拐杖→改每任务·guard→留);而 command_plan_for/runner 本体/manual_relay 本体/沙箱/安全闸 0-diff?」** 是 → 做;否 → 停、回主导线。

## 2. 建什么

### 2.1 合流 new 分支退 S0·走 C1

- `run_confirm_and_start_authorized_run_inner` 的 `"new" =>` 分支:**删掉** S0 单条建+绑(2311-2352 的 create_initialized_session 一次 + bind 循环)——不再开头建 S0;
- **推进调用**(2373):按 `session_choice` 分流——`new` → `run_auto_advance_authorized_role_loop_with_session_creator`(传 `session_creator`·每任务先生后绑·**复用[接着跑]已落的 Some 变体·别造第二套**);`existing` → 保持 `run_auto_advance_authorized_role_loop`(None·跑手动挡绑的那条);
- **new_session_notice**:改成 C1 语义的人话(或移除;每任务新会话的说明由 chain 侧已有机制给·别重复);
- **失败即停不回落**:每任务建会话失败 → chain 侧 fix3 留档停(首轮已立)·不静默回落 existing/拐杖。

### 2.2 那 4 个 S0 测·逐个判(不一锅端)

- `texts.len()==1`(S0 一次)这类断言在 C1 下变每任务 N 次 → **拐杖测·改成**「每任务建新会话·texts.len()==任务数·thread/target_session_id 互异」(可仿新的 `c1_auto_advance_new_conversation_path...` 或 `c1_chain_...` 范式);
- **但逐个判**:这 4 个里若有测的是**守卫**(如「new 会话建失败 → Err 不回落 existing」`confirm_and_start_new_session_failure_audits_no_fallback` 7208、或 path-lock/非测试项目拒),那类 C1 后**仍成立·保留或仅微调**(失败无回落在 per-task 下照样要成立);
- **判据**:每处改前问「拐杖(S0 一次性) 还是 守卫(失败无回落/path-lock/授权)」——**拿不准=停手报回**,别按"退 S0"一锅端。

### 2.3 明确不做

existing 的「逐任务人工指定会话」新交互(canon 提·手动挡独立功能·非本包)/ C2-C6 / fork。

### 2.4 文件边界

- **允许**:`director_agent.rs`(合流 new 分支退 S0 + 推进 mode 分流 + 自测)/ `lib.rs` **仅** 2.2 那 4 个 S0 拐杖测(**逐个判·守卫零改**·本包唯一 lib.rs 死线释放·范围锁死 S0 测);
- **0-diff**:`command_plan_for` 及 runner 本体 / `manual_relay.rs` 本体 / 沙箱 / 安全闸 / existing 手动挡分支(2294-2309)/ 合流 5 处守卫里的 path-lock(7311)·reject·auth / worker_report / 各 agent / 前端。

## 3. 安全死线

- **existing 手动挡分支(2294-2309)神圣不动**;**path-lock 守卫(7311)/auth/reject 神圣不动**(高危#1);
- lib.rs 释放**只限 S0 拐杖测**·守卫误伤=红线·拿不准停手;
- 先生后绑机制保留(改每任务用)·cwd 仍写死测试项目;memories 观察模式·不加旗;`.codex` 凭据不碰。

## 4. 验收

- **守卫回归(重中之重)**:existing 手动挡测 + path-lock(7311)+ auth + reject **全绿·git diff 证未改**;
- **新断言**:合流 `new` 3 任务 → 每任务建新会话(texts.len==任务数·thread 互异·target_session_id 互异物化)·existing 仍绑单条现有;
- **失败无回落**:per-task 建会话失败 → Err·不回落 existing(守卫语义 C1 下仍成立);
- **真跑**(`#[ignore]`·测试项目):一条合流 new 链端到端·codex home 见 N 条任务命名新会话·无孤儿 S0;
- 三闸绿 + 冻结核 0-diff 自证 + lib.rs diff **只含 S0 拐杖测**(逐处 justify) + 计数不降 + fmt **自己真跑 `rustfmt --check`·别自报**(前两轮前科)。

## 5. 回交

- §4 证据 + 守卫未改 git diff 自证 + lib.rs 改动逐处「拐杖 not 守卫」justify + 落点清单 → 主导线核实物。**子线不 commit。**

## 7. 不接受为

- 动 existing 手动挡分支 / 一锅端改那 4 测(含误伤失败无回落/path-lock 守卫) / 静默回落 existing/拐杖 / 留孤儿 S0(退就退干净) / 碰 command_plan_for·runner 本体·manual_relay 本体·沙箱·安全闸 / lib.rs 越出 S0 测范围 / 自报 fmt 不真跑(前科) / 做 existing 逐任务手动指定新交互(非本包)。
