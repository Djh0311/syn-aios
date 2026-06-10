# Codex Native Conversation Diagnostic

Status: repair helper for `2026-06-03-codex-native-app-conversation-list-repair-v1`.

These helpers repair Codex native app conversation list metadata in small
steps:

1. `repair-global-state-hints.mjs` repairs project/thread sidebar metadata in
   `.codex-global-state.json`.
2. `repair-session-index.mjs` deduplicates and fills missing non-archived
   entries in `session_index.jsonl` from sqlite thread metadata.
   Newly generated `thread_name` values are whitespace-normalized and capped at
   36 characters.
3. `promote-saved-projects-in-session-index.mjs` promotes one representative
   thread per saved workspace into the recent `session_index.jsonl` window. This
   is a display workaround for the native sidebar/list API when older saved
   projects fall outside the app's recent-thread window.
4. `promote-saved-projects-in-state-sqlite.mjs` promotes one representative
   thread per saved workspace into the sqlite `threads.updated_at_ms` recent
   window. Use this only after confirming the native app reads sqlite
   `listThreads(sortKey: "updated_at")` instead of `session_index.jsonl`.

It does not read rollout bodies, does not write rollout JSONL, and does not
touch auth/token/config files. The sqlite promotion helper writes
`state_5.sqlite` only when run with `--apply`.

## Dry Run

```bash
node tools/codex-native-conversation-diagnostic/repair-global-state-hints.mjs --dry-run
node tools/codex-native-conversation-diagnostic/repair-session-index.mjs --dry-run
node tools/codex-native-conversation-diagnostic/promote-saved-projects-in-session-index.mjs --dry-run
node tools/codex-native-conversation-diagnostic/promote-saved-projects-in-state-sqlite.mjs --dry-run
```

## Apply

Close the Codex native app first. If Codex is running, it can overwrite
`.codex-global-state.json` from its in-memory state.

```bash
node tools/codex-native-conversation-diagnostic/repair-global-state-hints.mjs \
  --apply \
  --confirm "repair codex native app conversation list global state"

node tools/codex-native-conversation-diagnostic/repair-session-index.mjs \
  --apply \
  --confirm "repair codex native app conversation list session index"

node tools/codex-native-conversation-diagnostic/promote-saved-projects-in-session-index.mjs \
  --apply \
  --confirm "promote codex native app saved project representatives"

node tools/codex-native-conversation-diagnostic/promote-saved-projects-in-state-sqlite.mjs \
  --apply \
  --confirm "promote codex native app sqlite thread window"
```

The scripts write backups before mutation:

```text
/Users/yoyi/.codex/backups_state/native-conversation-list-repair/<timestamp>/.codex-global-state.json.before
/Users/yoyi/.codex/backups_state/native-conversation-list-repair/<timestamp>/session_index.jsonl.before
/Users/yoyi/.codex/backups_state/native-conversation-list-repair/<timestamp>/session_index.jsonl.before-promote
/Users/yoyi/.codex/backups_state/native-conversation-list-repair/<timestamp>/state_5.sqlite.before-promote
```
