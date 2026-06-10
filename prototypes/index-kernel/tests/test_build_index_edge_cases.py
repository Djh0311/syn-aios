from __future__ import annotations

import importlib.util
import json
import sqlite3
import sys
import tempfile
import unittest
import contextlib
from pathlib import Path
from unittest import mock


MODULE_PATH = Path(__file__).resolve().parents[1] / "build_index.py"


def load_build_index_module():
    spec = importlib.util.spec_from_file_location("index_kernel_build_index_edge", MODULE_PATH)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load module from {MODULE_PATH}")
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


build_index = load_build_index_module()


class IndexKernelEdgeFixtureTests(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory(prefix="index-kernel-edge-")
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

    def create_threads_table(self) -> None:
        definitions = []
        for field in build_index.THREAD_FIELDS:
            column_type = "INTEGER" if field.endswith("_ms") or field in {"archived", "tokens_used", "has_user_event"} else "TEXT"
            definitions.append(f"{build_index.quote_identifier(field)} {column_type}")
        with contextlib.closing(sqlite3.connect(self.sources.sqlite_path)) as conn, conn:
            conn.execute(f"CREATE TABLE threads ({', '.join(definitions)})")

    def insert_thread(self, **overrides: object) -> None:
        record = {
            "id": "thread-edge",
            "rollout_path": str(self.sessions_dir / "thread-edge.jsonl"),
            "created_at": "2026-05-27T00:00:00Z",
            "updated_at": "2026-05-27T00:01:00Z",
            "created_at_ms": 1_779_840_000_000,
            "updated_at_ms": 1_779_840_060_000,
            "cwd": "/tmp/project",
            "title": "edge fixture thread",
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

    def test_corrupt_sqlite_file_degrades_with_sqlite_warning(self) -> None:
        self.sources.sqlite_path.write_bytes(b"this is not a sqlite database")

        index = self.build_fixture_index()

        self.assertEqual(index["threads"], [])
        self.assertEqual(index["source_stats"]["sqlite"]["thread_count"], 0)
        self.assertTrue(
            any(
                str(warning).startswith("sqlite_open_failed:")
                or str(warning).startswith("sqlite_read_failed:")
                for warning in index["warnings"]
            )
        )

    def test_unreadable_global_state_records_warning_or_skips_when_permissions_are_not_enforced(self) -> None:
        self.create_threads_table()
        global_state_path = self.sources.global_state_path
        global_state_path.chmod(0)
        try:
            index = self.build_fixture_index()
        finally:
            global_state_path.chmod(0o600)

        expected = f"read_failed:{global_state_path}:PermissionError"
        if expected not in index["warnings"]:
            self.skipTest("chmod 000 did not produce PermissionError on this filesystem/user context")

        self.assertFalse(index["source_stats"]["global_state"]["loaded"])

    def test_rollout_symlink_inside_sessions_to_outside_file_is_blocked(self) -> None:
        self.create_threads_table()
        outside = self.fixture_root / "outside-session.jsonl"
        outside.write_text("", encoding="utf-8")
        symlink_path = self.sessions_dir / "linked-outside.jsonl"
        try:
            symlink_path.symlink_to(outside)
        except (NotImplementedError, OSError) as exc:
            self.skipTest(f"symlink fixture unavailable: {exc.__class__.__name__}")
        self.insert_thread(rollout_path=str(symlink_path))

        index = self.build_fixture_index()

        sqlite_stats = index["source_stats"]["sqlite"]
        self.assertEqual(sqlite_stats["rollout_files"]["checked"], 0)
        self.assertEqual(sqlite_stats["rollout_files"]["outside_allowed_session_dirs"], 1)
        self.assertFalse(index["threads"][0]["rollout_exists"])
        self.assertIn("rollout_path_outside_allowed_session_dirs", index["threads"][0]["warnings"])

    def test_large_rollout_jsonl_body_is_not_opened_or_serialized(self) -> None:
        self.create_threads_table()
        sentinel = "EDGE_ROLLOUT_BODY_SHOULD_NOT_APPEAR"
        rollout = self.sessions_dir / "large-rollout.jsonl"
        rollout.write_text(json.dumps({"payload": sentinel + ("x" * 1_000_000)}) + "\n", encoding="utf-8")
        self.insert_thread(rollout_path=str(rollout))
        original_open = Path.open

        def guarded_open(path_self: Path, *args: object, **kwargs: object):
            if Path(path_self) == rollout:
                raise AssertionError("rollout body was opened")
            return original_open(path_self, *args, **kwargs)

        with mock.patch.object(Path, "open", guarded_open):
            index = self.build_fixture_index()

        self.assertTrue(index["threads"][0]["rollout_exists"])
        self.assertNotIn(sentinel, json.dumps(index, ensure_ascii=False))

    def test_large_session_index_payload_is_not_serialized(self) -> None:
        self.create_threads_table()
        rollout = self.sessions_dir / "thread-edge.jsonl"
        rollout.write_text("", encoding="utf-8")
        self.insert_thread(rollout_path=str(rollout))
        sentinel = "EDGE_SESSION_INDEX_BODY_SHOULD_NOT_APPEAR"
        self.sources.session_index_path.write_text(
            json.dumps(
                {
                    "id": "thread-edge",
                    "first_user_message": sentinel + ("x" * 500_000),
                    "preview": sentinel,
                    "payload": {"content": sentinel},
                }
            )
            + "\n",
            encoding="utf-8",
        )

        index = self.build_fixture_index()

        stats = index["source_stats"]["session_index"]
        self.assertTrue(stats["loaded"])
        self.assertEqual(stats["line_count"], 1)
        self.assertEqual(stats["parsed_count"], 1)
        self.assertNotIn(sentinel, json.dumps(index, ensure_ascii=False))


if __name__ == "__main__":
    unittest.main()
