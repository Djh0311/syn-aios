from __future__ import annotations

import contextlib
import importlib.util
import json
import sqlite3
import sys
import tempfile
import unittest
from pathlib import Path


MODULE_PATH = Path(__file__).resolve().parents[1] / "build_index.py"


def load_build_index_module():
    spec = importlib.util.spec_from_file_location("index_kernel_build_index_folder_harness", MODULE_PATH)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load module from {MODULE_PATH}")
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


build_index = load_build_index_module()


class IndexKernelFolderHarnessTests(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory(prefix="index-kernel-folder-harness-")
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

    def insert_thread(self) -> None:
        rollout = self.sessions_dir / "thread-folder-harness.jsonl"
        rollout.write_text("", encoding="utf-8")
        record = {
            "id": "thread-folder-harness",
            "rollout_path": str(rollout),
            "created_at": "2026-05-28T00:00:00Z",
            "updated_at": "2026-05-28T00:01:00Z",
            "created_at_ms": 1_779_926_400_000,
            "updated_at_ms": 1_779_926_460_000,
            "cwd": str(self.project_root),
            "title": "folder harness fixture",
            "archived": 0,
            "archived_at": None,
            "thread_source": "user",
            "model_provider": "fixture-provider",
            "model": "fixture-model",
            "reasoning_effort": "low",
            "tokens_used": 10,
            "has_user_event": 1,
        }
        with contextlib.closing(sqlite3.connect(self.sources.sqlite_path)) as conn, conn:
            columns = [row[1] for row in conn.execute("PRAGMA table_info(threads)").fetchall()]
            present = {key: value for key, value in record.items() if key in columns}
            placeholders = ", ".join("?" for _ in present)
            column_sql = ", ".join(build_index.quote_identifier(column) for column in present)
            conn.execute(f"INSERT INTO threads ({column_sql}) VALUES ({placeholders})", list(present.values()))

    def build_project(self) -> dict[str, object]:
        self.insert_thread()
        index = build_index.build_index(self.sources)
        projects = index["projects"]
        self.assertEqual(len(projects), 1)
        return projects[0]

    def test_folder_harness_resource_reads_manifest_metadata_without_body_or_commands(self) -> None:
        harness_dir = self.project_root / "codex-harness"
        harness_dir.mkdir()
        sentinel = "FOLDER_HARNESS_BODY_SHOULD_NOT_APPEAR"
        self.write_json(
            harness_dir / "harness.json",
            {
                "name": "Codex Verify Harness",
                "version": "1.2.3",
                "harness_kind": "verification",
                "capabilities": ["verify", "codex"],
                "command": sentinel,
            },
        )
        (harness_dir / "README.md").write_text(sentinel, encoding="utf-8")
        (harness_dir / "verify.py").write_text(f"print('{sentinel}')\n", encoding="utf-8")

        project = self.build_project()
        serialized = json.dumps(project, ensure_ascii=False)
        resources = project["harness_resources"]

        self.assertNotIn(sentinel, serialized)
        self.assertEqual(len(resources), 1)
        resource = resources[0]
        self.assertEqual(resource["root_path"], str(harness_dir))
        self.assertEqual(resource["display_name"], "Codex Verify Harness")
        self.assertEqual(resource["harness_kind"], "verification")
        self.assertEqual(resource["source_kind"], "project_file")
        self.assertEqual(resource["agent_type"], "codex")
        self.assertEqual(resource["adapter_id"], "codex-local")
        self.assertEqual(resource["capabilities"], ["codex", "verify"])
        self.assertEqual(resource["manifest_path"], str(harness_dir / "harness.json"))
        self.assertEqual(resource["readme_path"], str(harness_dir / "README.md"))
        self.assertEqual(resource["version"], "1.2.3")
        self.assertEqual(resource["warnings"], [])
        self.assertTrue(any(item["name"] == "verify.py" for item in resource["entrypoints"]))

    def test_folder_harness_missing_manifest_is_derived_with_warning(self) -> None:
        harness_dir = self.project_root / "verify-harness"
        harness_dir.mkdir()
        (harness_dir / "README.md").write_text("candidate", encoding="utf-8")
        (harness_dir / "verify.sh").write_text("echo verify\n", encoding="utf-8")

        project = self.build_project()
        resource = project["harness_resources"][0]

        self.assertEqual(resource["root_path"], str(harness_dir))
        self.assertEqual(resource["source_kind"], "project_file")
        self.assertIsNone(resource["manifest_path"])
        self.assertIn("missing_manifest", resource["warnings"])
        self.assertIn("missing_version", resource["warnings"])

    def test_folder_harness_without_entrypoints_records_warning(self) -> None:
        harness_dir = self.project_root / "validation-harness"
        harness_dir.mkdir()

        project = self.build_project()
        resource = project["harness_resources"][0]

        self.assertEqual(resource["entrypoints"], [])
        self.assertIn("missing_manifest", resource["warnings"])
        self.assertIn("missing_readme", resource["warnings"])
        self.assertIn("missing_entrypoints", resource["warnings"])

    def test_plain_directory_without_harness_signal_is_not_resource(self) -> None:
        ordinary = self.project_root / "scripts" / "ordinary"
        ordinary.mkdir(parents=True)
        (ordinary / "note.txt").write_text("not a harness", encoding="utf-8")

        project = self.build_project()

        self.assertEqual(project["harness_resources"], [])

    def test_existing_file_level_harness_candidates_remain_compatible(self) -> None:
        scripts_dir = self.project_root / "scripts"
        scripts_dir.mkdir()
        (scripts_dir / "verify.py").write_text("print('ok')\n", encoding="utf-8")

        project = self.build_project()

        self.assertTrue(any(item["name"] == "verify.py" for item in project["harness_candidates"]))
        self.assertEqual(project["harness_resources"], [])


if __name__ == "__main__":
    unittest.main()
