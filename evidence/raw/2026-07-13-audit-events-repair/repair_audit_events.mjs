#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";

const EXPECTED_LEGACY_IDS = new Set([
  "audit:uiwork-workflow-registered:1780224144221",
  "audit:uiwork-work-item-ready:1780224144221",
  "audit:uiwork-four-role-sessions-bound:1780224144221",
  "audit:workflow-machine-stale-run-cleaned:1780227015824",
]);

const REPAIR_RULES = {
  authorized_prepared_dispatch_created: {
    auditKind: "authorized-prepared-dispatch-created",
    entitySource(event, state) {
      const targetRef = requiredString(event.target_ref, "target_ref");
      const plannedTaskId = requiredString(
        event.project_director_planned_task_id,
        "project_director_planned_task_id",
      );
      const timestamp = requiredString(event.created_at, "created_at");
      const candidates = state.workflow_node_dispatches.filter(
        (dispatch) =>
          isObject(dispatch) &&
          dispatch.work_item_id === targetRef &&
          dispatch.c4_planned_task_id === plannedTaskId &&
          String(dispatch.created_at_ms) === timestamp &&
          typeof dispatch.dispatch_id === "string" &&
          dispatch.dispatch_id.startsWith("authorized-prepared-dispatch:"),
      );
      if (candidates.length !== 1) {
        throw new Error(
          `prepared-dispatch source is not unique for ${targetRef}: ${candidates.length}`,
        );
      }
      return candidates[0].dispatch_id;
    },
  },
  project_director_task_plan_created: {
    auditKind: "project-director-task-plan-created",
    entitySource(event) {
      return requiredString(event.target_ref, "target_ref");
    },
  },
};

function usage() {
  return [
    "Usage:",
    "  node repair_audit_events.mjs --input <workflow-state.v0.json>",
    "  node repair_audit_events.mjs --input <workflow-state.v0.json> --apply --in-place --mapping-dir <empty-dir>",
    "",
    "Without --apply this is a read-only preflight. --apply --in-place is intentionally explicit",
    "because it writes only the supplied workflow-state.v0.json after all checks pass.",
  ].join("\n");
}

function fail(message) {
  throw new Error(message);
}

function requiredString(value, name) {
  if (typeof value !== "string" || value.length === 0) {
    fail(`missing non-empty ${name}`);
  }
  return value;
}

function isObject(value) {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}

function stableId(value) {
  let output = "";
  for (const character of value) {
    if (/^[A-Za-z0-9]$/.test(character)) {
      output += character.toLowerCase();
    } else if (!output.endsWith("-")) {
      output += "-";
    }
  }
  return output.replace(/^-+|-+$/g, "");
}

function stableId96(value) {
  return [...stableId(value)].slice(0, 96).join("");
}

function sha256(value) {
  return crypto.createHash("sha256").update(value).digest("hex");
}

function canonicalJson(value) {
  if (Array.isArray(value)) {
    return `[${value.map(canonicalJson).join(",")}]`;
  }
  if (isObject(value)) {
    return `{${Object.keys(value)
      .sort()
      .map((key) => `${JSON.stringify(key)}:${canonicalJson(value[key])}`)
      .join(",")}}`;
  }
  return JSON.stringify(value);
}

function parseArgs(argv) {
  const args = { apply: false, inPlace: false };
  for (let index = 0; index < argv.length; index += 1) {
    const token = argv[index];
    if (token === "--apply") {
      args.apply = true;
      continue;
    }
    if (token === "--in-place") {
      args.inPlace = true;
      continue;
    }
    if (token === "--input" || token === "--mapping-dir") {
      const value = argv[index + 1];
      if (!value || value.startsWith("--")) {
        fail(`missing value for ${token}`);
      }
      args[token.slice(2).replace(/-([a-z])/g, (_, letter) => letter.toUpperCase())] = value;
      index += 1;
      continue;
    }
    if (token === "--help" || token === "-h") {
      process.stdout.write(`${usage()}\n`);
      process.exit(0);
    }
    fail(`unknown argument: ${token}`);
  }
  if (!args.input) {
    fail("--input is required");
  }
  if (args.inPlace && !args.apply) {
    fail("--in-place requires --apply");
  }
  if (args.apply && !args.inPlace) {
    fail("--apply requires --in-place; the script never chooses a write target itself");
  }
  if (args.apply && !args.mappingDir) {
    fail("--apply requires --mapping-dir");
  }
  return args;
}

function groupsByEventId(events) {
  const groups = new Map();
  for (const [index, event] of events.entries()) {
    if (!isObject(event)) {
      fail(`audit_events[${index}] is not an object`);
    }
    if (event.event_id === undefined) {
      continue;
    }
    const eventId = requiredString(event.event_id, `audit_events[${index}].event_id`);
    const members = groups.get(eventId) ?? [];
    members.push({ index, event });
    groups.set(eventId, members);
  }
  return [...groups.entries()].filter(([, members]) => members.length > 1);
}

function collectLegacyEvents(events) {
  const legacy = [];
  for (const [index, event] of events.entries()) {
    if (event.event_id !== undefined) {
      continue;
    }
    const legacyId = requiredString(event.audit_event_id, `audit_events[${index}].audit_event_id`);
    legacy.push({ index, legacyId, eventType: requiredString(event.event_type, "event_type") });
  }
  const found = new Set(legacy.map((entry) => entry.legacyId));
  if (
    legacy.length !== EXPECTED_LEGACY_IDS.size ||
    found.size !== EXPECTED_LEGACY_IDS.size ||
    [...EXPECTED_LEGACY_IDS].some((eventId) => !found.has(eventId))
  ) {
    fail("legacy audit_event_id set differs from the approved four-record repair scope");
  }
  return legacy;
}

function buildRepairPlan(before) {
  if (!Array.isArray(before.audit_events)) {
    fail("workflow-state has no audit_events array");
  }
  if (!Array.isArray(before.workflow_node_dispatches)) {
    fail("workflow-state has no workflow_node_dispatches array");
  }

  const collisions = groupsByEventId(before.audit_events);
  if (collisions.length === 0) {
    fail("no duplicate audit event_id groups found; refusing a no-op repair");
  }

  const changedIndexes = new Set();
  for (const [, members] of collisions) {
    for (const { index } of members) {
      changedIndexes.add(index);
    }
  }
  const untouchedIds = new Set(
    before.audit_events
      .filter((event, index) => !changedIndexes.has(index))
      .map((event) => event.event_id)
      .filter((eventId) => typeof eventId === "string" && eventId.length > 0),
  );
  const plannedIds = new Set();
  const mappings = [];
  const groupSummaries = [];

  for (const [oldId, members] of collisions) {
    const payloads = new Set(members.map(({ event }) => canonicalJson(event)));
    if (payloads.size !== members.length) {
      fail(`duplicate group has identical payloads and is not an approved collision: ${oldId}`);
    }
    const eventTypes = new Set(members.map(({ event }) => event.event_type));
    if (eventTypes.size !== 1) {
      fail(`duplicate group mixes event types: ${oldId}`);
    }
    const eventType = [...eventTypes][0];
    const rule = REPAIR_RULES[eventType];
    if (!rule) {
      fail(`duplicate group has an unapproved event_type: ${eventType}`);
    }

    const indexes = [];
    for (const { index, event } of members) {
      const timestamp = requiredString(event.created_at, "created_at");
      const entity = rule.entitySource(event, before);
      const expectedOldId = `audit:${rule.auditKind}:${stableId96(entity)}:${timestamp}`;
      if (expectedOldId !== oldId) {
        fail(`stored event_id does not match approved legacy template at audit_events[${index}]`);
      }
      let newId = `audit:${rule.auditKind}:${stableId(entity)}:${timestamp}`;
      if (untouchedIds.has(newId) || plannedIds.has(newId)) {
        newId = `${newId}:${sha256(entity).slice(0, 12)}`;
      }
      if (untouchedIds.has(newId) || plannedIds.has(newId)) {
        fail(`new event_id still collides after required hash suffix at audit_events[${index}]`);
      }
      plannedIds.add(newId);
      mappings.push({
        audit_event_index: index,
        event_type: eventType,
        old_event_id: oldId,
        new_event_id: newId,
        source_entity: entity,
        source_entity_sha256: sha256(entity),
        timestamp,
      });
      indexes.push(index);
    }
    groupSummaries.push({ old_event_id: oldId, event_type: eventType, indexes });
  }

  const legacy = collectLegacyEvents(before.audit_events);
  return { collisions, groupSummaries, legacy, mappings };
}

function applyPlan(before, plan) {
  const after = JSON.parse(JSON.stringify(before));
  for (const mapping of plan.mappings) {
    after.audit_events[mapping.audit_event_index].event_id = mapping.new_event_id;
  }
  for (const legacy of plan.legacy) {
    after.audit_events[legacy.index].event_id = legacy.legacyId;
  }
  return after;
}

function validateResult(before, after, plan) {
  const collisionIndexes = new Set(plan.mappings.map((entry) => entry.audit_event_index));
  const legacyIndexes = new Set(plan.legacy.map((entry) => entry.index));
  const changedIndexes = new Set([...collisionIndexes, ...legacyIndexes]);
  if (changedIndexes.size !== plan.mappings.length + plan.legacy.length) {
    fail("collision and legacy repair targets overlap");
  }

  const topLevelKeys = new Set([...Object.keys(before), ...Object.keys(after)]);
  for (const key of topLevelKeys) {
    if (key !== "audit_events" && canonicalJson(before[key]) !== canonicalJson(after[key])) {
      fail(`unexpected top-level change: ${key}`);
    }
  }
  let changed = 0;
  for (const [index, beforeEvent] of before.audit_events.entries()) {
    const afterEvent = after.audit_events[index];
    if (!changedIndexes.has(index)) {
      if (canonicalJson(beforeEvent) !== canonicalJson(afterEvent)) {
        fail(`unexpected audit event mutation at index ${index}`);
      }
      continue;
    }
    const beforeWithoutId = { ...beforeEvent };
    const afterWithoutId = { ...afterEvent };
    delete beforeWithoutId.event_id;
    delete afterWithoutId.event_id;
    if (canonicalJson(beforeWithoutId) !== canonicalJson(afterWithoutId)) {
      fail(`non-event_id mutation at audit_events[${index}]`);
    }
    if (typeof afterEvent.event_id !== "string" || afterEvent.event_id.length === 0) {
      fail(`invalid repaired event_id at audit_events[${index}]`);
    }
    changed += 1;
  }

  const ids = after.audit_events.map((event, index) => requiredString(event.event_id, `audit_events[${index}].event_id`));
  if (new Set(ids).size !== ids.length) {
    fail("post-repair audit event_id values are not unique");
  }
  return {
    changed_event_count: changed,
    changed_event_id_values: plan.mappings.length,
    added_legacy_event_id_fields: plan.legacy.length,
    untouched_event_count: after.audit_events.length - changed,
    post_repair_event_id_count: ids.length,
    post_repair_unique_event_id_count: new Set(ids).size,
  };
}

function writeJson(filePath, value) {
  fs.writeFileSync(filePath, `${JSON.stringify(value, null, 2)}\n`, { encoding: "utf8", flag: "wx" });
}

function prepareMappingDirectory(mappingDir) {
  if (fs.existsSync(mappingDir)) {
    if (!fs.statSync(mappingDir).isDirectory() || fs.readdirSync(mappingDir).length !== 0) {
      fail(`mapping directory must be absent or empty: ${mappingDir}`);
    }
  } else {
    fs.mkdirSync(mappingDir, { recursive: true });
  }
}

function main() {
  const args = parseArgs(process.argv.slice(2));
  const inputPath = path.resolve(args.input);
  if (path.basename(inputPath) !== "workflow-state.v0.json") {
    fail("--input must name workflow-state.v0.json");
  }
  const sourceText = fs.readFileSync(inputPath, "utf8");
  const sourceHash = sha256(sourceText);
  const before = JSON.parse(sourceText);
  const plan = buildRepairPlan(before);
  const after = applyPlan(before, plan);
  const semanticDiff = validateResult(before, after, plan);
  const rendered = JSON.stringify(after, null, 2);
  const roundTripByteEqual = sourceText === JSON.stringify(before, null, 2);
  const report = {
    source_path: inputPath,
    source_sha256: sourceHash,
    source_byte_length: Buffer.byteLength(sourceText),
    serialization_round_trip_byte_equal: roundTripByteEqual,
    source_ends_with_newline: sourceText.endsWith("\n"),
    collision_group_count: plan.collisions.length,
    collision_record_count: plan.mappings.length,
    collision_groups: plan.groupSummaries,
    legacy_additions: plan.legacy,
    semantic_diff: semanticDiff,
    repaired_sha256: sha256(rendered),
    repaired_byte_length: Buffer.byteLength(rendered),
  };

  if (!args.apply) {
    process.stdout.write(`${JSON.stringify({ mode: "preflight", ...report }, null, 2)}\n`);
    return;
  }

  const currentHash = sha256(fs.readFileSync(inputPath, "utf8"));
  if (currentHash !== sourceHash) {
    fail("input changed during preflight; refusing to write");
  }
  const mappingDir = path.resolve(args.mappingDir);
  prepareMappingDirectory(mappingDir);
  writeJson(path.join(mappingDir, "event-id-mapping.json"), plan.mappings);
  writeJson(path.join(mappingDir, "legacy-event-id-additions.json"), plan.legacy);
  writeJson(path.join(mappingDir, "repair-report.json"), { mode: "applied", ...report });
  fs.writeFileSync(inputPath, rendered, "utf8");
  const writtenHash = sha256(fs.readFileSync(inputPath, "utf8"));
  if (writtenHash !== report.repaired_sha256) {
    fail("written file hash differs from the verified repaired payload");
  }
  writeJson(path.join(mappingDir, "state-write-receipt.json"), {
    source_sha256: sourceHash,
    written_sha256: writtenHash,
    changed_event_count: semanticDiff.changed_event_count,
  });
  process.stdout.write(`${JSON.stringify({ mode: "applied", ...report, written_sha256: writtenHash }, null, 2)}\n`);
}

try {
  main();
} catch (error) {
  process.stderr.write(`repair_audit_events: ${error.message}\n`);
  process.exit(1);
}
