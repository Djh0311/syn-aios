# 实现任务包:交办 fix8 后端(codex 供给类失败分类:额度/订阅/登录 → 人话错误 + 不空重试)· 主导线 → 执行线 v1

日期:2026-07-05　性质:**轻档**(失败**报告**路径加缝;confinement/命令计划/guard 本体 0-diff——见 §3 精确圈界)。

## 0. 接手须知(冷启即读,本包自包含)

- 你是**执行线**(后端)。**子线不 commit。** 全程中文。
- **背景(用户今晚真实撞的·主导线已抓到原始错误)**:codex 订阅到期时,`codex exec` 先重连 5 次(~1 分钟)再死,stderr 是:
  `ERROR: unexpected status 403 Forbidden: {"code":"SUBSCRIPTION_NOT_FOUND","message":"No active subscription found for this group"}`
  但 `readonly_codex_consult` 的失败路径只返回 `consult_last_message_read_failed:No such file or directory`(codex 没写 last-message 文件)——**403 真相丢了**。后果:① 界面无法区分"额度死"和"网络抽风"(同一个错误串);② fix3 的 `is_director_plan_flaky_early_exit` 认这个串 → **额度死也被自动重试一次**,又白等一分钟。用户排了一晚上才靠人肉想起额度。
- **先读**:`codex_local_runner.rs::readonly_codex_consult` 失败路径(~354-361)与 `RealCodexLocalPhaseBProcessRunner::run_phase_b` 的结果结构(stderr/输出**捕没捕、捕到哪**——manual_relay 有 capture_to_files 先例,phase_b runner 大概率同款,找到它);`director_agent.rs::is_director_plan_flaky_early_exit` / `is_tier1_early_exit`。
- **一句话**:consult/worker 真跑失败时,把捕获的 stderr 尾巴**读出来分类**——命中供给类特征(`403`/`SUBSCRIPTION_NOT_FOUND`/`usage limit`/`quota`/`401`/`unauthorized`/`Reconnecting.*5/5`)→ 返回**带前缀的人话错误** `codex_provider_unavailable:…`(附 stderr 尾巴);没命中 → 现状错误 + 附 stderr 尾巴(真相别再丢);两处 retry 判据**排除**供给类(额度死不重试)。

## 1. 拍板摘要

- **要做的事**:额度/订阅/登录类失败从"无声+误重试"变成"一句人话+不空等"。
- **为什么**:今晚实战——这类失败目前表现为彻底静默,用户逐层排查一晚;违「永不冻」。
- **代价**:一轮·后端(失败报告分类 + retry 判据排除;UI 上脸是姊妹包)。

## 一句话判据

**「是不是只在失败**报告**路径读 stderr 尾巴做分类(命中→codex_provider_unavailable 前缀·未命中→原错误+尾巴),并让两处 retry 判据排除该前缀——而 confinement 构造/command_plan/guard/execute 判决体一字未动?」** 是 → 做;否 → 停、回主导线。

## 2. 建什么

1. **失败分类帮手**(如 `classify_codex_provider_failure(stderr_tail: &str) -> Option<String>`):命中特征(§0 那串·大小写不敏感)→ 返回人话(如「codex 供给不可用(403 订阅/额度/登录):{原始码}——处理后重试」)。
2. **consult 失败路径接上**(readonly_codex_consult ~354-361 区):`real_codex_executed==false` 与 `consult_last_message_read_failed` 两支,都先取 stderr 尾巴(runner 结果里找;若 phase_b 没捕 stderr → 在 runner **调用层**补捕[临时文件·同 manual_relay 先例],**别改 runner 本体**)→ 分类命中 → `Err("codex_provider_unavailable:{人话}")`;未命中 → 现状错误串 + `｜stderr尾:{tail截200字}`。
3. **worker 派发同待遇**:`execute` 返回的 dispatch failure 里若能拿到 stderr(查现有 warnings/summary 通道),同样分类进 failure message(**不改 execute 判决体**,只在 director_agent 消费侧翻译也行——实现者选侵入最小的层)。
4. **retry 排除**:`is_director_plan_flaky_early_exit` 与 `is_tier1_early_exit` 对 `codex_provider_unavailable` 前缀/特征**返回 false**(额度死重试=白等一分钟)。
5. **审计沿用**:fix3 的失败留档照旧(现在留下的是有信息量的错误)。

## 3. 安全死线

- `codex_local_runner.rs` 只许动**失败消息构造**那一小段(§2.2):`build_readonly_consult_request`/confinement/豁免 reason 表/`command_plan_for`/runner 本体——**逐函数 0-diff**(回交贴自证);
- `commands.rs`(execute)/`control_core`/`c4_c6`/chain controller/全部既有死线 0-diff;
- 分类**只影响报告与 retry 判据**,不改任何成败判定(失败仍失败);特征表保守,拿不准不归类(宁可报原始错)。

## 4. 验收

- **单测**:403/SUBSCRIPTION 样本→分类命中·人话对;`usage limit`/`401` 命中;普通早退(空 stderr)不命中→走原错误+尾巴;retry 判据对 provider 前缀 false(注入假 provider 错→**不重试**·一次即报);flaky 早退照旧 retry 一次(回归)。
- **真机(用户额度正死着=天然夹具·和姊妹 UI 包一起验)**:app 点出方案 → ~1-2 分钟后**人话报错**(非静默);后端审计里留的是分类后错误。
- **regression**:计数不降;§3 逐函数 0-diff 自证;fmt(只本包文件)。

## 5. 不做

- UI 上脸/重试按钮 = 姊妹包 `2026-07-05-jiaoban-fix8-ui-failure-on-face-v1.md`;供给失败的自动降级/换 provider(不做);codex CLI 重连行为(外部,不碰)。

## 6. 回交

- §4 证据 + stderr 捕获点说明(runner 有现成 or 调用层补捕)+ 0-diff 逐函数自证 + 计数 → 主导线核实物。**子线不 commit。**
