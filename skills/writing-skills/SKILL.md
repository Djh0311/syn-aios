---
name: writing-skills
description: Use when creating, editing, slimming, or verifying skills before deployment
---

# Writing Skills

## Purpose

Skills are reusable operating instructions for agents. They should help future agents choose the right workflow, avoid known failure modes, and apply project-specific techniques without loading unnecessary context.

This skill treats process documentation like code: define the failure it prevents, make the smallest useful change, verify the skill can be followed, then refactor for clarity.

## When To Use

Use this skill for:

- creating a new skill
- editing an existing skill
- slimming or reorganizing a skill
- changing skill triggers, required steps, or stop conditions
- verifying that a skill is ready to be copied into a project

Do not use it for ordinary project conventions. Put project-specific rules in `AGENTS.md` or the relevant project docs.

## Required Inputs

- Target skill path.
- Intended trigger: when should agents read this skill?
- Failure mode: what mistake or repeated burden should this skill prevent?
- Scope: create, edit, slim, split, or verify.
- Verification approach: pressure scenario, checklist review, or real task trial.

## Core Workflow

1. Route the work with `using-superpowers`.
2. Declare read/write scope.
3. Identify the failure mode the skill is meant to prevent.
4. For meaningful behavior changes, create at least one pressure scenario or real task trial before editing.
5. Edit the smallest set of skill files needed.
6. Keep the main `SKILL.md` short and action-oriented.
7. Move heavy examples, long rationale, API references, and historical notes into separate reference files.
8. Verify the final skill against the trigger and failure mode.
9. Report what changed, what was verified, and what remains unverified.

## Skill Structure

Every skill should have:

```markdown
---
name: skill-name
description: Use when [clear trigger and symptoms]
---

# Skill Name

## Purpose

## When To Use

## Workflow

## Stop Conditions

## Verification
```

Add extra sections only when they remove ambiguity. Avoid long persuasion text in the main skill.

## Description Rules

The `description` field is the routing hook. It should answer: "Should I read this skill now?"

Use:

- `Use when...`
- concrete triggers, symptoms, or task types
- third-person or neutral wording
- no workflow summary
- no broad catch-all phrasing

Avoid:

- vague descriptions such as "For testing"
- process summaries such as "writes tests, runs them, fixes code"
- first-person wording
- triggers so broad that the skill loads for unrelated work

## Main File Size

Targets:

- Frequently loaded entry skills: about 100-200 lines.
- Normal workflow skills: about 50-150 lines.
- Heavy references: separate files, linked from `SKILL.md`.

If the skill needs long examples or rationale, put them in a reference file and mention when to read it.

## Verification Options

Use the lightest verification that proves the change:

- Copy/format edits: read-through against this checklist.
- Trigger changes: test several user prompts and confirm the skill would or would not load.
- Discipline or failure-prevention skills: run pressure scenarios that would tempt the agent to skip the rule.
- Technique skills: run a realistic application task.
- Reference skills: test lookup and application of the referenced material.

For meaningful skill behavior changes, verification should show:

- the trigger is clear
- the workflow is executable
- stop conditions are explicit
- the skill does not force unrelated work
- the main file does not carry unnecessary context

## Stop Conditions

Stop and ask or re-route when:

- the skill trigger overlaps heavily with another skill
- the rule belongs in `AGENTS.md` instead of a reusable skill
- the requested change weakens a Universal Gate
- verification would require subagents or tools that are unavailable
- the edit would touch example/archive/reference files outside the declared scope

## Checklist

- [ ] Trigger is specific and starts with `Use when`.
- [ ] Main workflow is short and ordered.
- [ ] Required inputs are explicit.
- [ ] Stop conditions are explicit.
- [ ] Heavy examples or rationale are outside the main file.
- [ ] Related skills are named without forcing unnecessary file loads.
- [ ] Verification method is recorded.
- [ ] No automatic `git add` or `git commit` instructions are included.

## Related References

Read only when needed:

- `skill-authoring-best-practices.md` for long-form skill-authoring guidance.
- `testing-skills-with-subagents.md` for pressure-scenario methodology.
- `persuasion-principles.md` for discipline-skill rationale.
