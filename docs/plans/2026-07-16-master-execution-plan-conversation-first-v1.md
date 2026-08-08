# 总执行计划(唯一计划入口)· 对话优先改造 v1

> **状态校正（2026-08-09）：HISTORICAL / SUPERSEDED。** 本文只保存 2026-07 当时的执行现场；标题和正文中的“唯一计划”“当前”“下一步”均已失效。当前产品方向看 `../product/syn-product-canon-v1.md`，当前排期看 `2026-08-01-syn-personal-ai-workbench-master-development-plan-v1.md`，具体执行另看当前轻量开发护栏链。

日期:2026-07-25 · 状态:**共享 Conversation Transport + Syn MCP Capability Plane 已完成代码与离线回归，三句真实 App 重验包已冻结但仍待新授权；L3 知识库 N0-N5 既有能力离线收口，N2R-R0 真实参考已冻结，首个 React-only 单壳收口包待用户 kickoff；N6 继续 HOLD** · 本文=全仓待办与计划的**唯一收敛入口**:CURRENT §三只留顺序指针;旧计划文档全部在 §三 归位。方向正本=`docs/plans/2026-07-16-conversation-first-direction-and-execution-plan-v1.md`；当前 transport/MCP 决策=`decisions/2026-07-22-shared-conversation-transport-and-syn-mcp-capability-plane-v1.md`；L3 当前决策=`decisions/2026-07-23-l3-syn-native-knowledge-workspace-route-v2.md`(本文不重复其内容,只管「怎么执行不跑偏」)。

## 〇、防跑偏总则(执行任何包前重读,违反=机械拒)

1. **智能在模型不在流程**:任何包不得新增用户确认点、治理工件或提示牌;"修一个坑立一块牌"的旧习按违方向打回。
2. **人闸只有三下**(确认执行/属实/记住);其余一切确认改自动+审计记账。
3. **对话=介质;消息=候选;升级事实唯控制核心**;UI 不写事实(蓝图 §5.1);对话消息底层落项目黑板既有条目(蓝图 §5.4)。
4. **自然信息流转双轨**:对话负责理解/追问/协商;MCP 负责方案/任务/状态/证据等结构化动作;Syn 负责路由、角色能力、校验、幂等、审计与权威事实;卡片只是实物投影。用户不在 Syn 与 Codex/其他 agent 之间搬话;工具结果回原 thread;工具失败不得吞对话。普通聊天、MCP 动作、用户执行授权三层不得混成一把闸。
5. **角色**:对话面唯项目主管;工人/全局主管/秘书不进对话(全局主管意见=右区留证;秘书=项目外,本轮不动)。
6. **执行闸零碰**:S1 三支/写域锁定/fail-safe/AGENTS 高危 5 条/memories 渗出观察——每包红线默认含此条,diff 扫到即打回。
7. **写包纪律**:红线必须先读透对应数据流源码(产/消/依赖三段,ledger 07-16 二犯在案);引用运行时 reason/日志必须整段原文,截断串不得进包题/前提/红线。
8. **验收纪律**:每包四闸亲跑(typecheck/离线套件/shape gate 基线 13-5-5 零净增/cargo 当前 **1031/0/44** 口径只增不减)+10 项回传模板;含 Rust production 路径的包另跑 `cargo check --lib` 或等价 non-test build，不能用 test cfg 代替生产编译；每真单落三数(点头/分钟/卡点)进 CURRENT。
9. **transport/MCP 复用纪律**:交办页主用、智能体页次要 / 待定；允许扩充共享底座，但不得复制 transport、继续堆 resident/private-home 主路线、新增交办私有 MCP/sidecar，或把 `workspace-write` 搬给主管。MCP 能力必须在服务端 registry/role allowlist 后才可宣称可用。

## 一、主线拆包

### 阶段一·主管对话化(P1-0 → P1-E,串行为主)

| 包 | 内容 | 前置 | 验收 |
|---|---|---|---|
| **P1-0 选型勘察**(轻·只读) | codex 驱动方式实测:shell `resume` vs `codex mcp-server`/`app-server`——长回合稳定性/工具调用超时上限/并发 stdio 子进程数/等答复的挂起行为;固定测试项目实跑 | 无 | 实测记录+选型建议回传,总指导核复后拍 |
| **P1-A 主管常驻会话**(后端) | 项目级主管会话生命周期(创建/复用/**换代**=事实在核心不靠聊天记录);咨询并入主管(consult 改走主管会话多轮,tier-1 一次性注入路退役);咨询期只读挂项目 | P1-0 | 同一单多轮问答同会话续;会话换代后靠注入事实重建上下文;四闸 |
| **P1-B 问答接线**(后端+协议) | `RequestUserDecision` → 对话消息;用户回答注入续跑(不重开整圈);终版方案=回合末 schema 契约照旧+确认时重校 | P1-A | 真单「问一句-答一句-出方案」闭环;审计每步留痕 |
| **P1-C 中栏对话 UI**(前端) | ~~方案卡/交货卡=消息~~(07-17 用户实物否定)→**对话=纯话**(你/主管消息+主管人话短讯)、**方案卡/交货卡挂右区**(实体卡+批准动作随卡)、**唯一常驻输入框**(07-18 拍:无标题无按钮·Enter 发送·按状态路由既有三通道)、⚠ 提示牌退场;右区四视图(方案/工序图/治理保证/怎么跑);左栏锚点 | P1-B 可并行起 | 实渲量尺+离线套件+**改后截图先过总指导看形,再用户真机**(07-17 流程改进) |
| **P1-D 人闸收敛**(前后端) | 绑定确认/派发两连确认退场→自动+记账;下单表单/批态卡状态机/修改框/开放问题块(07-16 止血件)退场 | P1-C | 一单人闸仅【确认执行】(交货侧属 P3);审计事件不缺笔 |
| **P1-E 旧路退役+修宪** | 咨询塞纸条路死码清扫;UI 宪法(07-14 交互宪法/交办冻结令)逐条修订清单落 decisions;**修宪必扫测试面翻案断言**(07-15 教训) | P1-D | 死码 grep 清零;违宪断言全翻案;四闸 |

**阶段一收口**:真单三数达及格线(小单 1-2 句话/点头 ≤3 下/分钟级/零死卡)+渗出复巡②面。**确认后仍走现有拆任务+薄链(阶段一不碰)。**

### 阶段二·拆任务并入主管(2 包)

- **P2-A（已收口）** 终版方案自带任务图→走既有「所批即所跑」引擎路;主管重拆一跳退场(「空任务列表」失败类消失)。
- **P2-B（已收口·`e9ad7f3`）** 绑定默认自动新会话；挑会话降为右区「怎么跑」逐节点可选项，工序图保持只读；旧公开绑定命令保留未删。

### 当前串行线·共享 transport 与统一 MCP 能力层（07-22 重排）

1. **既有 UI / 运行期资产（已收口，继续复用）**:P3-A 的链事件/工序消息、S1C 的「对话左｜历届方案索引中｜方案/交货卡右」布局、P2-A/P2-B 的任务图与会话选择规则均保留；交办页是主产品消费面。
2. **阶段 A capability audit（已完成·只读）**:核对 Agent existing/new session、event mapping、manual relay 生命周期/权限、交办 resident 接法、MCP 配置、工具回执和 proposal 刷新。结论=`handoffs/2026-07-22-jiaoban-conversation-module-reuse-and-syn-mcp-capability-guidance-v1.md`；架构正本=`decisions/2026-07-22-shared-conversation-transport-and-syn-mcp-capability-plane-v1.md`。
3. **旧 resident 线（历史保留，暂停主排期）**:S1B/H1/H2 与 R3B/R4E/R4F/R4F-R1 的代码、离线、复制店、真实 App 诊断仍是有效历史 evidence；但 `supervisor_resident_oneshot_session`、私有 `CODEX_HOME`、generation/rotate/invalid-resume 自愈不再作为交办主运输继续加固，旧 live 合同退出当前执行入口。
4. **共享 transport 正式实施包（07-23 离线已收口）**:`tasks/2026-07-22-shared-conversation-transport-and-syn-mcp-capability-plane-offline-implementation-package-v1.md` 已在用户重档 #3 kickoff 下完成白名单内代码与离线回归；证据=`evidence/2026-07-22-shared-conversation-transport-and-syn-mcp-capability-plane-offline-verification-v1.md`。在该 07-22 包的基线范围，主管保持 `read-only + 空写根`，MCP 精确开放 `submit_proposal`；L3 v2 另行授权 `knowledge_search/read/open/cite` 四项只读能力，仍需 Active trusted binding，且没有外推为真实 App 可用。
5. **已落的离线范围（不继续扩包）**:profile-driven transport 复用 relay existing/new/poll/Stop/event core；固定 `supervisor-read-only`、可信 binding 与服务端 MCP registry/role allowlist 已接入，交办页已改接共享 transport。智能体页只守编译与基本不误伤，不是产品验收阻塞项。
6. **真实 App 替代性验收（对话线 live 候选；已有包、另授权）**:普通首句、同 thread 第二句、一次 `submit_proposal`、一张 Pending 卡、第三句续聊、工具失败不吞自然回复、chain/项目不动；替代验收通过前不删除旧 resident 路径。合同入口=`tasks/2026-07-23-shared-conversation-transport-real-app-reacceptance-package-v2.md`。它不得与知识线代码写入、构建或任何真实 App 验收并发。
7. **底1 / 底2 重排**:真实替代 transport 验收后，底1从用户点 Pending 卡继续；底2再扩主管代答、答不了@你、worker/reviewer 与交货收尾。
8. **知识库第一片 + P2-B（均已完成）**:知识库首片 `b9f7e34`，P2-B `e9ad7f3`；07-21 知识库 audit 已纠偏为固定 Batch 2 production write。这两项不再占当前排期；P3-D 信息小件仍挂尾部。
9. **L3 知识库第二片 v2（07-25 路线修订）**:Syn 原生底座、Markdown/Frontmatter/附件/JSON Canvas 真相源和 N0-N5 既有能力保留；N2R-R0 已用官方 Obsidian `1.12.7` 无隐私演示 vault 完成真实参考冻结。当前首包=`tasks/2026-07-25-l3-syn-n2r-r1-single-shell-convergence-package-v1.md`，只做 React-only 单壳结构收口、旧主容器退场和离线验证，尚未 kickoff。真嵌入、伴随窗口、Electron、受限品牌资产、插件生态和私有 API 仍停止；Home-only UI discovery、Gate 0 与 N6 十二项继续 HOLD。小阶段计划=`docs/plans/2026-07-23-l3-syn-native-knowledge-workspace-small-stage-plan-v2.md`。

### 统一当前执行图（07-25）

1. **K1 知识库离线施工**：用户 kickoff 后，由独立知识前端执行线完成 N2R-R1 单壳结构收口；指导线只派发和验收，不亲自施工。
2. **C1 对话线只读准备**：可以与 K1 并行核对既有三句合同、冻结输入和停止条件，但不得启动 Syn、构建 binary 或读取真实 store。
3. **真实运行串行门**：K1 写入停止、指导验收完成且工作树/hash 可冻结后，用户才能另行选择启动对话三句重验或知识 Home-only discovery；同一时间只能有一个真实 App 包。
4. **后续知识顺序**：N2R-R1 指导验收 → 后续视觉/交互包 → N2R 离线收口 → 用户授权 Home-only discovery → UI 门 → 用户授权 Gate 0/N6 十二项。

## 二、清桌与并行(不占主线)

- **第 0 步·清桌**(用户在场两次几分钟):①真单走完(白捡三验收:批1 交货卡真数据/复核实证闸首秀/记忆环验收①=交货→[属实,沉淀]→inbox→采纳→召回);②重 seed 恢复 DB 主写(观察期重开重计)。
- **S1B-H1/H2 与 R3B/R4E/R4F/R4F-R1（历史线）**:测试 harness、产品配置、canonical、私有 home 和工具归因证据均保留；不再追加 resident 主运输修复，也不再把旧两句→Pending live 当当前闸。可复用的错误分层、文案、幂等和审计要求并入共享 transport 实施包。
- **视觉治理线 G1→G4（07-20 用户拍「设计方向=三栏归真」·轻档离线·可立即串行开）**：决策 `decisions/2026-07-20-jiaoban-design-direction-three-column-truth-restoration-v1.md`（含逐包表）；施工参照样张 `prototypes/design-mockups/jiaoban-redesign-specimen-v1.html`，手账概念版留档下一代。G1 token 归真（单 :root 正典+`--bg` 定回拍板值+桌面皮违规退役+237 hex 归 token+gate 禁新硬编码 hex）→ G2 定式扶正（spec-* 唯一·事实行 4 式/pill 5 式收敛+死壳清扫）→ G3 盖章时刻（批=石绿印章·全 App 唯一重彩·需用户最后一眼）→ G4 死重清扫（1196 行死视图/占位页/门面瘦身）。不修宪、行为不变、13/5/5 零净增；架构两刀（写操作注册表/状态容器+事件订阅）另议不混视觉线。
- **并行自动**:记忆素材积累(真单自然攒,大梁 B 第一刀已通血);DB 观察期攒天数→M6 停写 JSON(用户另授权;届时 audit_ledger 读源 JSON→DB 一并)。

## 三、旧待办全量收敛(原散落 CURRENT §二/§三/§四·挂账各处,此表为唯一归位)

| 旧条目 | 归位 |
|---|---|
| 前端②⑤⑥视觉真机走查(未完) | **缩范围**:仅存活页(首页/记忆中心/审计账本/设置/右区视图)保留走查义务,随 P1-C/P3-A 各包验收;交办中栏旧脸走查**作废**(将重建) |
| 交办右区跑态/交货逐态视图(冻结前遗留) | 并入 **P3-A** |
| 信息批小件(`.run` 色/S2 律5/recon 口径) | 挂在 **底2 后尾部**，不抢 S1C/真机首单/知识库主线 |
| E 主管意见传参/「最近交货」接线(ProjectOverview) | 首页/总览侧=小件并入 **P3-D**;交办侧作废 |
| 阶段3拆巨石尾刀(ProjectJiaobanPanel 主组件 Browser ~1300 行五态状态机重构) | **作废**——五态状态机整个被 P1-C/P1-D 对话化取代,重构旧机=白干 |
| 卡住乙型回话框 disabled 占位(07-15 甲案「先不通电」) | 随 **底2** 卡住页态一并退场(对话内回话=真通电替代) |
| 丙·worker 追问生命周期(单独立包计划) | 并入 **底2**,不再单独立包 |
| 批1 交货卡真机验证+复核实证闸首秀+记忆环验收① | **第 0 步真单**白捡 |
| 知识库⑦ fs 写入面+L3 原生工作区 | **第一片已收口，第二片既有能力离线完成、界面未完成**：07-25 新增 N2R，将双容器收拢为一个 Obsidian 核心桌面高保真工作台；N2R、UI 先行门和原十二项均未验收。Obsidian 仍不是运行前置 |
| Syn 自有 agent 层 | **搁置**(用户 07-16:「考虑不清楚的事」)。共享 transport / MCP capability plane 只为现有外部 agent adapter 提供受控连接，不等于新建 Syn agent 层；当前真实 adapter 只有 Codex |
| M6 停写 JSON+audit_ledger 读源切 DB | C 线,观察期攒够+用户授权 |
| 两轴路由「默认单 agent」未实现 | 不再单独实现——**长进主管判断**(方向正本 §二.4) |
| 五站快车道计划(07-11) | 已收官,历史正本 |
| post-m5 stage plan v2(能力层排布) | **大梁 B 参照仍有效**(记忆→skill/harness→知识库次序);具体出包在阶段一收口后按需,经本文更新 |
| master-roadmap(06-18)/完整路线图(06-27) | 历史参照;per-task 状态一律以 CURRENT+本文为准(AGENTS §三原则照旧) |
| backlog.md kt-erp 并发写入条目 | 原样挂(他线代收,措辞与落地未拍) |
| 修包群/接线微批/craft 扫除/存储修复包 | 已全清(07-15/16 收口,见 git) |
| 秘书兑现(蓝图 §7 全职责)/自动连环真实项目(Phase E)/五域版图 | 远景,不排期;秘书与主管分工缝=方向正本 §十.2 |
| **人话工程**(07-18 用户拍立项;口径同日总指导建议后用户认可) | **三件套,不做大翻译层**(治本=字串出生即人话,大层=重流程+治标):①散装 humanize* 收编成前后端各一人话模块(照 `run_error_translation.rs` 单一真源样);②shape gate 加机械规则=UI 组件禁直渲机器格式错误串;③用户真机点名的机器话清单逐条进模块(随走查攒)。时机:P1-D/P1-E 旧面退场后动手(现在收编=收进将死面白干) |

## 四、决策点日历(只这几下要你)

1. ~~方向、阶段一、P2-A、P3-A、底1/S1B 开工~~（均已拍并完成各自已声明验证）。
2. ~~S1C 开工令与用户最后一眼~~（07-19 已确认；代码、量尺与四闸已收口）。
3. ~~S1B-H1 harness 可达性~~（07-19 已收口；完整 ignored live 1/0）。
4. ~~**共享 transport / MCP 架构方向与允许扩充现有模块**~~（07-22 用户确认；智能体页降为次要 / 待定，不直接删除）。
5. ~~**共享 transport 实施包开工**~~：07-23 用户已按包内 kickoff 对重档 #3 明确授权；白名单内代码与离线回归已收口，见对应 evidence。不得把该离线结果外推为真实 App/store 授权。
6. **真实 App 替代性验收**：对话线 live 候选，已有 v2 包但仍 HOLD；用户在场且另行授权后，验到一张 Pending 卡再按包决定是否进入第三句，不批准卡、不起链。
7. **底1真机首单**：替代 transport 通过后，用户在新布局里点批并允许起链；三数过目后再拍底2开工令。知识库第一片与 P2-B 已提前完成，不再排在底2之后。
8. 包外仍需另授权：M6 停写 JSON；非测试真实项目真执行；自动连环/多项目。
9. **L3 原生知识工作区 v2 修订路线**：07-25 用户拍板在 Syn 原生底座上高保真复刻 Obsidian 核心桌面界面；R0 真实参考已冻结，当前精确首包是 N2R-R1 React-only 单壳结构收口，待用户 kickoff。其他 vault、真实 App、任意 filesystem/shell、stage/commit/push 仍是停点。
