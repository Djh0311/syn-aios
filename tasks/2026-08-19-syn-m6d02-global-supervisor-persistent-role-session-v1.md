# Grok task package: M6D02 Global Supervisor persistent RoleSession

You are the preferred implementation writer for current Syn Harness leaf `M6D02-top-level-global-supervisor-role-session.md`. Work in `/home/synadmin/workspace/syn`. One writer only; do not spawn subagents. Do not commit, stage, reset, stash, clean, push, rebase, merge, tag, deploy, or release. Codex will independently review, repair if needed, validate in a detached checkout, and commit.

## Read first

1. `AGENTS.md`
2. `docs/harness/leaves/M6D02-top-level-global-supervisor-role-session.md`
3. `docs/contracts/m6-cross-project-and-organization-v1.md`
4. `docs/plans/2026-08-01-syn-stage-6-global-supervisor-and-internal-organization-plan-v1.md` §§2–5
5. `m3_role_session.rs`, `m3_role_session_repository.rs`, `m3_role_session_schema.rs`
6. `m4_secretary_domain.rs` only as the accepted pattern for installing/restoring a server-fixed identity through the M3 repository
7. `lib.rs`, `commands.rs`, `command_registry.rs` for ordinary AppState and real Tauri registration

Do not read or use the protected untracked `m6_*.rs`, `.bak`, or `gen/schemas/linux-schema.json`; they are not implementation inputs and must remain byte-identical/untracked.

## Exact product write scope

You may create/edit only:

- create `prototypes/productized-desktop-shell/src-tauri/src/m6_org_global_role_session.rs`
- create `m6_org_dto.rs`, `m6_org_schema.rs`, or `m6_org_store.rs` only if genuinely required; do not create a parallel RoleSession store/schema or in-memory truth
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`: one module declaration, ordinary AppState slot/install wiring, and required test literals only
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`: one narrow read/status command plus directly related tests
- `prototypes/productized-desktop-shell/src-tauri/src/command_registry.rs`: only one new handler-list entry
- `lib_read_model_boundary_tests.rs`, `m1_project_index.rs`, and `m3_project_role_session_authority.rs`: Codex reconciled these after compilation as necessary adjacent test assembly paths; only add `m6_org_global_role_session: Default::default()` to existing `AppState` literals
- `m3_role_session.rs`, `m3_role_session_repository.rs`, `m3_role_session_schema.rs`: only an unavoidable `pub(crate)` visibility change or new trait implementation; explain every such change and do not alter M3 semantics/fields

Do not edit Harness, contracts, plans, tasks, frontend, Cargo manifests, any other source, or any protected WIP.

## Required implementation

1. Build one server-fixed Global Supervisor RoleSession on the existing M3 `RoleSession` aggregate and `M3RoleSessionSqliteRepository`. Reuse the exact repository installed by ordinary M4/M3 product composition; do not create another SQLite RoleSession truth or a process-memory registry.
2. Use explicit canonical server-owned actor/role/global-scope/current-object/execution-channel/permission refs. Identity must not derive from cwd, project path, display/session name, provider/model/thread/process, renderer input, env gate, or fixture.
3. Startup must create once or restore exactly the same M3 session. On restart, ambiguous/mismatched/quarantined/closed/missing-after-established/corrupt state fails closed. Do not silently pick newest, guess, default, auto-import, or rebuild from provider/path. An idempotent M3 create receipt may be the establishment witness if it mechanically makes a deleted session fail rather than recreate.
4. Default permission is read-only/global: the M6 runtime exposes no project write capability, provider handle is never authority, and an explicit attempted project-write authorization path returns a stable fail-closed error before any mutation.
5. Context shape may contain only minimal summary refs and source refs. It must have no raw file, raw summary, transcript, secret, untrimmed memory, provider response, prompt, stdout/stderr, or tool-output field/body.
6. Project Supervisor and Secretary bindings must fail exact global-binding validation; discriminate by explicit canonical role/scope fields, never a display name.
7. Install the runtime only in the ordinary AppState product profile. Historical legacy and current isolated-uninstalled profiles must remain unavailable; do not use an env/acceptance gate to claim the ordinary consumer.
8. Add one host-fixed Tauri read/status command. Renderer input must not carry actor/role/scope/path/provider/permission/session identity claims. Define it in `commands.rs`, add exactly one `generate_handler!` entry in `command_registry.rs`, and route through the ordinary AppState slot to the new module.
9. Status/read DTO must make the evidence limit honest: stable role_session_id/revision/state, `scope_kind=GLOBAL`, `read_only=true`, `project_write_capability=false`, `provider_handle_authorizes=false`, minimal ref container, and a stable unavailable/fail-closed error. Do not claim cross-project query/advisory/UI.
10. Add `m6d02_` tests covering at least: M3 persistence round trip; drop/reopen restart same identity; explicit project/Secretary scope rejection; read-only/project-write rejection with zero mutation; minimal context shape; missing/corrupt established source fail-closed; ordinary AppState consumer; command registry entry. Tests may use only scratch temp roots and deterministic fake refs, never real provider/account/data.

Prefer one cohesive new module and zero M3 visibility/semantic changes. If the accepted M4 installation already exposes a cloneable M3 repository, clone that exact handle before the installation is moved into M4 conversation composition.

## Validation and return

Run only offline/local checks:

- format only touched Rust files with repository rustfmt 1.9.0; do not format the whole tree
- `cargo test --lib m6d02_ --offline`
- `cargo check --lib --offline`
- `git diff --check`
- `git status --short` and list only files you changed; do not touch unrelated runtime/Harness WIP

Return: files changed; complete ordinary call chain; persistence and fail-closed design; exact tests/pass counts/exits; every M3 visibility change (or explicitly none); any blocker. Do not claim cross-project query, advisory, GUI, real provider, deployment, or release.
