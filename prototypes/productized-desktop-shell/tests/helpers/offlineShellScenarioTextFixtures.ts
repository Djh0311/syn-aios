export const shellScenarioTextFixtures = {
  // ⑤ C 定稿(hifi `C · 首页(系统总览)`)重做首页后同步：首页 = 统计行(项目/跑着的单/等我的事/
  // 系统健康) + 四区块(等我的事/最近项目/记忆动态/系统状态)。
  // 删掉的 "技能"/"harness"/"运行中工作流" 锁的是旧首页形态(技能/harness 计数入口块 +「运行中」面板)，
  // 定稿 C 段没有这三块；三者在左导航的覆盖由下面 primaryNavLabels 断言独立保住，未失守。
  // "不是真实使用事件" 保留：最近项目的时间取自索引 latest_updated_at_ms(近似口径)，这句诚实声明仍成立。
  homeExpectedTexts: [
    "项目",
    "智能体",
    "跑着的单",
    "等我的事",
    "最近项目",
    "记忆动态",
    "系统状态",
    // 「系统健康」是统计格的名字，不是定稿要求上脸的串(hifi C 段那格渲染的是「● 正常」+ 小字明细)，
    // 故不断言它——否则读模型接线后那串消失，断言会假摔。
    // 系统状态读模型(后端包 §A)未接线 → 必须留位 + 人话「接线中」，不许编数据。
    "接线中",
    "不是真实使用事件",
  ],
  // 原断言 message = 「首页不应显示数量」，锁的是旧首页把原始计数块摆上脸(Skills 1 / Plugins 1 /
  // 「系统」计数块)。裸串 "系统" 与定稿 C 段强制的「系统状态」「系统健康」直接撞车 → 换成旧计数块
  // 自己的标签文案(可复用能力 / 运行器资源)，反向意图(不摆原始计数块)不弱化。
  homeForbiddenTexts: ["Skills 1", "Plugins 1", "可复用能力", "运行器资源"],
  primaryNavLabels: ["项目", "智能体", "想法箱", "知识库", "记忆层", "技能", "harness", "实验画布"],
  primaryNavGlyphs: [
    ["projects", "▤"],
    ["agents", "◍"],
    ["ideas", "✎"],
    ["knowledge", "▢"],
    ["memory", "◐"],
    ["skills", "✦"],
    ["harness", "⬡"],
    ["workflow", "⊹"],
  ],
  settingsExpectedTexts: [
    "开发者",
    "建议方案",
    "模型/凭据",
    "适配器",
    "供应方",
    "边车文件",
    "原始状态",
    "诊断",
    "不读取凭据",
    "不从设置页触发",
  ],
  settingsForbiddenTexts: ["执行 codex", "恢复会话", "密钥值", "令牌值", "读取并展示"],
  sourceEntryExpectedTexts: [
    "想法入口",
    "任务线索",
    "想法边界",
    "转任务后置",
    "方案入口",
    "项目边界",
    "方案边界",
    "确认后置",
    "工具入口",
    "运行器资源",
    "工具边界",
    "执行后置",
    "模型入口",
    "供应方状态",
    "凭据边界",
    "凭据不可见",
  ],
  sourceEntryForbiddenTexts: ["读取密钥值", "启动 runner", "调用真实 Codex", "自动转成项目任务"],
  runningWorkflowsExpectedTexts: [
    "运行中工作流",
    "只显示运行、等待、复核、重试和读回异常摘要",
    "读回异常",
    "未知 / 不可用不显示成 0 条结果",
    // 画布优先重画后的新结构（标题区 / 状态带 / 右栏详情 / 底部模式与操作）
    "CANVAS · 工作流画布",
    "当前节点",
    "运行状态",
    "建议方案",
    "手动编排",
    "展开任务包",
    "点节点只切换详情，不触发真实执行",
    // P4：右栏运行上下文（读回 / 失败 / 状态原因）。后置A 后：阶段进度段带按真实节点渲染，
    // 本 fixture 工作流无 derived 节点 → 忠实地不画段带（无阶段就无进度），故不再期望「阶段进度」。
    "读回",
    "失败 / 阻断",
    "状态原因",
  ],
  // 空数据（无 workflowState）必须走空画布文案，绝不显示成 0 条成功结果。
  runningWorkflowsEmptyExpectedTexts: ["CANVAS · 工作流画布", "当前没有运行中的工作流。", "打开项目工作流"],
  runningWorkflowsEmptyForbiddenTexts: ["自动执行已启用", "已成功", "0 条成功结果", "结果数：0"],
  agentSessionExpectedTexts: ["新对话", "搜索会话", "Offline interaction fixture", "重新读取", "打开记录文件"],
  // ⑥ H 定稿(hifi `H · 智能体页(会话中心·回顾面 B1 同构)`)重做后同步：智能体页 =
  // 左会话列表(搜索+项目分组+三元素行)+ 右 transcript + 底部 composer(写根/沙箱一行常显)。
  //
  // 删掉的 25 条("适配器能力"/"codex-local"/"会话索引读取"/…/"does_not_change_codex_execution_semantics")
  // 全部来自**开发者 11 面板**，定稿明令它们从本页退场(→审计账本页)。这些断言**一条没删**，只是跟着组件走：
  // 改测 AgentDeveloperPanels 组件本体(见 offline-permission-dialog.test.tsx 的 developerPanelsText 段)，
  // 读模型语义覆盖不丢；「面板不该出现在智能体页」另由 agentRetiredDeveloperPanelMarkers 正向断言锁死。
  //
  // 留下的是定稿 H 段真正要求上脸的东西(已用真实渲染核过，非猜)。
  agentViewExpectedTexts: [
    "新对话",
    "搜索会话",
    // 三元素会话行：claim + 时间；项目分组头
    "Offline interaction fixture",
    "codex-workbench",
    // 右 transcript 区
    "重新读取",
    "打开记录文件",
    "offline-model",
    // composer + ⑥ H 新增：发送前写根/沙箱常显(治体检 P1「批态可见性缺席」)。
    // 这行是本包最要紧的正向锁：它必须与真实发送的 sandbox/allowed_write_roots 同源(MANUAL_RELAY_SANDBOX)。
    "将以 workspace-write 写入 codex-workbench",
    "给 Codex",
    "发送",
  ],
  agentViewForbiddenTexts: ["请进入对应项目查看具体会话与正文", "选 择 智 能 体", "启动 OpenClaw", "绑定 Claude", "凭据已配置"],
  // ⑥ G 定稿(hifi `G · 项目页·总览`)重做总览后同步：总览 = 项目事实卡**单卡** + 第二卡位留白。
  // 删掉的 "项目概览"/"智能体入口"/"会话列表和对话界面已放到智能体页" 锁的是**旧四块形态**(项目概览卡 +
  // 智能体入口卡)；定稿 G 段把会话入口整个挪去智能体页(H)，这两张卡不再存在。会话入口未失守：
  // 「在智能体中打开」按钮的断言仍在(见 offline-permission-dialog.test.tsx，改测 selectedTool="agent-sessions"
  // 的 ProjectAgentMovedPanel —— 定稿口径下会话入口本来就该在那儿)。
  // "缺少项目默认 workflow" 保留：无 workflow 时「工单」事实行仍如实这么说(且按 D7 补了下一步)。
  projectOverviewExpectedTexts: [
    "总览",
    "工作流",
    "交接",
    "资源",
    "设置",
    // 定稿 G 段事实卡的段标题 + 五个字段名
    "项目事实",
    "路径",
    "最近交货",
    "工单",
    "写授权",
    "文件",
    "缺少项目默认 workflow",
    // 最近交货 / 文件两行无源(见 ProjectOverviewPanels.tsx 注释)→ 必须留位 + 人话「接线中」，不许编数据。
    "接线中",
    // 定稿 G 段：第二卡位留白 + 动作行
    "第二卡位留白",
    "去交办",
    "看工作流",
  ],
  projectOverviewForbiddenTexts: ["任务包", "Codex 角色编排", "任务包 Markdown 预览"],
  projectAgentSessionExpectedTexts: ["项目内 Agent 会话", "Offline interaction fixture", "codex-workbench", "重新读取"],
  projectAgentSessionForbiddenTexts: ["发送消息", "新建会话", "codex resume", "删除会话", "移动会话"],
  emptyProjectAgentSessionExpectedTexts: ["没有索引推断关联的 Codex 会话", "当前项目没有索引推断关联的 Codex 会话。"],
  workflowProjectDraftExpectedTexts: ["项目工作流草稿", "当前项目还没有本地工作流草稿", "创建默认工作流草稿", "请先创建默认工作流草稿，再登记任务包草稿", "不会派发给真实 Codex 会话"],
  bootstrapDialogExpectedTexts: ["目标路径", "写入边界", "默认节点", "默认边", "不写 Codex 状态库"],
  workflowProjectWithDraftExpectedTexts: [
    "当前项目已有本地工作流草稿",
    "任务草稿",
    "2 个",
    "创建任务包草稿",
    "Codex 开发线",
    "已有任务草稿",
    "第二个任务草稿",
    "task_package",
    "当前选中",
    "选择",
    "任务包 Markdown 预览",
    "预览，不是已派发任务包",
    "有任务草稿时可以点“预览 Markdown”查看只读文本",
    "编辑字段表单会绑定当前选中的任务草稿",
  ],
  workflowCanvasWithDraftExpectedTexts: [
    // P1/P2 项目面（两面一引擎，2026-06-21 真机反馈版）：规则状态条（运行性）+「新建/编辑工作流」
    // 动作（编辑是动作不是视图，已删方案/运行视图切换）。默认渲染只读运行状态治理面。
    // 2026-06-23 P1 全屏壳：删了只读头部 eyebrow「项目工作流主入口」，动作条挪顶边悬浮 HUD；
    // 改断「▶ 运行选中节点」（HUD 里稳定渲染的动作锚点）替代被删的头部文案，不弱化安全断言。
    // 2026-06-28 细调：删了底边「项目规则状态条」（运行性：…）+ 项目名标签 → 对应断言移除。
    "新建工作流",
    "编辑工作流",
    "▶ 运行选中节点",
    "方案与授权",
    "waiting_for_permission",
    "1 pending",
    "开发线",
    "权限",
    "节点详情",
    "节点状态",
    "会话绑定",
    "权限请求",
    "当前工作项",
    "负责角色",
    "当前位置",
    "派发位置",
    "Codex 开发线",
    "下一步：标记执行中",
    "节点会话绑定",
    "派发位置已有绑定",
    "Offline interaction fixture",
    "读取状态：可读取",
    "打开会话",
    "解除绑定",
    "派发指令",
    "执行目录：/Users/yoyi",
    "沙箱模式：workspace-write",
    "允许写入根目录：/Users/yoyi/codex-workflow-mario-test",
    "dispatch:offline:001",
    "事件：12 / 命中：1",
    "重试",
    "超时",
    "900 秒",
    "用户审核业务指令",
    "用户审核业务指令夹具",
    "权限请求队列",
    "待确认 / write_workflow_state",
    "需要用户确认是否允许写协议字段。",
    "项目咨询方案草案",
    "确认任务包、角色、读写范围、工具和停止条件后，再进入全局复核。",
    "确认方案范围",
    "要求修改",
    "拒绝方案",
    "plan-auth:offline:active",
    "blocked / 写入范围超出方案授权",
    "批准",
    "拒绝",
  ],
  workflowCanvasWithDraftForbiddenTexts: ["组件状态样例", "后续画布开发基准", "空画布", "四角色", "工作者已执行", "工作者已启动", "已自动执行"],
  confirmedProposalExpectedTexts: ["全局边界复核", "方案已由用户确认；等待全局主管复核", "待全局复核", "plan-auth:offline:pending-global", "批准并生效", "要求修改", "阻断方案"],
  confirmedProposalForbiddenTexts: ["工作者已执行", "自动派发已开始"],
  globalReviewDialogExpectedTexts: ["批准并生效", "复核结论", "复核摘要", "授权对象", "方案标题", "目标摘要", "读写范围", "工具 / 检查", "停止条件", "无阻断发现", "只让授权有效", "仍未派发工作者", "不写 /Users/yoyi/.codex"],
  projectDirectorTaskPlanCardExpectedTexts: ["项目主管拆任务", "授权范围内可准备", "C4 准备态子任务", "授权检查通过", "生成拆任务草案", "准备授权范围内派发"],
  projectDirectorTaskPlanCardForbiddenTexts: ["工作者已执行", "自动派发已开始", "Codex 已收到任务"],
  prepareAuthorizedDialogExpectedTexts: ["准备授权范围内派发", "授权对象", "plan-auth:offline:active", "方案对象", "proposal:offline:c4:confirmed", "计划摘要", "任务计数", "planned 1 / prepared 0 / blocked 0 / needs_binding 0", "记忆快照", "只创建准备记录", "不启动工作者", "不执行 codex exec resume", "不写 /Users/yoyi/.codex", "仍未执行工作者"],
  prepareAuthorizedDialogForbiddenTexts: ["工作者已执行", "自动派发已开始", "Codex 已收到任务"],
  derivedWorkflowForbiddenTexts: ["拖拽已保存", "连线已保存", "节点已删除", "已修改 workflow 事实", "画布编辑器已完成"],
  blockedRunCheckExpectedTexts: ["缺模型；系统不会自动选择模型。", "没有读范围；不能运行。", "会写文件但没有写范围；不能运行。", "节点没有声明工具；工具白名单为空。", "模型", "阻断", "读取范围", "工具白名单", "警告"],
  runnableRunCheckExpectedTexts: ["模型", "通过", "任务包已显式指定模型。", "记忆引用", "任务包没有声明需要记忆引用。"],
  projectsViewExpectedTexts: ["项 目 入 口", "方块入口", "codex-workbench", "最近更新", "会话", "工作流", "文件", "警告"],
  instructionDialogExpectedTexts: ["确认用户审核业务指令边界", "指令摘要", "用户审核业务指令夹具", "审核状态", "reviewed", "不执行真实业务任务", "不启动 Codex", "不恢复会话", "不发送消息", "不写 /Users/yoyi/.codex"],
  c5PanelExpectedTexts: ["C5 工作者汇报 / 过程事实", "待主管确认", "离线桩结果：已接收任务，没有执行真实 Codex 会话。", "记录汇报", "确认为过程事实", "要求返工", "阻断并上报"],
  c5PanelForbiddenTexts: ["工作者汇报已成为正式事实", "系统已记住", "最终结果已通过", "自动化工作流已完成"],
  c6PanelExpectedTexts: ["C6 结果 / 阶段验收", "阶段 C 验收门禁已通过", "最终复核", "最终复核通过", "用户已接受", "全局主管已完成最终复核", "用户已查看结果并作出决定", "记录最终复核通过", "记录用户接受", "生成验收摘要"],
  c6PanelForbiddenTexts: ["中间版本已完成", "完整记忆系统已完成", "工作者汇报已成为正式事实", "系统已记住", "真实工作者已执行"],
  globalFinalReviewDialogExpectedTexts: ["记录全局最终复核", "复核结论", "最终复核通过", "过程事实", "不代表用户已接受", "不写正式记忆", "不代表中间版本整体完成"],
  userDecisionDialogExpectedTexts: ["记录用户结果决定", "用户决定", "用户已接受", "关联复核", "只记录本次结果决定", "不代表未来任务默认接受", "不写正式记忆"],
  stageSummaryDialogExpectedTexts: ["生成阶段 C 验收摘要", "产物", "审计事件", "生成门禁摘要和后置项", "不执行真实工作者", "不写正式记忆", "不代表中间版本整体完成"],
  workerReportDialogExpectedTexts: ["记录工作者结构化汇报", "汇报摘要", "证据", "只记录工作者汇报", "不把汇报写成正式事实或正式记忆", "不启动 Codex"],
  processFactDialogExpectedTexts: ["确认为过程事实", "确认事实", "确认后只记录过程事实观察", "不写正式记忆", "不完成最终验收"],
  permissionDialogExpectedTexts: ["记录权限结论：批准", "权限请求", "permission:offline:001", "权限结论", "批准", "控制核心", "写入工作台自己的工作流状态", "审计事件", "不启动 Codex", "不恢复会话", "不发送消息", "不写 /Users/yoyi/.codex"],
  workflowReviewProjectExpectedTexts: ["总指导回收", "记录派发结果判断", "待回收", "WORKFLOW_NODE_DISPATCH_OK_2026_05_29", "事件：12", "命中：1", "警告：session_cwd_differs_from_project_root", "接受", "需要修改", "暂停", "废弃"],
  directorDialogExpectedTexts: ["记录总指导回收：接受", "派发记录", "dispatch:offline:001", "回收结论", "接受", "复核记录", "审计事件", "不启动 Codex", "不恢复会话", "不发送消息", "不写 /Users/yoyi/.codex", "不读取完整会话记录"],
  bindDialogExpectedTexts: ["绑定节点 Codex 会话", "Codex 会话", "offline-thread-001", "不启动 Codex", "不发送消息", "不读取完整会话正文"],
  unbindDialogExpectedTexts: ["解除节点会话绑定", "绑定对象", "binding:offline:codex-dev", "不删除", "不移动", "不归档"],
  advanceDialogExpectedTexts: ["推进工作项到执行中", "目标状态", "执行中", "推进工作台自己的工作项状态并追加审计事件", "不启动 Codex 命令行", "不恢复会话", "不运行运行器"],
  taskDraftDialogExpectedTexts: ["不生成真实任务包文件、不派发真实 Codex 会话", "任务标题", "登记任务包草稿", "目标说明", "写入 work_items 和 artifacts", "默认指派", "codex-dev"],
  copyPreviewDialogExpectedTexts: ["复制任务包 Markdown 预览", "复制对象", "work-item:offline:002", "只复制预览文本", "不写真实任务文件、不派发真实 Codex 会话"],
  taskFileGenerationExpectedTexts: ["真实任务包文件", "从当前草稿生成文件", "生成任务包文件"],
  generateDialogExpectedTexts: ["生成任务包文件", "生成对象", "work-item:offline:001", "写入目录", "/Users/yoyi/workspace/product-line/tasks/", "不派发真实 Codex 会话", "不运行运行器", "不写 /Users/yoyi/.codex 或 Codex 状态库"],
  generatedTaskFileExpectedTexts: ["该草稿已有生成文件", "已生成", "/Users/yoyi/workspace/product-line/tasks/2026-05-29-generated-task-package-offline-001.md"],
  notReadyShellExpectedTexts: ["派发准备", "任务包还不能派发", "not_ready", "检查派发准备", "生成可派发版本"],
  renderedNotReadyExpectedTexts: ["任务名为空、待补充或仍像测试草稿。", "禁止事项仍包含和当前生成行为冲突的历史禁令。", "/Users/yoyi/workspace/product-line/tasks/2026-05-29-generated-task-package-offline-001.md"],
  correctionPreviewExpectedTexts: ["字段级预览", "字段已填写，可复检 readiness", "派发准备字段修正", "用户提供真实目标。"],
  missingPreviewExpectedTexts: ["仍有字段缺失", "目标缺失", "允许写入缺失"],
  correctionEditorExpectedTexts: ["修正任务字段", "保存前先看字段预览", "不自动补编", "保存派发字段修正"],
  correctionDialogExpectedTexts: ["保存派发字段修正", "修正对象", "work-item:offline:001", "不生成真实任务包文件", "不派发真实 Codex 会话", "不运行运行器"],
  saveFieldsDialogExpectedTexts: ["保存任务包字段", "更新对象", "work-item:offline:002", "字段编辑任务", "不生成真实任务文件、不派发真实 Codex 会话"],
  statePanelExpectedTexts: ["本地事实层 v0", "存在状态", "不存在", "结构版本", "工作流版本", "工作流", "节点", "连线", "复核", "审计事件", "状态文件不存在；不会自动创建。"],
  initDialogExpectedTexts: ["写入边界", "workflow-state.v0.json", "备份", "不写 .codex", "追加审计事件", "原子替换"],
  // ⑥ I 定稿(hifi `I · 技能 / harness(回顾面 B1 同构·全量不截断)`)重做后同步。
  // 删掉的 "技能能力库"/"可复用能力"/"适用场景"/"最近使用"/"当前可用性"/"开发者详情：来源和字段缺口"
  // 锁的是**旧四列看板形态**(四个 SummaryTile 列名 + 底部 <details>)；定稿 I 段是 B1 双栏(工具条+三元素行+右详情)，
  // 这六个串随旧形态一起消失。反向意图没弱化：
  //   - 「不摆假事实」由下面 "接线中" + skillForbiddenTexts 接手(旧的「最近使用=未接入，不伪造热度」同义)；
  //   - 「可见≠已加载」这句边界声明仍在(见 SkillsBoardView 详情页脚注)，断言保留。
  // 新增 "全量可滚，零截断" = 治体检 P0 的正向锁：旧版 skills.slice(0, 6) 把 90 条砍到 6 条且无展开入口，
  // 这条断言防它复发。
  skillExpectedTexts: [
    // B1 工具条(真实总数 + 过滤 chip)。搜索框走 placeholder/aria-label，不是可见文本
    // (visibleText 会剥掉标签属性)，故这里只断 chip；与记忆中心 B1 同构，同样不断搜索框。
    "全部",
    "插件",
    // 三元素行 + 右详情事实行(定稿 I 段字段：来源 / 适用)
    "来源",
    "适用",
    "路径",
    // 状态/登记无源 → 留位「接线中」，按钮不做(见 SkillsBoardView.tsx 注释)
    "接线中",
    // 全量零截断(治体检 P0)
    "全量可滚，零截断",
    // 边界声明(旧「当前可用性」列的真实内容，B1 化后归详情脚注)
    "可见不等于已加载、已推荐或已绑定项目",
  ],
  // 定稿把「状态：候选(未登记)」和[登记为正式技能][查看 SKILL.md]画了出来，但 SkillRecord 没有 registration 字段、
  // 全仓也没有对应后端命令 → 硬编码状态 = 装饰不是事实；无命令的按钮 = 假按钮(宪法 §四.3)。两者都不许出现。
  skillForbiddenTexts: ["候选(未登记)", "登记为正式技能", "查看 SKILL.md"],
  // harness 页同理(定稿：「harness 页同款，不再单画」)。
  // 词表(2026-07-14 拍板)：产品域 UI 名 = harness，不译；「运行器」译名已废止 → 本页新写文案改用 harness，
  // 故删掉 "运行器能力库"/"运行器能力"/"可运行范围"/"最近运行"/"等待配置 / 不可用原因"(旧四列列名)
  // 和 "运行器类型"→"harness 类型"、"不自动运行运行器"→"不自动运行 harness"。
  // 左导航 label 已同步为「harness」；由 primaryNavLabels 独立断言保住，未失守。
  // 「资源=文件夹级 / 候选=文件级」这个必须保留的区分：B1 化后靠过滤 chip + 详情脚注保住，断言保留。
  harnessExpectedTexts: [
    // 搜索框同上：placeholder 不是可见文本，不断。
    "全部",
    "文件夹级 harness 资源",
    "文件级 harness 候选",
    // 详情事实行(旧「开发者详情」里的资源字段，B1 化后直接上脸，不再藏折叠)
    "显示名",
    "根路径",
    "harness 类型",
    "智能体类型",
    "适配器编号",
    "来源类型",
    "能力",
    "清单路径",
    "说明路径",
    "版本",
    "入口",
    "node_script:check.js",
    "权限级别",
    "缺清单",
    "缺说明",
    "缺入口",
    "缺版本",
    "全量可滚，零截断",
    // 边界声明原样保留(不新增运行按钮 / 不自动运行 / 不代表可运行或已验证)
    "不新增运行按钮",
    "不自动运行 harness",
    "不代表可运行或已验证",
  ],
} as const;

export function shellProposalDialogExpectedTexts(projectRoot: string): readonly string[] {
  return [
    "确认方案范围",
    "目标摘要",
    "允许读取",
    projectRoot,
    "允许写入",
    "/offline-fixture/projects/codex-workbench/src",
    "工具 / 检查",
    "read_file / npm run typecheck",
    "停止条件",
    "超出读写范围或需要权限升级时必须停下。",
    "待全局复核",
    "不会启动真实工作者",
    "不写 /Users/yoyi/.codex",
  ];
}

export function shellDerivedWorkflowExpectedTexts(projectRoot: string): readonly string[] {
  return [
    // 2026-06-23 P1 全屏壳：头部 eyebrow「项目工作流主入口」已删，改断顶边 HUD 稳定动作锚点。
    "▶ 运行选中节点",
    "黑板候选",
    "任务包",
    "attention",
    "用户摘要",
    "为什么停下",
    "下一步",
    "节点详情",
    "权限请求",
    "模型",
    "允许读取",
    projectRoot,
    "允许写入",
  ];
}
