#!/usr/bin/env python3
from __future__ import annotations

import argparse
from dataclasses import dataclass
import datetime as dt
import json
import re
from pathlib import Path
import sqlite3
import sys
from typing import Any


DEFAULT_CODEX_HOME = Path("/Users/yoyi/.codex")

DEFAULT_OUTPUT = (
    Path("/Users/yoyi/workspace")
    / "product-line"
    / "prototypes"
    / "index-kernel"
    / "codex-index.json"
)

THREAD_FIELDS = [
    "id",
    "rollout_path",
    "created_at",
    "updated_at",
    "created_at_ms",
    "updated_at_ms",
    "cwd",
    "title",
    "archived",
    "archived_at",
    "thread_source",
    "model_provider",
    "model",
    "reasoning_effort",
    "tokens_used",
    "has_user_event",
]

MAX_TITLE_CHARS = 160
INVENTORY_THREAD_COUNT = 289
MAX_CONTEXT_FILES_PER_KIND = 24
MAX_HARNESS_CANDIDATES = 40
MAX_HARNESS_RESOURCES = 24
MAX_HARNESS_ENTRYPOINTS = 24
SKIP_CONTEXT_DIRS = {
    ".git",
    ".next",
    ".nuxt",
    ".svelte-kit",
    ".turbo",
    ".venv",
    "__pycache__",
    "build",
    "coverage",
    "dist",
    "node_modules",
    "target",
    "vendor",
}
AUTHORITY_FILE_TYPES = {
    "AGENTS.md": "agents",
    "CLAUDE.md": "claude",
    "README.md": "readme",
    "README": "readme",
    "STAGE_PLAN.md": "stage_plan",
    "TASK_QUEUE.md": "task_queue",
    "task-queue.md": "task_queue",
    "current-state.md": "current_state",
    "decisions.md": "decisions",
    "open-questions.md": "open_questions",
}
AUTHORITY_DIRS = ("", "docs", ".codex", "product-line")
HANDOFF_DIR_NAMES = {"handoff", "handoffs", "hand-off", "hand-offs"}
EVIDENCE_DIR_NAMES = {"evidence", "evidences"}
SCRIPT_SUFFIX_TYPES = {
    ".js": "node_script",
    ".mjs": "node_script",
    ".cjs": "node_script",
    ".ts": "typescript_script",
    ".mts": "typescript_script",
    ".cts": "typescript_script",
    ".py": "python_script",
    ".sh": "shell_script",
}
HARNESS_DIR_NAME_MARKERS = ("harness", "validation", "verify", "codex")
HARNESS_CONTAINER_DIRS = ("scripts", "tools", ".codex", "harness", "tests")
HARNESS_MANIFEST_NAMES = (
    "harness.json",
    "harness.toml",
    "codex-harness.json",
    "codex-harness.toml",
    "manifest.json",
    "manifest.toml",
    "package.json",
)
HARNESS_README_NAMES = ("README.md", "README")
HARNESS_ENTRYPOINT_NAMES = (
    "run.sh",
    "verify.sh",
    "validate.sh",
    "test.sh",
    "run.py",
    "verify.py",
    "validate.py",
    "index.js",
    "index.ts",
    "main.py",
)

REQUIRED_THREAD_OUTPUT_KEYS = {
    "thread_id",
    "title",
    "project_root",
    "rollout_path",
    "rollout_exists",
    "created_at_ms",
    "updated_at_ms",
    "archived",
    "thread_source",
    "model",
    "model_provider",
    "reasoning_effort",
    "tokens_used",
    "has_user_event",
    "warnings",
}

REQUIRED_PROJECT_OUTPUT_KEYS = {
    "project_root",
    "thread_count",
    "active_thread_count",
    "archived_thread_count",
    "latest_updated_at_ms",
    "from_saved_workspace_roots",
    "active_hint",
    "order_hint",
    "authority_files",
    "handoff_files",
    "evidence_files",
    "harness_candidates",
    "harness_resources",
    "context_warnings",
    "warnings",
}


@dataclass(frozen=True)
class IndexSources:
    codex_home: Path
    sqlite_path: Path
    session_index_path: Path
    global_state_path: Path
    sessions_dir: Path
    archived_sessions_dir: Path
    skills_dir: Path
    plugins_dir: Path
    plugin_cache_dir: Path
    memories_dir: Path

    @classmethod
    def from_codex_home(cls, codex_home: Path) -> "IndexSources":
        codex_home = codex_home.expanduser()
        plugins_dir = codex_home / "plugins"
        return cls(
            codex_home=codex_home,
            sqlite_path=codex_home / "state_5.sqlite",
            session_index_path=codex_home / "session_index.jsonl",
            global_state_path=codex_home / ".codex-global-state.json",
            sessions_dir=codex_home / "sessions",
            archived_sessions_dir=codex_home / "archived_sessions",
            skills_dir=codex_home / "skills",
            plugins_dir=plugins_dir,
            plugin_cache_dir=plugins_dir / "cache",
            memories_dir=codex_home / "memories",
        )


def utc_now_iso() -> str:
    return dt.datetime.now(dt.UTC).isoformat(timespec="seconds").replace("+00:00", "Z")


def path_string(path: Path | None) -> str | None:
    if path is None:
        return None
    return str(path)


def is_relative_to(path: Path, parent: Path) -> bool:
    try:
        path.resolve(strict=False).relative_to(parent.resolve(strict=False))
        return True
    except ValueError:
        return False


def safe_stat_mtime_ms(path: Path) -> int | None:
    try:
        return int(path.stat().st_mtime * 1000)
    except OSError:
        return None


def file_metadata(path: Path, kind: str) -> dict[str, Any]:
    warnings: list[str] = []
    size_bytes = None
    updated_at_ms = None
    try:
        stat = path.stat()
        size_bytes = stat.st_size
        updated_at_ms = int(stat.st_mtime * 1000)
    except OSError as exc:
        warnings.append(f"metadata_read_failed:{exc.__class__.__name__}")
    return {
        "path": str(path),
        "kind": kind,
        "size_bytes": size_bytes,
        "updated_at_ms": updated_at_ms,
        "warnings": warnings,
    }


def count_lines(path: Path) -> int | None:
    try:
        with path.open("rb") as handle:
            return sum(1 for _ in handle)
    except OSError:
        return None


def load_json(path: Path) -> tuple[Any | None, list[str]]:
    warnings: list[str] = []
    try:
        with path.open("r", encoding="utf-8") as handle:
            return json.load(handle), warnings
    except FileNotFoundError:
        warnings.append(f"missing_file:{path}")
    except json.JSONDecodeError as exc:
        warnings.append(f"invalid_json:{path}:{exc.lineno}")
    except OSError as exc:
        warnings.append(f"read_failed:{path}:{exc.__class__.__name__}")
    return None, warnings


def coerce_ms(value: Any) -> int | None:
    if value is None or value == "":
        return None
    if isinstance(value, (int, float)):
        number = float(value)
        if number > 10_000_000_000:
            return int(number)
        if number > 1_000_000_000:
            return int(number * 1000)
        return int(number)
    if isinstance(value, str):
        text = value.strip()
        if not text:
            return None
        if text.isdigit():
            return coerce_ms(int(text))
        try:
            parsed = dt.datetime.fromisoformat(text.replace("Z", "+00:00"))
            return int(parsed.timestamp() * 1000)
        except ValueError:
            return None
    return None


def normalize_bool(value: Any) -> bool:
    if isinstance(value, bool):
        return value
    if isinstance(value, (int, float)):
        return value != 0
    if isinstance(value, str):
        return value.strip().lower() in {"1", "true", "yes", "y"}
    return False


def compact_text(value: Any, limit: int) -> tuple[str, bool]:
    if not isinstance(value, str):
        return "", False
    text = " ".join(value.split())
    if len(text) <= limit:
        return text, False
    return text[:limit].rstrip() + "...", True


def quote_identifier(identifier: str) -> str:
    return '"' + identifier.replace('"', '""') + '"'


def empty_project_context() -> dict[str, Any]:
    return {
        "authority_files": [],
        "handoff_files": [],
        "evidence_files": [],
        "harness_candidates": [],
        "harness_resources": [],
        "context_warnings": [],
    }


def should_skip_context_dir(path: Path) -> bool:
    return path.name in SKIP_CONTEXT_DIRS


def safe_resolve(path: Path) -> Path | None:
    try:
        return path.resolve(strict=True)
    except OSError:
        return None


def context_candidate_allowed(path: Path, project_root: Path, warnings: list[str]) -> bool:
    if path.is_symlink():
        resolved = safe_resolve(path)
        if resolved is None:
            warnings.append(f"symlink_unreadable:{path}")
            return False
        if not is_relative_to(resolved, project_root):
            warnings.append(f"symlink_outside_project:{path}")
            return False
    return True


def append_limited(
    items: list[dict[str, Any]],
    item: dict[str, Any],
    limit: int,
    warnings: list[str],
    warning_code: str,
) -> None:
    if len(items) >= limit:
        if warning_code not in warnings:
            warnings.append(warning_code)
        return
    items.append(item)


def scan_authority_files(project_root: Path, context_warnings: list[str]) -> list[dict[str, Any]]:
    authority_files: list[dict[str, Any]] = []
    seen: set[Path] = set()
    for directory_name in AUTHORITY_DIRS:
        directory = project_root / directory_name if directory_name else project_root
        try:
            is_dir = directory.is_dir()
        except OSError as exc:
            context_warnings.append(f"authority_dir_read_failed:{directory}:{exc.__class__.__name__}")
            continue
        if not is_dir:
            continue
        for filename, kind in AUTHORITY_FILE_TYPES.items():
            path = directory / filename
            try:
                exists = path.is_file()
            except OSError as exc:
                context_warnings.append(f"authority_file_read_failed:{path}:{exc.__class__.__name__}")
                continue
            if not exists:
                continue
            if path in seen:
                continue
            seen.add(path)
            if not context_candidate_allowed(path, project_root, context_warnings):
                continue
            append_limited(
                authority_files,
                file_metadata(path, kind),
                MAX_CONTEXT_FILES_PER_KIND,
                context_warnings,
                "authority_candidates_truncated",
            )
    return authority_files


def iter_named_dirs(project_root: Path, names: set[str]) -> list[Path]:
    matched: list[Path] = []
    for base in [project_root, project_root / "docs", project_root / "product-line"]:
        try:
            children = list(base.iterdir()) if base.is_dir() else []
        except OSError:
            continue
        for child in children:
            if child.name in names and child.is_dir():
                matched.append(child)
    return matched


def scan_context_file_group(
    project_root: Path,
    dir_names: set[str],
    kind: str,
    context_warnings: list[str],
    warning_prefix: str,
) -> list[dict[str, Any]]:
    files: list[dict[str, Any]] = []
    seen: set[Path] = set()
    for directory in iter_named_dirs(project_root, dir_names):
        if not context_candidate_allowed(directory, project_root, context_warnings):
            continue
        try:
            candidates = sorted(path for path in directory.glob("*.md") if path.is_file())
        except OSError as exc:
            context_warnings.append(f"{warning_prefix}_dir_read_failed:{directory}:{exc.__class__.__name__}")
            continue
        for path in candidates:
            if path in seen:
                continue
            seen.add(path)
            if not context_candidate_allowed(path, project_root, context_warnings):
                continue
            append_limited(
                files,
                file_metadata(path, kind),
                MAX_CONTEXT_FILES_PER_KIND,
                context_warnings,
                f"{warning_prefix}_candidates_truncated",
            )
    return files


def add_harness_candidate(
    candidates: list[dict[str, Any]],
    context_warnings: list[str],
    *,
    entry_type: str,
    source: str,
    path: Path,
    name: str | None = None,
) -> None:
    append_limited(
        candidates,
        {
            "entry_type": entry_type,
            "source": source,
            "path": str(path),
            "name": name,
            "size_bytes": file_metadata(path, entry_type)["size_bytes"],
            "updated_at_ms": safe_stat_mtime_ms(path),
            "warnings": [],
        },
        MAX_HARNESS_CANDIDATES,
        context_warnings,
        "harness_candidates_truncated",
    )


def scan_package_scripts(
    package_path: Path, project_root: Path, candidates: list[dict[str, Any]], context_warnings: list[str]
) -> None:
    if not context_candidate_allowed(package_path, project_root, context_warnings):
        return
    data, warnings = load_json(package_path)
    context_warnings.extend(f"package_json_{warning}" for warning in warnings)
    if not isinstance(data, dict):
        return
    scripts = data.get("scripts")
    if not isinstance(scripts, dict):
        return
    for name in sorted(key for key in scripts.keys() if isinstance(key, str)):
        add_harness_candidate(
            candidates,
            context_warnings,
            entry_type="package_script",
            source="package.json",
            path=package_path,
            name=name,
        )


MAKE_TARGET_RE = re.compile(r"^([A-Za-z0-9_.%/-]+):(?:\s|$)")


def scan_makefile_targets(
    makefile_path: Path, project_root: Path, candidates: list[dict[str, Any]], context_warnings: list[str]
) -> None:
    if not context_candidate_allowed(makefile_path, project_root, context_warnings):
        return
    try:
        with makefile_path.open("r", encoding="utf-8", errors="ignore") as handle:
            for line in handle:
                if line.startswith(("\t", "#", ".")):
                    continue
                match = MAKE_TARGET_RE.match(line)
                if not match:
                    continue
                name = match.group(1)
                if "=" in name:
                    continue
                add_harness_candidate(
                    candidates,
                    context_warnings,
                    entry_type="make_target",
                    source=makefile_path.name,
                    path=makefile_path,
                    name=name,
                )
    except OSError as exc:
        context_warnings.append(f"makefile_read_failed:{makefile_path}:{exc.__class__.__name__}")


def scan_script_files(project_root: Path, candidates: list[dict[str, Any]], context_warnings: list[str]) -> None:
    for directory_name in ("scripts", "tools"):
        directory = project_root / directory_name
        try:
            is_dir = directory.is_dir()
        except OSError as exc:
            context_warnings.append(f"script_dir_read_failed:{directory}:{exc.__class__.__name__}")
            continue
        if not is_dir or not context_candidate_allowed(directory, project_root, context_warnings):
            continue
        try:
            children = sorted(directory.iterdir())
        except OSError as exc:
            context_warnings.append(f"script_dir_read_failed:{directory}:{exc.__class__.__name__}")
            continue
        for child in children:
            if child.is_dir() or should_skip_context_dir(child):
                continue
            entry_type = SCRIPT_SUFFIX_TYPES.get(child.suffix)
            if entry_type is None:
                continue
            if not context_candidate_allowed(child, project_root, context_warnings):
                continue
            add_harness_candidate(
                candidates,
                context_warnings,
                entry_type=entry_type,
                source=directory_name,
                path=child,
                name=child.name,
            )


def scan_config_harness(project_root: Path, candidates: list[dict[str, Any]], context_warnings: list[str]) -> None:
    config_candidates = [
        ("vite_config", "vite", ["vite.config.ts", "vite.config.js", "vite.config.mjs"]),
        ("godot_project", "godot", ["project.godot"]),
        ("python_project", "python", ["pyproject.toml"]),
        ("node_project", "node", ["package.json"]),
    ]
    for entry_type, source, names in config_candidates:
        for name in names:
            path = project_root / name
            try:
                exists = path.is_file()
            except OSError as exc:
                context_warnings.append(f"config_read_failed:{path}:{exc.__class__.__name__}")
                continue
            if not exists or not context_candidate_allowed(path, project_root, context_warnings):
                continue
            add_harness_candidate(
                candidates,
                context_warnings,
                entry_type=entry_type,
                source=source,
                path=path,
                name=name,
            )


def scan_harness_candidates(project_root: Path, context_warnings: list[str]) -> list[dict[str, Any]]:
    candidates: list[dict[str, Any]] = []
    package_path = project_root / "package.json"
    if package_path.is_file():
        scan_package_scripts(package_path, project_root, candidates, context_warnings)

    for makefile_name in ("Makefile", "makefile"):
        makefile_path = project_root / makefile_name
        if makefile_path.is_file():
            scan_makefile_targets(makefile_path, project_root, candidates, context_warnings)

    scan_script_files(project_root, candidates, context_warnings)
    scan_config_harness(project_root, candidates, context_warnings)
    return candidates


def directory_has_harness_name(path: Path) -> bool:
    name = path.name.lower()
    return any(marker in name for marker in HARNESS_DIR_NAME_MARKERS)


def first_existing_file(directory: Path, names: tuple[str, ...]) -> Path | None:
    for name in names:
        path = directory / name
        try:
            if path.is_file():
                return path
        except OSError:
            continue
    return None


def read_harness_manifest(manifest_path: Path, warnings: list[str]) -> dict[str, Any]:
    if manifest_path.suffix.lower() != ".json":
        return {}
    data, load_warnings = load_json(manifest_path)
    warnings.extend(f"manifest_{warning}" for warning in load_warnings)
    if isinstance(data, dict):
        return data
    return {}


def infer_harness_kind(root_path: Path, manifest: dict[str, Any]) -> str:
    for key in ("harness_kind", "kind", "type"):
        value = manifest.get(key)
        if isinstance(value, str) and value.strip():
            return value.strip()
    name = root_path.name.lower()
    if "validation" in name or "verify" in name:
        return "validation"
    if "codex" in name:
        return "codex_harness"
    return "folder_harness"


def infer_harness_capabilities(root_path: Path, manifest: dict[str, Any], entrypoints: list[dict[str, Any]]) -> list[str]:
    raw_capabilities = manifest.get("capabilities")
    if isinstance(raw_capabilities, list):
        capabilities = sorted({item for item in raw_capabilities if isinstance(item, str) and item})
        if capabilities:
            return capabilities

    capabilities: set[str] = {"harness"}
    name = root_path.name.lower()
    entrypoint_names = " ".join(str(item.get("name") or "") for item in entrypoints).lower()
    joined = f"{name} {entrypoint_names}"
    if "verify" in joined:
        capabilities.add("verify")
    if "validat" in joined:
        capabilities.add("validate")
    if "test" in joined:
        capabilities.add("test")
    if "codex" in joined:
        capabilities.add("codex")
    return sorted(capabilities)


def harness_entrypoint_from_path(path: Path, root_path: Path, source_kind: str, warnings: list[str]) -> dict[str, Any] | None:
    if not context_candidate_allowed(path, root_path, warnings):
        return None
    if path.name in HARNESS_MANIFEST_NAMES:
        entry_type = "manifest"
    elif path.name in HARNESS_README_NAMES:
        entry_type = "readme"
    else:
        entry_type = SCRIPT_SUFFIX_TYPES.get(path.suffix, "file")
    metadata = file_metadata(path, entry_type)
    return {
        "entry_type": entry_type,
        "source_kind": source_kind,
        "path": str(path),
        "name": path.name,
        "size_bytes": metadata["size_bytes"],
        "updated_at_ms": metadata["updated_at_ms"],
        "warnings": metadata["warnings"],
    }


def scan_harness_entrypoints(root_path: Path, warnings: list[str]) -> list[dict[str, Any]]:
    entrypoints: list[dict[str, Any]] = []
    seen: set[Path] = set()
    try:
        children = sorted(root_path.iterdir())
    except OSError as exc:
        warnings.append(f"entrypoints_read_failed:{exc.__class__.__name__}")
        return entrypoints

    for child in children:
        if child.is_dir() or should_skip_context_dir(child):
            continue
        if child.name not in HARNESS_MANIFEST_NAMES and child.name not in HARNESS_README_NAMES and child.suffix not in SCRIPT_SUFFIX_TYPES:
            continue
        if child in seen:
            continue
        seen.add(child)
        entrypoint = harness_entrypoint_from_path(child, root_path, "project_file", warnings)
        if entrypoint is None:
            continue
        append_limited(
            entrypoints,
            entrypoint,
            MAX_HARNESS_ENTRYPOINTS,
            warnings,
            "entrypoints_truncated",
        )
    return entrypoints


def candidate_directory_has_signal(directory: Path) -> bool:
    if directory_has_harness_name(directory):
        return True
    for name in HARNESS_MANIFEST_NAMES + HARNESS_README_NAMES + HARNESS_ENTRYPOINT_NAMES:
        try:
            if (directory / name).is_file():
                return True
        except OSError:
            return False
    return False


def candidate_directory_has_resource_signal(directory: Path) -> bool:
    if directory_has_harness_name(directory):
        return True
    for name in HARNESS_MANIFEST_NAMES + HARNESS_README_NAMES:
        try:
            if (directory / name).is_file():
                return True
        except OSError:
            return False
    return False


def find_harness_directories(
    project_root: Path, harness_candidates: list[dict[str, Any]], context_warnings: list[str]
) -> list[Path]:
    directories: list[Path] = []
    seen: set[Path] = set()

    def add_directory(directory: Path) -> None:
        if directory in seen:
            return
        seen.add(directory)
        if should_skip_context_dir(directory):
            return
        if not context_candidate_allowed(directory, project_root, context_warnings):
            return
        directories.append(directory)

    try:
        root_children = sorted(project_root.iterdir())
    except OSError as exc:
        context_warnings.append(f"harness_root_scan_failed:{exc.__class__.__name__}")
        root_children = []
    for child in root_children:
        try:
            if child.is_dir() and directory_has_harness_name(child):
                add_directory(child)
        except OSError as exc:
            context_warnings.append(f"harness_dir_stat_failed:{child}:{exc.__class__.__name__}")

    for container_name in HARNESS_CONTAINER_DIRS:
        container = project_root / container_name
        try:
            if not container.is_dir():
                continue
            children = sorted(container.iterdir())
        except OSError as exc:
            context_warnings.append(f"harness_container_read_failed:{container}:{exc.__class__.__name__}")
            continue
        if container.name == "harness" or container.name == ".codex":
            add_directory(container)
        for child in children:
            try:
                if child.is_dir() and candidate_directory_has_signal(child):
                    add_directory(child)
            except OSError as exc:
                context_warnings.append(f"harness_dir_stat_failed:{child}:{exc.__class__.__name__}")

    for candidate in harness_candidates:
        path_value = candidate.get("path")
        if not isinstance(path_value, str):
            continue
        path = Path(path_value)
        parent = path.parent
        if parent == project_root:
            continue
        if not is_relative_to(parent, project_root):
            continue
        if candidate_directory_has_resource_signal(parent):
            add_directory(parent)

    return directories


def build_harness_resource(root_path: Path, project_root: Path) -> dict[str, Any]:
    warnings: list[str] = []
    manifest_path = first_existing_file(root_path, HARNESS_MANIFEST_NAMES)
    readme_path = first_existing_file(root_path, HARNESS_README_NAMES)
    manifest = read_harness_manifest(manifest_path, warnings) if manifest_path else {}
    entrypoints = scan_harness_entrypoints(root_path, warnings)

    source_kind = "project_file" if manifest_path or readme_path else "derived"
    if manifest_path is None:
        warnings.append("missing_manifest")
    if readme_path is None:
        warnings.append("missing_readme")
    if not entrypoints:
        warnings.append("missing_entrypoints")
    if not directory_has_harness_name(root_path) and manifest_path is None:
        warnings.append("weak_harness_signal")

    version = manifest.get("version")
    if not isinstance(version, str) or not version.strip():
        version = None
        warnings.append("missing_version")

    display_name = manifest.get("display_name") or manifest.get("name")
    if not isinstance(display_name, str) or not display_name.strip():
        display_name = root_path.name

    adapter_id = manifest.get("adapter_id")
    if not isinstance(adapter_id, str) or not adapter_id.strip():
        adapter_id = "codex-local"

    agent_type = manifest.get("agent_type")
    if not isinstance(agent_type, str) or not agent_type.strip():
        agent_type = "codex"

    metadata = file_metadata(root_path, "harness_resource")
    return {
        "root_path": str(root_path),
        "display_name": display_name,
        "harness_kind": infer_harness_kind(root_path, manifest),
        "source_kind": source_kind,
        "agent_type": agent_type,
        "adapter_id": adapter_id,
        "capabilities": infer_harness_capabilities(root_path, manifest, entrypoints),
        "entrypoints": entrypoints,
        "manifest_path": str(manifest_path) if manifest_path else None,
        "readme_path": str(readme_path) if readme_path else None,
        "version": version,
        "size_bytes": metadata["size_bytes"],
        "updated_at_ms": metadata["updated_at_ms"],
        "permission_level": "read_only",
        "warnings": warnings + metadata["warnings"],
    }


def scan_harness_resources(
    project_root: Path, harness_candidates: list[dict[str, Any]], context_warnings: list[str]
) -> list[dict[str, Any]]:
    resources: list[dict[str, Any]] = []
    for directory in find_harness_directories(project_root, harness_candidates, context_warnings):
        if len(resources) >= MAX_HARNESS_RESOURCES:
            if "harness_resources_truncated" not in context_warnings:
                context_warnings.append("harness_resources_truncated")
            break
        if not is_relative_to(directory, project_root):
            context_warnings.append(f"harness_resource_outside_project:{directory}")
            continue
        resource = build_harness_resource(directory, project_root)
        resources.append(resource)
    return resources


def scan_project_context(project: dict[str, Any]) -> dict[str, Any]:
    context = empty_project_context()
    root_value = project.get("project_root")
    if not isinstance(root_value, str) or not root_value or root_value == "__missing_project_root__":
        context["context_warnings"].append("missing_project_root")
        return context

    project_root = Path(root_value)
    try:
        is_dir = project_root.is_dir()
    except OSError as exc:
        context["context_warnings"].append(f"project_root_read_failed:{exc.__class__.__name__}")
        return context
    if not is_dir:
        context["context_warnings"].append("project_root_missing")
        return context
    if project_root.is_symlink():
        resolved = safe_resolve(project_root)
        if resolved is None:
            context["context_warnings"].append("project_root_symlink_unreadable")
            return context
        project_root = resolved

    context["authority_files"] = scan_authority_files(project_root, context["context_warnings"])
    context["handoff_files"] = scan_context_file_group(
        project_root, HANDOFF_DIR_NAMES, "handoff", context["context_warnings"], "handoff"
    )
    context["evidence_files"] = scan_context_file_group(
        project_root, EVIDENCE_DIR_NAMES, "evidence", context["context_warnings"], "evidence"
    )
    context["harness_candidates"] = scan_harness_candidates(project_root, context["context_warnings"])
    context["harness_resources"] = scan_harness_resources(
        project_root, context["harness_candidates"], context["context_warnings"]
    )
    return context


def attach_project_context(projects: list[dict[str, Any]]) -> dict[str, Any]:
    stats = {
        "role": "project_context_candidate_source",
        "projects_scanned": 0,
        "projects_missing": 0,
        "authority_file_count": 0,
        "handoff_file_count": 0,
        "evidence_file_count": 0,
        "harness_candidate_count": 0,
        "harness_resource_count": 0,
        "harness_resource_warning_count": 0,
        "context_warning_count": 0,
    }
    for project in projects:
        context = scan_project_context(project)
        project.update(context)
        stats["projects_scanned"] += 1
        if "project_root_missing" in context["context_warnings"]:
            stats["projects_missing"] += 1
        stats["authority_file_count"] += len(context["authority_files"])
        stats["handoff_file_count"] += len(context["handoff_files"])
        stats["evidence_file_count"] += len(context["evidence_files"])
        stats["harness_candidate_count"] += len(context["harness_candidates"])
        stats["harness_resource_count"] += len(context["harness_resources"])
        stats["harness_resource_warning_count"] += sum(
            len(resource.get("warnings", [])) for resource in context["harness_resources"]
        )
        stats["context_warning_count"] += len(context["context_warnings"])
    return stats


def fetch_threads(sources: IndexSources, index_warnings: list[str]) -> tuple[list[dict[str, Any]], dict[str, Any]]:
    stats: dict[str, Any] = {
        "path": str(sources.sqlite_path),
        "role": "primary_thread_index",
        "opened_readonly": False,
        "query_only_enabled": False,
        "threads_table_exists": False,
        "thread_count": 0,
        "schema_columns": [],
        "missing_fields": [],
    }

    if not sources.sqlite_path.exists():
        index_warnings.append(f"missing_sqlite:{sources.sqlite_path}")
        return [], stats

    uri = f"file:{sources.sqlite_path}?mode=ro"
    try:
        conn = sqlite3.connect(uri, uri=True)
    except sqlite3.Error as exc:
        index_warnings.append(f"sqlite_open_failed:{exc.__class__.__name__}")
        return [], stats

    conn.row_factory = sqlite3.Row
    try:
        conn.execute("PRAGMA query_only = ON")
        query_only = conn.execute("PRAGMA query_only").fetchone()
        stats["opened_readonly"] = True
        stats["query_only_enabled"] = bool(query_only and query_only[0] == 1)

        table_row = conn.execute(
            "SELECT name FROM sqlite_master WHERE type='table' AND name='threads'"
        ).fetchone()
        if table_row is None:
            index_warnings.append("missing_table:threads")
            return [], stats
        stats["threads_table_exists"] = True

        columns = [row["name"] for row in conn.execute("PRAGMA table_info(threads)")]
        stats["schema_columns"] = columns
        available = [field for field in THREAD_FIELDS if field in columns]
        missing = [field for field in THREAD_FIELDS if field not in columns]
        stats["missing_fields"] = missing
        for field in missing:
            index_warnings.append(f"missing_threads_field:{field}")

        if "id" not in columns:
            index_warnings.append("missing_threads_id_field")
            return [], stats

        select_sql = ", ".join(quote_identifier(field) for field in available)
        rows = conn.execute(f"SELECT {select_sql} FROM threads").fetchall()
        stats["thread_count"] = len(rows)
    except sqlite3.Error as exc:
        index_warnings.append(f"sqlite_read_failed:{exc.__class__.__name__}")
        return [], stats
    finally:
        conn.close()

    threads: list[dict[str, Any]] = []
    rollout_checked = 0
    rollout_existing = 0
    rollout_missing = 0
    rollout_outside_allowed = 0

    for row in rows:
        record = {field: row[field] if field in row.keys() else None for field in THREAD_FIELDS}
        warnings: list[str] = []

        thread_id = record.get("id")
        project_root = record.get("cwd")
        rollout_path_raw = record.get("rollout_path")
        rollout_path = Path(rollout_path_raw) if isinstance(rollout_path_raw, str) and rollout_path_raw else None
        rollout_exists = False

        if rollout_path is None:
            warnings.append("missing_rollout_path")
        elif not (
            is_relative_to(rollout_path, sources.sessions_dir)
            or is_relative_to(rollout_path, sources.archived_sessions_dir)
        ):
            rollout_outside_allowed += 1
            warnings.append("rollout_path_outside_allowed_session_dirs")
        else:
            rollout_checked += 1
            rollout_exists = rollout_path.exists()
            if rollout_exists:
                rollout_existing += 1
            else:
                rollout_missing += 1
                warnings.append("missing_rollout_file")

        created_at_ms = coerce_ms(record.get("created_at_ms"))
        if created_at_ms is None:
            created_at_ms = coerce_ms(record.get("created_at"))
            if created_at_ms is None:
                warnings.append("missing_created_at_ms")
            else:
                warnings.append("created_at_ms_derived_from_created_at")

        updated_at_ms = coerce_ms(record.get("updated_at_ms"))
        if updated_at_ms is None:
            updated_at_ms = coerce_ms(record.get("updated_at"))
            if updated_at_ms is None:
                warnings.append("missing_updated_at_ms")
            else:
                warnings.append("updated_at_ms_derived_from_updated_at")

        raw_source = record.get("thread_source")
        if raw_source in {"user", "subagent"}:
            thread_source = raw_source
        elif raw_source in {None, ""}:
            thread_source = "unknown"
        else:
            thread_source = "unknown"
            warnings.append("unrecognized_thread_source")

        if not project_root:
            warnings.append("missing_project_root")

        title, title_truncated = compact_text(record.get("title"), MAX_TITLE_CHARS)
        if title_truncated:
            warnings.append("title_truncated")

        threads.append(
            {
                "thread_id": thread_id,
                "title": title,
                "project_root": project_root,
                "rollout_path": path_string(rollout_path),
                "rollout_exists": rollout_exists,
                "created_at_ms": created_at_ms,
                "updated_at_ms": updated_at_ms,
                "archived": normalize_bool(record.get("archived")),
                "thread_source": thread_source,
                "model": record.get("model"),
                "model_provider": record.get("model_provider"),
                "reasoning_effort": record.get("reasoning_effort"),
                "tokens_used": record.get("tokens_used") if record.get("tokens_used") is not None else 0,
                "has_user_event": normalize_bool(record.get("has_user_event")),
                "warnings": warnings,
            }
        )

    threads.sort(key=lambda item: (item.get("updated_at_ms") or 0, item.get("thread_id") or ""), reverse=True)
    stats["rollout_files"] = {
        "checked": rollout_checked,
        "existing": rollout_existing,
        "missing": rollout_missing,
        "outside_allowed_session_dirs": rollout_outside_allowed,
        "existence_rate": (rollout_existing / rollout_checked) if rollout_checked else None,
    }
    return threads, stats


def parse_global_state(sources: IndexSources, index_warnings: list[str]) -> tuple[dict[str, Any], dict[str, Any]]:
    stats = {
        "path": str(sources.global_state_path),
        "role": "ui_state_and_project_hint_source",
        "loaded": False,
        "used_to_override_thread_cwd": False,
        "saved_workspace_roots_count": 0,
        "project_order_count": 0,
        "active_workspace_roots_count": 0,
        "thread_workspace_root_hints_count": 0,
    }
    data, warnings = load_json(sources.global_state_path)
    index_warnings.extend(warnings)
    if not isinstance(data, dict):
        return {"saved_roots": [], "project_order": [], "active_roots": []}, stats

    saved_roots = data.get("electron-saved-workspace-roots")
    project_order = data.get("project-order")
    active_roots = data.get("active-workspace-roots")
    root_hints = data.get("thread-workspace-root-hints")

    if not isinstance(saved_roots, list):
        saved_roots = []
        index_warnings.append("global_state_saved_workspace_roots_unavailable")
    if not isinstance(project_order, list):
        project_order = []
        index_warnings.append("global_state_project_order_unavailable")
    if not isinstance(active_roots, list):
        active_roots = []
        index_warnings.append("global_state_active_workspace_roots_unavailable")

    stats.update(
        {
            "loaded": True,
            "saved_workspace_roots_count": len(saved_roots),
            "project_order_count": len(project_order),
            "active_workspace_roots_count": len(active_roots),
            "thread_workspace_root_hints_count": len(root_hints) if isinstance(root_hints, dict) else 0,
        }
    )
    return {
        "saved_roots": [item for item in saved_roots if isinstance(item, str)],
        "project_order": [item for item in project_order if isinstance(item, str)],
        "active_roots": [item for item in active_roots if isinstance(item, str)],
    }, stats


def build_projects(threads: list[dict[str, Any]], global_state: dict[str, Any]) -> list[dict[str, Any]]:
    saved_roots = set(global_state.get("saved_roots", []))
    active_roots = set(global_state.get("active_roots", []))
    order_map = {
        root: index
        for index, root in enumerate(global_state.get("project_order", []))
        if isinstance(root, str)
    }

    projects: dict[str, dict[str, Any]] = {}
    for thread in threads:
        root = thread.get("project_root")
        if not isinstance(root, str) or not root:
            root = "__missing_project_root__"
        project = projects.setdefault(
            root,
            {
                "project_root": root,
                "thread_count": 0,
                "active_thread_count": 0,
                "archived_thread_count": 0,
                "latest_updated_at_ms": None,
                "from_saved_workspace_roots": root in saved_roots,
                "active_hint": root in active_roots,
                "order_hint": order_map.get(root),
                "warnings": [],
            },
        )
        project["thread_count"] += 1
        if thread.get("archived"):
            project["archived_thread_count"] += 1
        else:
            project["active_thread_count"] += 1
        updated_at_ms = thread.get("updated_at_ms")
        if isinstance(updated_at_ms, int):
            current = project.get("latest_updated_at_ms")
            project["latest_updated_at_ms"] = max(current or updated_at_ms, updated_at_ms)

    for root in saved_roots:
        projects.setdefault(
            root,
            {
                "project_root": root,
                "thread_count": 0,
                "active_thread_count": 0,
                "archived_thread_count": 0,
                "latest_updated_at_ms": None,
                "from_saved_workspace_roots": True,
                "active_hint": root in active_roots,
                "order_hint": order_map.get(root),
                "warnings": ["no_threads_in_sqlite"],
            },
        )

    for project in projects.values():
        root = project["project_root"]
        project["from_saved_workspace_roots"] = root in saved_roots
        project["active_hint"] = root in active_roots
        project["order_hint"] = order_map.get(root)

    return sorted(
        projects.values(),
        key=lambda item: (
            item.get("order_hint") is None,
            item.get("order_hint") if item.get("order_hint") is not None else 999999,
            -(item.get("latest_updated_at_ms") or 0),
            item.get("project_root") or "",
        ),
    )


def extract_skill_text(skill_path: Path) -> tuple[str, str, list[str]]:
    warnings: list[str] = []
    title = skill_path.parent.name
    description = ""
    try:
        with skill_path.open("r", encoding="utf-8") as handle:
            for index, line in enumerate(handle):
                text = line.strip()
                if not title and text.startswith("# "):
                    title = text[2:].strip()
                elif text.startswith("# ") and title == skill_path.parent.name:
                    title = text[2:].strip()
                if text.lower().startswith("description:") and not description:
                    description = text.split(":", 1)[1].strip().strip('"')
                if index >= 80 and title and description:
                    break
    except UnicodeDecodeError:
        warnings.append("skill_read_decode_failed")
    except OSError as exc:
        warnings.append(f"skill_read_failed:{exc.__class__.__name__}")
    return title, description, warnings


def scan_local_skills(sources: IndexSources, index_warnings: list[str]) -> tuple[list[dict[str, Any]], dict[str, Any]]:
    skills: list[dict[str, Any]] = []
    stats = {"path": str(sources.skills_dir), "role": "local_skill_source", "skill_count": 0}

    if not sources.skills_dir.exists():
        index_warnings.append(f"missing_skills_dir:{sources.skills_dir}")
        return skills, stats

    for skill_path in sorted(sources.skills_dir.rglob("SKILL.md")):
        if not is_relative_to(skill_path, sources.skills_dir):
            continue
        rel = skill_path.relative_to(sources.skills_dir)
        source_type = "system" if rel.parts and rel.parts[0] == ".system" else "user"
        title, description, warnings = extract_skill_text(skill_path)
        skill_id = "/".join(rel.parts[:-1])
        skills.append(
            {
                "skill_id": skill_id,
                "source_type": source_type,
                "title": title,
                "description": description,
                "path": str(skill_path),
                "plugin_name": None,
                "plugin_version": None,
                "warnings": warnings,
            }
        )

    stats["skill_count"] = len(skills)
    return skills, stats


def plugin_identity(
    manifest_path: Path, manifest: dict[str, Any] | None, sources: IndexSources
) -> tuple[str, str | None, Path]:
    plugin_root = manifest_path.parent.parent
    rel_parts = manifest_path.relative_to(sources.plugin_cache_dir).parts
    fallback_name = rel_parts[1] if len(rel_parts) > 1 else plugin_root.name
    fallback_version = rel_parts[2] if len(rel_parts) > 2 else None
    name = fallback_name
    version = fallback_version
    if manifest:
        for key in ("name", "id", "displayName", "title"):
            value = manifest.get(key)
            if isinstance(value, str) and value:
                name = value
                break
        value = manifest.get("version")
        if isinstance(value, str) and value:
            version = value
    return name, version, plugin_root


def has_manifest_section(manifest: dict[str, Any] | None, names: set[str]) -> bool:
    if not manifest:
        return False
    for name in names:
        value = manifest.get(name)
        if isinstance(value, dict) and value:
            return True
        if isinstance(value, list) and value:
            return True
    return False


def scan_plugins(
    sources: IndexSources, index_warnings: list[str]
) -> tuple[list[dict[str, Any]], list[dict[str, Any]], dict[str, Any]]:
    plugins: list[dict[str, Any]] = []
    plugin_skills: list[dict[str, Any]] = []
    stats = {
        "path": str(sources.plugin_cache_dir),
        "role": "plugin_manifest_and_skill_source",
        "manifest_count": 0,
        "plugin_skill_count": 0,
    }

    if not sources.plugin_cache_dir.exists():
        index_warnings.append(f"missing_plugin_cache_dir:{sources.plugin_cache_dir}")
        return plugins, plugin_skills, stats

    for manifest_path in sorted(sources.plugin_cache_dir.rglob(".codex-plugin/plugin.json")):
        if not is_relative_to(manifest_path, sources.plugin_cache_dir):
            continue
        manifest_raw, warnings = load_json(manifest_path)
        index_warnings.extend(warnings)
        manifest = manifest_raw if isinstance(manifest_raw, dict) else None
        plugin_name, plugin_version, plugin_root = plugin_identity(manifest_path, manifest, sources)

        description = ""
        homepage = None
        if manifest:
            description_value = manifest.get("description")
            homepage_value = manifest.get("homepage") or manifest.get("repository")
            if isinstance(description_value, str):
                description = description_value
            if isinstance(homepage_value, str):
                homepage = homepage_value

        skill_paths: list[str] = []
        skills_dir = plugin_root / "skills"
        if skills_dir.exists():
            for skill_path in sorted(skills_dir.glob("*/SKILL.md")):
                title, skill_description, skill_warnings = extract_skill_text(skill_path)
                skill_paths.append(str(skill_path))
                plugin_skills.append(
                    {
                        "skill_id": f"{plugin_name}:{skill_path.parent.name}",
                        "source_type": "plugin",
                        "title": title,
                        "description": skill_description,
                        "path": str(skill_path),
                        "plugin_name": plugin_name,
                        "plugin_version": plugin_version,
                        "warnings": skill_warnings,
                    }
                )

        plugins.append(
            {
                "plugin_name": plugin_name,
                "plugin_version": plugin_version,
                "manifest_path": str(manifest_path),
                "description": description,
                "homepage": homepage,
                "has_mcp_servers": has_manifest_section(manifest, {"mcpServers", "mcp_servers", "servers"}),
                "has_apps": has_manifest_section(manifest, {"apps", "applications"}),
                "skill_paths": skill_paths,
                "warnings": [] if manifest else ["manifest_unreadable_or_invalid"],
            }
        )

    stats["manifest_count"] = len(plugins)
    stats["plugin_skill_count"] = len(plugin_skills)
    return plugins, plugin_skills, stats


def scan_memories(sources: IndexSources, index_warnings: list[str]) -> tuple[list[dict[str, Any]], dict[str, Any]]:
    memories: list[dict[str, Any]] = []
    stats = {"path": str(sources.memories_dir), "role": "metadata_only_reference_source", "memory_count": 0}

    if not sources.memories_dir.exists():
        index_warnings.append(f"missing_memories_dir:{sources.memories_dir}")
        return memories, stats

    candidates: list[tuple[Path, str]] = [
        (sources.memories_dir / "MEMORY.md", "memory"),
        (sources.memories_dir / "memory_summary.md", "summary"),
        (sources.memories_dir / "raw_memories.md", "raw"),
    ]
    rollout_dir = sources.memories_dir / "rollout_summaries"
    if rollout_dir.exists():
        candidates.extend((path, "rollout_summary") for path in sorted(rollout_dir.glob("*.md")))
    omx_logs_dir = sources.memories_dir / ".omx" / "logs"
    if omx_logs_dir.exists():
        candidates.extend((path, "omx_log") for path in sorted(omx_logs_dir.glob("*.jsonl")))

    for path, kind in candidates:
        if not is_relative_to(path, sources.memories_dir):
            continue
        if not path.exists():
            continue
        memories.append(
            {
                "memory_path": str(path),
                "kind": kind,
                "line_count": count_lines(path),
                "updated_at_ms": safe_stat_mtime_ms(path),
                "confidence": "low",
                "warnings": [],
            }
        )

    stats["memory_count"] = len(memories)
    return memories, stats


def parse_session_index(
    sources: IndexSources, sqlite_thread_ids: set[str], index_warnings: list[str]
) -> dict[str, Any]:
    stats = {
        "path": str(sources.session_index_path),
        "role": "auxiliary_thread_list",
        "loaded": False,
        "line_count": 0,
        "parsed_count": 0,
        "unique_thread_ids": 0,
        "ids_not_in_sqlite_count": 0,
        "sqlite_ids_not_in_session_index_count": len(sqlite_thread_ids),
    }

    if not sources.session_index_path.exists():
        index_warnings.append(f"missing_session_index:{sources.session_index_path}")
        return stats

    ids: set[str] = set()
    try:
        with sources.session_index_path.open("r", encoding="utf-8") as handle:
            for line_number, line in enumerate(handle, start=1):
                stats["line_count"] += 1
                text = line.strip()
                if not text:
                    continue
                try:
                    item = json.loads(text)
                except json.JSONDecodeError:
                    index_warnings.append(f"session_index_invalid_json_line:{line_number}")
                    continue
                stats["parsed_count"] += 1
                thread_id = item.get("id") if isinstance(item, dict) else None
                if isinstance(thread_id, str):
                    ids.add(thread_id)
    except OSError as exc:
        index_warnings.append(f"session_index_read_failed:{exc.__class__.__name__}")
        return stats

    stats["loaded"] = True
    stats["unique_thread_ids"] = len(ids)
    stats["ids_not_in_sqlite_count"] = len(ids - sqlite_thread_ids)
    stats["sqlite_ids_not_in_session_index_count"] = len(sqlite_thread_ids - ids)
    return stats


def build_index(
    sources: IndexSources | None = None,
    *,
    expected_thread_count: int | None = None,
) -> dict[str, Any]:
    sources = sources or IndexSources.from_codex_home(DEFAULT_CODEX_HOME)
    warnings: list[str] = []

    threads, sqlite_stats = fetch_threads(sources, warnings)
    global_state, global_state_stats = parse_global_state(sources, warnings)
    projects = build_projects(threads, global_state)
    project_context_stats = attach_project_context(projects)
    local_skills, local_skill_stats = scan_local_skills(sources, warnings)
    plugins, plugin_skills, plugin_stats = scan_plugins(sources, warnings)
    memories, memory_stats = scan_memories(sources, warnings)
    session_index_stats = parse_session_index(
        sources,
        {thread["thread_id"] for thread in threads if isinstance(thread.get("thread_id"), str)},
        warnings,
    )

    skills = sorted(
        local_skills + plugin_skills,
        key=lambda item: (item.get("source_type") or "", item.get("plugin_name") or "", item.get("skill_id") or ""),
    )

    if expected_thread_count is not None and sqlite_stats.get("thread_count") != expected_thread_count:
        warnings.append(f"sqlite_thread_count_differs_from_expected:{sqlite_stats.get('thread_count')}")

    source_stats = {
        "codex_home": {
            "path": str(sources.codex_home),
            "role": "data_source_root",
        },
        "sqlite": sqlite_stats,
        "session_index": session_index_stats,
        "global_state": global_state_stats,
        "project_context": project_context_stats,
        "skills": {
            **local_skill_stats,
            "plugin_skill_count": plugin_stats["plugin_skill_count"],
            "total_skill_count": len(skills),
        },
        "plugins": plugin_stats,
        "memories": memory_stats,
    }

    return {
        "generated_at": utc_now_iso(),
        "warnings": warnings,
        "threads": threads,
        "projects": projects,
        "skills": skills,
        "plugins": plugins,
        "memories": memories,
        "source_stats": source_stats,
    }


def collect_warnings(index: dict[str, Any]) -> list[str]:
    warnings: list[str] = []
    root_warnings = index.get("warnings", [])
    if isinstance(root_warnings, list):
        warnings.extend(str(item) for item in root_warnings)

    for collection_name in ("threads", "projects", "skills", "plugins", "memories"):
        collection = index.get(collection_name, [])
        if not isinstance(collection, list):
            continue
        for item in collection:
            if not isinstance(item, dict):
                continue
            item_warnings = item.get("warnings", [])
            if isinstance(item_warnings, list):
                warnings.extend(str(warning) for warning in item_warnings)
            context_warnings = item.get("context_warnings", [])
            if isinstance(context_warnings, list):
                warnings.extend(str(warning) for warning in context_warnings)
            harness_resources = item.get("harness_resources", [])
            if isinstance(harness_resources, list):
                for resource in harness_resources:
                    if not isinstance(resource, dict):
                        continue
                    resource_warnings = resource.get("warnings", [])
                    if isinstance(resource_warnings, list):
                        warnings.extend(str(warning) for warning in resource_warnings)
    return warnings


def warning_matches(actual: str, expected: str) -> bool:
    return actual == expected or actual.startswith(f"{expected}:")


def warning_summary(index: dict[str, Any]) -> dict[str, int]:
    summary: dict[str, int] = {}
    for warning in collect_warnings(index):
        key = warning.split(":", 1)[0]
        summary[key] = summary.get(key, 0) + 1
    return dict(sorted(summary.items()))


def validate_warning_semantics(
    index: dict[str, Any],
    *,
    required_warnings: list[str] | None = None,
    forbidden_warnings: list[str] | None = None,
) -> list[str]:
    problems: list[str] = []
    all_warnings = collect_warnings(index)
    for required in required_warnings or []:
        if not any(warning_matches(actual, required) for actual in all_warnings):
            problems.append(f"missing_required_warning:{required}")
    for forbidden in forbidden_warnings or []:
        if any(warning_matches(actual, forbidden) for actual in all_warnings):
            problems.append(f"forbidden_warning_present:{forbidden}")
    return problems


def validate_index(
    index: dict[str, Any],
    *,
    required_warnings: list[str] | None = None,
    forbidden_warnings: list[str] | None = None,
) -> list[str]:
    problems: list[str] = []
    for key in ["generated_at", "warnings", "threads", "projects", "skills", "plugins", "memories", "source_stats"]:
        if key not in index:
            problems.append(f"missing_top_level_key:{key}")

    for idx, thread in enumerate(index.get("threads", [])):
        if not isinstance(thread, dict):
            problems.append(f"thread_not_object:{idx}")
            continue
        missing = REQUIRED_THREAD_OUTPUT_KEYS - set(thread.keys())
        if missing:
            problems.append(f"thread_missing_keys:{idx}:{','.join(sorted(missing))}")

    for idx, project in enumerate(index.get("projects", [])):
        if not isinstance(project, dict):
            problems.append(f"project_not_object:{idx}")
            continue
        missing = REQUIRED_PROJECT_OUTPUT_KEYS - set(project.keys())
        if missing:
            problems.append(f"project_missing_keys:{idx}:{','.join(sorted(missing))}")

    source_stats = index.get("source_stats", {})
    if isinstance(source_stats, dict):
        sqlite_stats = source_stats.get("sqlite", {})
        if isinstance(sqlite_stats, dict) and not sqlite_stats.get("opened_readonly"):
            problems.append("sqlite_not_marked_readonly")
        session_index = source_stats.get("session_index", {})
        if isinstance(session_index, dict) and session_index.get("role") != "auxiliary_thread_list":
            problems.append("session_index_not_marked_auxiliary")
        global_state = source_stats.get("global_state", {})
        if isinstance(global_state, dict) and global_state.get("used_to_override_thread_cwd"):
            problems.append("global_state_overrode_thread_cwd")

    problems.extend(
        validate_warning_semantics(
            index,
            required_warnings=required_warnings,
            forbidden_warnings=forbidden_warnings,
        )
    )
    return problems


def write_index(index: dict[str, Any], output_path: Path, pretty: bool) -> None:
    output_path.parent.mkdir(parents=True, exist_ok=True)
    with output_path.open("w", encoding="utf-8") as handle:
        json.dump(index, handle, ensure_ascii=False, indent=2 if pretty else None, sort_keys=True)
        handle.write("\n")


def load_index(path: Path) -> dict[str, Any]:
    with path.open("r", encoding="utf-8") as handle:
        data = json.load(handle)
    if not isinstance(data, dict):
        raise ValueError("index root is not an object")
    return data


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description="Build a read-only Codex local index.")
    parser.add_argument("--codex-home", type=Path, default=DEFAULT_CODEX_HOME, help="Codex data root to read.")
    parser.add_argument("--output", type=Path, default=DEFAULT_OUTPUT)
    parser.add_argument("--pretty", action="store_true")
    parser.add_argument("--check", type=Path, help="Validate an existing index file and exit.")
    parser.add_argument(
        "--expect-thread-count",
        type=int,
        help="Enable real-environment regression warning when SQLite thread count differs.",
    )
    parser.add_argument(
        "--require-warning",
        action="append",
        default=[],
        help="With --check, require a warning code or exact warning value. Can be repeated.",
    )
    parser.add_argument(
        "--forbid-warning",
        action="append",
        default=[],
        help="With --check, fail if a warning code or exact warning value is present. Can be repeated.",
    )
    parser.add_argument(
        "--warning-summary",
        action="store_true",
        help="With --check, print warning counts grouped by warning code.",
    )
    args = parser.parse_args(argv)

    if args.check:
        try:
            index = load_index(args.check)
        except (OSError, json.JSONDecodeError, ValueError) as exc:
            print(f"validation_failed:cannot_load_index:{exc}", file=sys.stderr)
            return 2
        problems = validate_index(
            index,
            required_warnings=args.require_warning,
            forbidden_warnings=args.forbid_warning,
        )
        if problems:
            for problem in problems:
                print(problem, file=sys.stderr)
            return 1
        if args.warning_summary:
            print(json.dumps(warning_summary(index), ensure_ascii=False, sort_keys=True))
            return 0
        print("validation_ok")
        return 0

    sources = IndexSources.from_codex_home(args.codex_home)
    index = build_index(sources, expected_thread_count=args.expect_thread_count)
    problems = validate_index(index)
    if problems:
        for problem in problems:
            print(problem, file=sys.stderr)
        return 1
    write_index(index, args.output, args.pretty)
    stats = index["source_stats"]
    print(
        json.dumps(
            {
                "output": str(args.output),
                "thread_count": stats["sqlite"]["thread_count"],
                "project_count": len(index["projects"]),
                "skill_count": len(index["skills"]),
                "plugin_count": len(index["plugins"]),
                "memory_count": len(index["memories"]),
                "warning_count": len(index["warnings"]),
                "rollout_existing": stats["sqlite"]["rollout_files"]["existing"],
                "rollout_checked": stats["sqlite"]["rollout_files"]["checked"],
            },
            ensure_ascii=False,
            sort_keys=True,
        )
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
