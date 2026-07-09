# 实现任务包:B2·C4a 主管七查+终标(确定性初筛+LM兜底)· 主导线 → 执行线 v1

日期:2026-07-09　性质:**较重**(改完成判定语义·加主管治理动作+LM 兜底)。主导线已 measure-first 亲读(见 §0)。正本:任务包设计 §4.7 + 定稿 `decisions/2026-07-08-phase-b2-execution-loop-final-v1.md` 拍2(成本策略)+ 七拍拍6。**C4 拆三片·本片=核心;C4b(总结→记忆候选)/C4c(failed 四选一)另包后置。**

## 0. 接手须知(冷启即读·前提已核到底)

- 你是**执行线**(后端)。**子线不 commit。** 全程中文。
- **现状缺口(主导线亲读)**:链的完成分支(director_agent.rs:1359-1397)是 **worker 报文解析成功即自动 `completed`**——1382 注释「完成路只归档不驱动·任务恒算 completed」。**没有一个独立于链自动 completed 之外的「主管读七项、点终标」动作**(C0 §5.6 确认的最大缺口)。
- **七查七项(设计正本 §4.7·原文)**:①任务目标是否完成 ②验收标准是否满足 ③证据是否足够 ④审查或 harness 是否通过 ⑤是否有未处理风险 ⑥是否需要生成记忆候选 ⑦是否需要生成用户汇报。**⑥⑦属工作流末(§4.8)→ 归 C4b·本片只做①-⑤的每任务终标。**
- **成本策略(定稿拍2·硬约束)**:**确定性初筛 + LM 兜底**——口供全绿的任务**确定性直过·不烧 LM**;主管 LM **只判黄牌任务**(过/退回)。
- **C3 已立的料(直接用)**:worker 报文真源含 status/acceptance_status/evidence/direction_risks(C3a);黄牌 = report_status 非 done 或无报文(worker_report:report_status_field);求助已在 C3 走 waiting_decision(本片不重复处理求助)。
- **知悉(七拍拍6)**:C4 主管终标是**完成判定·单一判定者·不构成对抗式复核**;审查智能体可选位照旧后置(不在本片)。

## 1. 拍板摘要

- **做什么**:链完成分支从「解析成功即 completed」改为「主管七查(①-⑤)+终标」:**确定性初筛全绿→直过 completed**(不烧 LM);**非全绿(黄牌)→ 主管 LM 判**→ 过(completed)/ 退回(needs_rework·反馈边预算制回 worker)。
- **canon**:完成判定=主管终标这一独立治理动作(不再是链自动);人闸一寸不动。
- **不做**:⑥⑦记忆候选/用户汇报+主管总结(C4b)/ failed 四选一(C4c)/ 审查智能体(后置)/ 求助(C3 已做)。

## 一句话判据

**「是不是只:完成分支插主管七查(①-⑤)+终标——确定性初筛全绿直过 completed(零 LM)·黄牌走主管 LM 判过/退回(退回=needs_rework 反馈边预算制)——而求助路(C3)/失败路/沙箱/授权/execute/worker_report/c4_c6 record 0-diff、人闸不动、退回有预算不无限循环?」** 是 → 做;否 → 停、回主导线。

## 2. 建什么

### 2.1 确定性初筛(①-⑤·全绿直过·零 LM)

- 完成分支拿到 worker 报文后,**确定性判**①-⑤:①status=="done" ②acceptance_status=="reported_completed" ③evidence 非空 ④harness/required_checks 若配置则通过(没配则视为不适用·不卡) ⑤direction_risks 空(C3 真源);
- **①-⑤全绿 → 主管终标直接 completed**(记一条「主管终标·确定性直过」审计·**不调 LM**·守定稿成本策略);
- 语义:确定性初筛是**主管终标的一部分**(不是绕过主管·是主管授权的确定性快路)。

### 2.2 LM 兜底(黄牌→主管判过/退回)

- ①-⑤任一不绿(黄牌:partial/无报文/证据空/有未处理风险)→ **主管 LM(CliDirectorAgent)判**:读口供+任务目标+验收标准 → 出「过」或「退回」+人话理由;
- **过** → 终标 completed(记审计含 LM 理由);
- **退回** → 任务置 needs_rework、**反馈边回 worker 重做**(复用超时重拆的**预算制**·质量债先例 budget=1·防无限退回循环)、链停该任务待重做结果或耗预算;
- LM 判读失败/额度断供 → 保守(供给类人话·不谎报完成·可退回人处理·别自动 completed 蒙混)。

### 2.3 退回的预算与终止(防循环)

- 退回预算(如 1 次·同重拆先例):预算内退回→worker 重做→再判;**耗尽预算仍黄牌 → 停在待人决策**(不无限退回·不自动 completed);
- 退回走反馈边(把黄牌理由+原口供喂回 worker·同质量债重拆的授权窗口供喂法)。

### 2.4 明确不做

⑥⑦(记忆候选/用户汇报)+ 主管总结(§4.8)= **C4b**;failed 四选一 = **C4c**;审查智能体(§4.6 可选位)= 后置;求助(C3 已做·本片完成分支不碰 waiting_decision 求助路)。

## 3. 安全死线

- **人闸一寸不动**(定稿铁律);求助路(C3 waiting_decision)/ 失败即停路 / 沙箱/path-lock/授权/execute/runner/relay/commands / `worker_report.rs`(C3)/ `c4_c6` record 本体 — **0-diff**;
- **确定性初筛不谎报**:①-⑤有一条拿不准就走黄牌 LM·不蒙混直过(保守·同 acceptance_status 保守归一化);
- 退回**必须有预算**(不无限循环·runaway 护栏);LM 断供保守不自动 completed;真跑圈测试项目;memories 观察模式不加旗。

## 4. 验收

- **单测**:①口供全绿(done+reported_completed+evidence+无风险)→ 确定性直过 completed·**审计标「确定性直过」·无 LM 调用**(stub LM 被调 0 次);②黄牌(partial)→ 主管 LM 判(stub 返「过」→completed / 返「退回」→needs_rework+预算减);③退回耗尽预算→停待人·不自动 completed·不无限循环;④LM 断供→保守不 completed;⑤求助路(C3)不受影响(仍走 waiting_decision);
- **回归**:C3 的 738 测全绿(证求助路没碰);既有完成/失败测按新语义调整处说明;
- **真跑**(`#[ignore]`):一条链全绿任务确定性直过·一条黄牌任务走 LM 判;
- 三闸绿 + 死线 0-diff 自证 + 计数不降 + fmt **`cargo fmt --check` 真跑**(别 ad-hoc)。

## 5. 回交

- §4 证据(尤其「全绿零 LM 直过」+「黄牌走 LM」+「退回有预算不循环」)+ 死线 0-diff 自证 + LM 调用计数证据 + 落点清单 → 主导线核实物。**子线不 commit。**

## 7. 不接受为

- 全绿也烧 LM(违定稿成本策略确定性初筛)/ 确定性初筛谎报直过(拿不准要走黄牌)/ 退回无预算无限循环 / LM 断供自动 completed 蒙混 / 碰求助路 waiting_decision(C3)/ 碰 worker_report/c4_c6 record/沙箱/授权/execute / 动人闸 / 提前做 C4b(总结候选)C4c(failed 四选一)/ 做审查智能体 / 自报 fmt 或 ad-hoc rustfmt。
