# 实现任务包:A·工作台会话在智能体页可见(修显示 bug)· 主导线 → 执行线 v1

日期:2026-07-09　性质:**轻档**(后端读模型合并 + 前端徽标·不碰死线)。缘起:C1 真机验成但**智能体页看不见任务会话**(Cmd+R/重启都没用)。主导线已核根因到底(见 §0)。

## 0. 接手须知(冷启即读·本包自包含·前提已核到底)

- 你是**执行线**。**子线不 commit。** 全程中文。
- **根因(主导线亲验·对当前代码+真数据)**:智能体页会话列表命令 `load_codex_session_page`(commands.rs:256)调 `codex_db::read_threads_page`,那条 SQL **过滤 `WHERE has_user_event=1`**(codex_db.rs:90 注释「Skips threads where has_user_event=0」);而**工作台用 `codex exec` 建的会话全是 `has_user_event=0`**(实测 state_5.sqlite:C1 四条任务会话 019f4617/4619/4623/4625 标题「交办任务专用会话:本会话只承接任务「…」」全 `has_user_event=0`)→ 被永久藏掉。不是缓存,重启无效。
- **钥匙已现成**:`codex_db::find_thread_by_id(db_path, thread_id) -> Result<Option<CodexThreadRow>>`(codex_db.rs:118)按主键只读、**绕过 has_user_event 过滤**(旁边测试 `find_thread_by_id_sees_exec_thread_hidden_from_list`:490 印证能看见列表藏的 exec 会话)。
- **工作台会话的判据信号**:workflow store 的 `workflow_node_session_bindings[].native_thread_id`(= 工作台真绑到工作流节点在用的会话)。这是「工作台的会话」最有据的信号(不靠标题猜)。
- **映射器现成**:`session_record_from_codex_thread(CodexThreadRow) -> SessionRecord`(index_host_app_entrypoints.rs:35)。`SessionRecord` 定义在 `workbench_snapshot_types.rs:69`。

## 1. 拍板摘要

- **做什么**:智能体页会话列表 = 「有用户事件的会话(现状)」**并上**「工作台绑过工作流节点的会话」(即使 has_user_event=0)。让 C1 任务会话看得见。
- **怎么做**:**不动** `read_threads_page` 的过滤(codex 空占位噪音照样藏);在 `load_codex_session_page` 里**加一步合并**——取 store 绑定的 thread、去重、`find_thread_by_id` 解析被过滤掉的、映射+标记、并进列表。
- **为什么这么划**:过滤有用(codex 一堆空会话该藏),不能删;只**定向补上工作台真在用的**。

## 一句话判据

**「是不是只:`load_codex_session_page` 加『合并 store 绑定会话』一步(find_thread_by_id 解析·标 workbench_bound·去重)+ SessionRecord 加 workbench_bound 标记 + 前端徽标——而 `read_threads_page` 过滤本体 0-diff、`find_thread_by_id` 0-diff、codex 会话创建/relay/runner 不碰?」** 是 → 做;否 → 停、回主导线。

## 2. 建什么

### 2.1 列表合并工作台会话(commands.rs·主改)

- `load_codex_session_page`(commands.rs:256)成功分支里,建好过滤后的 `sessions` 之后:
  1. **仅首页**(`offset==0`·后页这些已在首页显示过·避免重复与分页错乱)读 workflow store 的 `workflow_node_session_bindings`,取所有 `native_thread_id`(非空);
  2. 去掉已在 `sessions` 里的(按 thread_id 去重);
  3. 剩下的逐个 `codex_db::find_thread_by_id` 解析(拿 CodexThreadRow);None/archived 跳过;
  4. 映射成 SessionRecord(复用 `session_record_from_codex_thread`)、**标 `workbench_bound=true`**;
  5. 并进 `sessions`,按 `updated_at_ms` 倒序重排(与现状排序一致·自然交错;若你判「置顶更合用」留注释说明并二选一);
- **软着陆**:读 store 绑定失败 → 只出 warning、返回原列表(别 Err 断列表·显示是增益不是闸);
- **不动** `read_threads_page` 过滤本体、不动 `find_thread_by_id`。

### 2.2 SessionRecord 加标记(workbench_snapshot_types.rs)

- `SessionRecord` 加 `workbench_bound: bool`(`#[serde(default)]`·旧路径零改);`session_record_from_codex_thread` 默认给 false(§2.1 合并处才置 true);其余 SessionRecord 构造点补默认 false(机械必需·如 commands.rs:494 / index_host:128 / lib 测夹具——逐个补,别漏致编译错)。

### 2.3 前端徽标(agents 视图)

- 会话列表项:`workbench_bound===true` 的加一个小徽标(如「工作台任务」)让用户一眼认出;不改其它交互。

### 2.4 明确不做

改 has_user_event 过滤本体(噪音还得藏)/ 动会话创建·relay·runner(那是死线,本 bug 在读侧)/ 分页把工作台会话铺到每页(只首页并一次)/ C2 的活。

## 3. 安全死线

- `read_threads_page` 过滤 SQL 本体 0-diff、`find_thread_by_id` 0-diff、`codex_local_runner`/`manual_relay`/沙箱/安全闸 0-diff(本 bug 纯读侧显示·不碰任何执行/创建/凭据);
- `.codex` 只读(现状 codex_db 已只读打开);不写 `~/.codex`。

## 4. 验收

- **单测**(仿 `find_thread_by_id_sees_exec_thread_hidden_from_list` 场景):造一条 has_user_event=0、且在 store `workflow_node_session_bindings` 里绑过的 thread → `load_codex_session_page` 首页结果**含它且 `workbench_bound=true`**;不在绑定里的 has_user_event=0 会话**仍不出现**(证只补工作台的、没把噪音也放出来);后页(offset>0)不重复注入;
- **软着陆测**:store 绑定读失败 → 返回原列表 + warning、不 Err;
- **真跑/真机**(你的原始场景):智能体页现在能看见今天那 4 条「交办任务专用会话」;
- 三闸绿 + 计数不降 + fmt **自己真跑 `rustfmt --check` 别自报**(前科)。

## 5. 回交

- §4 证据(尤其「绑定的出现 / 没绑定的噪音仍藏」两侧都测)+ 落点清单 → 主导线核实物。**子线不 commit。**

## 7. 不接受为

- 删/放宽 has_user_event 过滤本体(噪音会灌进来) / 靠标题字符串猜「是不是工作台会话」(要靠 store 绑定这个硬信号) / 动会话创建/relay/runner/沙箱 / Err 断列表(软着陆) / 自报 fmt 不真跑 / 顺手做 C2。
