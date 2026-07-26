# S1B-H2-R3 lifecycle 与私有 runner 脱敏矩阵

## 逐条生命周期

| message | recorded | prepared | registry | binding / `thread.started` | runner exit | injected / reply | 最早可证边界 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `…1878362000` | 是 | 无 | 无历史证据 | 无 | 无 | `0 / 0` | recorded 后、成功 output 初始化 / prepared 前 |
| `…6842452000` | 是 | 无 | 无历史证据 | 无 | 无 | `0 / 0` | 同上 |
| `…0019151000` | 是 | 无 | 无历史证据 | 无 | 无 | `0 / 0` | 同上 |

说明：registry 当前 entries 为 `0`，且 R2 时间窗没有 registry audit；常规 register/unregister 本身不保留历史 audit，故此列不能单独推导“从未 spawn”。但 supervisor JSON 与 SQLite 两类存储均没有本窗 `turn_prepared`、session created/reused/replaced、invalid-resume、binding、turn exit 或 tool-call 事实。

三条都没有 `prepared.active_message_id`，所以不存在可把其中任一 message 归属到 resident run 的持久桥。唯一可读到的是**历史** session：`R6#18ad059764393529`、generation `6`、thread `T6#fa73c9bdbd15c4c3`、host PID `0`、`resident_exited` / `turn_failed`，最后时间为 07-19。它不是三条任一消息的 run 证据；只有在 `load_resident_session` 成功返回该非空 thread 时，源码才会选择 resume 预检。

## 会话和私有面

- 当前目标 resident session 仍是历史 generation `6`，历史 thread 仅以 `T6#fa73c9bdbd15c4c3` 表示；`host_pid=0`，`launch_status=resident_exited`，terminal family=`turn_failed`，active message/proposal 均为空，最后生命周期时间为 07-19。
- 因此不存在 `resident_turn_cleanup_failed` 的已证状态；不能把后两条直接裁为清理残留的 fail-closed。
- R2 时间窗没有 g6-resume 预期产物目录（manifest id `174bc4a1b6bf50ed`），也没有可能 invalid-resume 轮转后的 g7-initial / g7-resume 目录（manifest id `f7cbae236977d816` / `6f0b5cad2bebc43d`）。
- 所有现存 supervisor runner 产物的最新 mtime 为 `1784430552`，早于首条案发记录约 65 小时 35 分；active home 元数据/白名单文件均止于该历史时点，权限仍为 owner-only。认证链接未解引用。
- 历史 g5-resume 可被受控分类为 `invalid_resume`，但时间、generation、message identity 均不关联 R2；仅标为历史，不得倒推本案。历史 g6-initial 亦同。

结论：私有面不是“最后一次覆盖前三次”，而是本窗没有任何 runner 侧输出目录或文件可关联。没有私有正文、stderr、路径、认证资料或 `CODEX_HOME` 内容进入本证据。
