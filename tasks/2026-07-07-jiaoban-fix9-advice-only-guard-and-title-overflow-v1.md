# 实现任务包:fix9·纯建议方案不许无声开工(守卫+诚实脸+停因改口)+ 会话行标题溢出 · 主导线 → 执行线 v1

日期:2026-07-07　性质:**轻档**(单线双面·文件边界 §2.5;死线 0-diff)。

## 0. 接手须知(冷启即读,本包自包含)

- 你是**执行线**(单线双面)。**子线不 commit。** 全程中文。
- **事故背景(2026-07-07 16:48/16:55 用户真机两撞·主导线盘上定谳)**:用户目标明明是改文件(「加回 1 个怪」),tier-1 咨询**两次都没交 `execution_scope` 字段** → `map_consultation_to_c1_input` 的分流走了 None 支=**忠实映射成纯咨询只读方案**(scope_draft:写根空·roles=[project_consultant]·tools=[read_file]) → 授权卡没把「这方案不改任何文件」喊出来,用户批了 → 合流照 scope_draft 建了**空写根授权**(盘上 1783411685711/1783412134850 两份 active 实锤) → prepare 逐任务拦(「授权写入范围为空」+「目标角色不在授权范围内」×3) → blocked。**代码没退化**(同日 13:12 的方案档位灌得好好的·分流/档位单测全绿)——这是 tier-1 输出不稳家族第三案(前两案:consult 早退/suggest_workflow 摇摆)。
- **主导线已核的事实(直接用)**:分流在 `consultant_agent.rs::map_consultation_to_c1_input`(match `proposal.execution_scope`·None 支注释自称「忠实映射不是兜底」——语义没错,错在**None 分不清「LM 判定纯咨询」和「LM 忘了给字段」**,且下游没人把纯建议喊响);合流=`director_agent.rs::confirm_and_start_authorized_run`(1829);停因老话在 `director_agent.rs:1388`(还在教用户「在方案里补上」写范围——档位时代这是死胡同);前端 `willWrite`(=写根非空)判据已存在(授权卡 🔓 行就是它)。

## 1. 拍板摘要

- **要做的事**:三层堵死「纯建议方案被当施工单批走」:① 开工口守卫(确定性·零 LM 依赖)② 批卡诚实脸(批前就喊出来)③ 停因文案改口(不再指死胡同);顺带修会话行标题溢出(用户实测:收起+展开都溢)。
- **为什么**:交办第一次在真机上让用户批了两份空转方案还查不出所以然;「永不冻」不光是有按钮,还得是**按钮指对路**。
- **代价**:一轮。后端两处守卫+一句文案+prompt 硬化;前端一条黄条+主按钮改道+CSS。

## 一句话判据

**「是不是只:开工口对空写根方案人话拒(不建授权不起链)+ 批卡对纯建议方案响脸 + 1388 文案改口 + prompt 硬化一段 + 溢出量了才修——而分流本体/档位函数/人闸语义/prepare guard/判决体全 0-diff?」** 是 → 做;否 → 停、回主导线。

## 2. 建什么

### 2.1 后端 a·开工口守卫(fix9 主件·确定性)

- `confirm_and_start_authorized_run`:人闸校验之后、**建授权之前**——方案 `scope_draft.allowed_write_roots` 为空 → **拒绝开工**,人话:「这份方案是纯建议(咨询判定不需要改文件),没有可执行范围,开工只会空转。想让 AI 动手:点[重新出方案],把要改什么说清楚(带上文件名/功能名更稳)。」**不建授权、不绑会话、不起链**(16:48 那种空授权垃圾不再入库);审计留档走现有 stopped 语义事件,别新开事件族;
- `run_auto_advance_authorized_role_loop`(=[接着跑]口):查到 active 授权后同判——授权 `allowed_write_roots` 为空 → 同款人话停(**盘上已有两份空 active 授权残留**,这道守卫让它们从「坑」变「哑」,零 store 手术);
- 注释注明:当前档位世界「写根空 ⇔ 纯建议」;将来若出现「只读但要跑检查」的新档位形态,此守卫须随分流一起升级。

### 2.2 后端 b·停因文案改口 + prompt 硬化

- `director_agent.rs:1388`:老话「方案缺了它该写的内容(如写范围/工具/检查)…或在方案里补上」= 档位前时代误导(写范围早已由档位装配,用户没处「补」)→ 改:「这单的授权没带可执行范围(多半是方案被判成了纯建议)。请点[重新出方案]把要动手的内容说清楚——写范围由系统自动装配,不需要你手填。」;
- 咨询 prompt(consultant_agent 内 prompt 常量段)硬化一句:「凡用户目标涉及创建/修改/删除任何文件或功能,**必须**输出 execution_scope(target_files 按最合理猜测填);仅当目标是纯提问/纯分析时才省略。」——如实标注:prompt 改动只是提高概率,**不承诺根治**(tier-1 不稳是既定认知,根治靠 2.1 守卫+2.3 响脸)。**分流 match 本体与 `profile_edit_test_project_scope` 函数体 0-diff。**

### 2.3 前端 c·批卡纯建议诚实脸

- 授权卡:`willWrite === false`(现成判据)→ 卡顶醒目警条:「⚠ 这份方案**不会改任何文件**——它是纯建议。你的目标若是要动手改东西,别批这份,点下面[重新出方案]。」;主按钮位换 **[重新出方案(要动手)]**(调现成 backToSay);[允许并开始] 降为次按钮(留着=用户真想收下纯建议也有路;点了会被 2.1 拒并人话上脸——现有失败上脸机制接得住);
- `willWrite === true` 一切原样(零回退);离线 DOM 断言两态。

### 2.4 前端 d·会话行标题溢出(用户实测:批卡会话收纳行,收起+展开都溢)

- **先量后改(硬要求·此仓渲染坑规矩)**:照刀A headless 真渲染先例,造长标题 fixture(两种:≥120 字连续中文、≥120 字符无空格英文串),真渲染 `JiaobanSessionPicker`(已 export)收起+展开两态,**量 scrollWidth>clientWidth 定位溢出盒**(数值/截图留证)→ 修 → 复量=0 溢出,前后证据都进回交;
- 主导线代码侧候选弱链(供比对,**以量到的为准修,不许全撒**):① 收起行 `.jiaoban-session-summary-row` 是 `.jiaoban-session-pick`(grid)的 item、缺 `min-width:0`;② 展开列表 `.jiaoban-radio` 里标题是**裸文本节点**(匿名 flex item 选不中、约不住)→ 包一层 span 再加 `min-width:0`+`overflow-wrap:anywhere`;③ `.jiaoban-session-expand`/`.jiaoban-session-rest` 同为 grid item 缺 `min-width:0`;④ `.project-jiaoban-col` 作为 flex item 缺 `min-width:0`;
- 真机最终判据=用户(他的真实长标题会话)。

### 2.5 文件边界(越界即停)

- 允许:`director_agent.rs`(合流守卫+auto_advance 守卫+1388 文案)/ `consultant_agent.rs`(**仅 prompt 常量段**)/ `ProjectJiaobanPanel.tsx` / `projectWorkflowSidePanel.css` / `tests/` 新文件 + 跑器 1 行 / 量具临时脚本(scratch 或 scripts 下临时文件·不进产线依赖·回交后可删);
- **0-diff**:`map_consultation_to_c1_input` 分流 match 本体 / `profile_edit_test_project_scope` / c4_c6(prepare guard 本体)/ controller / commands / runner / control_core / worker_report / global_supervisor_agent / 两执行 store / manual_relay / lib.rs。

## 3. 安全死线

- 守卫**只收紧不放宽**:人闸语义原样(守卫在人闸后);prepare guard 原样(守卫是提前拦,兜底仍在);**绝不许**把「LM 没给 execution_scope」默认成档位(那是反向放权=把纯咨询错当施工,违最小权);
- 渲染类真机过;fmt 老规矩(skip_children)。

## 4. 验收

- **单测**:纯建议方案(写根空)→ 合流拒·人话对·**授权店零新增**;正常档位方案 → 照旧全通;空写根 active 授权 + auto_advance → 人话停不再进 prepare;1388 新文案断言;
- **离线 DOM**:willWrite=false → 警条在+主按钮=[重新出方案];willWrite=true → 原样;
- **溢出**:量具前后数值/截图(§2.4);
- **真机(用户)**:①故意等一份纯建议方案(或用盘上 16:53 那份 user_confirmed 的)→ 批卡应见警条+主按钮改道;②正常方案照旧跑通;③长标题会话收起/展开不再溢出;
- 三闸绿 + §2.5 0-diff 自证 + 计数不降。

## 5. 回交

- §4 证据(含溢出前后量值)+ 落点清单 → 主导线核实物。**子线不 commit。**

## 7. 不接受为

- 分流 None 支默认成档位 / 动分流 match 或档位函数 / 守卫放在人闸前(改人闸语义)/ 新开审计事件族 / 溢出不量就撒 CSS / [允许并开始] 在纯建议卡上被删死(降级可以,删死不行——按钮永远有路)。
