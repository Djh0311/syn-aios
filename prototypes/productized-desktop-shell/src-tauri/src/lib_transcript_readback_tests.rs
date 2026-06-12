    #[test]
    fn transcript_reader_rejects_thread_outside_index() {
        let index = json!({
          "threads": [
            {
              "thread_id": "indexed-thread",
              "title": "Indexed",
              "rollout_path": "/tmp/indexed.jsonl",
              "rollout_exists": true
            }
          ]
        });

        assert!(find_index_thread(&index, "indexed-thread").is_some());
        assert!(find_index_thread(&index, "missing-thread").is_none());
    }

    #[test]
    fn parses_transcript_reader_output() {
        let transcript = json!({
          "thread_id": "indexed-thread",
          "rollout_path": "/tmp/indexed.jsonl",
          "project_path": "/tmp/project",
          "title": "Indexed",
          "created_at_ms": 1,
          "updated_at_ms": 2,
          "events": [
            {
              "event_id": "line-000001",
              "event_type": "user_message",
              "actor": "user",
              "role": "user",
              "text": "hello",
              "warnings": []
            },
            {
              "event_id": "line-000002",
              "event_type": "command_output",
              "actor": "tool",
              "stdout": "ok",
              "stderr": "",
              "exit_code": 0,
              "warnings": ["sample_warning"]
            }
          ],
          "summary": {
            "total_events": 2,
            "event_type_counts": {
              "user_message": 1,
              "command_output": 1
            },
            "unknown_event_count": 0,
            "warning_count": 1,
            "encrypted_content_event_count": 0,
            "sensitive_like_event_count": 0
          },
          "warnings": [],
          "source_stats": {
            "jsonl": {
              "line_count": 2,
              "parsed_line_count": 2,
              "bad_json_line_count": 0
            }
          }
        });

        let parsed = parse_codex_transcript(&transcript).expect("transcript should parse");

        assert_eq!(parsed.thread_id, "indexed-thread");
        assert_eq!(parsed.summary.total_events, 2);
        assert_eq!(
            parsed.summary.event_type_counts.get("command_output"),
            Some(&1)
        );
        assert_eq!(parsed.events[0].text.as_deref(), Some("hello"));
        assert_eq!(parsed.events[1].stdout.as_deref(), Some("ok"));
        assert_eq!(parsed.events[1].warnings, vec!["sample_warning"]);
    }

    #[test]
    fn transcript_catalog_reads_sqlite_thread_not_in_index() {
        let fixture = transcript_catalog_fixture("transcript-catalog-sqlite-only", "sqlite-thread");
        let index = transcript_index(&fixture.codex_home, Vec::new());

        let transcript =
            load_codex_session_transcript_with_catalog(&index, "sqlite-thread", &fixture.db_path)
                .expect("sqlite-only transcript should read");

        assert_eq!(transcript.thread_id, "sqlite-thread");
        assert_eq!(transcript.title.as_deref(), Some("Sqlite thread"));
        assert_eq!(
            transcript.events[0].text.as_deref(),
            Some("hello from sqlite")
        );
        assert_eq!(transcript.source_stats["catalog_source"], json!("sqlite"));
    }

    #[test]
    fn transcript_catalog_sqlite_overrides_stale_index_rollout_status() {
        let fixture =
            transcript_catalog_fixture("transcript-catalog-sqlite-overrides", "same-thread");
        let stale_index = transcript_index(
            &fixture.codex_home,
            vec![json!({
                "thread_id": "same-thread",
                "title": "Stale index",
                "project_root": "/tmp/stale",
                "rollout_path": fixture.codex_home.join("sessions").join("missing.jsonl").display().to_string(),
                "rollout_exists": false
            })],
        );

        let transcript = load_codex_session_transcript_with_catalog(
            &stale_index,
            "same-thread",
            &fixture.db_path,
        )
        .expect("sqlite authority should override stale index");

        assert_eq!(transcript.title.as_deref(), Some("Sqlite thread"));
        assert_eq!(
            transcript.events[0].text.as_deref(),
            Some("hello from sqlite")
        );
        assert_eq!(transcript.source_stats["catalog_source"], json!("sqlite"));
    }

    #[test]
    fn transcript_catalog_rejects_sqlite_rollout_outside_allowed_dirs() {
        let dir = test_temp_dir("transcript-catalog-outside");
        fs::create_dir_all(&dir).expect("create temp dir");
        let codex_home = dir.join("fake-codex-home");
        fs::create_dir_all(codex_home.join("sessions")).expect("create sessions");
        fs::create_dir_all(codex_home.join("archived_sessions")).expect("create archived");
        let outside = dir.join("outside.jsonl");
        write_test_rollout(&outside, "outside");
        let db_path = codex_home.join("state_5.sqlite");
        create_test_threads_db(&db_path, "outside-thread", &outside);
        let index = transcript_index(&codex_home, Vec::new());

        let error = load_codex_session_transcript_with_catalog(&index, "outside-thread", &db_path)
            .expect_err("outside rollout should be rejected");

        assert!(error.starts_with("rollout_outside_allowed_dirs:"));
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn transcript_catalog_classifies_missing_sqlite_rollout() {
        let dir = test_temp_dir("transcript-catalog-missing");
        fs::create_dir_all(&dir).expect("create temp dir");
        let codex_home = dir.join("fake-codex-home");
        let sessions_dir = codex_home.join("sessions");
        fs::create_dir_all(&sessions_dir).expect("create sessions");
        fs::create_dir_all(codex_home.join("archived_sessions")).expect("create archived");
        let missing = sessions_dir.join("missing.jsonl");
        let db_path = codex_home.join("state_5.sqlite");
        create_test_threads_db(&db_path, "missing-thread", &missing);
        let index = transcript_index(&codex_home, Vec::new());

        let error = load_codex_session_transcript_with_catalog(&index, "missing-thread", &db_path)
            .expect_err("missing rollout should be classified");

        assert!(error.starts_with("rollout_missing:"));
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn transcript_catalog_falls_back_to_index_when_sqlite_unavailable() {
        let dir = test_temp_dir("transcript-catalog-index-fallback");
        let codex_home = dir.join("fake-codex-home");
        let sessions_dir = codex_home.join("sessions");
        fs::create_dir_all(&sessions_dir).expect("create sessions");
        fs::create_dir_all(codex_home.join("archived_sessions")).expect("create archived");
        let rollout = sessions_dir.join("index-thread.jsonl");
        write_test_rollout(&rollout, "hello from index");
        let index = transcript_index(
            &codex_home,
            vec![json!({
                "thread_id": "index-thread",
                "title": "Index thread",
                "project_root": "/tmp/index-project",
                "rollout_path": rollout.display().to_string(),
                "rollout_exists": true,
                "updated_at_ms": 55
            })],
        );

        let transcript = load_codex_session_transcript_with_catalog(
            &index,
            "index-thread",
            &codex_home.join("missing-state.sqlite"),
        )
        .expect("index fallback should read when sqlite unavailable");

        assert_eq!(
            transcript.events[0].text.as_deref(),
            Some("hello from index")
        );
        assert_eq!(
            transcript.source_stats["catalog_source"],
            json!("index_fallback_sqlite_unavailable")
        );
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn transcript_catalog_main_path_does_not_need_python_reader() {
        let fixture = transcript_catalog_fixture("transcript-catalog-no-python", "sqlite-thread");
        let index = transcript_index(&fixture.codex_home, Vec::new());

        let transcript =
            load_codex_session_transcript_with_catalog(&index, "sqlite-thread", &fixture.db_path)
                .expect("native reader should not need transcript_reader.py");

        assert_eq!(transcript.source_stats["catalog_source"], json!("sqlite"));
        assert_eq!(transcript.summary.total_events, 1);
    }

    #[test]
    fn transcript_catalog_reads_sqlite_thread_without_index_catalog() {
        let fixture = transcript_catalog_fixture("transcript-catalog-no-index", "sqlite-thread");

        let transcript = load_codex_session_transcript_with_optional_catalog(
            None,
            "sqlite-thread",
            &fixture.db_path,
            Some("索引 JSON 解析失败".to_string()),
        )
        .expect("sqlite transcript should read even when index is unavailable");

        assert_eq!(transcript.source_stats["catalog_source"], json!("sqlite"));
        assert_eq!(
            transcript.events[0].text.as_deref(),
            Some("hello from sqlite")
        );
    }

    #[test]
    fn dispatch_readback_stats_reads_sqlite_only_native_rollout() {
        let fixture = transcript_catalog_fixture("dispatch-readback-sqlite-only", "sqlite-thread");
        let index = transcript_index(&fixture.codex_home, Vec::new());

        let stats = dispatch_readback_stats_native(
            Some(&index),
            &fixture.db_path,
            "sqlite-thread",
            "hello from sqlite",
        )
        .expect("sqlite-only native readback should read");

        assert_eq!(stats.transcript_event_count, 1);
        assert_eq!(stats.transcript_target_hits, 1);
        assert!(!fixture
            .codex_home
            .parent()
            .expect("fixture codex home should have parent")
            .join("transcript_reader.py")
            .exists());
    }

    #[test]
    fn dispatch_readback_stats_reads_sqlite_when_index_unavailable() {
        let fixture = transcript_catalog_fixture("dispatch-readback-no-index", "sqlite-thread");

        let stats = dispatch_readback_stats_native(
            None,
            &fixture.db_path,
            "sqlite-thread",
            "hello from sqlite",
        )
        .expect("sqlite readback should not need index");

        assert_eq!(stats.transcript_event_count, 1);
        assert_eq!(stats.transcript_target_hits, 1);
    }

    #[test]
    fn dispatch_readback_stats_falls_back_to_index_when_sqlite_unavailable() {
        let dir = test_temp_dir("dispatch-readback-index-fallback");
        let codex_home = dir.join("fake-codex-home");
        let sessions_dir = codex_home.join("sessions");
        fs::create_dir_all(&sessions_dir).expect("create sessions");
        fs::create_dir_all(codex_home.join("archived_sessions")).expect("create archived");
        let rollout = sessions_dir.join("index-thread.jsonl");
        write_test_rollout(&rollout, safe_probe_target());
        let index = transcript_index(
            &codex_home,
            vec![json!({
                "thread_id": "index-thread",
                "title": "Index thread",
                "project_root": "/tmp/index-project",
                "rollout_path": rollout.display().to_string(),
                "rollout_exists": true,
                "updated_at_ms": 55
            })],
        );

        let stats = dispatch_readback_stats_native(
            Some(&index),
            &codex_home.join("missing-state.sqlite"),
            "index-thread",
            safe_probe_target(),
        )
        .expect("index fallback should read when sqlite unavailable");

        assert_eq!(stats.transcript_event_count, 1);
        assert_eq!(stats.transcript_target_hits, 1);
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn dispatch_readback_stats_hits_safe_probe_target_in_text_and_stdout() {
        let fixture = dispatch_readback_fixture(
            "dispatch-readback-target-hit",
            "thread-hit",
            vec![
                dispatch_text_event("noise"),
                dispatch_text_event(safe_probe_target()),
                dispatch_stdout_event(safe_probe_target()),
            ],
        );
        let index = transcript_index(&fixture.codex_home, Vec::new());

        let stats = dispatch_readback_stats_native(
            Some(&index),
            &fixture.db_path,
            "thread-hit",
            safe_probe_target(),
        )
        .expect("native readback should count target hits");

        assert_eq!(stats.transcript_event_count, 3);
        assert_eq!(stats.transcript_target_hits, 2);
    }

    #[test]
    fn dispatch_readback_stats_returns_zero_hits_when_target_missing() {
        let fixture = dispatch_readback_fixture(
            "dispatch-readback-target-missing",
            "thread-missing",
            vec![
                dispatch_text_event("noise"),
                dispatch_stdout_event("more noise"),
            ],
        );
        let index = transcript_index(&fixture.codex_home, Vec::new());

        let stats = dispatch_readback_stats_native(
            Some(&index),
            &fixture.db_path,
            "thread-missing",
            safe_probe_target(),
        )
        .expect("native readback should return stats");

        assert_eq!(stats.transcript_event_count, 2);
        assert_eq!(stats.transcript_target_hits, 0);
    }

    #[test]
    fn dispatch_readback_stats_failure_preserves_zero_zero_downgrade() {
        let index = transcript_index(&test_temp_dir("dispatch-readback-empty-index"), Vec::new());

        let stats = dispatch_readback_stats_native(
            Some(&index),
            &PathBuf::from("/tmp/missing-dispatch-readback-state.sqlite"),
            "missing-thread",
            safe_probe_target(),
        )
        .expect("readback failure should keep compatibility downgrade");

        assert_eq!(stats.transcript_event_count, 0);
        assert_eq!(stats.transcript_target_hits, 0);
    }

    struct TranscriptCatalogFixture {
        codex_home: PathBuf,
        db_path: PathBuf,
    }

    fn transcript_catalog_fixture(prefix: &str, thread_id: &str) -> TranscriptCatalogFixture {
        let dir = test_temp_dir(prefix);
        fs::create_dir_all(&dir).expect("create temp dir");
        let codex_home = dir.join("fake-codex-home");
        let sessions_dir = codex_home.join("sessions");
        fs::create_dir_all(&sessions_dir).expect("create sessions");
        fs::create_dir_all(codex_home.join("archived_sessions")).expect("create archived");
        let rollout = sessions_dir.join(format!("{thread_id}.jsonl"));
        write_test_rollout(&rollout, "hello from sqlite");
        let db_path = codex_home.join("state_5.sqlite");
        create_test_threads_db(&db_path, thread_id, &rollout);
        TranscriptCatalogFixture {
            codex_home,
            db_path,
        }
    }

    fn dispatch_readback_fixture(
        prefix: &str,
        thread_id: &str,
        events: Vec<Value>,
    ) -> TranscriptCatalogFixture {
        let dir = test_temp_dir(prefix);
        fs::create_dir_all(&dir).expect("create temp dir");
        let codex_home = dir.join("fake-codex-home");
        let sessions_dir = codex_home.join("sessions");
        fs::create_dir_all(&sessions_dir).expect("create sessions");
        fs::create_dir_all(codex_home.join("archived_sessions")).expect("create archived");
        let rollout = sessions_dir.join(format!("{thread_id}.jsonl"));
        write_test_rollout_events(&rollout, events);
        let db_path = codex_home.join("state_5.sqlite");
        create_test_threads_db(&db_path, thread_id, &rollout);
        TranscriptCatalogFixture {
            codex_home,
            db_path,
        }
    }

    fn dispatch_text_event(message: &str) -> Value {
        json!({
            "timestamp": "2026-06-03T00:00:00Z",
            "type": "event_msg",
            "payload": {
                "type": "user_message",
                "message": message
            }
        })
    }

    fn dispatch_stdout_event(stdout: &str) -> Value {
        json!({
            "timestamp": "2026-06-03T00:00:01Z",
            "type": "response_item",
            "payload": {
                "type": "function_call_output",
                "call_id": "call-dispatch-readback",
                "output": json!({
                    "stdout": stdout,
                    "stderr": "",
                    "exit_code": 0
                }).to_string()
            }
        })
    }

    fn transcript_index(codex_home: &Path, threads: Vec<Value>) -> Value {
        json!({
            "threads": threads,
            "source_stats": {
                "codex_home": {
                    "path": codex_home.display().to_string(),
                    "role": "data_source_root"
                }
            }
        })
    }

    fn write_test_rollout(path: &Path, message: &str) {
        let row = json!({
            "timestamp": "2026-06-03T00:00:00Z",
            "type": "event_msg",
            "payload": {
                "type": "user_message",
                "message": message
            }
        });
        fs::write(path, format!("{row}\n")).expect("write rollout");
    }

    fn write_test_rollout_events(path: &Path, events: Vec<Value>) {
        let text = events
            .into_iter()
            .map(|event| event.to_string())
            .collect::<Vec<_>>()
            .join("\n");
        fs::write(path, format!("{text}\n")).expect("write rollout events");
    }

    fn create_test_threads_db(db_path: &Path, thread_id: &str, rollout_path: &Path) {
        let conn = rusqlite::Connection::open(db_path).expect("open sqlite");
        conn.execute_batch(
            r#"
            CREATE TABLE threads (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                cwd TEXT NOT NULL,
                updated_at_ms INTEGER,
                archived INTEGER NOT NULL,
                rollout_path TEXT NOT NULL,
                model TEXT,
                reasoning_effort TEXT,
                thread_source TEXT,
                has_user_event INTEGER NOT NULL
            );
            "#,
        )
        .expect("create threads table");
        conn.execute(
            r#"
            INSERT INTO threads (
                id,
                title,
                cwd,
                updated_at_ms,
                archived,
                rollout_path,
                model,
                reasoning_effort,
                thread_source,
                has_user_event
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
            "#,
            rusqlite::params![
                thread_id,
                "Sqlite thread",
                "/tmp/sqlite-project",
                1000_i64,
                0_i64,
                rollout_path.display().to_string(),
                "gpt-test",
                "medium",
                "codex",
                1_i64,
            ],
        )
        .expect("insert thread");
    }
