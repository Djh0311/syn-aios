---
name: finishing-a-development-branch
description: Use when implementation is verified and the user wants merge, PR, branch cleanup, worktree cleanup, or discard decisions; all risky git actions require explicit confirmation
---

# Finishing A Development Branch

## Purpose

Finish verified development work without accidentally merging, pushing, deleting, committing, or discarding changes. This skill presents safe integration options and waits for explicit user confirmation before risky Git operations.

## When To Use

Use this skill when:

- implementation work is complete enough to discuss integration
- the user asks about merging, PR creation, branch cleanup, worktree cleanup, or discarding work
- a plan or multi-agent workflow reaches its final integration decision

Do not use it to claim implementation is complete. Use `verification-before-completion` first.

## Core Rules

- Verify current status before presenting integration options.
- Do not run `git add`, `git commit`, merge, push, branch delete, worktree remove, reset, or discard actions without explicit user confirmation.
- Treat discard, force delete, reset, and cleanup as destructive unless proven otherwise.
- If this directory is not a Git repo or worktrees are unavailable, report the limitation and use the nearest safe file-based summary.
- If tests or required checks have not passed, present the real status and do not recommend merge/PR as ready.

## Workflow

1. **Confirm status**
   - Read the current task goal, changed files, verification evidence, and residual risks.
   - Run or inspect the relevant fresh checks required by `verification-before-completion`.
   - Check whether the worktree has uncommitted, untracked, or user-owned changes.

2. **Inspect Git context read-only**
   - Identify current branch, base branch if knowable, upstream status, worktree path, and whether this is a Git worktree.
   - If the base branch is ambiguous, ask before proposing a merge target as fact.
   - Do not stage or commit during this inspection.

3. **Present options**
   - Summarize verification status and known risks first.
   - Offer only options that are safe for the actual state.
   - Typical options:
     - keep current branch/worktree as-is
     - prepare a PR after user confirms push/PR actions
     - merge locally after user confirms base branch and merge action
     - clean up worktree/branch after user confirms exact paths/names
     - discard work only after typed destructive confirmation

4. **Wait for confirmation**
   - Ask for the user's chosen option.
   - For merge, push, PR creation, branch deletion, worktree removal, or discard, name the exact command class and target before acting.
   - For destructive actions, require explicit typed confirmation such as the branch/path name or `discard`.

5. **Execute only the confirmed action**
   - Keep actions scoped to the confirmed option.
   - Re-run required verification after merge or other integration changes.
   - If a command fails or produces conflicts, stop and report the exact state.

6. **Report final state**
   - State what was done, what was verified after the action, what remains unverified, and any branch/worktree cleanup still pending.
   - Do not claim merge/PR/release readiness without fresh evidence.

## Stop Conditions

Stop and ask when:

- Git repository, branch, upstream, or base branch cannot be determined safely
- there are unrelated or user-owned changes in the worktree
- required verification failed or did not run
- merge would create conflicts
- push/PR requires authentication or remote decisions
- cleanup target is ambiguous
- the user asks to discard, delete, reset, or force-update anything
- the action would touch paths outside declared write scope

## Reporting Template

```markdown
Branch finish status:
- Current branch:
- Base branch:
- Changed files:
- Verification:
- Risks:
- Recommended options:
- Needs confirmation:
```

## Related Skills

- `verification-before-completion` before readiness or completion claims.
- `using-git-worktrees` when the work happened in an isolated worktree.
- `receiving-code-review` when final review feedback changes scope or code.
