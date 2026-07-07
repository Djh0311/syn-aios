# 回交:fix9·纯建议方案守卫+诚实脸+停因改口+标题溢出 · 执行线(单线双面)→ 主导线 v1

日期:2026-07-07 · 包:`tasks/2026-07-07-jiaoban-fix9-advice-only-guard-and-title-overflow-v1.md`。**子线未 commit。** 轻档。

## 一句话结论

三层堵死「纯建议方案被当施工单批走」(开工口/接着跑口确定性守卫+批卡诚实脸+1388 改口+prompt 硬化),单测复刻 16:48 事故形态全绿;溢出件**先量后改**——真 Chromium 量出根因=主导线候选①(收纳行缺 `min-width:0`),只改 2 处 CSS,复量文档横滚 1937→577(=零横滚)。死线全 0-diff。剩真机(§6)。

## 1. 落点清单

**a·开工口守卫**(`director_agent.rs`·确定性零 LM 依赖):
- 合流:人闸校验(PendingUserConfirmation)**之后**、record_decision(建授权)**之前**——`proposal.scope_draft.allowed_write_roots` 空 → 人话拒「这份方案是纯建议…点[重新出方案]…」;**不建授权、不绑会话、不起链**;留档走现有 `role_loop_auto_advance_stopped` 事件族(不新开);注释注明「写根空⇔纯建议」的档位世界前提与升级条款;
- [接着跑]口(`run_auto_advance_authorized_role_loop`):拿到 active 授权后 `active.scope.allowed_write_roots` 空 → 同款人话停,在 started 审计**之前**拦(没开始就不记 started);**盘上 16:48/16:55 两份空授权残留由此从「坑」变「哑」,零 store 手术**。

**b·停因改口+prompt 硬化**:
- 1388 老话(「或在方案里补上」死胡同)→「这单的授权没带可执行范围(多半是方案被判成了纯建议)。请点[重新出方案]…写范围由系统自动装配,不需要你手填。」(保留 reasons_text 具体原因);
- `consultant_agent.rs` **仅 +1 行** prompt:「凡目标涉及创建/修改/删除文件或功能,必须输出 execution_scope(target_files 按最合理猜测填)…漏给=空转单」。如实标注:**只提高概率不承诺根治**(tier-1 不稳第三案),根治靠 a+c。
- 白捡的一致性:`classifyBlocked` 规则 2 正则(`重新.{0,4}方案`)**天然命中**全部三句新话 → 守卫 Err 上卡住脸时主按钮自动=[重新说目标出新方案],指对路,前端配对表零改动。

**c·批卡诚实脸**(`ProjectJiaobanPanel.tsx`+css):
- `willWrite===false`(现成判据)→ 顶部警条「⚠ 这份方案**不会改任何文件**——它是纯建议…」(复用 stale banner 样式族·纯 selector 加法)+ 主按钮位改 **[重新出方案(要动手)]**(调现成 onRePlan)+ [允许并开始] 降次按钮改文案「仍要允许并开始(纯建议)」——**不删死**(§7:按钮永远有路;硬点会被 a 守卫人话拒,现有失败上脸机制接住);
- 分支优先级:纯建议 > 旧方案 > 正常;`willWrite===true` 零回退(离线断言)。

**d·会话行标题溢出(先量后改·§2.4)**——见 §3。

## 2. §4 机器证据

- **单测**(`director_agent.rs` 自包含 `fix9_tests` mod·3/3 绿):
  ① `fix9_confirm_rejects_advice_only_proposal_before_authorization`:**真分流**造事故形态(execution_scope=None→map→断言写根空)→ 合流拒·人话对·**授权店零新增**·方案仍 Pending(人闸语义没动)·stopped 留档·runner/director/creator 三 panic 桩全没炸;
  ② `fix9_auto_advance_rejects_empty_write_root_active_authorization`:**全程生产 API 复刻 16:48 存量链路**(确认+边界复核→真建出空写根 active 授权·前置断言事故形态成立)→ [接着跑]人话停·stopped 在·**started 不在**·panic 桩没炸=没进 LM/prepare;
  ③ `fix9_guard_does_not_touch_profile_backed_proposal`:档位方案(写根非空)不被误伤——走到绑会话步才因假会话失败(证明穿过守卫走的正常路径)。
- **离线 DOM**(新 `tests/advice-only-authorize-face.test.tsx`+跑器 1 行·2 组全过):纯建议→警条+primary 位=改道按钮+[仍要允许]在;正常→无警条+主按钮原样+🔓 行原样。offline 全套 15 passed。
- **全量**:`cargo test --lib` = **693/0/40**(基线 690+3;计数不降)。三闸:tsc 绿/offline 绿/build ✓。fmt:director/consultant `skip_children` check CLEAN。

## 3. §2.4 溢出量值(前后全量·真 Chromium)

**量具**:Claude Preview MCP(2026-06-17 拍板的「agent 眼睛」先例)——esbuild 把**真组件**(`JiaobanSessionPicker` 产线 export)+**真 CSS**(styles.css+面板 css 全量)打包成自包含页,480px 容器复刻真实容器链(`.project-jiaoban>.project-jiaoban-col>.project-canvas-detail-card`),fixture=两条 ≥120 字长标题(连续中文/无空格英文)。**临时件全在仓库外**(量具在会话 scratchpad、launch.json 在 `/Users/yoyi/workspace/.claude/`,`git status` 仓库零足迹)。

**BEFORE(修前·收起态与展开态同病)**:
| 盒 | clientW | scrollW |
|---|---|---|
| `section.project-jiaoban` | 480 | **1921** |
| `.project-jiaoban-col` | 480 | **1921** |
| `.project-canvas-detail-card` | 480 | **1921** |
| `.jiaoban-session-pick` | 478 | **1920** |
| `.jiaoban-session-summary-value` | **1726**(被撑开·ellipsis 失效) | 1788 |

文档横滚 `docScrollX=1937`(视口 577)。截图:收起行与展开列表全部横向撑破卡片(与用户实测一致)。

**根因(量到实锤)**= 主导线候选①:`.jiaoban-session-summary-row`(flex 行)是 `.jiaoban-session-pick`(grid)的 item,**grid item 默认 `min-width:auto`** → 长标题把整行撑到 1920 → 行内 summary 按钮跟着宽 → value 的 `min-width:0+ellipsis` 失去约束基准。

**修(只改量到的,2 处 CSS)**:① `.jiaoban-session-summary-row { min-width:0 }`;② 复量时暴露的连带刺:行宽被约束后 label「用哪个对话干:」被 value 挤成竖排 4 行 → `.jiaoban-session-summary .jiaoban-field-label { flex:none }`(压缩全让给本就 ellipsis 的 value)。

**候选②③④量证不需要**:展开列表 radio 裸文本(候选②)修①后 `clientW==scrollW==454`、中英文都正常换行(高 107px/86px)——面板 CSS **既有** `overflow-wrap:anywhere`(css 30/38/91 行)已覆盖;候选③④(expand/rest/col 的 min-width)在①修后链上无一溢出。「以量到的为准修,不许全撒」执行到位。

**AFTER(复量终值)**:收起+展开两态,溢出清单仅剩 `.jiaoban-session-summary-value`(scrollW 1788>clientW 268,`overflow:hidden+text-overflow:ellipsis`——**这是 ellipsis 的工作原理本身**,非视觉溢出);`docScrollX=577`(=视口,零横滚);label 高 21px(恒一行)。截图:收起行「用哪个对话干: 接现有·这是一条特别长的连续中文会话标…▾ 看原始对话」一行齐;展开列表两条长标题整齐换行在卡片内。

## 4. 0-diff 自证(§2.5 全名单)

改动面=允许名单精确吻合:`director_agent.rs`(+392/-1:守卫×2+文案+自包含测试 mod)/ `consultant_agent.rs`(**+1/-0,全部 diff 行肉眼核过=仅 prompt 一句**;`grep scope_draft|profile_edit_test_project_scope|match &proposal.execution_scope` 命中 **0** = 分流 match 与档位函数 0-diff)/ Panel / css / 新测试 / 跑器 1 行。死线逐一 `git diff --stat` 空:**c4_c6(prepare guard)/ controller / commands / codex_local_runner / control_core / worker_report / global_supervisor_agent / global_supervisor_review_store / manual_relay / lib.rs 全 0-diff**。安全死线:守卫只收紧(在人闸后·prepare guard 兜底仍在);**没**把 None 默认成档位(反向放权红线没碰);[允许并开始] 降级未删死。

## 5. 真机待验(§4·用户)

1. 故意等一份纯建议方案(或用盘上 16:53 那份 user_confirmed 的走[接着跑])→ 批卡应见 ⚠ 警条+主按钮=[重新出方案(要动手)];硬点[仍要允许并开始(纯建议)] → 卡住脸人话「纯建议…重新出方案」且主按钮指[重新说目标];
2. 正常方案(带 execution_scope)照旧跑通(零回退);
3. 你的真实长标题会话:批卡收纳行收起+展开都不再溢出(量具已过,真机终判在你);
4. [接着跑] 对旧空授权:人话停、不再进 prepare 撞「授权写入范围为空」×N。

## 6. 回交动作

§4 证据+溢出前后量值(§3 表)+落点清单如上 → 主导线核实物。临时件(scratchpad 量具+workspace 根 launch.json)不进仓库、可随手清。**子线不 commit。**
