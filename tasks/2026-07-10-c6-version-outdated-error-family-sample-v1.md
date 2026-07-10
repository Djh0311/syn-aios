# 微件:C6 错误族补「版本过旧」样本 · 主导线 → 执行线 v1(可派·可搭任一包顺手)

日期:2026-07-10 · 轻档微件。来源:真机验收实撞——codex CLI 0.134 被服务端 400 拒(`The 'gpt-5.6-sol' model requires a newer version of Codex. Please upgrade`),现分类器把它归供给类翻成「codex 服务不可用」,**不算错但不够准**(该叫「codex 版本太旧,要升级」并给动作)。

- **做**:`run_error_translation.rs` 加「版本过旧」判据(保守:命中 `requires a newer version` / `please upgrade`(不区分大小写)且语境为 codex 拒答)→ 独立族或供给类子话术,人话=「codex 版本太旧,升级 CLI 后重试(npm install -g @openai/codex@latest)」;原文进 raw_snippet 照旧;
- **样本测试**:用上面 400 原文串做命中单测 + 不误伤(普通含 upgrade 字样的输出不归此族·拿不准归 unknown 的保守纪律照守);
- **死线**:C6 原包 §3 全套照守(runner 冻结核 0-diff·不改成败判定·director retry 供给类信号不断——若把它从供给类拆出独立族,**必须**验 `codex_provider_unavailable:` 前缀对原供给类照旧产出、retry 测试逐条绿);
- **验收**:单测新增 ≥2(命中+不误伤)·764+ 计数不降·权威 fmt 净。**子线不 commit。**
