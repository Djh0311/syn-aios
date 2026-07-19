# 任务包:底1b·主管传输层换一次一发 resume(照搬对话界面同构)+看门狗重做 v1

日期:2026-07-18 深夜 · 档位:**轻档·后端为主**(高危 5 条零碰) · 执行者:执行线 · 上位:用户 23:5x 拍板原话「**直接照搬对话界面的模块就好了,把入口接过去,对话界面至少调的差不多了**」+底1 收口 `97fca19` · 起因=真机首验两连败根因勘察(本包 §勘察定案)。

## 勘察定案(2026-07-18 深夜真机实证,写包依据)

1. **两次失败都是 420s 整超时**:`SUPERVISOR_RESIDENT_TURN_TIMEOUT=420`(supervisor_resident_session.rs:21);23:20:38→23:27:39、23:35:36→23:42:36,分秒吻合。
2. **超时后毁灭式收场**:清了临时家(`action=cleaned; trigger=resident_process_terminated`),但 codex 宿主(pid 97177)与 `__mc` 桥进程(pid 97429)**没杀死=孤儿泄漏**(总指导手工收割);会话记录(sidecar sessions)`resident_running`/started_at 15:17/pid 34587=**陈账不刷**。
3. **宿主是起来就僵**:孤儿 10 分钟仅 1.04s CPU、桥 0.00s CPU——不是在干大活,是出生即卡(桥进程一行活没干=首嫌疑:workbench-as-MCP-server 在 app 并存时对生产 store 的打开/锁竞争;**未定罪**,防御性修)。
4. **额度/认证/机制均无罪**:总指导探针(s1_freeform live 真模型)两跑 30-35s 全绿——同机制在测试隔离 store 下完全健康。
5. 结构判断:**全系统唯一常驻守护进程=唯一连环暴毙者**;worker/manual relay/agent 页聊天全是一次一发 codex 进程,天天真跑稳定。P1-0 选型本就留了「备胎=shell resume」,本包=备胎转正。

## A·干什么

1. **A1 传输层换轨(核心)**:主管回合从「常驻 mcp-server 宿主」换成**一次一发 codex resume**(与 manual relay/worker 同构):每条用户消息=persist(canonical 不变)→spawn codex(私有家配置)→resume 同 threadId→收回文→落 supervisor_message→进程退出。**协议面零变**:三 canonical 事件/黑板派生/审计/换代事实注入(=新 thread 首轮注入)全原样;`submit_proposal` 工具照旧走私有家 MCP 配置挂载(config 挂载不依赖常驻宿主)。
1b. **A1b 出方案挂卡链路(用户 07-19 点名,显式钉死)**:「照搬」只搬**跑法**,不搬**配置**——出方案挂卡靠的不是常驻进程,是**私有家 config 挂载的 `submit_proposal` 工具**:一次一发进程启动时读项目常驻私有家(A2)→挂同一份 MCP 白名单(syn 桥+submit_proposal)→回合内主管调工具→写既有 proposal store→右侧卡出现(与底1 完全同链)。机制先例=worker 一次一发进程天天挂真家 MCP 干活(firecrawl token 过期崩过 worker=反向实证 config 挂载对 exec/resume 生效)。**验收硬断言:mock+live 都必须有「一次一发回合内工具调用→PendingUserConfirmation 卡出现」**;若实测发现 codex resume 不挂 config MCP(与先例矛盾),停手报回,不许静默降级成纯聊天。
2. **A2 私有家改项目常驻**:CODEX_HOME 从「随宿主生灭的 /tmp 临时家」改**随项目常驻目录**(app-data 下,含 MCP 白名单 config+auth 符号链接;threadId 连续性靠它);清家时机=换代(replace)时轮转旧家归档,**绝不在回合中清**。
3. **A3 看门狗重做**:420 死数退役→**按动静判卡**(子进程 stdout/rollout 持续有事件=活着不管;静默超阈值[建议 120s]→杀**该回合进程**[先 SIGTERM 后 SIGKILL 双段]→家保留→自动重试一次→仍败=人话上脸「主管这句没接上——再发一次或换个说法」);回合进程退出=天然清场,无守护进程可泄漏。
4. **A4 陈账卫生**:sidecar sessions 记录随回合真实状态刷(started/ended/termination_reason);启动时扫「记录活着但 pid 已死」的陈账→改 exited 并记审计(exec-registry 孤儿收割先例同款);`__mc` 桥若仍需(MCP serve)→**桥进程打开生产 store 必须只读/惰性**,防御 §勘察 3 嫌疑。
5. **A5 验收**:①离线:回合生命周期状态机断言(runner 预登记);②live ignored(总指导亲跑):同 thread 三轮 resume 续接+换代注入+submit_proposal 落卡(把底1 欠的「聊→工具落卡」live 一并补);③四闸+s1 定向全绿(1000/0/44 口径只增不减·退旧宿主测试走删测预登记);④真机=用户重发「改标题成小马里奥」那句(底1 首单欠账继续在此单清)。

## B·红线(违者停手报回)

1. canonical/审计/黑板派生/人闸/所批即所跑零变;`submit_proposal` server-owned 字段与严格校验原样;S1 三支/终标/闸零碰。
2. manual_relay/consult/agent 页两脸零碰(同构≠合并;共用底层器官可列建议进回传,不动手)。
3. 换轨后**杀掉整类死法**:不留任何常驻守护进程;每回合进程必须「退出即清」可证明(回传附进程收割断言)。
4. 私有家路径/auth 符号链接=既有 P1-A 口径(高危#2:不写真 ~/.codex);serde 双件;gate 13 基线零净增。
5. P1-0 选型定案的改判随本包收口落 decision 一行(备胎转正·理由=本包勘察定案),不翻写历史。

## C·交付

1. 代码+离线断言+live 证据(三轮 resume+工具落卡 transcript);
2. 进程生命周期证明(回合前后 ps 对账·零残留);
3. 10 项回传;真机走查点=用户那句「改标题」重发走通(聊→出方案落卡→批准→跑)。
