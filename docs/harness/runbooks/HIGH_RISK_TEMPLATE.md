# High-Risk Action: <short description>

Status: `DRAFT | READY_FOR_CONFIRMATION | AUTHORIZED | EXECUTED | RECONCILE | CLOSED`

## Exact Action

- Target:
- Intended state change:
- Affected people/system:
- Why this is necessary:

## Preconditions

- Current authority:
- Identity and target verification:
- Backup, rollback, or safe-close condition:
- Required redaction:

Preflight is repeatable and does not execute or consume the action.

## Explicit Confirmation

Ask immediately before the external side effect:

> Confirm execution of `<exact action>` against `<exact target>` with
> `<expected effect>`?

Record the confirmation context without copying secrets or private payloads.

## Execution Boundary

- Last repeatable preflight:
- First operation that can change external state:
- Duplicate-action risk:
- Automatic retry allowed after boundary: `No`, unless the action is proven
  idempotent and the user explicitly authorizes retry behavior.

## Stop Conditions

- target identity differs;
- precondition changed;
- required rollback/safe close is unavailable;
- output would expose secrets or raw private data;
- result is ambiguous;
- requested action exceeds the confirmed effect.

## Post-State and Reconciliation

- Authoritative post-state check:
- Redacted result:
- Rollback or safe close performed:
- Remaining unknown:
- User decision required:
