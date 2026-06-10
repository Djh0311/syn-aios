from __future__ import annotations

import importlib.util
import json
import sqlite3
import sys
import tempfile
import unittest
import contextlib
from pathlib import Path


MODULE_PATH = Path(__file__).resolve().parents[1] / "build_index.py"


def load_build_index_module():
    spec = importlib.util.spec_from_file_location("index_kernel_build_index_project_context", MODULE_PATH)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load module from {MODULE_PATH}")
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


build_index = load_build_index_module()


class IndexKernelProjectContextTests(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory(prefix="index-kernel-project-context-")
        self.addCleanup(self.tmp.cleanup)

        self.fixture_root = Path(self.tmp.name)
        self.codex_home = self.fixture_root / "fake-codex-home"
        self.project_root = self.fixture_root / "project"
        self.sessions_dir = self.codex_home / "sessions"
        self.archived_sessions_dir = self.codex_home / "archived_sessions"
        self.skills_dir = self.codex_home / "skills"
        self.plugin_cache_dir = self.codex_home / "plugins" / "cache"
        self.memories_dir = self.codex_home / "memories"
        self.sources = build_index.IndexSources.from_codex_home(self.codex_home)
        for path in [
            self.project_root,
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
        self.create_threads_table()

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
        rollout = self.sessions_dir / "thread-project-context.jsonl"
        rollout.write_text("", encoding="utf-8")
        record = {
            "id": "thread-project-context",
            "rollout_path": str(rollout),
            "created_at": "2026-05-27T00:00:00Z",
            "updated_at": "2026-05-27T00:01:00Z",
            "created_at_ms": 1_779_840_000_000,
            "updated_at_ms": 1_779_840_060_000,
            "cwd": str(self.project_root),
            "title": "project context fixture",
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

    def only_project(self, index: dict[str, object]) -> dict[str, object]:
        projects = index["projects"]
        self.assertIsInstance(projects, list)
        self.assertEqual(len(projects), 1)
        return projects[0]

    def test_project_context_scans_metadata_candidates_without_reading_doc_bodies(self) -> None:
        self.insert_thread()
        sentinel = "PROJECT_CONTEXT_BODY_SHOULD_NOT_APPEAR"
        (self.project_root / "README.md").write_text(sentinel, encoding="utf-8")
        (self.project_root / "AGENTS.md").write_text(sentinel, encoding="utf-8")
        (self.project_root / "docs").mkdir()
        (self.project_root / "docs" / "current-state.md").write_text(sentinel, encoding="utf-8")
        (self.project_root / "handoffs").mkdir()
        (self.project_root / "handoffs" / "handoff-one.md").write_text(sentinel, encoding="utf-8")
        (self.project_root / "evidence").mkdir()
        (self.project_root / "evidence" / "evidence-one.md").write_text(sentinel, encoding="utf-8")
        (self.project_root / "scripts").mkdir()
        (self.project_root / "scripts" / "verify.py").write_text("print('do not run')\n", encoding="utf-8")
        self.write_json(
            self.project_root / "package.json",
            {
                "scripts": {
                    "test": "echo should-not-be-indexed",
                    "dev": "vite --host 0.0.0.0",
                }
            },
        )
        (self.project_root / "Makefile").write_text("verify:\n\tpytest\n", encoding="utf-8")
        (self.project_root / "vite.config.ts").write_text("export default {}\n", encoding="utf-8")

        index = self.build_fixture_index()
        project = self.only_project(index)
        serialized = json.dumps(index, ensure_ascii=False)

        self.assertNotIn(sentinel, serialized)
        self.assertNotIn("echo should-not-be-indexed", serialized)
        self.assertGreaterEqual(len(project["authority_files"]), 3)
        self.assertEqual(len(project["handoff_files"]), 1)
        self.assertEqual(len(project["evidence_files"]), 1)
        harness_names = {item["name"] for item in project["harness_candidates"]}
        self.assertIn("test", harness_names)
        self.assertIn("dev", harness_names)
        self.assertIn("verify", harness_names)
        self.assertIn("verify.py", harness_names)
        self.assertIn("vite.config.ts", harness_names)
        self.assertEqual(project["context_warnings"], [])
        self.assertEqual(index["source_stats"]["project_context"]["projects_scanned"], 1)
        self.assertGreaterEqual(index["source_stats"]["project_context"]["harness_candidate_count"], 5)

    def test_missing_project_root_adds_context_warning(self) -> None:
        missing_root = self.fixture_root / "missing-project"
        self.insert_thread(cwd=str(missing_root))

        index = self.build_fixture_index()
        project = self.only_project(index)

        self.assertIn("project_root_missing", project["context_warnings"])
        self.assertEqual(project["authority_files"], [])
        self.assertEqual(index["source_stats"]["project_context"]["projects_missing"], 1)

    def test_symlink_candidates_outside_project_are_blocked(self) -> None:
        self.insert_thread()
        outside = self.fixture_root / "outside.md"
        outside.write_text("outside", encoding="utf-8")
        linked = self.project_root / "README.md"
        try:
            linked.symlink_to(outside)
        except (NotImplementedError, OSError) as exc:
            self.skipTest(f"symlink fixture unavailable: {exc.__class__.__name__}")

        index = self.build_fixture_index()
        project = self.only_project(index)

        self.assertEqual(project["authority_files"], [])
        self.assertTrue(
            any(str(warning).startswith("symlink_outside_project:") for warning in project["context_warnings"])
        )

    def test_handoff_candidates_are_truncated_with_warning(self) -> None:
        self.insert_thread()
        handoff_dir = self.project_root / "handoffs"
        handoff_dir.mkdir()
        for index in range(build_index.MAX_CONTEXT_FILES_PER_KIND + 3):
            (handoff_dir / f"handoff-{index:02d}.md").write_text("candidate", encoding="utf-8")

        index = self.build_fixture_index()
        project = self.only_project(index)

        self.assertEqual(len(project["handoff_files"]), build_index.MAX_CONTEXT_FILES_PER_KIND)
        self.assertIn("handoff_candidates_truncated", project["context_warnings"])


if __name__ == "__main__":
    unittest.main()
