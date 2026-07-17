# 任务包:P1-A 主管常驻会话(mcp-server 驱动)v1

日期:2026-07-17 · 档位:**轻档·后端**(仓内改码;不碰高危 5 条) · 执行者:执行线 · 上位:总执行计划 §一 P1-A(防跑偏总则 7 条先读)· 驱动方式已拍(P1-0b 二轮核复):**主=`codex mcp-server`(codex/codex-reply/threadId)·「问你一句」=转交+唤醒(A5 直证)·备胎=shell resume**。

## 背景(一句)

今天所有 LM 通道都是「塞纸条」:主管/咨询每问都 `codex exec` 或 `readonly_codex_consult` 重开整圈(`operation_id=new_session`·`session_id=None`),多轮=重注入,没有活会话。P1-A 把项目主管变成**一条常驻 mcp-server 会话**:同项目多轮问答走 `codex-reply` 续 threadId,咨询并入主管,塞纸条路只留退役标记(P1-E 清扫)。

## 现有资产(先读透再动,勘察发现≠重造清单)

1. `supervisor_session_launcher.rs`:临时私有 CODEX_HOME 三件套**原样沿用**——`create_private_supervisor_home_dir`(0700)+`create_auth_symlink`(真家 auth.json 符号链接=认证零复制)+`supervisor_mcp_config_toml`(**MCP 白名单已是事实**:config 只写 `supervisor_orchestrator` 一个条目,真家私人 MCP 结构性进不来——P1-0b 二轮 B1 渗出案的现成正解,勿另立新机制)。
2. 同文件 `SupervisorCommandPlan`/`CodexSupervisorProcess`:现为 `codex exec` 一次性进程+stdin opening message+wait 收尾——**这层是本包要换的**(exec→`codex mcp-server` stdio 常驻子进程,JSON-RPC `initialize→tools/list→tools/call codex/codex-reply`)。
3. `mcp/supervisor_orchestrator.rs`:七动作(dispatch/inspect/follow_up/wait_worker/finalize/report_user+read_key_file 族)照旧挂白名单 config,零改动作语义。
4. `codex_local_runner.rs::readonly_codex_consult`(:320)+`build_readonly_consult_request`(:275):咨询塞纸条正身;guard 六道只读豁免先例(:330 `CONSULT_READONLY_EXEMPT_GUARD_REASONS`)——常驻会话的咨询期沿同一豁免思路,**guard 本体零碰**。
5. `exec_process_registry`:进程登记/孤儿收割现成——mcp-server 常驻子进程**必须登记**,复用不重造。
6. 记忆召回 top5 拼 prompt 的位点(CURRENT §一「记忆环」)本包勘察时定位并在回传里报坐标;换代注入复用它,别新造召回。

## A·干什么

1. **A1 mcp-server 会话宿主**:新模块(或 launcher 内新层)管 `codex mcp-server` 子进程生命周期:起(临时 home 三件套+read-only 沙箱+cwd=项目根)/JSON-RPC client(node 探针工艺转 Rust,或 stdio 直写;零新依赖)/threadId 记账/退出收割(登记表)。**A2 实测参数**:回合均值 3.3s、3 并发健康、`tools/call` 返回≠回合结束(以 `task_complete` 事件为准——P1-0 首轮实证,消费侧必须等对信号)。
2. **A2 项目级会话生命周期**:project_id→{threadId, 宿主进程} 映射落 store(新 sidecar 禁——用既有 store 面,写点走 M5 显式桥,DB 主写+投影同笔);创建(项目首问)/复用(同项目续问=codex-reply)/**换代**(进程死/thread 失效→新 thread+事实注入重建:项目黑板既有条目+记忆召回 top5,**事实在核心不靠聊天记录**——换代后回答质量靠注入,不靠捞旧 transcript)。
3. **A3 咨询并入主管**:咨询入口(director/consult 调用面)改走常驻会话 `codex-reply`;`readonly_codex_consult` 调用点**不删**,标 `// P1-E 退役候选` 注释+回传列清单(全局主管两钩点 4 处调用**本包零碰**——它们是 advisory 钩子,不是项目主管对话)。
4. **A4 验收路径**:固定测试项目真跑——同一项目连续 3+ 轮问答(threadId 不变·上下文续接实证:后轮引用前轮内容)/杀宿主进程再问(换代自动发生·注入重建·审计留痕)/mock 面离线测试。

## B·红线(违者停手报回)

1. S1 三支/写域锁定/guard 本体/`final_mark` 复核实证闸/高危 5 条零碰;主管会话恒 read-only 沙箱+写根空(consult 同款结构性只读)。
2. MCP 白名单=只挂 `supervisor_orchestrator`(Syn 总插座);**禁把真家 config/私人 MCP 引入会话**;auth 只许符号链接,禁读/复制/打印凭据内容。
3. 不新增用户确认点/提示牌(防跑偏总则 1);人闸三下之外零新闸。
4. 本包只做「常驻+咨询并入」;`RequestUserDecision`→对话消息=P1-B,**不越包**(转交+唤醒机制本包不接 UI,只保证协议面可续)。
5. 新 sidecar 禁;store 写点全走 M5 显式桥(mode-on=DB 主写+投影同笔;22 表口径不破);审计每步留痕(会话创建/复用/换代/咨询并入各有 event_type)。
6. tier-1 输出不稳警报器照旧:LM 新字段必配确定性兜底;mcp-server 事件解析失败=保守停,不猜。

## C·交付

1. 代码+离线测试(mock 宿主);固定测试项目真跑证据(3+ 轮 threadId 续接原文+换代重建原文);
2. `readonly_codex_consult` 退役候选清单(调用点坐标+各自去留判断);记忆召回位点坐标;
3. 10 项回传模板:第 7 项 shape gate 三数原文(基线 13/5/5 仓根跑·**新 [error] 零容忍**);cargo 976/45 只增不减;typecheck 0;离线套件全过;不 commit。
