# 任务包:前端施工总包——照定稿逐页实现(②-⑧)v1

日期:2026-07-15 · 档位:轻档(纯前端·照图施工) · 执行者:前端施工对话(Claude·可长跑) · **设计已全部定稿,零设计决策——任何「我觉得这样更好」都是违纪,照图干**。

## 先读(顺序,一个不省)

1. `prototypes/productized-desktop-shell/DESIGN.md`——信息规范(四问+两禁令)+三面定式+布局总纲(壳固定容器滚/紧凑一档)·**施工宪法**;
2. `decisions/2026-07-14-interaction-model-canon-v1.md`——交互宪法(五态/打断三级/回顾面/§四修宪后布局);
3. `docs/design/2026-07-14-stage-b-hifi-fullapp-v1.html`——高保真定稿 11 张 A-K(浏览器打开对照着写);
4. `src/components/SpecPrimitives.tsx`——设计系统基座,**一律复用,禁止另造同类件**;
5. `src/views/MemoryCenterView.tsx` 的 B1 双栏区——已完工样板(commit `877d54e`),照它的模式干。

## 施工纪律(与包同效力)

- 每页一刀;每刀 `npm run typecheck`+`npm run test:offline-interaction` **全过**才进下一页;
- 组件保持**无 hooks 可平铺**约定(状态提升父级;样板先例=DailyMemoryCandidateInbox showAll);
- 测试断言随新语义更新**合法**(样板先例=B1 改 4 处 fixture),但只许改「锁旧形态」的断言,不许删测试;
- **不 commit**(总指导核后 commit);不碰 src-tauri(后端件另有包);Tauri 真机=用户走查,你不用跑 App;
- 回传:每页收口回一段(页名/改动文件/套件结果/断言更新清单),全部完成后总回传。

## 逐页清单(按序,定稿段落=hifi html 对应 <h2>)

**② A·交办三栏(修宪版,最大刀)**:`ProjectWorkspaceShell`/`ProjectJiaobanPanel` 布局区——左工作历史独立栏(可一键收起成窄条,含筛选 chips)+中交办主卡+右画布动态宽(未开工=窄提示条「批准后这里画工序图」;开工后=宽)。renderLayout 机制已有(M2 布局器),历史栏从悬浮覆盖迁独立栏。壳固定:此页起主区不整页滚,各栏 `.spec-scroll` 内滚。

**③ F·卡住态两型**:`jiaoban/JiaobanBlockedStates.tsx`——甲型(等确认:现状按钮式,收敛文案)/乙型(出问题:停因+「直接回它一句」输入框+[发送并继续]主按钮+次按钮)。乙型发送走后端 follow-up 通道——**命令面若未就绪(后端包在做),UI 先立形态+disabled 带人话「通道接线中」**,零假按钮。

**④ D·右栏+审计账本页(新页)**:先立**审计账本页**(回顾面 B1 同构:过滤+审计流列表+详情;数据=现有 workflowState 审计/后端包的账本读模型,未就绪部分先渲染现有可得数据);右栏 rail 计数角标+抽屉行改真实阶段+**点击带上下文直达**(治「点哪都跳同一页」——RightDetailPanel.tsx:410 族);然后各处「开发者详情」折叠逐一移除,机器信息注「见审计账本」。

**⑤ C·首页系统总览**:`HomeView` 重做——统计行(项目/跑着/等我/系统健康)+四区块(等我的事/最近项目/记忆动态/系统状态)固定网格内滚。「系统状态」数据命令未就绪前显现有可得(项目数/工单态从 workflowState 可派生),缺的字段留位+「接线中」。

**⑥ G/H/I 三页(B1 同构,便宜)**:G 项目页总览=事实卡单卡+第二卡位留白(虚线框注「等真实需求」);H 智能体页=左会话列表(搜索+项目分组+三元素行)+右 transcript,composer 上一行显沙箱与写根,开发者 11 面板退场(注「见审计账本」);I 技能/harness=B1 同构全量零截断(SkillsBoardView/HarnessBoardView 重做)。

**⑦ J·知识库(独立大件)**:Obsidian 式编辑器=内置组件——左 vault 文件树(读目录)+右 MD 编辑器(编辑/预览切换·即时写盘·[[双链]]渲染可点·#标签高亮)。选型自定(CodeMirror 6 级),依赖新增合规;vault 路径:先做「选择文件夹」空态(答下一步),选后记忆(本地配置)。「存为记忆候选」走现有 create-memory-candidate PendingAction。**写盘走 Tauri fs——若需新 command 则列缺口报回,不自开后端**。

**⑧ K/E+批1回炉**:设置页人话四行(照定稿 K)+想法箱空态句式;E 弹层对照定稿核(三段式,现 PermissionDialog 结构近似=对齐文案与三要素顺序);批 1 交货卡回炉对照 A 定稿(pill 行/干了什么/体检单/动作行——已实施件核对齐)。

## 验收(总包)

全部页:typecheck+离线套件全过;每页对照定稿逐块自查表(在回传里逐块打勾);shape gate(仓根)前端零净增 error;新文件 <2000 行;styles.css 新增只许 spec-*/页面级 class,禁改既有 token 值。
