# High-Risk Boundary

Use this component only when the next concrete action may be destructive,
difficult to reverse, external, shared, costly, privacy-sensitive, or capable of
affecting a real device or person.

Examples include:

- production, remote, or shared writes;
- deployment, upload, publication, or remote configuration changes;
- destructive local cleanup or broad replacement;
- use, rotation, or transmission of real credentials;
- paid provider calls with material cost;
- payment, notification, MQTT, device, lock, or physical-world actions.

Ordinary local work and normal read-only inspection do not become high-risk
merely because the surrounding project is important. Sensitive reads still use
the smallest necessary scope and redacted output.

## Decision Rule

The model evaluates the actual next action:

1. What state can it change?
2. Who or what can be affected?
3. Can it be safely and completely reversed?
4. Is the exact target known?
5. Can a duplicate action cause harm?

If the action crosses the boundary, obtain fresh confirmation for the exact
target and effect. Generic project approval is not production-action approval.

## Retry Rule

Preflight, validation, dry-run, and read-only reconciliation are repeatable and
do not consume execution authority.

One-shot or no-automatic-retry behavior begins only after the action could have
changed external state. If the result is ambiguous, do not repeat it. Inspect the
authoritative post-state or ask the user how to proceed.

## Evidence

Capture only enough redacted post-state to establish what happened. Do not store
credentials, raw private data, unrestricted command output, or unrelated
environment details. A runbook is not evidence that the action succeeded.
