# Skill Authoring Best Practices

Use this reference when a skill needs deeper design work than the main `writing-skills` workflow covers.

## Core Principle

Skills should add operational knowledge the agent would otherwise miss. They should not restate general intelligence, basic coding practice, or project rules already owned by `AGENTS.md`.

Good skills:

- have a clear trigger
- prevent a specific failure mode
- are short enough to load during real work
- point to references only when needed
- include stop conditions
- can be tested with realistic pressure scenarios

## Progressive Disclosure

Put only the routing and execution essentials in `SKILL.md`.

Keep in the main file:

- purpose
- when to use
- required inputs
- workflow
- stop conditions
- verification
- links to related references

Move out of the main file:

- long examples
- historical notes
- API tables
- long rationale
- pressure-test transcripts
- broad background material

Reference files should be one level away from `SKILL.md` and named by the decision they help with.

## Trigger Design

The `description` field is the routing hook. It should let an agent answer: "Should I load this skill now?"

Use descriptions that name:

- the task type
- symptoms or failure modes
- high-risk surfaces
- direct user intent

Avoid:

- vague labels
- broad catch-all triggers
- workflow summaries
- triggers that pull the skill into unrelated work

## Context Budget

Every loaded line competes with task context. A skill should be long only when the added specificity prevents likely mistakes.

Use shorter instructions when:

- the agent already knows the general technique
- the task is low risk
- the rule is already in `AGENTS.md`
- examples are illustrative rather than required

Use stronger or longer instructions when:

- the failure is repeated or costly
- agents routinely rationalize around the rule
- the workflow has strict ordering
- external tools or exact file formats matter

## Specificity Beats Persuasion

Prefer actionable constraints over motivational text.

Better:

```markdown
Before claiming UI completion, open the route in a browser harness, exercise the changed interaction, check console/network status, and report unverified gaps.
```

Weaker:

```markdown
UI verification is very important. Skipping it causes bad outcomes.
```

Use bright-line rules when a failure mode must never happen, but remove repeated persuasion once the stop condition and workflow are clear.

## Reference Files

Reference files are useful when they are:

- loaded only for a specific variant or technique
- directly linked from `SKILL.md`
- easy to scan with headings
- not required for ordinary use

For long references, include a short table of contents or clear section headings. Avoid nested reference chains that require agents to chase several files before acting.

## Scripts And Assets

Bundle scripts when deterministic tooling is better than prompting.

Good scripts:

- validate inputs
- produce actionable errors
- avoid destructive defaults
- document required dependencies
- can be run without reading their whole source

Do not add scripts that merely move uncertainty from the skill into opaque code.

## Testing Skills

Test discipline skills with pressure scenarios that tempt the agent to skip the rule.

Useful pressures:

- time pressure
- sunk cost
- apparent obvious fix
- user urgency
- partial prior success
- fatigue or context loss

Capture the exact failure mode, then update the skill only enough to prevent that failure. Retest after every meaningful change.

## Maintenance

Revise a skill when:

- the trigger fires too often or too rarely
- agents skip a required step
- agents load too much context for simple work
- a repeated mistake should become prevention
- project rules move into `AGENTS.md`
- a reference becomes stale or unused

Remove or split a skill when it becomes a general essay rather than an executable workflow.

## Review Checklist

- [ ] Trigger starts with `Use when`.
- [ ] Failure mode is explicit.
- [ ] Workflow is ordered and executable.
- [ ] Stop conditions are clear.
- [ ] Main file is concise.
- [ ] References are directly linked and optional.
- [ ] Verification method is stated.
- [ ] No automatic commit/stage/destructive instructions are included.
