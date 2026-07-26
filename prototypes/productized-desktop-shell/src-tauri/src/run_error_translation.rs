// A·运行错误人话翻译层（C6 观测补强）。
//
// 任务包：tasks/2026-07-09-run-error-plain-language-surface-A-v1.md
//
// 定位：把 codex/worker 运行的**原始错误** → 翻译成人话 → 结构化 `{family, human, raw_snippet}`。
// 单一真源：fix8 的供给类判据从 `codex_local_runner::classify_codex_provider_failure` 搬到这里
// （族①），runner 那个函数改成薄委托——收编成一套、不留散的两套翻译器。
//
// 安全属性（延续 fix8「只影响报告不改成败」）：
// - **纯函数·零副作用·不改任何成败判定**——只把已发生的错误串翻成人话，供呈现层用；
// - **保守归一化**：拿不准 → `unknown` + 原文，绝不硬编假人话骗人（unknown 必带 raw_snippet）；
// - **retry 承重信号不在这层动**：`codex_provider_unavailable:` / `consult_last_message_read_failed:`
//   两个前缀是 director retry 的承重标记（`director_agent.rs:1384/1396/1399`），本模块只**读**不改；
//   runner 侧路由时保留前缀（见 codex_local_runner.rs 委托点）。
// - **大小写不敏感** + **下划线归一**（dispatch.failure_reason 把 stderr 压成下划线形，一并识别）。

/// 错误族（稳定机器键·前端自映射人话/配色）。
///
/// 判据顺序 = 具体优先（version_outdated→provider→readback→timeout→codex_subsystem→sandbox→network→command→unknown），
/// 防「exit≠0 的命令失败」把更具体的子系统/沙箱错误盖掉。
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub(crate) struct RunErrorHuman {
    /// "codex_version_outdated" | "provider_unavailable" | "network" | "timeout" | "sandbox_denied"
    /// | "command_failed" | "codex_subsystem" | "readback_failed" | "unknown"
    pub(crate) family: String,
    /// 人话摘要（默认脸显这个）。
    pub(crate) human: String,
    /// 原文片段（截断安全·下钻看这个）——**永远保留原文**，不做「人话/原文二选一」。
    pub(crate) raw_snippet: String,
}

const RAW_SNIPPET_MAX: usize = 400;

fn snippet(raw: &str) -> String {
    let trimmed = raw.trim();
    trimmed.chars().take(RAW_SNIPPET_MAX).collect()
}

/// fix8·供给类失败判据（**单一真源**·从 codex_local_runner 搬来·字面判据一字未改）。
/// 命中→人话（同 fix8 原文·保 director retry 与既有测试不回归）；否则 None。
/// 保守表（大小写不敏感）：subscription_not_found / usage limit / quota / unauthorized / 403 / 401 / reconnecting+5/5。
pub(crate) fn classify_provider_failure_human(stderr_tail: &str) -> Option<String> {
    let text = stderr_tail.trim();
    if text.is_empty() {
        return None;
    }
    let lower = text.to_lowercase();
    let hit = lower.contains("subscription_not_found")
        || lower.contains("usage limit")
        || lower.contains("quota")
        || lower.contains("unauthorized")
        || lower.contains("403")
        || lower.contains("401")
        || (lower.contains("reconnecting") && lower.contains("5/5"));
    if !hit {
        return None;
    }
    let snip: String = text.chars().take(200).collect();
    Some(format!(
        "codex 供给不可用（403 订阅/额度/登录类，非网络抽风）：{snip}——请检查订阅/额度/登录，别空重试。"
    ))
}

/// 错误族全谱分类器。输入原始错误串（可含 dispatch.failure_reason 的下划线压缩形）→ 结构化人话。
/// unknown 兜底：未命中任何族 → `family=unknown` + 保守人话 + 原文（不装假人话）。
pub(crate) fn classify_run_error(raw: &str) -> RunErrorHuman {
    let raw_snippet = snippet(raw);
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return RunErrorHuman {
            family: "unknown".to_string(),
            human: "未识别错误（没有可读的错误信息）".to_string(),
            raw_snippet,
        };
    }
    // 已翻译前缀（`codex_provider_unavailable:<人话>`）= 上游 fix8 已归类的供给类错误：原样取内嵌人话
    // （humanize_consult_error×2 的老语义·别用本模块通用人话盖掉上游已给的具体供给人话）。
    if let Some(rest) = trimmed.strip_prefix("codex_provider_unavailable:") {
        return RunErrorHuman {
            family: "provider_unavailable".to_string(),
            human: rest.trim().to_string(),
            raw_snippet,
        };
    }
    // 归一：小写 + 下划线→空格（dispatch.failure_reason 用 `compact_failure_warning` 把 stderr 压成下划线形，
    // 一并识别，别让「no_such_table」漏判成 unknown）。
    let lower = trimmed.to_lowercase();
    let spaced = lower.replace('_', " ");
    let has = |needle: &str| lower.contains(needle) || spaced.contains(needle);

    // ⑧ codex CLI 版本过旧（服务端明确拒答，升级后再试；普通 upgrade 文本不归此族）。
    let has_version_upgrade_signal = has("requires a newer version") || has("please upgrade");
    let is_codex_server_rejection = has("requires a newer version of codex")
        || (has("model") && has("codex"))
        || ((has("400") || has("bad request")) && has("codex"));
    if has_version_upgrade_signal && is_codex_server_rejection {
        return RunErrorHuman {
            family: "codex_version_outdated".to_string(),
            human: "codex 版本太旧，升级 CLI 后重试（npm install -g @openai/codex@latest）"
                .to_string(),
            raw_snippet,
        };
    }
    // ① 供给类（最具体·复用单一真源判据）。
    if classify_provider_failure_human(trimmed).is_some() {
        return RunErrorHuman {
            family: "provider_unavailable".to_string(),
            human:
                "codex 供给不可用（订阅/额度/登录类，非网络抽风）——请检查订阅/额度/登录，别空重试。"
                    .to_string(),
            raw_snippet,
        };
    }
    // ⑦ 口供读取失败（specific 信号·director 据此 retry 一次）。
    if has("consult last message read failed") || has("readback failed") || has("readback_failed") {
        return RunErrorHuman {
            family: "readback_failed".to_string(),
            human: "worker 干完了但口供没读回来（偶发·系统会自动重试一次）。".to_string(),
            raw_snippet,
        };
    }
    // ③ 超时。
    if has("timed out") || has("timeout") {
        return RunErrorHuman {
            family: "timeout".to_string(),
            human: "这一步超时被中止了（一般是活太大）；系统可能已自动打回主管拆细重来。"
                .to_string(),
            raw_snippet,
        };
    }
    // ⑥ codex 内部子系统（memories/索引/缺表等·比沙箱/命令更具体的自身报错）。
    if has("memories") || has("no such table") || has("memory subsystem") || has("codex_memories") {
        return RunErrorHuman {
            family: "codex_subsystem".to_string(),
            human: "codex 自身某个子系统报错（如记忆/索引），一般不影响本次任务结果。".to_string(),
            raw_snippet,
        };
    }
    // ④ 沙箱/权限拒绝（判据一致于 `phase_b_mentions_codex_state_error`：state-db 只读/权限）。
    if has("permission denied")
        || has("read-only")
        || has("readonly")
        || has("attempt to write a readonly database")
        || has("sandbox")
        || has("state db")
    {
        return RunErrorHuman {
            family: "sandbox_denied".to_string(),
            human: "写入被沙箱/权限挡下（目标不在允许写入范围，或本地库只读）。".to_string(),
            raw_snippet,
        };
    }
    // ② 断供/网络（reconnecting 未到 5/5·或断流）。
    if has("reconnecting") || has("stream disconnected") || has("connection reset") {
        return RunErrorHuman {
            family: "network".to_string(),
            human: "网络/连接抽风（断流或反复重连），可重试。".to_string(),
            raw_snippet,
        };
    }
    // ⑤ 命令失败（exit≠0·最泛·放最后免盖住更具体的族）。
    if has("exit nonzero")
        || has("exit code")
        || has("resume exit")
        || has("nonzero")
        || has("exited with")
    {
        return RunErrorHuman {
            family: "command_failed".to_string(),
            human: "命令没跑成（退出码非 0），看原文定位。".to_string(),
            raw_snippet,
        };
    }
    // unknown 兜底（保守·带原文·不装假人话）。
    RunErrorHuman {
        family: "unknown".to_string(),
        human: "未识别错误（附原文供排查）。".to_string(),
        raw_snippet,
    }
}

/// 呈现用人话（收编 `humanize_consult_error`×2 的单一真源）：命中任一族 → 取翻译人话；
/// unknown → **回退原文**（保 humanize 老语义「拿不准就把原始错误原样给」·不硬塞「未识别」把原文盖掉）。
/// secretary/global_supervisor 的 `unavailable_reason` 用它——单一真源、不动 director retry。
pub(crate) fn humanize_error_for_display(raw: &str) -> String {
    let translated = classify_run_error(raw);
    if translated.family == "unknown" {
        raw.trim().to_string()
    } else {
        translated.human
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // 收编 humanize_consult_error×2：已翻译前缀取内嵌人话（不被通用人话盖）；unknown 回退原文。
    #[test]
    fn humanize_for_display_matches_old_humanize_semantics() {
        // 供给类前缀 → 剥前缀取内嵌人话（同 secretary/global 老测试）。
        let out =
            humanize_error_for_display("codex_provider_unavailable:codex 额度用完了，明天再试");
        assert!(out.contains("额度用完"), "取内嵌人话：{out}");
        assert!(!out.contains("codex_provider_unavailable:"), "前缀剥掉");
        assert_eq!(
            classify_run_error("codex_provider_unavailable:codex 额度用完了").family,
            "provider_unavailable"
        );
        // 非供给非任何族的陌生错误 → 回退原文（不硬塞「未识别」·保 humanize 老语义）。
        let raw = "some totally novel error blorp";
        assert_eq!(humanize_error_for_display(raw), raw, "unknown 回退原文");
        // 可识别的非供给错误 → 翻人话（改进·比老 humanize 原样透传更好）。
        assert!(
            humanize_error_for_display("consult_last_message_read_failed: gone").contains("口供")
        );
    }

    // 供给类判据搬家后**不回归**（与 fix8_classify_codex_provider_failure_hits_and_misses 同样本）。
    #[test]
    fn provider_failure_human_matches_fix8_samples() {
        let real = "Reconnecting... 5/5 ERROR: unexpected status 403 Forbidden: {\"code\":\"SUBSCRIPTION_NOT_FOUND\"}";
        assert!(classify_provider_failure_human(real).is_some());
        for sample in [
            "unexpected status 401 UNAUTHORIZED",
            "you have hit your USAGE LIMIT for this month",
            "quota exceeded",
            "subscription_not_found",
            "Reconnecting... 5/5 then gave up",
        ] {
            assert!(
                classify_provider_failure_human(sample).is_some(),
                "应命中：{sample}"
            );
        }
        assert!(classify_provider_failure_human("").is_none());
        assert!(
            classify_provider_failure_human("thread 'main' panicked at foo.rs line 12").is_none()
        );
        assert!(classify_provider_failure_human("Reconnecting... 1/5").is_none());
    }

    // C6：服务端 400 明示 CLI 版本过旧 → 独立错误族、给升级动作、原文照留。
    #[test]
    fn codex_version_outdated_real_400_is_translated() {
        let raw = "400 The 'gpt-5.6-sol' model requires a newer version of Codex. Please upgrade";
        let out = classify_run_error(raw);
        assert_eq!(out.family, "codex_version_outdated");
        assert_eq!(
            out.human,
            "codex 版本太旧，升级 CLI 后重试（npm install -g @openai/codex@latest）"
        );
        assert_eq!(out.raw_snippet, raw, "原文照留供下钻");
    }

    // C6：没有 codex 拒答上下文的普通升级提示，保守回 unknown，不能误报 CLI 版本过旧。
    #[test]
    fn ordinary_upgrade_text_is_not_codex_version_outdated() {
        for raw in [
            "Please upgrade your Rust toolchain before building",
            "This plugin requires a newer version; see its README",
            "The Codex docs say: please upgrade your Rust toolchain",
        ] {
            assert_eq!(classify_run_error(raw).family, "unknown", "raw={raw}");
        }
    }

    // 七族各一命中样本 + 大小写不敏感 + 结构化三段齐 + 原文必带。
    #[test]
    fn seven_families_each_hit_and_carry_raw() {
        let cases: [(&str, &str); 7] = [
            (
                "unexpected status 403 SUBSCRIPTION_NOT_FOUND",
                "provider_unavailable",
            ),
            ("Reconnecting... 2/5 stream disconnected", "network"),
            ("worker dispatch TIMED OUT after 600s", "timeout"),
            (
                "attempt to write a readonly database (state db)",
                "sandbox_denied",
            ),
            (
                "codex_exec_resume_exit_code_1 exit nonzero",
                "command_failed",
            ),
            (
                "codex_memories_write::phase2::job: failed to claim job (no such table: jobs)",
                "codex_subsystem",
            ),
            (
                "consult_last_message_read_failed: file gone",
                "readback_failed",
            ),
        ];
        for (raw, family) in cases {
            let out = classify_run_error(raw);
            assert_eq!(out.family, family, "raw={raw}");
            assert!(!out.human.trim().is_empty(), "human 非空：{raw}");
            assert!(
                !out.raw_snippet.trim().is_empty(),
                "raw_snippet 必带原文：{raw}"
            );
            assert!(
                raw.to_lowercase().contains(
                    &out.raw_snippet
                        .to_lowercase()
                        .chars()
                        .take(10)
                        .collect::<String>()
                ),
                "raw_snippet 是原文片段"
            );
        }
        // 大小写不敏感：全大写超时也命中。
        assert_eq!(classify_run_error("FATAL: TIMEOUT").family, "timeout");
    }

    // 具体优先：exit≠0 + no-such-table 同现 → 归 codex_subsystem（不被 command_failed 盖）。
    #[test]
    fn subsystem_beats_generic_command_failed() {
        let raw = "exit code 1: codex_memories_write no such table: jobs";
        assert_eq!(classify_run_error(raw).family, "codex_subsystem");
    }

    // 下划线压缩形（dispatch.failure_reason）也识别：no_such_table → 子系统。
    #[test]
    fn underscore_compacted_reason_recognized() {
        let raw = "codex_resume_exit_nonzero, no_such_table:_jobs";
        assert_eq!(classify_run_error(raw).family, "codex_subsystem");
    }

    // unknown 兜底：陌生错误 → family=unknown + 保守人话 + 原文（不装假人话）。
    #[test]
    fn unknown_falls_back_conservatively_with_raw() {
        let raw = "totally novel gibberish blorp 42";
        let out = classify_run_error(raw);
        assert_eq!(out.family, "unknown");
        assert!(
            out.human.contains("未识别"),
            "保守人话·不装懂：{}",
            out.human
        );
        assert!(out.raw_snippet.contains("blorp"), "原文必带");
        // 空输入也不炸、也 unknown。
        assert_eq!(classify_run_error("   ").family, "unknown");
    }

    // 族④判据与 `phase_b_mentions_codex_state_error` 对同一 state-db 只读样本判定一致（不造矛盾探测）。
    // （此处证 classify_run_error 对该样本归 sandbox_denied；探测器一致性在 codex_local_runner 侧测。）
    #[test]
    fn state_db_readonly_is_sandbox_family() {
        for raw in [
            "attempt to write a readonly database",
            "state db permission denied",
            "state_5.sqlite read-only",
        ] {
            assert_eq!(
                classify_run_error(raw).family,
                "sandbox_denied",
                "raw={raw}"
            );
        }
    }
}
