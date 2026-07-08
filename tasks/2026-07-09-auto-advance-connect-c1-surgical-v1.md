# 实现任务包:auto_advance 接 C1(外科手术式·mode-aware)· 主导线 → 执行线 v1

日期:2026-07-09　性质:**轻档**(后端·测试项目圈内)。这是 **C1 的最后一根尾巴**(§8·③,首版执行线撞 lib.rs 停手报回·主导线核后重拆)。正本:canon 决策 `decisions/2026-07-09-session-mode-drives-per-task-creation-v1.md` + C1 定稿。

## 0. 接手须知(冷启即读·本包自包含)

- 你是**执行线**(后端)。**子线不 commit。** 全程中文。
- **背景**:C1 首轮已让**直起链**(start_project_director_chain)走 C1 每任务新会话;但**自动推进路**(独立[接着跑] + 合流命令的推进段)仍跑在预绑会话(拐杖)。本包接它。
- **首版为什么被打回**:首版执行线想让共享的 auto_advance 内层**无条件**建会话 → 撞了合流的手动挡/拒绝/path-lock 测试(它们传 `PanicJiaobanSessionCreator`·一建就 panic)→ 误判成「放开那 5 处退役语义」。**主导线核实:那 5 处是手动挡/拒绝/path-lock 守卫,不是退役语义,放开会误伤高危#1 path-lock。** 正解见下:**mode-aware**,不是无条件。

## 1. 拍板摘要

- **做什么**:自动推进这条免管路接 C1 每任务新会话;手动挡/拒绝/path-lock 路**原样不动**。
- **canon(决策正本)**:新对话模式=自动=C1 每任务;用现有=手动=逐任务人工指定(不掺自动推进)。独立[接着跑]无 `session_choice` = 天然新对话模式 = C1。
- **为什么外科手术**:共享内层无条件建会话会误伤守卫;mode-aware 让守卫全程保持绿。

## 一句话判据

**「是不是只:给自动推进路按『会话模式』分流——新/自动路传 C1 per-task creator、手动挡/拒绝/path-lock 路不建会话(那 5 处 PanicCreator + auth + path-lock 守卫**保持绿、零改**)+ 只更新真正断言『自动路跑在预绑拐杖上』的测试(逐个判)——而 command_plan_for/manual_relay/runner 本体/沙箱/安全闸 0-diff?」** 是 → 做;否 → 停、回主导线。

## 2. 建什么

### 2.1 自动推进路 mode-aware 接 C1

- **独立[接着跑]**(`run_auto_advance_authorized_role_loop` 系·director_agent.rs:1616/1645):现状无 creator 参、跑预绑会话。**加 session_creator 注入口**,每任务经现成先生后绑(`ManualRelayJiaobanNewSessionCreator`·cwd 写死测试项目)建新会话 → 绑 → dispatch 用新 thread → target_session_id 物化(**复用 C1 首轮已建的 `run_director_task_chain_with_session_creator` 那套·别另造第二套**);
- **合流命令推进段**:`session_choice=="new"` → 走 C1 每任务新会话(同上);`session_choice=="existing"` → **保持手动挡**(不自动建·沿用现绑定);path-lock/拒绝 → 一字不动;
- **失败即停**:建会话失败 → 该任务 fix3 留档停(人话「新建会话失败」)·**不静默回落拐杖/existing**(C1 §7 硬线·首轮已立);
- **别碰 runner/relay 本体**:先生后绑机器现成·只调不改。

### 2.2 测试处置(逐个判·防一锅端)

- **保持绿·零改(核对确认它们仍不触发 creator)**:
  - 合流 5 处 `PanicJiaobanSessionCreator`(lib.rs:7009 existing 手动挡 / 7066·7100 拒绝 / 7311 path-lock / 8647 fix3 拒绝)——mode-aware 后 existing/拒绝/path-lock 路仍不建会话·PanicCreator 永不被调·**必须仍绿**;
  - `auto_advance_rejects_without_active_authorization`(授权闸)/ `auto_advance_blocks_non_test_project`(**path-lock·高危#1**)——留;
- **随 C1 更新(仅这类·每处判「拐杖 or 守卫」再动)**:断言「自动路跑在预绑会话」的拐杖测,如 `auto_advance_runs_chain_when_authorized_and_bound`(6565)/ `auto_advance_stops_at_needs_binding_when_unbound`(6607)——改成断言「自动路每任务建新会话·target_session_id 互异物化」;
- **判据**:每处改动前问「这测试钉的是拐杖(自动路预绑)还是守卫(手动挡/拒绝/安全闸)」——**拿不准=停手报回**,不许按"退役语义"一锅端。

### 2.3 明确不做

`existing` 模式的「逐任务人工指定会话」新交互(canon 提到·是手动挡的独立功能·非本包·归后续)/ C2-C6 的活 / fork。

### 2.4 文件边界

- **允许**:`director_agent.rs`(auto_advance 内层加 creator 注入 + mode 分流 + 自测)/ `lib.rs` **仅** 2.2「随 C1 更新」那类拐杖测(**逐个判·守卫零改**·这是本包唯一获授权的 lib.rs 死线释放·范围锁死在拐杖测)/ `command_registry.rs` 若接线需要(仅调用参数·判决体 0 命中);
- **0-diff**:`command_plan_for` 及 runner 本体 / `manual_relay.rs` 本体 / 沙箱 / 安全闸 / c4_c6 判决体 / worker_report / 各 agent / secretary / run_history / 前端。

## 3. 安全死线

- 新会话全圈**固定测试项目**;人闸/授权复查/prepare guard/四护栏一字不动;**path-lock 守卫(7311/6689)神圣不可动**——它们是高危#1;
- lib.rs 释放**只限拐杖测**·守卫误伤=红线;拿不准停手;
- memories 观察模式·不加旗;`.codex` 凭据不碰。

## 4. 验收

- **守卫回归(重中之重)**:5 处 PanicCreator + auth + path-lock 测**全绿·且未被修改**(git diff 证这些行没动)——证 mode-aware 没误伤;
- **新断言**:独立[接着跑] 3 任务链 → 每任务建新会话·thread 互异·target_session_id 互异物化(director 自 mod·可仿 C1 首轮 `c1_chain_...` 范式);合流 `new` 同款、`existing` 仍手动挡不建会话;
- **真跑**(`#[ignore]`·测试项目):一条[接着跑]链端到端·codex home 见任务命名新会话·口供落库;
- 三闸绿 + 冻结核 0-diff 自证 + lib.rs diff **只含拐杖测**(逐行可justify) + 计数不降 + fmt 净(**自己真跑 `rustfmt --check`·别自报**)。

## 5. 回交

- §4 证据 + **守卫未改的 git diff 自证** + lib.rs 改动逐处「拐杖 not 守卫」justify + 落点清单 → 主导线核实物。**子线不 commit。**

## 7. 不接受为

- 无条件建会话(必 mode-aware) / 一锅端放开那 5 处 PanicCreator 或 path-lock 守卫(误伤高危#1) / 静默回落 existing/拐杖 / 碰 command_plan_for·runner 本体·manual_relay 本体·沙箱·安全闸 / lib.rs 改动越出「拐杖测」范围 / 自报 fmt 净不真跑(首轮前科) / 做 existing 逐任务手动指定新交互(非本包)。
