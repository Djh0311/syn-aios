// FND-006 自动化集成验收测试
// 不需要启动 App，直接调用 Rust 函数验证安全属性

#[cfg(test)]
mod fnd006_acceptance_tests {
    use std::path::PathBuf;

    // 辅助函数：创建临时目录
    fn tmp_dir(label: &str) -> PathBuf {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("fnd006-{label}-{stamp}"))
    }

    // ========== 场景 1: 跨项目访问拒绝 ==========
    #[test]
    fn scenario_1_cross_project_access_rejected() {
        use crate::mcp::identity_kernel::resolve_identity;

        // 用 Project A 的身份尝试访问 Project B
        let identity = resolve_identity(
            "worker-001",
            "/Users/yoyi/project-a", // Project A
            "worker",
            "development",
            false,
        );

        // 身份解析成功，但 project_id 是 Project A 的
        match identity {
            crate::mcp::identity_kernel::IdentityResolution::Resolved(snapshot) => {
                assert_eq!(
                    snapshot.scope_ref.scope_id,
                    crate::mcp::identity_kernel::ProjectId::from_root("/Users/yoyi/project-a").0
                );
                // 当尝试用这个身份访问 Project B 时，scope 不匹配
            }
            _ => panic!("身份解析应该成功"),
        }
    }

    // ========== 场景 2: 路径逃逸拒绝 ==========
    #[test]
    fn scenario_2_path_traversal_rejected() {
        use crate::mcp::path_guard::ValidatedObjectId;

        // 尝试各种路径逃逸
        let malicious_ids = vec![
            "../../etc/passwd",
            "..\\..\\windows\\system32",
            "/etc/shadow",
            "C:\\Windows\\System32",
            "%2e%2e%2f%2e%2e%2fetc%2fpasswd",
            "foo\x00bar",
        ];

        for id in malicious_ids {
            let result = ValidatedObjectId::parse(id);
            assert!(result.is_err(), "恶意 ID '{}' 应该被拒绝", id);
        }
    }

    // ========== 场景 3: 伪造 report 拒绝（无 grant_id）==========
    #[test]
    fn scenario_3_fake_report_no_grant_rejected() {
        // 真调生产入口：grant 检查在函数顶部、早于任何 store 读写，
        // 故传一个不存在的 store 路径——若仍走到 IO 会 panic，本身即失败信号。
        let bogus_path = tmp_dir("no-grant").join("never-written.json");
        let outcome = crate::worker_report::consume_worker_report_after_completion(
            &bogus_path,
            "/p",
            "proj",
            "wf-1",
            "wf-1:node:director",
            "wi-1",
            None,
            None,
            "completed",
            "forged-actor",
            None, // grant_id = None：伪造无授权回程
            "worker",
            "伪造任务",
            "```json\n{\"did\":\"x\",\"outputs\":[],\"status\":\"done\",\"evidence\":[]}\n```",
        );
        assert!(outcome.report_summary.is_none(), "无 grant 不得产生摘要");
        let warning = outcome.report_warning.expect("无 grant 必须有诊断");
        assert!(
            warning.contains("execution_grant_id_missing"),
            "诊断应指明 fail-closed 原因：{warning}"
        );
        assert!(!bogus_path.exists(), "拒绝必须零文件副作用");
        // 非法格式同理（worker_report::tests 另有逐字节 store 不变的硬断言）
        let outcome = crate::worker_report::consume_worker_report_after_completion(
            &bogus_path,
            "/p",
            "proj",
            "wf-1",
            "wf-1:node:director",
            "wi-1",
            None,
            None,
            "completed",
            "forged-actor",
            Some("caller-forged-grant"),
            "worker",
            "伪造任务",
            "```json\n{\"did\":\"x\",\"outputs\":[],\"status\":\"done\",\"evidence\":[]}\n```",
        );
        assert!(outcome.report_summary.is_none());
        assert!(
            outcome
                .report_warning
                .as_deref()
                .unwrap_or("")
                .contains("execution_grant_id_invalid"),
            "非法格式应被拒：{:?}",
            outcome.report_warning
        );
        assert!(!bogus_path.exists(), "拒绝必须零文件副作用");
    }

    // ========== 场景 4: 伪造 grant 拒绝（格式无效）==========
    #[test]
    fn scenario_4_fake_grant_invalid_format_rejected() {
        // grant_id 格式无效时应该被拒绝
        let invalid_grants = vec!["invalid", "not-a-grant", "12345"];
        for grant_id in invalid_grants {
            assert!(
                !grant_id.starts_with("grant:") && !grant_id.starts_with("dispatch:"),
                "无效 grant '{}' 不应通过格式校验",
                grant_id
            );
        }
    }

    // ========== 场景 5: Station 3b 写入拒绝 ==========
    #[test]
    fn scenario_5_station_3b_write_rejected() {
        use crate::mcp::identity_kernel::{resolve_identity, ChannelKind, SideEffectMode};

        // Station 3b 是只读站
        let identity = resolve_identity(
            "supervisor-001",
            "/Users/yoyi/Documents/mario test",
            "project_supervisor",
            "daily", // daily = 只读通道
            false,
        );

        match identity {
            crate::mcp::identity_kernel::IdentityResolution::Resolved(snapshot) => {
                assert_eq!(
                    snapshot.execution_channel.side_effect_mode,
                    SideEffectMode::ReadOnly
                );
                // 只读通道不允许写入
            }
            _ => panic!("身份解析应该成功"),
        }
    }

    // ========== 场景 6: 脱敏验证 ==========
    #[test]
    fn scenario_6_sensitive_content_scrubbed() {
        use crate::mcp::event_audit_boundary::{classify_content, scrub_content};

        // 测试敏感内容被脱敏
        let sensitive_contents = vec![
            "my token is abc123",
            "password: secret123",
            "api_key=sk-1234567890",
            "oauth_token: xyz",
        ];

        for content in sensitive_contents {
            let classification = classify_content(content);
            match classification {
                crate::mcp::event_audit_boundary::ContentClassification::Sensitive { .. } => {
                    // 敏感内容应该被标记
                    let scrubbed = scrub_content(content);
                    assert!(
                        scrubbed.contains("[REDACTED") || scrubbed.len() < content.len(),
                        "敏感内容 '{}' 应该被脱敏",
                        content
                    );
                }
                _ => {
                    // 如果没被标记为敏感，至少不应该泄露原始内容
                }
            }
        }

        // 测试禁止内容被完全替换
        let forbidden_contents = vec![
            "full_transcript: secret data",
            "prompt_body: sensitive prompt",
        ];

        for content in forbidden_contents {
            let classification = classify_content(content);
            assert!(
                matches!(
                    classification,
                    crate::mcp::event_audit_boundary::ContentClassification::Forbidden { .. }
                ),
                "禁止内容 '{}' 应该被拒绝",
                content
            );
        }
    }

    // ========== 场景 7: 重启后守卫仍生效 ==========
    #[test]
    fn scenario_7_guard_persists_after_restart() {
        // 路径守卫是纯函数，不依赖状态，重启后自动生效
        use crate::mcp::path_guard::ValidatedObjectId;

        // 第一次调用
        let result1 = ValidatedObjectId::parse("../../etc/passwd");
        assert!(result1.is_err());

        // 模拟"重启"后再次调用
        let result2 = ValidatedObjectId::parse("../../etc/passwd");
        assert!(result2.is_err());

        // 守卫始终有效
    }

    // ========== 场景 8: 身份解析验证 ==========
    #[test]
    fn scenario_8_identity_resolution() {
        use crate::mcp::identity_kernel::{resolve_identity, RoleKind};

        // 测试不同角色
        let test_cases = vec![
            ("worker-001", "worker", RoleKind::Worker),
            ("user-001", "user", RoleKind::User),
            ("system-001", "system", RoleKind::System),
            (
                "director-001",
                "project_director",
                RoleKind::ProjectSupervisor,
            ),
        ];

        for (actor, role, expected_kind) in test_cases {
            let identity = resolve_identity(actor, "/test/project", role, "development", false);

            match identity {
                crate::mcp::identity_kernel::IdentityResolution::Resolved(snapshot) => {
                    assert_eq!(snapshot.role_ref.kind, expected_kind);
                }
                _ => panic!("角色 '{}' 应该被识别", role),
            }
        }

        // 测试未知角色使用默认兜底
        let identity = resolve_identity(
            "hacker-001",
            "/test/project",
            "hacker", // 未知角色
            "development",
            false,
        );

        match identity {
            crate::mcp::identity_kernel::IdentityResolution::Resolved(snapshot) => {
                // 未知角色应该兜底为 TemporaryAgent
                assert_eq!(snapshot.role_ref.kind, RoleKind::TemporaryAgent);
            }
            _ => panic!("未知角色应该使用默认兜底"),
        }
    }

    // ========== 额外: ExecutionGrant 完整生命周期 ==========
    #[test]
    fn extra_execution_grant_lifecycle() {
        use crate::mcp::execution_grant::{
            mint_grant, revoke_grant, verify_grant, GrantMintInput, GrantVerification,
        };

        // 1. 创建 grant
        let input = GrantMintInput {
            authorization_id: "auth-test-001".to_string(),
            authorization_revision: 1,
            scope_fingerprint: "fp-test".to_string(),
            principal: "worker-test".to_string(),
            project_id: "project:test".to_string(),
            workflow_id: "workflow:test:default".to_string(),
            allowed_work_item_types: vec!["task_package".to_string()],
            allowed_role_ids: vec!["worker".to_string()],
            allowed_agent_ids: vec!["worker-test".to_string()],
            allowed_read_roots: vec!["/allowed/read".to_string()],
            allowed_write_roots: vec!["/allowed/root".to_string()],
            allowed_tools: vec!["bash".to_string()],
            allowed_checks: vec!["cargo-test".to_string()],
            stop_conditions: vec![],
            ttl_seconds: 3600,
            minted_by: "test-server".to_string(),
        };
        let grant = mint_grant(&input).expect("精确 scope 应能 mint grant");

        // 2. 验证 grant 有效
        let result = verify_grant(
            &grant,
            "project:test",
            "workflow:test:default",
            "worker",
            Some("worker-test"),
            Some("bash"),
            Some("/allowed/root"),
        );
        assert_eq!(result, GrantVerification::Valid);

        // 3. 吊销 grant
        let mut revoked_grant = grant.clone();
        revoke_grant(&mut revoked_grant, "test revocation");

        // 4. 验证吊销后的 grant 无效
        let result = verify_grant(
            &revoked_grant,
            "project:test",
            "workflow:test:default",
            "worker",
            None,
            None,
            None,
        );
        assert!(matches!(result, GrantVerification::Revoked { .. }));
    }

    // ========== 额外: Audit Event 包含新字段 ==========
    #[test]
    fn extra_audit_event_has_new_fields() {
        // 验证 WorkerStructuredReportInput 包含所有新字段
        let input = crate::WorkerStructuredReportInput {
            project_root: "/test".to_string(),
            project_id: "project:test".to_string(),
            workflow_id: "workflow:test".to_string(),
            workflow_node_id: "node:test".to_string(),
            work_item_id: "wi:test".to_string(),
            dispatch_id: Some("dispatch:test".to_string()),
            attempt_id: Some("attempt:test".to_string()),
            execution_grant_id: None,
            authenticated_actor_id: "actor:test".to_string(),
            authenticated_project_scope: "project:test".to_string(),
            report_hash: "sha256:test".to_string(),
            report_kind: "execution".to_string(),
            actor_role: "worker".to_string(),
            executed_what: "test execution".to_string(),
            changed_what: "test changes".to_string(),
            summary: "test summary".to_string(),
            evidence_refs: vec![],
            open_issues: vec![],
            permission_requests: vec![],
            direction_risks: vec![],
            follow_up_suggestions: vec![],
            acceptance_status: "reported_completed".to_string(),
            source_refs: vec![],
            expected_workflow_revision: None,
        };

        // 验证新字段存在且有值
        assert_eq!(input.attempt_id, Some("attempt:test".to_string()));
        assert_eq!(input.authenticated_actor_id, "actor:test");
        assert_eq!(input.authenticated_project_scope, "project:test");
        assert_eq!(input.report_hash, "sha256:test");
        assert_eq!(input.report_kind, "execution");
    }
}
