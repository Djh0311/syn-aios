# 实现任务包:交办 fix8 UI(出方案失败上脸 + 供给类人话 + 重试)· UI 专线 v1

日期:2026-07-05　性质:**轻档·前端**(说脸失败呈现;`src-tauri` 0-diff)。

## 0. 接手须知(冷启即读,本包自包含)

- 你是 **UI 专线**。只改前端。**子线不 commit。**
- **背景(今晚实战)**:codex 额度/订阅死时,点「出方案」→ 确认弹层 → 后端转 ~1 分钟失败 → **失败只落在 App 顶部 notice(易错过),说脸上零显示** → 用户以为整个系统坏了、排查一晚。这是交办面最后一个已知「静默死角」,违「永不冻」。
- **现状路径**:说脸 `submitGoal` → `onRequestAction(buildRunProjectConsultationAction(…))` → App 确认弹层 → App 执行器 `runProjectConsultation` → 失败只 `setNotice`。面板拿不到失败,也没有 loading 态。
- **姊妹后端包**(可并行·文件面不撞):失败错误将带 `codex_provider_unavailable:` 前缀+人话(403/额度/登录已分类)。UI 不必自己猜原始英文,但 humanize 兜底仍要有(后端包未落地前旧错误串也要能显示)。
- **一句话**:出方案改为**面板直调**(带 loading/失败态·失败人话上脸+[重试]);咨询是只读、canon 无人闸要求(决策 2026-06-25 只读豁免),去掉那层确认弹层正好回归好用五拍(说→批一次,唯一的闸是[允许并开始])。

## 1. 拍板摘要

- **要做的事**:说脸有 loading("AI 正在读项目、想方案…约 1-2 分钟")、有失败脸(人话+`codex_provider_unavailable` 专句「codex 额度/订阅/登录不可用——处理后点重试」+[重试]按钮)、成功自动进批脸。**永不冻补完最后一块。**
- **为什么**:今晚一晚的排查成本 = 这个死角的真实代价。
- **代价**:一轮·前端(面板直调 + 一个"店刷新"回调 prop 穿三层)。

## 一句话判据

**「是不是只把出方案改成面板直调(loading/失败/重试上脸)+ 穿一个刷新回调,src-tauri 0-diff、人闸([允许并开始])不动、其它动作仍走确认弹层?」** 是 → 做;否 → 停。

## 2. 建什么

1. **面板直调**:`submitGoal`/`submitAmendment` 改调 `runProjectConsultation`(lib/tauri 现成封装),自管:`consultLoading`(说脸显「AI 正在读项目、想方案…(约 1-2 分钟)」·按钮 disabled)/`consultError`(失败脸:人话+[重试原目标]·目标文本保留不清空)。防重入(同 runningRef 套路)。**去掉出方案的确认弹层**(咨询只读·决策 2026-06-25 豁免·canon 人闸=方案授权那一下;卡上已有"AI 要花 1-2 分钟"说明,补一句"会读你的项目"即可)。
2. **成功后刷新方案店**:面板需要新 prop(如 `onProposalStoreRefresh: () => Promise<void>`),App 把现成 `reloadCandidateStores` 穿下来(App → ProjectsView → ProjectWorkspaceShell → 面板·纯 prop 穿线);成功后 await 刷新 → latestProposal 更新 → 自动进批脸(现有 phase 推导不用动)。
3. **humanize**:错误含 `codex_provider_unavailable` → 直接显后端人话;兜底匹配(`403`/`SUBSCRIPTION`/`quota`/`usage limit`/`401`/`unauthorized`/`consult_last_message_read_failed`)→「codex 服务不可用(常见:额度用完/订阅过期/登录失效)——处理后点重试;若是网络抽风,重试一次通常就过」。同一个 humanize 也接进 blocked 脸的报错显示(合流/接着跑撞供给死时同句)。
4. **词表**:照旧零黑话;失败脸必有 [重试] + [改要求](**绝不零按钮**——fix3 铁律延伸到说脸)。

## 3. 安全死线

- `src-tauri` 0-diff;**[允许并开始]的人闸一动不动**(去的只是出方案那层通用确认——只读动作·有决策背书);其它 action(确认/决策/派发类)仍走确认弹层不动;渲染类**必须真机过**。

## 4. 验收(真机·用户额度正死着 = 天然失败夹具)

- 点出方案 → **立刻**见 loading 脸 → ~1-2 分钟后**失败人话上脸**(供给类专句)+ [重试][改要求],目标文本还在——**全程无静默**;
- [重试] 再走一遍(还是失败也照样上脸·不冻);
- (额度恢复后补验)成功路:出方案 → loading → 自动进批脸;
- 三闸绿;`git diff` 仅前端。

## 5. 不做

- 后端分类 = 姊妹包;App notice 机制改造(留原样·面板自显后它只是冗余);额度监测/预警(以后)。

## 6. 回交

- §4 证据(失败态真机截图必须有——现在就能截)+ prop 穿线清单 + diff 仅前端 → 主导线核。**子线不 commit。**
