# 实现任务包:方案a 后端(交办「开个新的」= 先生后绑)· 主导线 → 执行线 v1

日期:2026-07-05　性质:**轻档**(编排侧接现成 relay 机器;execute/commands/死线 0-diff;真建会话=测试项目内真跑 codex 一次·用户点击直接效果)。

## 0. 接手须知(冷启即读,本包自包含)

- 你是**执行线**(后端)。**子线不 commit。** 决策正本:`decisions/2026-07-05-jiaoban-new-session-birth-before-bind-v1.md`(**先生后绑**·execute 的 resume-only 不反转)。
- **现状**:合流 `confirm_and_start_authorized_run` 的 `session_choice=new` 清错拒(director_agent.rs ~1474 注释);existing 分支走现有绑定。runner 层 new_session 能力在(codex_local_runner `operation_id ∈ {new_session,send_message,resume}`·new_session 必须绑 work_item·06-21 验过通),但那是别的入口的;**本包用的是 relay 的**:`manual_relay::run_manual_relay_gui_direct_new_session_once(input, timestamp) -> ManualRelayReceipt`(commands.rs:81 有现成 GUI 命令包装可参考调用形状)。
- **先读**:① `ManualRelayGuiDirectNewSessionInput` 字段与该路径内部的闸(沙箱/审批拒绝/在场类字段——**若有本设计无法诚实满足的门,停、回主导线,不许伪造**)② 回执 `ManualRelayReceipt` 里 thread_id 在哪 ③ 合流 inner 的绑会话步(existing 分支怎么绑——new 建完**复用同一绑定机器**)④ 刀1 包(合流骨架)。
- **一句话**:new 分支 = 调 relay 单次路径在**固定测试项目**真建一条会话(初始化消息写明用途)→ 回执取 thread_id → 走 existing 同款绑定 → 链照旧 resume。

## 1. 拍板摘要

- **要做的事**:「开个新的」从清错拒变真能用——不用用户先去智能体页开会话。
- **为什么**:好用(默认路径应该是"它自己开",这是当初授权卡的原设计);机器全现成,只差接。
- **代价**:一轮·后端(director_agent 的 new 分支 + lib 测试);每单多一次初始化 codex 调用(~15-60s,决策已认)。

## 一句话判据

**「是不是只在合流的 new 分支接了现成 relay new_session 单次路径(测试项目·初始化消息·回执取 id)然后复用现有绑定,而 execute/commands/codex_local_runner/relay 本体/所有死线 0-diff、无第二套会话创建、非测试项目照拒?」** 是 → 做;否 → 停、回主导线。

## 2. 建什么

1. **new 分支**(director_agent 合流 inner,替换清错拒):
   - 组 `ManualRelayGuiDirectNewSessionInput`:cwd=**固定测试项目根**(不可参数化)、消息=初始化文案(写明"交办新会话·承接方案 X 的 worker 任务",人话、简短)、其余字段照该路径要求老实填;
   - 调 `run_manual_relay_gui_direct_new_session_once` → 回执取 thread_id;失败 → 人话报错(「新会话没建起来:…」)+ 走 fix3 的失败留档(确认后失败必审计),**不静默回落 existing**;
   - 建成 → **复用 existing 分支同一套绑定逻辑**绑到节点 → 继续自动推进(后面与 existing 完全同路)。
2. **outcome 加一句人话 warning/说明**:「已为这单活新建会话(初始化 ~1 分钟)」——让等待有名目。
3. **前端解禁 = 不在本包**(避免与卫生包/别的 UI 活撞面板;backend 落地后另开 5 行小包解禁「开个新的」并传 `session_choice:"new"`)。

## 3. 安全死线

- **0-diff**:`commands.rs`(execute 的 resume 写死原样)/ `codex_local_runner.rs` / `manual_relay.rs` 本体(只调不改)/ 全部既有死线文件;
- cwd **写死测试项目**(同档位纪律·不可参数化);人闸不动(new 分支仍在 PendingUserConfirmation 校验之后);
- relay 路径内部的闸**原样生效**(它拒就是拒,人话转述,不许绕/不许伪造在场类字段——真伪造=改审批语义=高危#3);
- 不新建"自由会话"(relay 该路径要什么锚就给什么锚,老实来)。

## 4. 验收(执行线自己验)

- **单测**:stub relay(注入假回执)→ new 分支建→绑→推进全通;relay 失败 → 人话错 + stopped 审计 + 不回落 existing;existing 分支回归不变;非测试 root 照拒。
- **真跑**(`#[ignore]`·测试项目·用户在场那次点击语义):new 一路到 proof——真建的会话在 codex home 里能看到、绑定记录对、链 resume 的就是它、`.codex` 凭据没碰(auth mtime)。
- **regression**:计数不降;§3 全 0-diff 扫 diff 自证;fmt(只本包文件)。

## 5. 不做

- 前端解禁(另 5 行小包);非测试项目;runner 层直接 new_session 进链(execute 不碰);会话复用策略优化(每单一条即可)。

## 6. 回交

- §4 证据(含真跑的会话实物:thread_id/rollout 存在)+ relay 门面核实结论(有没有过不去的门)+ 0-diff 自证 + 计数 → 主导线核实物。**子线不 commit。**

## 7. 不接受为

- 不接受为:碰了 execute/runner/relay 本体 / cwd 可参数化 / 伪造 relay 门字段 / 失败静默回落 existing / 造了第二套会话创建。
