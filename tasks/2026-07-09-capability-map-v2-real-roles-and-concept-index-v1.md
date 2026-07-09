# 调研任务包:能力地图 v2(真角色 + 概念→在哪反查索引)· 主导线 → 执行线(便宜模型) v1

日期:2026-07-09　性质:**纯只读调研·升级地图**(grep+读头注释+填反查·零代码改)。缘起:v1(`docs/2026-07-09-codebase-capability-map-v1.md`)是骨架——角色多套话、关键文件 pub 面空、**没有"概念→在哪"反查**(写加新能力包前真正要查的)。v2 补这三样。**便宜模型可做**:结构卡死·播种概念清单让你只 grep 填位·判断留主导线。

## 0. 接手须知(冷启即读·本包自包含)

- 你是**执行线**(调研·**便宜模型**)。**只读·产/改地图文档·不改任何代码·不 commit。** 全程中文。
- **接 v1 升级**:读 `docs/2026-07-09-codebase-capability-map-v1.md`,**在它基础上产 v2** `docs/2026-07-09-codebase-capability-map-v2.md`(别推翻·补三样:真角色/补全 pub 面/概念反查)。仓根 `/Users/yoyi/workspace/product-line`;后端 `prototypes/productized-desktop-shell/src-tauri/src`、前端 `prototypes/productized-desktop-shell/src`。

## 1. 要补的三样

### 1.1 真角色(禁套话)

- 每个文件的"一句话角色"**必须来自文件自己的头注释(`//`/`///` 开头几行)或主函数名**,禁止套话。**禁词表**(出现即算没做):「后端能力封装」「XX 相关核心能力」「工作流核心能力实现」「前端通用类型、状态派生和 Tauri 适配」这类**填充分类**;
- 做法:每文件读**前 20 行 + 头注释**,一句话说清"它到底管什么"(如 `codex_local_runner.rs`=「codex exec/resume 真跑进程封装+命令拼装+供给类错误分类」,不是「codex 相关核心能力」)。

### 1.2 补全 pub 面(尤其命令文件)

- v1 里标「(无显式pub/导出)」的文件多是漏了 `#[tauri::command]`/非 pub `fn`——**逐个补**:`grep -nE '#\[tauri::command\]|pub(\(crate\))? fn |pub(\(crate\))? struct |^fn |^async fn ' <文件>` 取代表性 5-8 个;命令文件(commands.rs 等)重点列**命令名**。

### 1.3 概念→在哪 反查索引(★v2 核心·防重造用)

- 新一节 `## 概念→在哪（写"加新能力"包前先查这里）`:**每个能力概念一行 → 它散落/落地在哪些 file:fn**。**下方是主导线播种的起点清单,你逐个 grep 补全 file:fn + 补你 grep 时发现的新概念**:

| 概念 | grep 关键词(种子) | 你填:落在哪些 file:fn |
|---|---|---|
| 错误人话翻译 | classify_codex_provider_failure / humanize_consult_error / classify_codex_resume_failure | (已知散四处·填全) |
| 会话列表(含工作台会话) | load_codex_session_page / read_threads_page / find_thread_by_id | |
| codex 真跑(exec/resume) | run_phase_b / command_plan_for / resume | |
| 先生后绑建会话 | create_and_bind_task_session / ManualRelayJiaobanNewSessionCreator | |
| worker 回程契约/求助 | parse_worker_report / help_signal_from_raw / WORKER_REPORT_CONTRACT | |
| 主管终标/退回 | director_final_screen / reset_work_item_for_director_rework | |
| 主管总结→记忆候选 | capture_director_summary_candidate / capture_event | |
| 记忆五层写入 | memory_candidate_store / formal_memory / observation | |
| 授权闸/人闸 | require_active_authorization / plan_authorization / boundary_review | |
| path-lock/沙箱 | legacy_product_command_blocked / sandbox / path-lock | |
| prepare 物化/就绪闸 | prepare_authorized_auto_dispatch / needs_binding | |
| 状态机迁移校验 | workflow_transition_allowed / NODE_ALLOWED_TRANSITIONS | |
| 读模型投影 | derive_subagent_reports / read_model | |
| 记忆采集总线 | memory_capture_bus / CaptureSource | |
| relay 手动指挥 | manual_relay / new_session | |
| 审计事件 | append_chain_audit / audit_events / event_type | |
| 秘书(零写入) | secretary_agent / run_secretary_explain | |
| 全局主管两钩点 | global_supervisor / boundary_review | |
| 画布/HUD | WorkflowCanvasEngine / canvas | |
| 前端会话中心 | AgentSessionList / focusedThreadId | |

- **每个概念填 file:fn 时,若发现同一概念有"疑似两套/多套"实现,标 ⚠️** 给主导线(这正是防重造要的)。

## 2. 死线

- **只读**:不改任何 `.rs`/`.tsx`/v1 地图/其它现有文档,只新增 v2 文档,不 commit,不跑写盘命令;
- **不判决**:不下"这是重复该合"结论(⚠️ 标出来·主导线判);真角色只如实转述文件注释、不脑补;
- 不碰 `.codex`/沙箱。

## 3. 回交格式(v2 文档结构)

```
# 代码库能力地图 v2（2026-07-09·便宜模型·主导线待核）
## 后端能力（按 8 区域·每行:文件 — 真角色[来自头注释] — 代表 pub 面/命令）
## 前端能力（按六面·同上）
## 概念→在哪（★种子清单 + 你补全的 file:fn + ⚠️疑似多套）
## 覆盖率自评 + 撞见的疑似多套清单
```

## 4. 回交 → 主导线

- v2 文档路径 + 覆盖率自评(禁词残留几处、pub 面补了几个空、概念反查填了几行)+ ⚠️疑似多套清单 → 主导线核+替换 v1 为正本。**你不 commit。**

## 7. 不接受为

- 角色还用禁词套话(必来自文件注释)/ 命令文件 pub 面仍空 / 概念反查种子没填全 / 下"该合"结论(⚠️ 标·主导线判)/ 改代码或 v1 或其它文档 / commit / 脑补文件没有的角色 / 跑写盘命令。
