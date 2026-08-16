# D0C03 快速临时链路证明与完整回滚

阶段：stage-11 阶段11 Syn 5600X/WSL C1 临时链路证明
目标：用一个最长 300 秒的 nonce listener 和一条精确临时 portproxy，证明 Mac Tailscale 到 WSL 的完整链路；随后删除本轮全部对象并验证不可达。
干完的标准：取得 `C1_READY`、Mac 成功 nonce 回执、Windows/WSL 回滚回执和 Mac 回滚后失败回执，最终结论为 `C1_PASS_ROLLED_BACK`；任一缺失均不归档。

允许动：

- docs/harness/authorization.json
- docs/harness/plan.md
- docs/harness/stages/stage-11.md
- docs/harness/leaves/D0C03-syn-5600x-wsl-fast-temporary-transport-proof.md
- docs/harness/unfinished/D0C03-syn-5600x-wsl-fast-temporary-transport-proof.md [新增]
- docs/harness/done/2026-08/D0C03-syn-5600x-wsl-fast-temporary-transport-proof.md [新增]
- docs/harness/done/2026-08/stage-11.md [新增]
- docs/harness/audit/2026-08.jsonl
- docs/harness/usage/.turn

## 步骤

1. 精确 external 授权就绪后，实时冻结 `DESKTOP-FRK8K62`、`Ubuntu-24.04`、`synadmin`、Windows Tailscale IPv4、WSL IPv4、`47123/TCP`、`python3` 和既有 portproxy 空状态。
2. 用户只在 5600X 的管理员 PowerShell 粘贴一次已审查脚本；脚本生成 run id/nonce、启动 WSL 临时 listener、新增精确 portproxy；Attempt3 再新增一条本轮唯一命名、仅允许 `100.120.223.16 -> 100.98.94.76:47123/TCP` 的临时 firewall rule，自检并输出 `C1_READY`，同时设置 300 秒自动回滚。
3. Mac 用 `curl --noproxy '*'` 访问 `http://<Windows-Tailscale-IP>:47123/<nonce>`，核 HTTP 200、remote IP、nonce 和 WSL 标识。
4. 等 5600X 脚本进入 `finally`，核本轮 firewall rule 和 portproxy 已删除、listener 已停止，未碰既有防火墙规则、Cockpit、19528 或其他现有对象。
5. Mac 对同 URL 再探测必须失败；成功和完整回滚同时成立才归档 D0C03 和 stage-11。

## 当前授权状态

- 用户在收到精确范围“临时启动 WSL 47123 nonce 服务、新增并删除一条精确 portproxy、Mac 实测后完整回滚”后明确回复“批准”。
- `authorization.json` 已绑定五个精确 external target；只允许本轮 300 秒临时 listener、精确 portproxy、Mac 成功探测、强制回滚与回滚后失败探测。

## C1 attempt 1 用户转回回执（2026-08-14）

- 5600X 管理员 PowerShell 在新增 `portproxy` 前停止，主错误为：`C1_ERROR=Windows 无法直接访问 WSL 临时服务。`
- 脚本随后报告：`ROLLBACK=PASS`、`PORTPROXY_47123=ABSENT`、`WINDOWS_47123_LISTENER=ABSENT`、`WSL_47123_LISTENER=ABSENT`。
- 当前结论为 `ATTEMPT_1_FAILED_ROLLED_BACK`：没有 `C1_READY`，没有 Mac 成功探测，不能归档 D0C03；回执表明本轮未留下已知 `47123` listener 或 portproxy。
- 失败点还不能区分“Python listener 启动后立即退出”和“listener 存在但 Windows 宿主无法直连 WSL IPv4”。下一步只做最小分层诊断或修正后重试，不扩大到防火墙、Cockpit、更新、SSH、源码或 Git。

结构化结论：

- `ATTEMPT1_RESULT=FAILED_BEFORE_PORTPROXY`
- `PORTPROXY_ADD=NOT_EXECUTED`（由已执行脚本的控制流位置确认）
- `C1_READY=NOT_REACHED`
- `MAC_SUCCESS_PROBE=NOT_EXECUTED`
- `DEVICE_RESIDUAL_CHECK=PASS`（用户转回的设备侧三项观测）
- `MAC_POST_ROLLBACK_NEGATIVE=NOT_APPLICABLE_BECAUSE_NO_PORTPROXY`
- `C1_PASS_ROLLED_BACK=NOT_ISSUED`
- 来源上限：聊天内用户转回文本，无独立附件 hash，也不是 Mac 直接取得的完整远端 transcript。

## C1 Attempt2 授权（2026-08-14）

- 用户在收到精确范围“仍用 `47123`、最长 300 秒，修正服务启动并分层诊断，然后执行同范围 portproxy、Mac 验证和完整回滚；再次失败就停止”后明确回复“批准”。
- Attempt2 必须在创建 portproxy 前依次证明：WSL 服务进程/作业仍存活、WSL `ss` 有精确监听、WSL 本地 HTTP nonce 成功、Windows 直连 WSL IPv4 HTTP nonce 成功。
- 五个 external target 均带 `attempt2`；Attempt2 若失败，先回滚并停止，不得自动运行 Attempt3。
- Mac target 精确覆盖同一 URL 的一次成功探测和回滚后一次失败探测；不覆盖其他地址、端口或持续探测。

## C1 Attempt2 运行中回执（2026-08-14）

5600X 用户转回的 READY 前证据：

- `DIAG_ATTEMPT1_ZERO_RESIDUAL=PASS`
- `DIAG_PY_JOB_ACTIVE=PASS pid=394`
- `DIAG_WSL_PID_MARKER=PASS`
- `DIAG_WSL_SS=PASS`
- `DIAG_WSL_LOCAL_HTTP=PASS`
- `DIAG_WINDOWS_DIRECT_HTTP=PASS`
- `DIAG_PRE_PROXY_REFREEZE=PASS`
- `DIAG_PORTPROXY=PASS`
- URL：`http://100.98.94.76:47123/syn-c1/f3c5da51aadc4c88ab1cf4cde5c69571`
- nonce：`cc2dd9db71ce4ff29c157b0859f574f6`
- expected body SHA-256：`f15c665baa9c6ef2ad82f7adb0ead058e18db657a5aeabe9931a1764518a35bb`
- WSL IPv4：`172.18.102.245`；绝对过期时间：`2026-08-14T06:16:19.1298916Z`；`FIREWALL_CHANGED=NO`。

Mac 在 `2026-08-14T06:13:38Z`、仍早于过期时间时执行获准的直连探测：

- `curl --noproxy '*'` 到上述精确 URL；退出码 `7`。
- `CURL_REMOTE_IP=`、`CURL_HTTP_CODE=000`、`CURL_TIME_TOTAL=0.002424`。
- 当前标签：`C1_FAILED_AT_C_MAC_PENDING_ROLLBACK`。A（WSL 本地）、B（Windows 直连 WSL）和 Windows 侧 portproxy 准备均通过，但 Mac 未取得 HTTP/nonce；不能签发 C1 通过。
- 根因边界：失败已缩小到 Tailnet 到 Windows 精确 listener 的入站路径；当前证据不能继续区分 Windows 入站防火墙、Tailnet policy/ACL 或该 listener 的远端可达性。

Mac 回滚后负向探测：

- `2026-08-14T06:17:49Z`（晚于绝对过期时间）对同一 URL 再次直连。
- curl 退出码 `7`；`POST_ROLLBACK_REMOTE_IP=`、`POST_ROLLBACK_HTTP_CODE=000`、`POST_ROLLBACK_TIME_TOTAL=0.001051`。
- `MAC_POST_ROLLBACK_NEGATIVE=PASS`：过期后 Mac 无法取得 HTTP 响应。仍需 5600X 窗口的 `ROLLBACK=...`、portproxy/Windows listener/WSL listener 三项输出，才能确认设备侧零残留。

5600X 用户转回的最终退场回执：

- `ROLLBACK=PASS`
- `PORTPROXY_47123=ABSENT`
- `WINDOWS_47123_LISTENER=ABSENT`
- `WSL_47123_LISTENER=ABSENT`

Attempt2 最终结论：

- `C1_FAILED_AT_C_MAC_ROLLED_BACK`
- `DEVICE_RESIDUAL_CHECK=PASS`
- `MAC_POST_ROLLBACK_NEGATIVE=PASS`
- `C1_ACCEPTED_ROLLED_BACK=NOT_ISSUED`
- D0C03 未达到“Mac 正向 nonce 成功”完成门，不能归档；按用户对 Attempt2 的明确限制，失败后停止，不运行 Attempt3。
- 最短后续候选是单独授权一轮：只临时新增一个精确的 Windows 入站 allow 规则（Windows Tailscale 本地地址、Mac Tailscale 远端地址、`47123/TCP`），重复同范围 nonce/portproxy 验证后连同规则一起删除。该候选当前未授权、未执行。

未完成原因：ATTEMPT2_FAILED_AT_C_MAC_ROLLED_BACK；设备侧与Mac侧均确认零残留；按用户授权停止，不运行Attempt3；等待新的精确临时防火墙规则授权

## C1 Attempt3 授权（2026-08-14）

- 用户在收到精确范围“同一 `47123`、最长 300 秒，临时允许 Mac Tailscale 地址访问 Windows Tailscale 地址；测试后删除该规则、portproxy 和 listener，失败即停止”后明确回复“批准”。
- Attempt3 只允许新增一条固定名为 `Syn-C1-Attempt3-47123` 的入站 allow rule：本地地址 `100.98.94.76`、远端地址 `100.120.223.16`、TCP、本地端口 `47123`、Profile Any；执行前该名称必须不存在，不得修改或删除既有 `Tailscale-In` 或其他规则。
- Attempt3 无论成功失败都必须删除本轮 rule、portproxy 和 listener并验零残留；失败后不自动进入 Attempt4。

## C1 Attempt3 正向回执（2026-08-14）

5600X 用户转回的 READY 前证据：

- `DIAG_ATTEMPT2_ZERO_RESIDUAL=PASS`
- `DIAG_FIREWALL_RULE_PRECHECK=PASS`
- `DIAG_PY_JOB_ACTIVE=PASS pid=396`
- `DIAG_WSL_LOCAL_HTTP=PASS`
- `DIAG_WINDOWS_DIRECT_HTTP=PASS`
- `DIAG_FIREWALL_RULE=PASS name=Syn-C1-Attempt3-47123 remote=100.120.223.16 local=100.98.94.76:47123/TCP`
- `DIAG_PORTPROXY=PASS`
- URL：`http://100.98.94.76:47123/syn-c1/de15bb6dff0d4c15894d071c403a73e4`
- nonce：`8ddc685828ad4443937c4403522ebccb`
- expected body SHA-256：`3c04b66c623f96191d264bf42e7961290b07302924c7f676234db35ea7e6e2b6`
- WSL PID：`396`；WSL IPv4：`172.18.102.245`；绝对过期时间：`2026-08-14T07:15:44.7564587Z`。
- 临时规则：`Syn-C1-Attempt3-47123`，`100.120.223.16 -> 100.98.94.76:47123/TCP`；`FIREWALL_CHANGED=TEMPORARY_EXACT_ALLOW`。

Mac 在 `2026-08-14T07:14:12Z` 之后、`2026-08-14T07:15:10Z` 之前且仍早于绝对过期时间时执行获准的正向探测：

- `curl --noproxy '*'` 退出码 `0`；`CURL_REMOTE_IP=100.98.94.76`、`CURL_HTTP_CODE=200`、`CURL_TIME_TOTAL=0.040989`。
- 响应中的 run id、nonce、`DESKTOP-FRK8K62`、`Ubuntu-24.04`、`synadmin`、两端 Tailscale IPv4、WSL IPv4、端口和规则名全部匹配。
- 本机按响应原始正文逐行复算 SHA-256 为 `3c04b66c623f96191d264bf42e7961290b07302924c7f676234db35ea7e6e2b6`，与 READY 的 expected SHA-256 一致。
- 当前标签：`C1_ATTEMPT3_POSITIVE_PASS_PENDING_ROLLBACK`。尚需 5600X 侧规则/portproxy/listener 零残留和 Mac 回滚后负向探测，当前不得归档。

Mac 回滚后负向探测：

- `2026-08-14T07:16:27Z`（晚于绝对过期时间）开始对同一 URL 探测。
- curl 退出码 `28`；`POST_ROLLBACK_REMOTE_IP=`、`POST_ROLLBACK_HTTP_CODE=000`、`POST_ROLLBACK_TIME_TOTAL=3.005054`。
- `MAC_POST_ROLLBACK_NEGATIVE=PASS`：Mac 已无法从该地址和端口取得 HTTP 响应。
- 当前仍需 5600X 管理员 PowerShell 的最终回滚输出，确认固定名规则、portproxy、Windows listener 与 WSL listener 全部不存在；在收到设备侧零残留回执前不得签发 `C1_PASS_ROLLED_BACK`。

5600X 用户转回的最终退场回执：

- `ROLLBACK=PASS`
- `FIREWALL_RULE_Syn-C1-Attempt3-47123=ABSENT`
- `PORTPROXY_47123=ABSENT`
- `WINDOWS_47123_LISTENER=ABSENT`
- `WSL_47123_LISTENER=ABSENT`

Attempt3 最终结论：

- `C1_ATTEMPT3_CHAIN=PASS`：Mac 在窗口内取得 HTTP 200、正确远端 IP、正确 nonce 和 WSL 身份正文，正文 SHA-256 与 READY 一致。
- `DEVICE_ROLLBACK=PASS`：用户转回的 PowerShell 退场回执确认本轮固定名 firewall rule、portproxy、Windows listener 与 WSL listener 均不存在。
- `MAC_POST_ROLLBACK_NEGATIVE=PASS`：绝对过期时间后对同一 URL 的新连接无法取得 HTTP 响应。
- `C1_PASS_ROLLED_BACK=ISSUED`
- `C1_ACCEPTED_ROLLED_BACK=ISSUED`
- 证据上限：仅证明本轮临时的 `Mac -> Windows Tailscale 100.98.94.76:47123 -> portproxy -> WSL 172.18.102.245:47123` 链路曾成功且随后零残留；不证明安全加固、长期稳定、SSH、源码迁移、独立 Headless Core、正式 Primary 或 authority epoch 切换。
- 来源边界：Mac 正向/负向 curl 为本任务直接执行；Windows/WSL READY 与最终退场状态为用户从 5600X PowerShell 转回的原始文本，无独立附件 hash。
