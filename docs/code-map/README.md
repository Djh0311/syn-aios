# Structured Code Map seed

This is a small, partial navigation map created in Phase 3. It is not a source
of runtime, product, or acceptance truth. Follow each source reference and its
tests before using a capability.

- `index.json` names the six deliberately bounded domains; each domain file is
  independent and records its coverage and verification commit.
- A `canonical` reference is only allowed when its repository-relative source
  path is tracked at `HEAD`. `needs-confirmation` deliberately uses `null` when
  current ownership or runtime selection cannot be proved from tracked code.
- Every `publicSymbols[].symbol` is a bare declared source identifier checked
  with `git show <verifiedAtCommit>:<path>`; a CLI without an appropriate public
  identifier uses an empty `publicSymbols` list instead of a descriptive label.
- `active` means a tracked code capability exists at its verification commit. It
  never grants real execution: the current plan, user authorization, and safety
  gates remain separately required.
- `legacy` entries are discoverable history, not a default route or a request
  to build another implementation. In particular, resident/private-home is not
  a third conversation transport.
- The map never writes source code or derives canonical facts from dirty files.
  `overlay` reports unstaged and untracked paths separately; `check --staged`
  reports staged rename/delete impact.

Use the explicit commands only:

    node scripts/harness-v2/codebase-map.js query --target . --query "conversation transport"
    node scripts/harness-v2/codebase-map.js overlay --target .
    node scripts/harness-v2/codebase-map.js check --target . --staged --strict

The map is intentionally not wired into hooks, config, or the default project
context route. A no-match means `NO_MATCH_IN_PARTIAL_MAP`, never that a
capability is absent from the repository.

`maintenance-audit` is an explicit, read-only drift report. It never consumes
overlay paths, rewrites this map, or enters Hook/CI/cron/default routing; a
finding asks for human review rather than an automatic map update.
