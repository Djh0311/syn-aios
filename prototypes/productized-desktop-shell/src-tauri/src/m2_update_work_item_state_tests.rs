// M2 接线测试：update_work_item_state 真实命令路径
// 证据等级：TEMP-INTEGRATION（需要真实 SQLite 连接）

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;
    use std::fs;
    use std::path::Path;
    use std::path::PathBuf;

    fn temp_dir(name: &str) -> PathBuf {
        super::temp_dir(&format!("linked-{name}"))
    }

    fn create_test_db(path: &Path) -> Connection {
        super::create_test_db(path)
    }

    /// 插入测试用 work_item 到 work_items 表
    fn insert_test_work_item(connection: &Connection, work_item_id: &str, workflow_id: &str, state: &str) {
        super::insert_test_work_item(connection, work_item_id, workflow_id, state);
    }

    #[test]
    fn test_update_work_item_state_allowed() {
        let dir = temp_dir("allowed");
        fs::create_dir_all(&dir).expect("create temp dir");
        let db_path = dir.join("test.sqlite");

        let connection = create_test_db(&db_path);
        // 插入 work_item，当前状态 draft → 允许转换到 ready_to_dispatch
        insert_test_work_item(&connection, "work-item-001", "workflow-001", "draft");

        let command = UpdateWorkItemStateCommand {
            command_id: "cmd-001".to_string(),
            idempotency_key: "idem-001".to_string(),
            actor_id: "user-001".to_string(),
            scope_ref: "scope-001".to_string(),
            project_id: "project-001".to_string(),
            workflow_id: "workflow-001".to_string(),
            work_item_id: "work-item-001".to_string(),
            expected_revision: None,
            new_status: Some(WorkItemStatus::ReadyToDispatch),
            new_state_json: None,
        };

        let result = update_work_item_state_m2(&connection, command);
        assert!(result.is_ok());

        let result = result.unwrap();
        assert_eq!(result.receipt.status, CommandReceiptStatus::Committed);
        assert_eq!(result.event.event_type, "WorkItemStateUpdated");
        assert_eq!(result.audit.action, AuditAction::Committed);
        assert!(result.snapshot.is_some());

        // 验证数据库中有记录
        let count: i64 = connection.query_row(
            "SELECT COUNT(*) FROM command_receipts",
            [],
            |row| row.get(0),
        ).unwrap();
        assert_eq!(count, 1);

        let count: i64 = connection.query_row(
            "SELECT COUNT(*) FROM events",
            [],
            |row| row.get(0),
        ).unwrap();
        assert_eq!(count, 1);

        let count: i64 = connection.query_row(
            "SELECT COUNT(*) FROM audit_records",
            [],
            |row| row.get(0),
        ).unwrap();
        assert_eq!(count, 1);

        let count: i64 = connection.query_row(
            "SELECT COUNT(*) FROM current_snapshots",
            [],
            |row| row.get(0),
        ).unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_update_work_item_state_denied() {
        let dir = temp_dir("denied");
        fs::create_dir_all(&dir).expect("create temp dir");
        let db_path = dir.join("test.sqlite");

        let connection = create_test_db(&db_path);
        // 插入 work_item，当前状态 draft → 非法转换到 failed（draft 只能到 ready_to_dispatch）
        insert_test_work_item(&connection, "work-item-002", "workflow-002", "draft");

        let command = UpdateWorkItemStateCommand {
            command_id: "cmd-002".to_string(),
            idempotency_key: "idem-002".to_string(),
            actor_id: "user-002".to_string(),
            scope_ref: "scope-002".to_string(),
            project_id: "project-002".to_string(),
            workflow_id: "workflow-002".to_string(),
            work_item_id: "work-item-002".to_string(),
            expected_revision: None,
            new_status: Some(WorkItemStatus::Failed),
            new_state_json: None,
        };

        let result = update_work_item_state_m2(&connection, command);
        assert!(result.is_ok(), "denied path should return Ok with denial receipt: {:?}", result.err());

        let result = result.unwrap();
        // 验证 receipt 是 denial 状态
        assert_eq!(result.receipt.status, CommandReceiptStatus::Denied);
        // 验证 denial audit record 落盘
        let audit_count: i64 = connection.query_row(
            "SELECT COUNT(*) FROM audit_records",
            [],
            |row| row.get(0),
        ).unwrap();
        assert_eq!(audit_count, 1);
        // 验证零业务变化：events 表无新行
        let event_count: i64 = connection.query_row(
            "SELECT COUNT(*) FROM events",
            [],
            |row| row.get(0),
        ).unwrap();
        assert_eq!(event_count, 0);
    }

    #[test]
    fn test_idempotency_same_key_same_hash_returns_same_receipt() {
        let dir = temp_dir("idem-same");
        fs::create_dir_all(&dir).expect("create temp dir");
        let db_path = dir.join("test.sqlite");

        let connection = create_test_db(&db_path);
        insert_test_work_item(&connection, "work-item-003", "workflow-003", "draft");

        let command = UpdateWorkItemStateCommand {
            command_id: "cmd-003".to_string(),
            idempotency_key: "idem-003".to_string(),
            actor_id: "user-003".to_string(),
            scope_ref: "scope-003".to_string(),
            project_id: "project-003".to_string(),
            workflow_id: "workflow-003".to_string(),
            work_item_id: "work-item-003".to_string(),
            expected_revision: None,
            new_status: Some(WorkItemStatus::ReadyToDispatch),
            new_state_json: None,
        };

        // 第一次执行
        let result1 = update_work_item_state_m2(&connection, command.clone()).unwrap();
        assert_eq!(result1.receipt.status, CommandReceiptStatus::Committed);

        // 验证数据库中有 1 条 receipt
        let count: i64 = connection.query_row(
            "SELECT COUNT(*) FROM command_receipts",
            [],
            |row| row.get(0),
        ).unwrap();
        assert_eq!(count, 1);

        // 第二次执行（相同 command_id + idempotency_key + 相同 request_hash）
        let result2 = update_work_item_state_m2(&connection, command).unwrap();
        // 返回既有 receipt（receipt_id 相同）
        assert_eq!(result2.receipt.receipt_id, result1.receipt.receipt_id);

        // 验证数据库中仍然只有 1 条 receipt（幂等，不新增行）
        let count: i64 = connection.query_row(
            "SELECT COUNT(*) FROM command_receipts",
            [],
            |row| row.get(0),
        ).unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_idempotency_same_key_different_hash_returns_conflict() {
        let dir = temp_dir("idem-conflict");
        fs::create_dir_all(&dir).expect("create temp dir");
        let db_path = dir.join("test.sqlite");

        let connection = create_test_db(&db_path);
        insert_test_work_item(&connection, "work-item-004", "workflow-004", "draft");

        // 第一次执行
        let command1 = UpdateWorkItemStateCommand {
            command_id: "cmd-004".to_string(),
            idempotency_key: "idem-004".to_string(),
            actor_id: "user-004".to_string(),
            scope_ref: "scope-004".to_string(),
            project_id: "project-004".to_string(),
            workflow_id: "workflow-004".to_string(),
            work_item_id: "work-item-004".to_string(),
            expected_revision: None,
            new_status: Some(WorkItemStatus::ReadyToDispatch),
            new_state_json: None,
        };
        let result1 = update_work_item_state_m2(&connection, command1).unwrap();
        assert_eq!(result1.receipt.status, CommandReceiptStatus::Committed);

        // 第二次执行（相同 command_id + idempotency_key + 不同 request_hash → 不同 new_status）
        let command2 = UpdateWorkItemStateCommand {
            command_id: "cmd-004".to_string(),
            idempotency_key: "idem-004".to_string(),
            actor_id: "user-004".to_string(),
            scope_ref: "scope-004".to_string(),
            project_id: "project-004".to_string(),
            workflow_id: "workflow-004".to_string(),
            work_item_id: "work-item-004".to_string(),
            expected_revision: None,
            new_status: Some(WorkItemStatus::Running),  // 不同的 new_status → 不同的 request_hash
            new_state_json: None,
        };
        let result2 = update_work_item_state_m2(&connection, command2);
        assert!(result2.is_err(), "different hash should return conflict error");
        assert!(result2.unwrap_err().contains("idempotent_conflict"));
    }
}
