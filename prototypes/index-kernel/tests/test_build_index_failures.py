from __future__ import annotations

import contextlib
import io
import importlib.util
import json
import sqlite3
import sys
import tempfile
import unittest
from pathlib import Path


MODULE_PATH = Path(__file__).resolve().parents[1] / "build_index.py"


def load_build_index_module():
    spec = importlib.util.spec_from_file_location("index_kernel_build_index", MODULE_PATH)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load module from {MODULE_PATH}")
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


build_index = load_build_index_module()


class IndexKernelFailureFixtureTests(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory(prefix="index-kernel-fixture-")
        self.addCleanup(self.tmp.cleanup)

        self.fixture_root = Path(self.tmp.name)
        self.codex_home = self.fixture_root / "fake-codex-home"
        self.sessions_dir = self.codex_home / "sessions"
        self.archived_sessions_dir = self.codex_home / "archived_sessions"
        self.skills_dir = self.codex_home / "skills"
        self.plugin_cache_dir = self.codex_home / "plugins" / "cache"
        self.memories_dir = self.codex_home / "memories"
        self.sources = build_index.IndexSources.from_codex_home(self.codex_home)
        for path in [
            self.sessions_dir,
            self.archived_sessions_dir,
            self.skills_dir,
            self.plugin_cache_dir,
            self.memories_dir,
        ]:
            path.mkdir(parents=True, exist_ok=True)

        self.write_json(
            self.sources.global_state_path,
            {
                "electron-saved-workspace-roots": [],
                "project-order": [],
                "active-workspace-roots": [],
                "thread-workspace-root-hints": {},
            },
        )
        self.sources.session_index_path.write_text("", encoding="utf-8")

    def write_json(self, path: Path, value: object) -> None:
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(json.dumps(value), encoding="utf-8")

    def create_threads_table(self, columns: list[str] | None = None) -> None:
        selected = columns or build_index.THREAD_FIELDS
        definitions = []
        for field in selected:
            column_type = "INTEGER" if field.endswith("_ms") or field in {"archived", "tokens_used", "has_user_event"} else "TEXT"
            definitions.append(f"{build_index.quote_identifier(field)} {column_type}")
        with contextlib.closing(sqlite3.connect(self.sources.sqlite_path)) as conn, conn:
            conn.execute(f"CREATE TABLE threads ({', '.join(definitions)})")

    def insert_thread(self, **overrides: object) -> None:
        record = {
            "id": "thread-ok",
            "rollout_path": str(self.sessions_dir / "thread-ok.jsonl"),
            "created_at": "2026-05-27T00:00:00Z",
            "updated_at": "2026-05-27T00:01:00Z",
            "created_at_ms": 1_779_840_000_000,
            "updated_at_ms": 1_779_840_060_000,
            "cwd": "/tmp/project",
            "title": "fixture thread",
            "archived": 0,
            "archived_at": None,
            "thread_source": "user",
            "model_provider": "fixture-provider",
            "model": "fixture-model",
            "reasoning_effort": "low",
            "tokens_used": 10,
            "has_user_event": 1,
        }
        record.update(overrides)
        with contextlib.closing(sqlite3.connect(self.sources.sqlite_path)) as conn, conn:
            columns = [row[1] for row in conn.execute("PRAGMA table_info(threads)").fetchall()]
            present = {key: value for key, value in record.items() if key in columns}
            placeholders = ", ".join("?" for _ in present)
            column_sql = ", ".join(build_index.quote_identifier(column) for column in present)
            conn.execute(f"INSERT INTO threads ({column_sql}) VALUES ({placeholders})", list(present.values()))

    def build_fixture_index(self) -> dict[str, object]:
        return build_index.build_index(self.sources)

    def run_main(self, args: list[str]) -> tuple[int, str, str]:
        stdout = io.StringIO()
        stderr = io.StringIO()
        with contextlib.redirect_stdout(stdout), contextlib.redirect_stderr(stderr):
            exit_code = build_index.main(args)
        return exit_code, stdout.getvalue(), stderr.getvalue()

    def test_missing_sqlite_file_degrades_with_warning(self) -> None:
        index = self.build_fixture_index()

        self.assertEqual(index["threads"], [])
        self.assertIn(f"missing_sqlite:{self.sources.sqlite_path}", index["warnings"])
        self.assertFalse(index["source_stats"]["sqlite"]["opened_readonly"])

    def test_sqlite_without_threads_table_degrades_with_warning(self) -> None:
        with contextlib.closing(sqlite3.connect(self.sources.sqlite_path)) as conn, conn:
            conn.execute("CREATE TABLE unrelated (id TEXT)")

        index = self.build_fixture_index()

        self.assertEqual(index["threads"], [])
        self.assertIn("missing_table:threads", index["warnings"])
        self.assertTrue(index["source_stats"]["sqlite"]["opened_readonly"])

    def test_threads_table_missing_noncritical_fields_still_indexes_thread(self) -> None:
        columns = ["id", "rollout_path", "created_at_ms", "updated_at_ms", "cwd", "title", "archived"]
        self.create_threads_table(columns)
        rollout = self.sessions_dir / "thread-ok.jsonl"
        rollout.write_text("", encoding="utf-8")
        self.insert_thread(rollout_path=str(rollout))

        index = self.build_fixture_index()

        self.assertEqual(len(index["threads"]), 1)
        self.assertIn("missing_threads_field:model", index["warnings"])
        self.assertIn("missing_threads_field:has_user_event", index["warnings"])
        self.assertTrue(index["threads"][0]["rollout_exists"])

    def test_threads_table_missing_id_returns_no_threads_and_clear_warning(self) -> None:
        self.create_threads_table(["rollout_path", "created_at_ms", "updated_at_ms", "cwd", "title"])

        index = self.build_fixture_index()

        self.assertEqual(index["threads"], [])
        self.assertIn("missing_threads_id_field", index["warnings"])

    def test_missing_rollout_file_adds_thread_warning(self) -> None:
        self.create_threads_table()
        self.insert_thread(rollout_path=str(self.sessions_dir / "missing.jsonl"))

        index = self.build_fixture_index()

        self.assertEqual(index["source_stats"]["sqlite"]["rollout_files"]["missing"], 1)
        self.assertFalse(index["threads"][0]["rollout_exists"])
        self.assertIn("missing_rollout_file", index["threads"][0]["warnings"])

    def test_rollout_outside_allowed_dirs_is_not_checked_as_existing(self) -> None:
        self.create_threads_table()
        outside = self.fixture_root / "outside-session.jsonl"
        outside.write_text("", encoding="utf-8")
        self.insert_thread(rollout_path=str(outside))

        index = self.build_fixture_index()

        self.assertEqual(index["source_stats"]["sqlite"]["rollout_files"]["checked"], 0)
        self.assertEqual(index["source_stats"]["sqlite"]["rollout_files"]["outside_allowed_session_dirs"], 1)
        self.assertFalse(index["threads"][0]["rollout_exists"])
        self.assertIn("rollout_path_outside_allowed_session_dirs", index["threads"][0]["warnings"])

    def test_bad_session_index_jsonl_records_warning_and_keeps_good_rows(self) -> None:
        self.create_threads_table()
        rollout = self.sessions_dir / "thread-ok.jsonl"
        rollout.write_text("", encoding="utf-8")
        self.insert_thread(rollout_path=str(rollout))
        self.sources.session_index_path.write_text(
            json.dumps({"id": "thread-ok"}) + "\n{bad json\n",
            encoding="utf-8",
        )

        index = self.build_fixture_index()

        stats = index["source_stats"]["session_index"]
        self.assertTrue(stats["loaded"])
        self.assertEqual(stats["line_count"], 2)
        self.assertEqual(stats["parsed_count"], 1)
        self.assertIn("session_index_invalid_json_line:2", index["warnings"])

    def test_bad_plugin_manifest_records_plugin_warning(self) -> None:
        self.create_threads_table()
        plugin_root = self.plugin_cache_dir / "owner" / "bad-plugin" / "1.0.0"
        manifest_path = plugin_root / ".codex-plugin" / "plugin.json"
        manifest_path.parent.mkdir(parents=True)
        manifest_path.write_text("{bad json", encoding="utf-8")

        index = self.build_fixture_index()

        self.assertEqual(len(index["plugins"]), 1)
        self.assertIn("manifest_unreadable_or_invalid", index["plugins"][0]["warnings"])
        self.assertTrue(
            any(str(item).startswith(f"invalid_json:{manifest_path}:1") for item in index["warnings"])
        )

    def test_skill_decode_failure_records_skill_warning(self) -> None:
        skill_path = self.skills_dir / "bad-encoding" / "SKILL.md"
        skill_path.parent.mkdir(parents=True)
        skill_path.write_bytes(b"\xff\xfe\x00\x00")

        index = self.build_fixture_index()

        self.assertEqual(len(index["skills"]), 1)
        self.assertIn("skill_read_decode_failed", index["skills"][0]["warnings"])

    def test_cli_codex_home_uses_injected_source_root(self) -> None:
        self.create_threads_table()
        rollout = self.sessions_dir / "thread-ok.jsonl"
        rollout.write_text("", encoding="utf-8")
        self.insert_thread(rollout_path=str(rollout))
        output_path = self.fixture_root / "index.json"

        exit_code, _, _ = self.run_main(
            [
                "--codex-home",
                str(self.codex_home),
                "--output",
                str(output_path),
            ]
        )

        self.assertEqual(exit_code, 0)
        index = json.loads(output_path.read_text(encoding="utf-8"))
        self.assertEqual(index["source_stats"]["codex_home"]["path"], str(self.codex_home))
        self.assertEqual(index["source_stats"]["sqlite"]["thread_count"], 1)
        self.assertNotIn("sqlite_thread_count_differs_from_expected:1", index["warnings"])

    def test_expected_thread_count_warning_is_opt_in(self) -> None:
        self.create_threads_table()

        default_index = self.build_fixture_index()
        checked_index = build_index.build_index(self.sources, expected_thread_count=289)

        self.assertFalse(
            any(str(warning).startswith("sqlite_thread_count_differs") for warning in default_index["warnings"])
        )
        self.assertIn("sqlite_thread_count_differs_from_expected:0", checked_index["warnings"])

    def test_check_can_require_warning_semantics(self) -> None:
        index_path = self.fixture_root / "warning-index.json"
        self.write_json(
            index_path,
            {
                "generated_at": "2026-05-27T00:00:00Z",
                "warnings": ["missing_table:threads"],
                "threads": [],
                "projects": [],
                "skills": [],
                "plugins": [],
                "memories": [],
                "source_stats": {
                    "sqlite": {"opened_readonly": True},
                    "session_index": {"role": "auxiliary_thread_list"},
                    "global_state": {"used_to_override_thread_cwd": False},
                },
            },
        )

        ok_code, _, ok_stderr = self.run_main(["--check", str(index_path), "--require-warning", "missing_table"])
        fail_code, _, fail_stderr = self.run_main(["--check", str(index_path), "--forbid-warning", "missing_table"])

        self.assertEqual(ok_code, 0)
        self.assertEqual(ok_stderr, "")
        self.assertEqual(fail_code, 1)
        self.assertIn("forbidden_warning_present:missing_table", fail_stderr)


if __name__ == "__main__":
    unittest.main()
