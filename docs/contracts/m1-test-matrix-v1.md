# SYN-FND-001-R1 M1 static test matrix

| ID | Requirement | Mechanical evidence | Expected |
|---|---|---|---|
| M1-C01 | Ten versioned contracts exist | manifest path set and file parse | PASS |
| M1-C02 | Owner is unique per contract | manifest owner uniqueness | PASS |
| M1-C03 | Exported type has one owner | global export uniqueness | PASS |
| M1-C04 | Owner dependency graph is acyclic | topological traversal plus negative fixtures | PASS |
| M1-C05 | Formal actions name six facets | action flow schema: command/policy/state/event/audit/outbox | PASS |
| M1-C06 | Failure closes safely | every flow and contract failure mode is fail-closed | PASS |
| M1-C07 | Sensitive values are excluded | contract deny lists plus nested forbidden-field fixtures | PASS |
| M1-C08 | Every contract has positive and negative fixtures | rule polarity plus required regression fixture coverage | 10/10 contracts |
| M1-I01 | Tauri inventory is exact | parse opening `generate_handler!` and compare ordered set | 171/171 |
| M1-I02 | Supervisor MCP inventory is exact | parse opening capability registry and compare ordered set | 8/8 |
| M1-I03 | Runner/background inventory is routed | source symbol, owner, scope, policy, bypass, migration fields | PASS |
| M1-I04 | Opening evidence is immutable | Git blob and SHA-256 for all 30 fixed-base source files | PASS |
| M1-S01 | SQLite table set is explicit | schema source extraction and inventory set equality | 68/68 |
| M1-S02 | Sidecars have owners, joins, and source closure | fixed-base importer/registry derivation plus storage inventory required fields | 18/18 |
| M1-S03 | Projections have owners, joins, and source closure | exact taxonomy-to-source declaration anchors plus storage inventory required fields | 7/7 |
| M1-S04 | Every storage record has traceable entries and fail-closed dispositions | resolved SOURCE/TAURI/MCP/HOLD refs plus unknown/corrupt/sensitive enum checks | 93/93 |
| M1-M01 | Legacy migration items are routed | unique ID, disposition, target owner/port, next stage or HOLD | PASS |
| M1-H01 | Every HOLD has a later owner | HOLD registry and contract references | PASS |
| M1-H02 | UNKNOWN never masquerades as migrated | inventory negative fixtures | PASS |
| M1-M2-01 | M2 minimum interfaces are complete | eight interface names and required fields | PASS |
| M1-M2-02 | Semantic parity is multi-dimensional | parity dimension set | PASS |
| M1-M2-03 | Rollback is non-destructive | rollback guard set and negative fixtures | PASS |
| M1-M2-04 | M1 does not freeze persistence tuning | forbidden premature-decision scan | PASS |
| M1-G01 | Product tree is unchanged | package required Git command | PASS |

The matrix is static M1 acceptance only. It does not establish runtime, desktop, database,
connector, deployment, or release acceptance.
