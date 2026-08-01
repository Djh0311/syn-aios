import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import { existsSync, readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const contractDir = dirname(fileURLToPath(import.meta.url));
const repoRoot = resolve(contractDir, "../..");
const expectedBaseOid = "2bf9406bd688db8eb84d2138f9b3c6994dac2fb9";
const failures = [];

function fail(code, detail) {
  failures.push({ code, detail: String(detail) });
}

function check(condition, code, detail) {
  if (!condition) fail(code, detail);
}

function readRepoText(relativePath) {
  try {
    return readFileSync(resolve(repoRoot, relativePath), "utf8");
  } catch (error) {
    fail("FILE_READ", relativePath + ": " + error.message);
    return "";
  }
}

function loadJson(relativePath) {
  const text = readRepoText(relativePath);
  try {
    return JSON.parse(text);
  } catch (error) {
    fail("JSON_PARSE", relativePath + ": " + error.message);
    return {};
  }
}

function gitText(args) {
  try {
    return execFileSync("git", args, {
      cwd: repoRoot,
      encoding: "utf8",
      stdio: ["ignore", "pipe", "pipe"]
    }).trimEnd();
  } catch (error) {
    const detail = error.stderr ? String(error.stderr).trim() : error.message;
    fail("GIT_READ", args.join(" ") + ": " + detail);
    return "";
  }
}

function gitBuffer(args) {
  try {
    return execFileSync("git", args, {
      cwd: repoRoot,
      stdio: ["ignore", "pipe", "pipe"]
    });
  } catch (error) {
    const detail = error.stderr ? String(error.stderr).trim() : error.message;
    fail("GIT_READ", args.join(" ") + ": " + detail);
    return Buffer.alloc(0);
  }
}

function sha256(buffer) {
  return createHash("sha256").update(buffer).digest("hex");
}

function asArray(value) {
  return Array.isArray(value) ? value : [];
}

function unique(values) {
  return new Set(values).size === values.length;
}

function sameArray(left, right) {
  return JSON.stringify(left) === JSON.stringify(right);
}

function sameSet(left, right) {
  return sameArray(sorted(left), sorted(right));
}

function sorted(values) {
  return [...values].sort((a, b) => String(a).localeCompare(String(b)));
}

function allNonEmptyStrings(values) {
  return Array.isArray(values) && values.length > 0 &&
    values.every((value) => typeof value === "string" && value.trim() !== "");
}

function escapeRegExp(value) {
  return String(value).replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

const allowedFrontKeys = new Set([
  "contract_id",
  "version",
  "status",
  "evidence_level",
  "schema_authority",
  "dependencies",
  "hold_refs"
]);

function parseFrontMatterResult(text) {
  const lines = String(text).replace(/\r\n?/g, "\n").split("\n");
  if (lines[0] !== "---") return { code: "FRONT_MATTER_MISSING", value: {} };
  const end = lines.indexOf("---", 1);
  if (end < 0) return { code: "FRONT_MATTER_MISSING", value: {} };
  const result = {};
  const seen = new Set();
  for (let index = 1; index < end; index += 1) {
    const line = lines[index];
    if (line.trim() === "") continue;
    const match = line.match(/^([A-Za-z0-9_-]+):[ \t]*(.*)$/);
    if (!match) {
      return { code: "FRONT_MATTER_MALFORMED_LINE", detail: index + 1, value: result };
    }
    const key = match[1];
    const raw = match[2];
    if (!allowedFrontKeys.has(key)) {
      return { code: "FRONT_MATTER_UNKNOWN_KEY", detail: key, value: result };
    }
    if (seen.has(key)) {
      return { code: "FRONT_MATTER_DUPLICATE_KEY", detail: key, value: result };
    }
    seen.add(key);
    if (raw.startsWith("[") || raw.startsWith("{")) {
      try {
        result[key] = JSON.parse(raw);
      } catch (error) {
        return { code: "FRONT_MATTER_JSON", detail: key + ":" + error.message, value: result };
      }
    } else if (/^[0-9]+$/.test(raw)) {
      result[key] = Number(raw);
    } else {
      result[key] = raw.replace(/^(["'])(.*)\1$/, "$2");
    }
  }
  return { code: "PASS", value: result, endLine: end + 1 };
}

function maskHtmlComments(text) {
  return String(text).replace(/<!--[\s\S]*?(?:-->|$)/g, (match) =>
    match.replace(/[^\n]/g, " ")
  );
}

function collectLevel2Headings(text) {
  const lines = String(text).replace(/\r\n?/g, "\n").split("\n");
  const headings = [];
  let fence = null;
  const rawHtmlOutsideFence = /^[ \t]{0,3}<(?:!--|\?|!\[CDATA\[|![A-Z]|\/?[A-Za-z][A-Za-z0-9-]*(?:\s|\/?>|$))/i;
  for (let index = 0; index < lines.length; index += 1) {
    const line = lines[index];
    if (fence !== null) {
      const closing = line.match(/^[ \t]{0,3}(`{3,}|~{3,})[ \t]*$/);
      if (closing && closing[1][0] === fence.char && closing[1].length >= fence.length) {
        fence = null;
      }
      continue;
    }
    const opening = line.match(/^[ \t]{0,3}(`{3,}|~{3,})(.*)$/);
    if (opening) {
      fence = { char: opening[1][0], length: opening[1].length };
      continue;
    }
    if (rawHtmlOutsideFence.test(line)) {
      return { headings, lines, error: { code: "CONTRACT_HTML_FORBIDDEN", line: index + 1 } };
    }
    const match = line.match(/^##[ \t]+(.+?)[ \t]*$/);
    if (match) headings.push({ name: match[1], line: index + 1, index });
  }
  return { headings, lines, error: null };
}

function headingResult(text, heading) {
  const result = collectLevel2Headings(text);
  if (result.error) return result.error.code;
  const count = result.headings.filter((item) => item.name === heading).length;
  if (count === 0) return "CONTRACT_SECTION_MISSING";
  if (count > 1) return "CONTRACT_SECTION_DUPLICATE";
  return "PASS";
}

function sectionBody(text, heading) {
  const { headings, lines, error } = collectLevel2Headings(text);
  if (error) return "";
  const matches = headings.filter((item) => item.name === heading);
  if (matches.length !== 1) return "";
  const current = matches[0];
  const next = headings.find((item) => item.index > current.index);
  return lines.slice(current.index + 1, next ? next.index : lines.length).join("\n").trim();
}

function fencedJsonBlocks(text, label) {
  const pattern = new RegExp(
    "^```json[ \\t]+" + escapeRegExp(label) + "[ \\t]*\\n([\\s\\S]*?)^```[ \\t]*$",
    "gm"
  );
  return [...String(text).matchAll(pattern)].map((match) => match[1]);
}

function dagResult(contracts) {
  if (!Array.isArray(contracts)) return "REQUIRED_FIELD";
  const ids = contracts.map((item) => item.id);
  const authorities = contracts.map((item) => item.schema_authority ?? item.owner);
  if (!unique(ids)) return "DUPLICATE_ID";
  if (!unique(authorities)) return "DUPLICATE_OWNER";
  const known = new Set(ids);
  for (const item of contracts) {
    for (const dependency of asArray(item.dependencies)) {
      if (!known.has(dependency)) return "UNKNOWN_DEPENDENCY";
    }
  }
  const visiting = new Set();
  const visited = new Set();
  const byId = new Map(contracts.map((item) => [item.id, item]));
  function visit(id) {
    if (visiting.has(id)) return false;
    if (visited.has(id)) return true;
    visiting.add(id);
    for (const dependency of asArray(byId.get(id).dependencies)) {
      if (!visit(dependency)) return false;
    }
    visiting.delete(id);
    visited.add(id);
    return true;
  }
  for (const id of ids) {
    if (!visit(id)) return "CYCLE";
  }
  return "PASS";
}

function collectHoldRefs(value, target) {
  if (Array.isArray(value)) {
    for (const item of value) collectHoldRefs(item, target);
    return;
  }
  if (!value || typeof value !== "object") return;
  for (const [key, item] of Object.entries(value)) {
    if (key === "hold_refs" && Array.isArray(item)) {
      for (const ref of item) target.add(ref);
    } else {
      collectHoldRefs(item, target);
    }
  }
}

function canonicalKey(value) {
  return String(value)
    .normalize("NFKD")
    .replace(/[\p{M}\p{Cf}]+/gu, "")
    .replace(/([A-Z]+)([A-Z][a-z])/g, "$1_$2")
    .replace(/([a-z0-9])([A-Z])/g, "$1_$2")
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "_")
    .replace(/_+/g, "_")
    .replace(/^_+|_+$/g, "");
}

function canonicalSensitiveText(value) {
  return String(value)
    .normalize("NFKD")
    .replace(/[\p{M}\p{Cf}]+/gu, "")
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "");
}

const opaqueRefPrefixes = {
  credential_ref: ["credential"],
  secret_ref: ["secret"],
  transcript_ref: ["object", "transcript"],
  tool_output_ref: ["object", "tool_output"],
  payload_ref: ["object", "payload"],
  prompt_ref: ["object", "prompt"],
  provider_response_ref: ["object", "provider_response"],
  stdout_ref: ["object", "stdout"],
  stderr_ref: ["object", "stderr"],
  provider_handle_ref: ["provider_handle"]
};

function opaqueReferenceResult(key, value, mode) {
  if (typeof value !== "string" || value !== value.normalize("NFKC") ||
      /[\p{Cc}\p{Cf}\s./:=]/u.test(value)) {
    return "SENSITIVE_REF_SHAPE";
  }
  for (const prefix of opaqueRefPrefixes[key] || []) {
    const marker = prefix + "_ref_";
    if (!value.startsWith(marker)) continue;
    const suffix = value.slice(marker.length);
    const fixtureSuffix = /^[0-9]{2,}$/;
    const artifactSuffix = /^(?:[0-9a-f]{32}|[0-9a-f]{8}(?:-[0-9a-f]{4}){3}-[0-9a-f]{12}|[0-9A-HJKMNP-TV-Z]{26}|[0-9a-f]{64})$/i;
    return (mode === "FIXTURE" ? fixtureSuffix : artifactSuffix).test(suffix) ? "PASS" : "SENSITIVE_REF_SHAPE";
  }
  return "SENSITIVE_REF_SHAPE";
}

function findSensitiveMaterial(value, forbiddenKeys, allowedKeys, sentinels, path = "$", mode = "ARTIFACT") {
  if (typeof value === "string") {
    const normalized = canonicalSensitiveText(value);
    for (const sentinel of sentinels) {
      if (normalized.includes(canonicalSensitiveText(sentinel))) {
        return { code: "SENSITIVE_SENTINEL", path };
      }
    }
    const credentialText = String(value).normalize("NFKD").replace(/[\p{M}\p{Cf}]+/gu, "");
    if (/\b(?:bearer|basic)\s+[A-Za-z0-9._~+/=-]+/i.test(credentialText) ||
        /-----BEGIN [A-Z0-9 -]*PRIVATE KEY(?: BLOCK)?-----/i.test(credentialText)) {
      return { code: "SENSITIVE_VALUE", path };
    }
    return null;
  }
  if (Array.isArray(value)) {
    for (let index = 0; index < value.length; index += 1) {
      const found = findSensitiveMaterial(value[index], forbiddenKeys, allowedKeys, sentinels, path + "[" + index + "]", mode);
      if (found) return found;
    }
    return null;
  }
  if (!value || typeof value !== "object") return null;
  const riskPatterns = [
    /(^|_)(secret|credential|password|api_key|private_key|client_secret)(_|$)/,
    /(^|_)(access_|refresh_|bearer_|auth_|session_)?token(_|$)/,
    /(^|_)(raw_|full_)?(transcript|stdout|stderr|prompt|prompt_body)(_|$)/,
    /(^|_)(raw_|full_)?tool_(call_)?(output|result|response|payload)(_|$)/,
    /(^|_)(raw_|full_)?(provider|model|function)_(output|result|response|payload)(_|$)/,
    /^(?:authorization|proxy_authorization|cookie|set_cookie|error_stack|stack_trace|exception_stack)$/
  ];
  for (const [rawKey, child] of Object.entries(value)) {
    const key = canonicalKey(rawKey);
    const nextPath = path + "." + rawKey;
    const forbidden = forbiddenKeys.has(key) || riskPatterns.some((pattern) => pattern.test(key));
    if (allowedKeys.has(key)) {
      const refResult = opaqueReferenceResult(key, child, mode);
      if (refResult !== "PASS") return { code: refResult, path: nextPath };
    } else if (forbidden) {
      return { code: "SENSITIVE_FIELD", path: nextPath };
    }
    const found = findSensitiveMaterial(child, forbiddenKeys, allowedKeys, sentinels, nextPath, mode);
    if (found) return found;
  }
  return null;
}

function findForbiddenField(value, forbiddenKeys, path = "$") {
  if (Array.isArray(value)) {
    for (let index = 0; index < value.length; index += 1) {
      const found = findForbiddenField(value[index], forbiddenKeys, path + "[" + index + "]");
      if (found) return found;
    }
    return null;
  }
  if (!value || typeof value !== "object") return null;
  for (const [rawKey, child] of Object.entries(value)) {
    const key = canonicalKey(rawKey);
    const nextPath = path + "." + rawKey;
    if (forbiddenKeys.has(key)) return nextPath;
    const found = findForbiddenField(child, forbiddenKeys, nextPath);
    if (found) return found;
  }
  return null;
}

function inventoryRecordResult(record, contractIds, holdIds, enums) {
  const required = [
    "owner_contract", "scope_rule", "policy_gateway", "bypass_status", "migration_status",
    "disposition", "migration_target", "next_task", "hold_refs"
  ];
  for (const key of required) {
    const value = record[key];
    if (value === undefined || value === null ||
        (typeof value === "string" && value.trim() === "") ||
        (key === "hold_refs" && !Array.isArray(value))) {
      return "REQUIRED_FIELD";
    }
  }
  if (contractIds && !contractIds.has(record.owner_contract)) return "UNKNOWN_OWNER";
  if (enums && (!enums.bypass_status.includes(record.bypass_status) ||
      !enums.migration_status.includes(record.migration_status) ||
      !enums.disposition.includes(record.disposition))) {
    return "UNKNOWN_ENUM";
  }
  if (holdIds && record.hold_refs.some((ref) => !holdIds.has(ref))) return "UNKNOWN_HOLD";
  if (record.migration_status === "GUARDED_LEGACY") {
    const proof = record.guard_proof;
    if (!proof || proof.authorization_source !== "SERVER" || proof.caller_influences_allow !== false ||
        !allNonEmptyStrings(proof.source_refs)) {
      return "GUARD_PROOF_REQUIRED";
    }
  }
  if (record.migration_status === "BLOCKED") {
    const proof = record.block_proof;
    if (!proof || !allNonEmptyStrings(proof.source_refs) ||
        typeof proof.mechanism !== "string" || proof.mechanism.trim() === "") {
      return "BLOCK_PROOF_REQUIRED";
    }
  }
  if (record.migration_status === "MIGRATED" && record.bypass_status !== "NONE_OBSERVED") {
    return "MIGRATED_WITH_BYPASS";
  }
  if (record.bypass_status === "STATICALLY_BLOCKED" && record.migration_status !== "BLOCKED") {
    return "STATIC_BLOCK_STATUS";
  }
  return "PASS";
}

function m2InputResult(input, forbiddenKeys) {
  if (!input || typeof input !== "object" ||
      typeof input.name !== "string" || input.name.trim() === "" ||
      typeof input.owner_contract !== "string" || input.owner_contract.trim() === "" ||
      !allNonEmptyStrings(input.required_fields)) {
    return "REQUIRED_FIELD";
  }
  if (findForbiddenField(input, forbiddenKeys)) return "PREMATURE_DECISION";
  const allowed = ["name", "owner_contract", "domain_owner", "required_fields", "persistence_owner", "runtime_state_machine"];
  return Object.keys(input).every((key) => allowed.includes(key)) ? "PASS" : "M2_UNKNOWN_KEY";
}

const m2RootKeys = [
  "schema", "status", "scope", "ownership_boundary", "outbox_runtime_hold",
  "storage_input_requirements", "interfaces", "shadow_write", "parity_dimensions",
  "classification_rules", "rollback_guards", "premature_decisions_forbidden_in_m1"
];
const m2OwnershipKeys = ["m1_owns", "m2_owns", "no_runtime_claim"];
const m2OutboxHoldKeys = [
  "semantic_interface_owner", "runtime_owner", "runtime_fields", "runtime_state_machine",
  "forbidden_m1_commands", "forbidden_m1_transitions"
];
const m2InterfaceBaseKeys = ["name", "owner_contract", "domain_owner", "required_fields"];
const m2InterfaceHoldKeys = [...m2InterfaceBaseKeys, "persistence_owner", "runtime_state_machine"];
const m2ShadowKeys = ["states", "invariants"];
const m2ClassificationKeys = ["unknown", "corrupt", "sensitive", "approved_difference", "bug"];

function exactKeySet(value, expectedKeys) {
  return value !== null && typeof value === "object" && !Array.isArray(value) &&
    sameSet(Object.keys(value), expectedKeys);
}

function foldM2Text(value) {
  return String(value).normalize("NFKD").replace(/[\p{M}\p{Cf}]+/gu, "").toLowerCase();
}

function m2TextVariants(value) {
  const folded = foldM2Text(value);
  const commentPattern = /\/\*[\s\S]*?(?:\*\/|$)|--[^\r\n]*|#[^\r\n]*/g;
  return [folded, folded.replace(commentPattern, " "), folded.replace(commentPattern, "")];
}

function collectStringLeaves(value, target) {
  if (typeof value === "string") {
    target.push(value);
    return;
  }
  if (Array.isArray(value)) {
    for (const item of value) collectStringLeaves(item, target);
    return;
  }
  if (!value || typeof value !== "object") return;
  for (const item of Object.values(value)) collectStringLeaves(item, target);
}

const sqliteIdentifierPattern = "(?:[a-z_][a-z0-9_]*|\"(?:\"\"|[^\"])+\"|`(?:``|[^`])+`|\\[(?:\\]\\]|[^\\]])+\\])";
const m2ConcretePatterns = [
  /\bcreate\s+(?:(?:or\s+replace|temp(?:orary)?|virtual|unique|materialized|unlogged)\s+)*(?:table|index|schema|trigger|view)\b/i,
  /\b(?:alter|drop|truncate)\s+(?:table|index|schema|trigger|view)\b/i,
  /\b(?:insert\s+into|delete\s+from|update\s+[a-z0-9_]+\s+set)\b/i,
  /\b(?:primary|foreign)\s+key\b/i,
  new RegExp("\\bupdate\\s+" + sqliteIdentifierPattern + "\\s+set\\b", "i"),
  new RegExp("\\breferences\\s+" + sqliteIdentifierPattern + "(?=\\s|\\(|;|$)", "i"),
  /\b(?:table_name|index_name|unique_index|primary_store|quarantine_table|live_store_path|provider_name|rollback_script)\s*[:=]/i,
  /\b(?:lease[\s._-]*seconds|retry[\s._-]*count|backoff[\s._-]*ms)\s*[:=]\s*\d+/i,
  /\bcutover(?:[\s._-]*at)?\s*[:=]\s*\S+/i,
  /\b(?:begin|rollback)\s+transaction\b/i,
  /\bpragma\b/i,
  /\battach\b/i,
  /\bdetach\b/i,
  /\bvacuum\b/i,
  /\breindex\b/i,
  /\banalyze\b/i
];

function m2ArtifactShapeResult(input, forbiddenKeys) {
  if (!exactKeySet(input, m2RootKeys)) return "M2_UNKNOWN_KEY";
  if (!exactKeySet(input.ownership_boundary, m2OwnershipKeys) ||
      !exactKeySet(input.outbox_runtime_hold, m2OutboxHoldKeys) ||
      !exactKeySet(input.shadow_write, m2ShadowKeys) ||
      !exactKeySet(input.classification_rules, m2ClassificationKeys)) {
    return "M2_UNKNOWN_KEY";
  }
  if (!Array.isArray(input.interfaces)) return "REQUIRED_FIELD";
  for (const item of input.interfaces) {
    const expected = ["OutboxLease", "UnknownQuarantineRef"].includes(item?.name) ?
      m2InterfaceHoldKeys : m2InterfaceBaseKeys;
    if (!exactKeySet(item, expected)) return "M2_UNKNOWN_KEY";
  }
  if (findForbiddenField(input, forbiddenKeys)) return "PREMATURE_DECISION";
  const freeText = [
    ...m2OwnershipKeys.map((key) => input.ownership_boundary[key]),
    ...asArray(input.storage_input_requirements),
    ...asArray(input.shadow_write.invariants),
    ...m2ClassificationKeys.map((key) => input.classification_rules[key]),
    ...asArray(input.rollback_guards)
  ];
  if (!freeText.every((value) => typeof value === "string" && value.trim() !== "")) return "REQUIRED_FIELD";
  const allText = [];
  collectStringLeaves(input, allText);
  for (const value of allText) {
    if (m2TextVariants(value).some((variant) =>
      m2ConcretePatterns.some((pattern) => pattern.test(variant)))) return "M2_PREMATURE_TEXT";
  }
  return "PASS";
}

function withFixtureMutation(value, mutation) {
  const clone = JSON.parse(JSON.stringify(value));
  if (!mutation) return clone;
  let target = clone;
  for (const key of mutation.path.slice(0, -1)) target = target[key];
  target[mutation.path[mutation.path.length - 1]] = mutation.value;
  return clone;
}

function migrationItemResult(item, statusEnum, dispositionEnum, classificationEnum) {
  if (!statusEnum.includes(item.migration_status)) return "MIGRATION_STATUS_ENUM";
  if (!dispositionEnum.includes(item.disposition)) return "MIGRATION_DISPOSITION_ENUM";
  if (!classificationEnum.includes(item.source_classification)) return "MIGRATION_CLASSIFICATION_ENUM";
  if (["UNKNOWN", "CORRUPT", "SENSITIVE"].includes(item.source_classification)) {
    if (!["BLOCKED", "HOLD"].includes(item.migration_status)) {
      return "MIGRATION_CLASSIFICATION_NOT_BLOCKED";
    }
    if (!item.quarantine || item.quarantine.mode !== "REQUIRED" ||
        typeof item.quarantine.ref !== "string" ||
        !/^quarantine_ref_(?:[0-9]{2,}|[0-9a-f]{32}|[0-9a-f]{8}(?:-[0-9a-f]{4}){3}-[0-9a-f]{12}|[0-9A-HJKMNP-TV-Z]{26}|[0-9a-f]{64})$/i.test(item.quarantine.ref) ||
        item.quarantine.owner !== "SYN-DAT-006") {
      return "MIGRATION_QUARANTINE_REQUIRED";
    }
    if (item.quarantine.payload_mode !== "REF_ONLY") {
      return "MIGRATION_SENSITIVE_REF_ONLY";
    }
  }
  if (item.migration_status === "GUARDED_LEGACY" && !allNonEmptyStrings(item.guard_refs)) {
    return "MIGRATION_GUARD_PROOF_REQUIRED";
  }
  return "PASS";
}

function maskRustCommentsAndLiterals(text) {
  const source = String(text);
  const output = source.split("");
  const maskAt = (index) => {
    if (output[index] !== "\n" && output[index] !== "\r") output[index] = " ";
  };
  let index = 0;
  while (index < source.length) {
    if (source.startsWith("//", index)) {
      while (index < source.length && source[index] !== "\n") maskAt(index++);
      continue;
    }
    if (source.startsWith("/*", index)) {
      let depth = 0;
      while (index < source.length) {
        if (source.startsWith("/*", index)) {
          maskAt(index++); maskAt(index++); depth += 1; continue;
        }
        if (source.startsWith("*/", index)) {
          maskAt(index++); maskAt(index++); depth -= 1;
          if (depth === 0) break;
          continue;
        }
        maskAt(index++);
      }
      continue;
    }
    const raw = source.slice(index).match(/^(?:br|cr|r)(#*)"/);
    if (raw) {
      const closing = "\"" + raw[1];
      for (let count = 0; count < raw[0].length; count += 1) maskAt(index++);
      while (index < source.length) {
        if (source.startsWith(closing, index)) {
          for (let count = 0; count < closing.length; count += 1) maskAt(index++);
          break;
        }
        maskAt(index++);
      }
      continue;
    }
    const stringPrefix = source.startsWith("b\"", index) || source.startsWith("c\"", index) ? 2 :
      source[index] === "\"" ? 1 : 0;
    if (stringPrefix > 0) {
      for (let count = 0; count < stringPrefix; count += 1) maskAt(index++);
      let escaped = false;
      while (index < source.length) {
        const char = source[index];
        maskAt(index++);
        if (escaped) escaped = false;
        else if (char === "\\") escaped = true;
        else if (char === "\"") break;
      }
      continue;
    }
    const charMatch = source.slice(index).match(/^(?:b)?'(?:\\.|[^\\'\n])'/);
    if (charMatch) {
      for (let count = 0; count < charMatch[0].length; count += 1) maskAt(index++);
      continue;
    }
    index += 1;
  }
  return output.join("");
}

function maskRustMacroTokenTrees(text) {
  const source = String(text);
  const output = source.split("");
  const openerToCloser = { "(": ")", "[": "]", "{": "}" };
  const rustIdent = String.raw`(?:r#)?(?:[_\p{ID_Start}])(?:[_\p{ID_Continue}]*)`;
  const rustPath = `${rustIdent}(?:[ \\t\\r\\n]*::[ \\t\\r\\n]*${rustIdent})*`;
  const macroPattern = new RegExp(
    `(?<![_\\p{ID_Continue}])(?:macro_rules[ \\t\\r\\n]*![ \\t\\r\\n]*${rustIdent}|${rustPath}[ \\t\\r\\n]*!)[ \\t\\r\\n]*([({\\[])`,
    "gu"
  );
  const macro2Pattern = new RegExp(
    `(?<![_\\p{ID_Continue}])macro[ \\t\\r\\n]+${rustIdent}[ \\t\\r\\n]*([({\\[])`,
    "gu"
  );
  const attributePattern = /#[ \t\r\n]*!?[ \t\r\n]*(\[)/g;
  const matches = [
    ...[...source.matchAll(macroPattern)].map((match) => ({ match, macro2: false })),
    ...[...source.matchAll(attributePattern)].map((match) => ({ match, macro2: false })),
    ...[...source.matchAll(macro2Pattern)].map((match) => ({ match, macro2: true }))
  ].sort((left, right) => left.match.index - right.match.index);
  const maskTree = (openerIndex) => {
    output[openerIndex] = " ";
    const stack = [openerToCloser[source[openerIndex]]];
    let index = openerIndex + 1;
    while (index < source.length && stack.length > 0) {
      const char = source[index];
      if (openerToCloser[char]) stack.push(openerToCloser[char]);
      else if (char === stack[stack.length - 1]) stack.pop();
      if (char !== "\n" && char !== "\r") output[index] = " ";
      index += 1;
    }
    return index;
  };
  for (const { match, macro2 } of matches) {
    const openerIndex = match.index + match[0].lastIndexOf(match[1]);
    if (output.slice(match.index, openerIndex + 1).every((char, offset) => char === source[match.index + offset])) {
      let index = maskTree(openerIndex);
      if (macro2 && match[1] !== "{") {
        while (index < source.length && /[ \t\r\n]/.test(source[index])) index += 1;
        if (source[index] === "{") maskTree(index);
      }
    }
  }
  return output.join("");
}

function rustFunctionHasBody(source, startIndex) {
  let parenDepth = 0;
  let bracketDepth = 0;
  let angleDepth = 0;
  let nestedCurlyDepth = 0;
  for (let index = startIndex; index < source.length; index += 1) {
    const char = source[index];
    if (char === "(") parenDepth += 1;
    else if (char === ")" && parenDepth > 0) parenDepth -= 1;
    else if (char === "[") bracketDepth += 1;
    else if (char === "]" && bracketDepth > 0) bracketDepth -= 1;
    else if (char === "<" && nestedCurlyDepth === 0) angleDepth += 1;
    else if (char === ">" && nestedCurlyDepth === 0 && angleDepth > 0) angleDepth -= 1;
    else if (char === "{") {
      if (parenDepth === 0 && bracketDepth === 0 && angleDepth === 0 && nestedCurlyDepth === 0) return true;
      nestedCurlyDepth += 1;
    } else if (char === "}" && nestedCurlyDepth > 0) {
      nestedCurlyDepth -= 1;
    } else if (char === ";" && parenDepth === 0 && bracketDepth === 0 &&
        angleDepth === 0 && nestedCurlyDepth === 0) {
      return false;
    }
  }
  return false;
}

function rustDefinitionLines(text, symbol) {
  const escaped = escapeRegExp(symbol);
  const pattern = new RegExp(
    "^[ \\t]*(?:pub(?:\\([^)]*\\))?[ \\t]+)?(?:(?:async|unsafe|const|extern(?:[ \\t]+\"[^\"]+\")?)[ \\t]+)*fn[ \\t]+" + escaped + "\\b",
    "gm"
  );
  const masked = maskRustMacroTokenTrees(maskRustCommentsAndLiterals(text));
  return [...masked.matchAll(pattern)]
    .filter((match) => rustFunctionHasBody(masked, match.index + match[0].length))
    .map((match) => masked.slice(0, match.index).split("\n").length);
}

function runnerDefinitionResult(source, symbol, declaredLines) {
  if (!/^[A-Za-z_][A-Za-z0-9_]*$/.test(symbol)) return "RUNNER_SYMBOL_IDENTIFIER";
  return sameArray(rustDefinitionLines(source, symbol), declaredLines) ? "PASS" : "RUNNER_DEFINITION_VARIANTS";
}

function rustStringConst(source, symbol) {
  const pattern = new RegExp(
    "^[ \\t]*(?:pub(?:\\([^)]*\\))?[ \\t]+)?const[ \\t]+" + escapeRegExp(symbol) +
      "\\b[^=\\n]*=[ \\t]*(\"(?:\\\\.|[^\"\\\\])*\")[ \\t]*;",
    "m"
  );
  const match = String(source).match(pattern);
  if (!match) return null;
  try {
    return JSON.parse(match[1]);
  } catch {
    return null;
  }
}

function rustStringArray(source, symbol, constants) {
  const pattern = new RegExp(
    "^[ \\t]*(?:pub(?:\\([^)]*\\))?[ \\t]+)?const[ \\t]+" + escapeRegExp(symbol) +
      "\\b[^=\\n]*=[ \\t]*&[ \\t]*\\[([\\s\\S]*?)^[ \\t]*\\];",
    "m"
  );
  const match = String(source).match(pattern);
  if (!match) return { code: "RUST_STRING_ARRAY_MISSING", values: [] };
  const body = match[1].replace(/\/\*[\s\S]*?\*\//g, "").replace(/\/\/.*$/gm, "");
  const values = [];
  for (const token of body.split(",").map((item) => item.trim()).filter(Boolean)) {
    if (Object.hasOwn(constants, token)) {
      values.push(constants[token]);
      continue;
    }
    if (/^\"(?:\\.|[^\"\\])*\"$/.test(token)) {
      try {
        values.push(JSON.parse(token));
        continue;
      } catch {
        return { code: "RUST_STRING_ARRAY_TOKEN", values: [] };
      }
    }
    return { code: "RUST_STRING_ARRAY_TOKEN", values: [] };
  }
  return { code: "PASS", values };
}

function sourceDeclarationAnchorResult(reference, sourceById, sourceTextById) {
  const match = String(reference).match(/^SOURCE::([a-z0-9-]+)::([A-Za-z_][A-Za-z0-9_]*)$/);
  if (!match || !sourceById.has(match[1])) return "PROJECTION_SOURCE_ANCHOR_REF";
  const source = sourceTextById.get(match[1]) || "";
  if (rustDefinitionLines(source, match[2]).length > 0) return "PASS";
  const masked = maskRustMacroTokenTrees(maskRustCommentsAndLiterals(source));
  const constPattern = new RegExp(
    "^[ \\t]*(?:pub(?:\\([^)]*\\))?[ \\t]+)?(?:const|static)[ \\t]+" + escapeRegExp(match[2]) + "\\b",
    "m"
  );
  return constPattern.test(masked) ? "PASS" : "PROJECTION_SOURCE_ANCHOR_DECLARATION";
}

function sourceRefLineResult(reference, sourceById) {
  const match = String(reference).match(/^([a-z0-9-]+):([1-9][0-9]*)$/);
  if (!match || !sourceById.has(match[1])) return "INVALID_SOURCE_REF";
  const source = sourceById.get(match[1]);
  const text = gitText(["show", expectedBaseOid + ":" + source.path]);
  return (text.split("\n")[Number(match[2]) - 1] || "").trim() === "" ? "EMPTY_SOURCE_LINE" : "PASS";
}

function storageEntryRefResult(reference, sourceById, sourceTextById, tauriCommands, mcpCapabilities, holdIds) {
  if (typeof reference !== "string" || reference.trim() !== reference) return "STORAGE_ENTRY_REF_SHAPE";
  if (reference.startsWith("SOURCE::")) {
    const remainder = reference.slice("SOURCE::".length);
    const separator = remainder.indexOf("::");
    if (separator <= 0 || separator === remainder.length - 2) return "STORAGE_SOURCE_REF_SHAPE";
    const sourceId = remainder.slice(0, separator);
    const symbol = remainder.slice(separator + 2);
    if (!sourceById.has(sourceId) || !/^[A-Za-z_][A-Za-z0-9_]*$/.test(symbol)) return "STORAGE_SOURCE_REF_UNKNOWN";
    const sourceText = sourceTextById.get(sourceId) || "";
    return new RegExp("\\b" + escapeRegExp(symbol) + "\\b").test(sourceText) ? "PASS" : "STORAGE_SOURCE_SYMBOL_UNKNOWN";
  }
  if (reference.startsWith("TAURI::")) {
    return tauriCommands.has(reference.slice("TAURI::".length)) ? "PASS" : "STORAGE_TAURI_REF_UNKNOWN";
  }
  if (reference.startsWith("MCP::")) {
    return mcpCapabilities.has(reference.slice("MCP::".length)) ? "PASS" : "STORAGE_MCP_REF_UNKNOWN";
  }
  if (reference.startsWith("HOLD::")) {
    return holdIds.has(reference.slice("HOLD::".length)) ? "PASS" : "STORAGE_HOLD_REF_UNKNOWN";
  }
  return "STORAGE_ENTRY_REF_SHAPE";
}

const requiredArtifacts = [
  "docs/contracts/README.md",
  "docs/contracts/manifest.v1.json",
  "docs/contracts/source-opening-manifest-v1.json",
  "docs/contracts/entrypoint-inventory-v1.json",
  "docs/contracts/legacy-migration-inventory-v1.json",
  "docs/contracts/storage-opening-inventory-v1.json",
  "docs/contracts/open-design-holds-v1.json",
  "docs/contracts/m1-test-matrix-v1.md",
  "docs/contracts/m2-shadow-write-parity-rollback-input-v1.json",
  "docs/contracts/fixtures/syn-fnd-001/contract-cases-v1.json",
  "docs/contracts/fixtures/syn-fnd-001/owner-dag-cases-v1.json",
  "docs/contracts/fixtures/syn-fnd-001/forbidden-field-cases-v1.json",
  "docs/contracts/fixtures/syn-fnd-001/inventory-cases-v1.json",
  "docs/contracts/fixtures/syn-fnd-001/m2-input-cases-v1.json",
  "docs/contracts/fixtures/syn-fnd-001/migration-cases-v1.json",
  "docs/contracts/fixtures/syn-fnd-001/document-shape-cases-v1.json",
  "docs/contracts/fixtures/syn-fnd-001/runner-definition-cases-v1.json",
  "docs/contracts/verify-syn-fnd-001.mjs"
];
for (const artifact of requiredArtifacts) {
  check(existsSync(resolve(repoRoot, artifact)), "ARTIFACT_MISSING", artifact);
}

const manifest = loadJson("docs/contracts/manifest.v1.json");
const sourceManifest = loadJson("docs/contracts/source-opening-manifest-v1.json");
const entryInventory = loadJson("docs/contracts/entrypoint-inventory-v1.json");
const migrationInventory = loadJson("docs/contracts/legacy-migration-inventory-v1.json");
const storageInventory = loadJson("docs/contracts/storage-opening-inventory-v1.json");
const holdRegistry = loadJson("docs/contracts/open-design-holds-v1.json");
const m2Input = loadJson("docs/contracts/m2-shadow-write-parity-rollback-input-v1.json");
const contractFixtures = loadJson("docs/contracts/fixtures/syn-fnd-001/contract-cases-v1.json");
const ownerFixtures = loadJson("docs/contracts/fixtures/syn-fnd-001/owner-dag-cases-v1.json");
const forbiddenFixtures = loadJson("docs/contracts/fixtures/syn-fnd-001/forbidden-field-cases-v1.json");
const inventoryFixtures = loadJson("docs/contracts/fixtures/syn-fnd-001/inventory-cases-v1.json");
const m2Fixtures = loadJson("docs/contracts/fixtures/syn-fnd-001/m2-input-cases-v1.json");
const migrationFixtures = loadJson("docs/contracts/fixtures/syn-fnd-001/migration-cases-v1.json");
const documentFixtures = loadJson("docs/contracts/fixtures/syn-fnd-001/document-shape-cases-v1.json");
const runnerFixtures = loadJson("docs/contracts/fixtures/syn-fnd-001/runner-definition-cases-v1.json");

const expectedForbiddenKeys = [
  "secret", "secret_value", "credential", "credential_value", "password", "token", "access_token",
  "refresh_token", "api_key", "private_key", "client_secret", "transcript", "raw_transcript",
  "transcript_body", "tool_output", "full_tool_output", "raw_tool_output", "tool_result",
  "tool_response", "tool_call_output", "function_output", "prompt", "prompt_body", "provider_response",
  "provider_payload", "provider_output", "provider_result", "model_response", "model_output", "model_result",
  "stdout", "stderr", "authorization", "proxy_authorization", "cookie", "set_cookie", "error_stack",
  "stack_trace", "exception_stack"
];
const expectedAllowedReferenceKeys = [
  "credential_ref", "secret_ref", "transcript_ref", "tool_output_ref", "payload_ref", "prompt_ref",
  "provider_response_ref", "stdout_ref", "stderr_ref", "provider_handle_ref"
];
const expectedSensitiveSentinels = [
  "SYN_FND_SECRET_SENTINEL", "SYN_FND_RAW_TRANSCRIPT_SENTINEL", "SYN_FND_TOOL_OUTPUT_SENTINEL"
];
const expectedM2ForbiddenKeys = [
  "table_name", "ddl", "sql", "index_name", "unique_index", "lease_seconds", "retry_count", "backoff_ms",
  "cutover_at", "provider_name", "secret_value", "live_store_path", "primary_store", "delete_legacy_at",
  "quarantine_table", "rollback_script"
];
check(sameArray(asArray(forbiddenFixtures.forbidden_keys).map(canonicalKey), expectedForbiddenKeys),
  "SECURITY_FORBIDDEN_KEYS_DRIFT", "fixture");
check(sameArray(asArray(forbiddenFixtures.allowed_reference_keys).map(canonicalKey), expectedAllowedReferenceKeys),
  "SECURITY_ALLOWED_KEYS_DRIFT", "fixture");
check(sameArray(asArray(forbiddenFixtures.sensitive_sentinels), expectedSensitiveSentinels),
  "SECURITY_SENTINELS_DRIFT", "fixture");
const forbiddenKeys = new Set(expectedForbiddenKeys);
const allowedKeys = new Set(expectedAllowedReferenceKeys);
const sensitiveSentinels = expectedSensitiveSentinels;

function requireFixtureIds(collection, ids, code) {
  const present = new Set(asArray(collection).map((item) => item.id));
  for (const id of ids) check(present.has(id), code, id);
}

requireFixtureIds(forbiddenFixtures.cases, [
  "opaque-refs-only", "camel-access-token", "fullwidth-access-token", "punctuated-access-token",
  "provider-output", "tool-result", "model-response", "secret-sentinel-punctuation",
  "secret-sentinel-zero-width", "secret-sentinel-combining-mark", "invalid-secret-ref-value",
  "invalid-tool-output-ref-value", "authorization-header", "cookie-header", "error-stack",
  "bearer-value-under-generic-key", "zero-width-authorization-header", "combining-secret-key",
  "wrapped-bearer-value", "assigned-bearer-value", "encrypted-private-key-value",
  "dsa-private-key-value", "short-bearer-value"
], "SECURITY_FIXTURE_REQUIRED");
requireFixtureIds(documentFixtures.heading_cases, [
  "heading-in-fence", "heading-in-html-comment", "heading-short-fence-close", "heading-in-script-block",
  "heading-in-pre-block", "heading-in-style-block", "heading-in-div-block", "heading-in-multiline-div-block",
  "heading-in-lone-cr-div-block"
], "DOCUMENT_FIXTURE_REQUIRED");
requireFixtureIds(runnerFixtures.cases, [
  "comment-is-not-definition", "block-comment-is-not-definition", "raw-string-is-not-definition",
  "macro-token-is-not-definition", "macro-token-spaced-bang-is-not-definition",
  "macro-rules-spaced-bang-is-not-definition", "unicode-macro-token-is-not-definition",
  "attribute-token-is-not-definition", "declarative-macro2-token-is-not-definition",
  "trait-declaration-is-not-definition", "extern-declaration-is-not-definition"
], "RUNNER_FIXTURE_REQUIRED");
requireFixtureIds(migrationFixtures.cases, [
  "unknown-valid-quarantine", "unknown-placeholder-ref", "unknown-missing-owner"
], "MIGRATION_FIXTURE_REQUIRED");
requireFixtureIds(m2Fixtures.cases, [
  "premature-table-top-level", "premature-ddl-deep", "premature-lease-camel",
  "premature-fullwidth-path", "premature-provider-punctuation", "interface-unknown-implementation-note"
], "M2_FIXTURE_REQUIRED");
requireFixtureIds(m2Fixtures.artifact_cases, [
  "artifact-baseline", "artifact-root-extra-key", "artifact-interface-extra-key",
  "artifact-ownership-concrete-ddl", "artifact-shadow-concrete-lease",
  "artifact-classification-concrete-ddl", "artifact-rollback-provider-selection",
  "artifact-ownership-comment-obfuscated-ddl", "artifact-outbox-owner-concrete-ddl",
  "artifact-ownership-temp-table-ddl", "artifact-ownership-unique-index-ddl",
  "artifact-pragma-journal-mode", "artifact-attach-database", "artifact-vacuum-into",
  "artifact-reindex", "artifact-analyze", "artifact-vacuum-schema-into",
  "artifact-attach-bracket-alias", "artifact-reindex-bracket-name", "artifact-analyze-bracket-name",
  "artifact-attach-expression"
], "M2_ARTIFACT_FIXTURE_REQUIRED");

check(manifest.schema === "syn.contract-manifest.v1", "MANIFEST_SCHEMA", manifest.schema);
check(manifest.task_id === "SYN-FND-001-R1", "TASK_ID", manifest.task_id);
check(manifest.contract_status === "FROZEN_V1", "CONTRACT_STATUS", manifest.contract_status);
check(manifest.evidence_level === "STATIC_OPENING_ONLY", "EVIDENCE_LEVEL", manifest.evidence_level);
check(manifest.opening && manifest.opening.base_oid === expectedBaseOid, "BASE_OID", "contract manifest");
check(manifest.opening && manifest.opening.tauri_command_count === 171, "OPENING_COUNT", "tauri");
check(manifest.opening && manifest.opening.supervisor_mcp_capability_count === 8, "OPENING_COUNT", "mcp");

const expectedContractIds = [
  "identity-scope-v1", "command-v1", "event-audit-outbox-v1", "role-session-v1", "handoff-v1",
  "attention-decision-v1", "project-orchestration-v1", "memory-personal-model-v1",
  "connector-capability-v1", "object-ref-navigation-v1"
];

const expectedDependencies = {
  "identity-scope-v1": [],
  "command-v1": ["identity-scope-v1"],
  "event-audit-outbox-v1": ["identity-scope-v1", "command-v1"],
  "role-session-v1": ["identity-scope-v1", "event-audit-outbox-v1"],
  "handoff-v1": ["identity-scope-v1", "command-v1", "event-audit-outbox-v1", "role-session-v1", "object-ref-navigation-v1"],
  "attention-decision-v1": ["identity-scope-v1", "command-v1", "event-audit-outbox-v1", "handoff-v1", "object-ref-navigation-v1"],
  "project-orchestration-v1": ["identity-scope-v1", "command-v1", "event-audit-outbox-v1", "role-session-v1", "handoff-v1", "attention-decision-v1", "object-ref-navigation-v1"],
  "memory-personal-model-v1": ["identity-scope-v1", "command-v1", "event-audit-outbox-v1", "attention-decision-v1", "object-ref-navigation-v1"],
  "connector-capability-v1": ["identity-scope-v1", "command-v1", "event-audit-outbox-v1", "attention-decision-v1", "object-ref-navigation-v1"],
  "object-ref-navigation-v1": ["identity-scope-v1", "command-v1", "event-audit-outbox-v1", "role-session-v1"]
};

const expectedExports = {
  "identity-scope-v1": ["ActorId", "ProjectId", "ProjectRootRef", "ScopeRef", "RoleRef", "CurrentObjectRef", "ExecutionChannel", "PermissionProfile", "PermissionSnapshotRef", "ScopeChain", "IdentitySnapshot"],
  "command-v1": ["CommandId", "CorrelationId", "CausationId", "TraceContext", "CommandEnvelope", "CommandReceipt"],
  "event-audit-outbox-v1": ["EventId", "WorkbenchEventEnvelope", "AuditRecord", "OutboxItem", "OutboxLease", "ProjectionCheckpoint", "CurrentSnapshot", "UnknownQuarantineRef"],
  "role-session-v1": ["RoleSessionId", "RoleSession", "TurnId", "Turn", "ProviderHandleRef", "ConversationContextRef", "SessionBinding"],
  "handoff-v1": ["HandoffId", "HandoffPermissionRequest", "Handoff", "HandoffReceipt"],
  "attention-decision-v1": ["InboxItem", "OpenLoop", "OpenLoopTodoRelation", "Todo", "Notification", "Reminder", "DecisionRequest", "DecisionRequestProjection"],
  "project-orchestration-v1": ["OrchestrationId", "Proposal", "PlanAuthorization", "WorkflowRun", "WorkItem", "PreparedAttempt", "ExecutionGrant", "Dispatch", "ExecutedReport", "ManualOfflineClaim", "Review", "AuthorizationDecision", "ResultUserDecision", "ProjectSummary"],
  "memory-personal-model-v1": ["Observation", "MemoryCandidate", "FormalMemory", "PersonalFact", "ModelAssertion", "MemoryPolicyResult", "MemoryConflict", "MemoryVersion"],
  "connector-capability-v1": ["ConnectorDefinition", "ConnectionAccount", "CredentialRef", "CapabilityId", "CapabilityGrant", "SyncCursor", "InboundItem", "ActionRequest", "ActionResult"],
  "object-ref-navigation-v1": ["ObjectRef", "ObjectKind", "NavigationIntent", "NavigationReceipt", "DeepLinkResolution"]
};

const expectedOwners = {
  ActorId:"identity_scope_kernel", ProjectId:"project_index", ProjectRootRef:"project_index",
  ScopeRef:"identity_scope_kernel", RoleRef:"role_catalog", CurrentObjectRef:"identity_scope_kernel",
  ExecutionChannel:"identity_scope_kernel", PermissionProfile:"permission_policy_catalog",
  PermissionSnapshotRef:"permission_snapshot_authority", ScopeChain:"identity_scope_kernel",
  IdentitySnapshot:"identity_scope_kernel", CommandId:"command_gateway", CorrelationId:"command_gateway",
  CausationId:"command_gateway", TraceContext:"command_gateway", CommandEnvelope:"command_gateway",
  CommandReceipt:"application_command_receipt_ledger", EventId:"event_ledger_repository",
  WorkbenchEventEnvelope:"event_ledger_repository", AuditRecord:"audit_ledger_repository",
  OutboxItem:"outbox_repository", OutboxLease:"outbox_claimer", ProjectionCheckpoint:"PROJECTOR_ID",
  CurrentSnapshot:"source_domain_projector", UnknownQuarantineRef:"unknown_quarantine_repository",
  RoleSessionId:"conversation_domain", RoleSession:"conversation_domain", TurnId:"role_session_aggregate",
  Turn:"role_session_aggregate", ProviderHandleRef:"conversation_role_session_repository",
  ConversationContextRef:"conversation_context_projector", SessionBinding:"conversation_role_session_repository",
  HandoffId:"handoff_aggregate", HandoffPermissionRequest:"handoff_aggregate", Handoff:"handoff_aggregate",
  HandoffReceipt:"handoff_aggregate", InboxItem:"personal_inbox_projector",
  OpenLoop:"secretary_coordination_domain", OpenLoopTodoRelation:"attention_relation_projector",
  Todo:"personal_action_aggregate", Notification:"notification_domain", Reminder:"reminder_domain",
  DecisionRequest:"SOURCE_OWNER_REF", DecisionRequestProjection:"attention_decision_projector",
  OrchestrationId:"project_orchestration", Proposal:"project_orchestration",
  PlanAuthorization:"project_orchestration", WorkflowRun:"execution_aggregate", WorkItem:"execution_aggregate",
  PreparedAttempt:"execution_aggregate", ExecutionGrant:"project_orchestration", Dispatch:"execution_aggregate",
  ExecutedReport:"claim_ledger", ManualOfflineClaim:"claim_ledger", Review:"review_domain",
  AuthorizationDecision:"project_authorization", ResultUserDecision:"review_domain", ProjectSummary:"project_projector",
  Observation:"memory_capture", MemoryCandidate:"memory_governance", FormalMemory:"formal_memory_repository",
  PersonalFact:"personal_fact_domain", ModelAssertion:"personal_model_domain",
  MemoryPolicyResult:"memory_governance", MemoryConflict:"memory_governance", MemoryVersion:"formal_memory_repository",
  ConnectorDefinition:"connector_registry", ConnectionAccount:"connector_domain", CredentialRef:"protected_vault",
  CapabilityId:"connector_registry", CapabilityGrant:"policy_grant_domain", SyncCursor:"connector_sync_repository",
  InboundItem:"connector_domain", ActionRequest:"action_domain", ActionResult:"action_domain",
  ObjectRef:"object_ref_registry", ObjectKind:"object_ref_registry",
  NavigationIntent:"object_navigation_application", NavigationReceipt:"object_navigation_application",
  DeepLinkResolution:"object_resolution_service"
};

const expectedPartial = new Set([
  "ProjectId", "ProjectRootRef", "CommandId", "CorrelationId", "CausationId", "Proposal",
  "PlanAuthorization", "WorkflowRun", "WorkItem", "PreparedAttempt", "Dispatch", "ExecutedReport",
  "Review", "AuthorizationDecision", "ResultUserDecision", "ProjectSummary", "Observation",
  "MemoryCandidate", "FormalMemory", "MemoryVersion"
]);

const criticalFields = {
  CurrentObjectRef:["object_type","object_id","source_owner_ref","scope_ref","binding_revision","binding_source_ref"],
  ObjectRef:["object_kind","object_id","scope_ref","source_owner_ref","source_ref","object_revision"],
  CommandEnvelope:["command_id","actor_id","scope_ref","current_object_ref","execution_channel","permission_snapshot_ref","expected_revision","idempotency_key","correlation_id","payload_ref","payload_hash"],
  CommandReceipt:["command_id","idempotency_key","request_hash","actor_id","scope_ref","current_object_ref","policy_decision_ref","status","result_hash","error_code"],
  WorkbenchEventEnvelope:["correlation_id","causation_id","schema_version","source_ref","sensitivity","summary_ref","payload_ref","payload_hash"],
  AuditRecord:["decision","actor_id","scope_ref","reason_code","scrub_result","source_refs"],
  OutboxItem:["owning_command_id","owning_command_receipt_ref","effect_id","capability_id","payload_ref","payload_hash","result_command_type","idempotency_key","correlation_id","status"],
  CurrentSnapshot:["object_ref","object_revision","source_watermark","snapshot_hash"],
  ProjectionCheckpoint:["projector_id","projector_version","last_event_id","source_watermark","status","error_receipt_ref"],
  RoleSession:["role_session_id","role_ref","scope_ref","current_object_ref","execution_channel","permission_snapshot_ref","owner_fingerprint","status","revision"],
  Handoff:["handoff_id","from_role_session_id","to_recipient_ref","scope_ref","requested_outcome_ref","object_refs","risk_class","permission_request","status","revision"],
  OpenLoop:["source_owner_ref","source_object_ref","source_revision","reason_code","priority_basis","coordination_state","dedupe_key","todo_ref"],
  Todo:["personal_scope_ref","created_by_actor_id","source_kind","status","revision","idempotency_key"],
  DecisionRequest:["source_owner_ref","source_object_ref","source_revision","required_actor_ref","required_scope_ref","decision_command_type","status","idempotency_key"],
  AuthorizationDecision:["authorization_decision_id","proposal_id","proposal_revision","deciding_actor_id","decision","idempotency_key"],
  ResultUserDecision:["result_decision_id","workflow_run_id","review_id","report_ref","deciding_actor_id","decision","result_revision","idempotency_key"],
  ExecutionGrant:["grant_id","authorization_id","authorization_revision","attempt_id","principal_actor_id","worker_role_session_id","scope_fingerprint","allowed_commands","cwd_ref","write_root_refs","object_refs","policy_decision_ref","expires_at","revoked_at","idempotency_key","effect_key","grant_hash"],
  ExecutedReport:["report_kind","project_id","workflow_run_id","work_item_id","node_id","dispatch_id","attempt_id","grant_id","worker_role_session_id","authoritative_execution_receipt_ref","authenticated_actor_id","report_hash","observed_attempt_state","claim_status"],
  ManualOfflineClaim:["report_kind","authenticated_submitter_id","source_refs","evidence_refs","claim_hash","claim_status"],
  PersonalFact:["subject_actor_id","statement_hash","provenance_kind","source_refs","valid_from","valid_until","status","correction_of_version_id"],
  ModelAssertion:["subject_actor_id","inference_hash","evidence_refs","confidence","valid_from","valid_until","contestability","status","supersedes_version_id"],
  CredentialRef:["credential_ref","credential_kind","status","rotation_revision"],
  CapabilityGrant:["grant_kind","subject_actor_id","role_ref","scope_ref","connection_account_id","capability_kind","constraints_ref","confirmation_ref","expires_at","revoked_at","status","grant_hash"],
  ActionRequest:["capability_grant_id","actor_id","scope_ref","target_object_ref","confirmation_ref","effect_id","payload_ref","payload_hash","outbox_item_id","status","idempotency_key"],
  ActionResult:["action_request_id","effect_id","external_receipt_ref","external_receipt_hash","readback_status","result_command_receipt_ref","status"]
};

const contracts = asArray(manifest.contracts);
check(contracts.length === 10, "CONTRACT_COUNT", contracts.length);
check(sameArray(contracts.map((item) => item.id), expectedContractIds), "CONTRACT_ORDER", contracts.map((item) => item.id).join(","));
check(dagResult(contracts) === "PASS", "CONTRACT_DAG", dagResult(contracts));
const contractIds = new Set(contracts.map((item) => item.id));
check(unique(contracts.map((item) => item.schema_authority)), "DUPLICATE_CONTRACT_AUTHORITY", "schema authorities");

const allManifestExports = contracts.flatMap((contract) => asArray(contract.exports));
check(unique(allManifestExports), "DUPLICATE_EXPORT_OWNER", allManifestExports.join(","));
check(sameSet(allManifestExports, Object.keys(expectedOwners)), "EXPECTED_EXPORT_CLOSURE", "manifest exports differ from hardcoded set");

for (const contract of contracts) {
  check(contract.path === contract.id + ".md", "CONTRACT_PATH", contract.id);
  check(sameArray(contract.dependencies, expectedDependencies[contract.id]), "CONTRACT_DEPENDENCIES", contract.id);
  check(sameArray(contract.exports, expectedExports[contract.id]), "CONTRACT_EXPORTS", contract.id);
  const hasPartial = contract.exports.some((name) => expectedPartial.has(name));
  const expectedImplementation = hasPartial ? "MIXED_OPENING" : "ABSENT";
  check(contract.implementation_status === expectedImplementation, "IMPLEMENTATION_STATUS", contract.id + ":" + contract.implementation_status);
}

const requiredSections = [
  "contract.owner", "contract.schema", "contract.truth-source", "contract.legal-states",
  "contract.cross-scope", "contract.formal-actions", "contract.events", "contract.audit",
  "contract.outbox", "contract.sensitivity", "contract.idempotency", "contract.failure",
  "contract.rollback", "contract.compatibility", "contract.fixtures", "contract.non-goals",
  "contract.holds"
];
const actionFields = ["id", "command", "policy", "state", "event", "audit", "outbox", "failure"];
const outboxModes = new Set(["NONE", "OPTIONAL", "REQUIRED"]);
const contractHoldRefs = new Set();
const schemaByContract = new Map();
const typeSchemas = new Map();
const actionsByContract = new Map();
let actionCount = 0;

for (const contract of contracts) {
  const relativePath = "docs/contracts/" + contract.path;
  const text = readRepoText(relativePath);
  const frontResult = parseFrontMatterResult(text);
  check(frontResult.code === "PASS", frontResult.code, relativePath + ":" + (frontResult.detail ?? ""));
  const front = frontResult.value;
  check(front.contract_id === contract.id, "CONTRACT_FRONT_ID", contract.id);
  check(front.version === 1, "CONTRACT_VERSION", contract.id);
  check(front.status === "FROZEN_V1", "CONTRACT_FRONT_STATUS", contract.id);
  check(front.evidence_level === "STATIC_OPENING_ONLY", "CONTRACT_FRONT_EVIDENCE", contract.id);
  check(front.schema_authority === contract.schema_authority, "CONTRACT_FRONT_AUTHORITY", contract.id);
  check(sameArray(front.dependencies, contract.dependencies), "CONTRACT_FRONT_DEPENDENCY", contract.id);
  check(Array.isArray(front.hold_refs), "CONTRACT_FRONT_HOLDS", contract.id);
  for (const ref of asArray(front.hold_refs)) contractHoldRefs.add(ref);
  for (const section of requiredSections) {
    const result = headingResult(text, section);
    check(result === "PASS", result, contract.id + ":" + section);
    check(sectionBody(text, section) !== "", "CONTRACT_SECTION_EMPTY", contract.id + ":" + section);
  }

  const schemaBlocks = fencedJsonBlocks(text, "contract-schema");
  check(schemaBlocks.length === 1, "CONTRACT_SCHEMA_COUNT", contract.id + ":" + schemaBlocks.length);
  let schema = {};
  if (schemaBlocks.length === 1) {
    try {
      schema = JSON.parse(schemaBlocks[0]);
    } catch (error) {
      fail("CONTRACT_SCHEMA_JSON", contract.id + ":" + error.message);
    }
  }
  schemaByContract.set(contract.id, schema);
  check(schema.schema_authority === contract.schema_authority, "CONTRACT_SCHEMA_AUTHORITY", contract.id);
  check(sameArray(asArray(schema.exports).map((item) => item.name), contract.exports), "CONTRACT_SCHEMA_EXPORTS", contract.id);
  for (const type of asArray(schema.exports)) {
    check(typeof type.name === "string" && type.name.trim() !== "", "TYPE_NAME", contract.id);
    check(!typeSchemas.has(type.name), "TYPE_DUPLICATE", type.name);
    typeSchemas.set(type.name, { ...type, contract_id: contract.id });
    check(type.domain_owner === expectedOwners[type.name], "TYPE_OWNER", type.name + ":" + type.domain_owner);
    check(allNonEmptyStrings(type.required_fields), "TYPE_FIELDS", type.name);
    check(unique(asArray(type.required_fields)), "TYPE_FIELD_DUPLICATE", type.name);
    const expectedStatus = expectedPartial.has(type.name) ? "PARTIAL_LEGACY" : "ABSENT";
    check(type.opening_status === expectedStatus, "TYPE_OPENING_STATUS", type.name + ":" + type.opening_status);
    for (const field of asArray(criticalFields[type.name])) {
      check(type.required_fields.includes(field), "TYPE_CRITICAL_FIELD", type.name + ":" + field);
    }
  }

  const actionBlocks = fencedJsonBlocks(text, "action-flow");
  check(actionBlocks.length === 1, "ACTION_FLOW_COUNT", contract.id + ":" + actionBlocks.length);
  let actions = [];
  if (actionBlocks.length === 1) {
    try {
      actions = JSON.parse(actionBlocks[0]);
    } catch (error) {
      fail("ACTION_FLOW_JSON", contract.id + ":" + error.message);
    }
  }
  actionsByContract.set(contract.id, actions);
  check(Array.isArray(actions) && actions.length > 0, "ACTION_FLOW_EMPTY", contract.id);
  check(unique(asArray(actions).map((item) => item.id)), "ACTION_ID_DUPLICATE", contract.id);
  for (const action of asArray(actions)) {
    actionCount += 1;
    for (const field of actionFields) {
      const value = action[field];
      check(value !== undefined && value !== null &&
        (typeof value !== "string" || value.trim() !== ""), "ACTION_FIELD", contract.id + ":" + action.id + ":" + field);
    }
    check(action.failure === "FAIL_CLOSED", "ACTION_FAILURE", contract.id + ":" + action.id);
    check(action.outbox && outboxModes.has(action.outbox.mode) &&
      typeof action.outbox.reason === "string" && action.outbox.reason.trim() !== "", "ACTION_OUTBOX", contract.id + ":" + action.id);
    const sensitive = findSensitiveMaterial(action, forbiddenKeys, allowedKeys, sensitiveSentinels);
    check(sensitive === null, sensitive ? sensitive.code : "ACTION_SENSITIVE", contract.id + ":" + (sensitive ? sensitive.path : ""));
  }
  const schemaSensitive = findSensitiveMaterial(schema, forbiddenKeys, allowedKeys, sensitiveSentinels);
  check(schemaSensitive === null, schemaSensitive ? schemaSensitive.code : "SCHEMA_SENSITIVE", contract.id + ":" + (schemaSensitive ? schemaSensitive.path : ""));
}

check(actionCount >= 35, "ACTION_COUNT", actionCount);
check(typeSchemas.size === allManifestExports.length, "TYPE_SCHEMA_CLOSURE", typeSchemas.size);
for (const [contractId, schema] of schemaByContract) {
  for (const imported of asArray(schema.imports)) {
    check(typeSchemas.has(imported), "TYPE_IMPORT_UNKNOWN", contractId + ":" + imported);
    if (typeSchemas.has(imported)) {
      const sourceContract = typeSchemas.get(imported).contract_id;
      const seen = new Set();
      const stack = [...expectedDependencies[contractId]];
      while (stack.length > 0) {
        const current = stack.pop();
        if (seen.has(current)) continue;
        seen.add(current);
        stack.push(...expectedDependencies[current]);
      }
      check(seen.has(sourceContract), "TYPE_IMPORT_WITHOUT_DEPENDENCY", contractId + ":" + imported + ":" + sourceContract);
    }
  }
}

function exactObject(actual, expected) {
  return JSON.stringify(actual) === JSON.stringify(expected);
}

function transitionTokens(state) {
  const parts = String(state).split("->");
  return parts.length === 2 ? parts.flatMap((part) => part.split("|")) : [];
}

function assertOwnedStateActions(contractId) {
  const legal = schemaByContract.get(contractId)?.legal_states || {};
  for (const action of asArray(actionsByContract.get(contractId))) {
    check(typeof action.state_owner === "string" && action.state_owner.trim() !== "",
      "ACTION_STATE_OWNER", contractId + ":" + action.id);
    check(typeof action.state_target === "string" && action.state_target.trim() !== "",
      "ACTION_STATE_TARGET", contractId + ":" + action.id);
    check(allNonEmptyStrings(action.preconditions), "ACTION_PRECONDITIONS", contractId + ":" + action.id);
    const [typeName, fieldName, ...extra] = String(action.state_target).split(".");
    const type = typeSchemas.get(typeName);
    check(extra.length === 0 && type?.contract_id === contractId,
      "ACTION_STATE_TARGET_TYPE", contractId + ":" + action.id + ":" + action.state_target);
    check(type?.required_fields.includes(fieldName),
      "ACTION_STATE_TARGET_FIELD", contractId + ":" + action.id + ":" + action.state_target);
    check(action.state_owner === type?.domain_owner, "ACTION_STATE_OWNER_DRIFT", contractId + ":" + action.id);
    check(Array.isArray(legal[action.state_target]),
      "ACTION_LEGAL_STATES_MISSING", contractId + ":" + action.state_target);
    for (const token of transitionTokens(action.state)) {
      if (token !== "NONE") {
        check(asArray(legal[action.state_target]).includes(token),
          "ACTION_ILLEGAL_STATE", contractId + ":" + action.id + ":" + token);
      }
    }
  }
}

function actionByCommand(contractId, command) {
  return asArray(actionsByContract.get(contractId)).find((item) => item.command === command);
}

function expectAction(contractId, command, expected) {
  const action = actionByCommand(contractId, command);
  check(action !== undefined, "ACTION_REQUIRED", contractId + ":" + command);
  for (const [field, value] of Object.entries(expected)) {
    check(action?.[field] === value, "ACTION_SEMANTIC_DRIFT", contractId + ":" + command + ":" + field);
  }
}

const expectedProjectLegalStates = {
  "Proposal.status":["DRAFT","SUBMITTED","WITHDRAWN","SUPERSEDED"],
  "AuthorizationDecision.decision":["APPROVED","REJECTED"],
  "PlanAuthorization.status":["ACTIVE","REVOKED","EXPIRED","SUPERSEDED","QUARANTINED"],
  "WorkflowRun.status":["CREATED","ACTIVE","SUCCEEDED","FAILED","CANCELLED","TIMED_OUT","UNKNOWN_READBACK"],
  "WorkItem.status":["READY","ACTIVE","SUCCEEDED","FAILED","CANCELLED","BLOCKED"],
  "PreparedAttempt.state":["PREPARED_NON_RUNNABLE","GRANT_PENDING_NON_RUNNABLE","GRANT_READY_NON_RUNNABLE","DISPATCHED","RUNNING","SUCCEEDED","FAILED","CANCELLED","TIMED_OUT","UNKNOWN_READBACK"],
  "ExecutionGrant.status":["MINT_PENDING","ACTIVE","REVOKED","EXPIRED","QUARANTINED"],
  "Dispatch.state":["PENDING_DELIVERY","DISPATCHED","FAILED","CANCELLED","UNKNOWN_READBACK"],
  "ExecutedReport.claim_status":["RECORDED_UNVERIFIED","QUARANTINED","SUPERSEDED"],
  "ManualOfflineClaim.claim_status":["RECORDED_UNVERIFIED","QUARANTINED","SUPERSEDED"],
  "Review.review_outcome":["VERIFIED","REJECTED","NEEDS_READBACK","UNKNOWN"],
  "ResultUserDecision.decision":["ACCEPTED_RESULT","REJECTED_RESULT","NEEDS_FOLLOWUP"]
};
const expectedAttentionLegalStates = {
  "OpenLoop.coordination_state":["OPEN","SNOOZED","DISMISSED","CLOSED","QUARANTINED"],
  "Todo.status":["OPEN","COMPLETED","CANCELLED","ARCHIVED"],
  "DecisionRequestProjection.projected_status":["PENDING","ROUTED","STALE","DENIED","ANSWERED","CANCELLED","EXPIRED","SUPERSEDED"]
};
const projectSchema = schemaByContract.get("project-orchestration-v1");
const attentionSchema = schemaByContract.get("attention-decision-v1");
check(exactObject(projectSchema.legal_states, expectedProjectLegalStates), "PROJECT_LEGAL_STATES", "exact");
check(exactObject(attentionSchema.legal_states, expectedAttentionLegalStates), "ATTENTION_LEGAL_STATES", "exact");
assertOwnedStateActions("project-orchestration-v1");
assertOwnedStateActions("attention-decision-v1");
const projectActions = asArray(actionsByContract.get("project-orchestration-v1"));
for (const target of Object.keys(expectedProjectLegalStates)) {
  check(projectActions.some((action) => action.state_target === target), "PROJECT_STATE_TARGET_COVERAGE", target);
}
const createGroupId = "create-authorized-run-and-prepared-attempt";
const expectedCreateGroup = {
  command:"CreateAuthorizedRunAndPreparedAttempt",
  facets:["WorkflowRun.status","WorkItem.status","PreparedAttempt.state"],
  commit_semantics:"ALL_OR_NONE",
  receipt_semantics:"ONE_SHARED_COMMAND_RECEIPT",
  event_semantics:"ONE_SHARED_EVENT"
};
check(exactObject(projectSchema.atomic_transition_groups?.[createGroupId], expectedCreateGroup),
  "PROJECT_ATOMIC_GROUP", createGroupId);
const createFacets = projectActions.filter((action) => action.command === "CreateAuthorizedRunAndPreparedAttempt");
check(createFacets.length === 3, "PROJECT_ATOMIC_FACET_COUNT", createFacets.length);
check(sameArray(createFacets.map((action) => action.state_target), expectedCreateGroup.facets),
  "PROJECT_ATOMIC_FACET_TARGETS", createFacets.map((action) => action.state_target).join(","));
check(sameArray(createFacets.map((action) => action.state),
  ["NONE->CREATED", "NONE->READY", "NONE->PREPARED_NON_RUNNABLE"]), "PROJECT_ATOMIC_FACET_STATES", "states");
for (const action of createFacets) {
  check(action.atomic_group === createGroupId, "PROJECT_ATOMIC_GROUP_REF", action.id);
  check(action.event === "AuthorizedRunAndPreparedAttemptCreated", "PROJECT_ATOMIC_SHARED_EVENT", action.id);
  check(action.audit === "SCRUBBED_AUTHORIZED_RUN_RECORD", "PROJECT_ATOMIC_SHARED_AUDIT", action.id);
  check(action.outbox?.mode === "NONE", "PROJECT_ATOMIC_OUTBOX", action.id);
}

check(!expectedExports["identity-scope-v1"].includes("ObjectRef"), "OBJECT_OWNER_SPLIT", "identity exports ObjectRef");
check(!typeSchemas.get("CurrentObjectRef").required_fields.includes("object_ref"), "CURRENT_OBJECT_NESTED_OWNER", "object_ref");
check(expectedExports["object-ref-navigation-v1"].includes("ObjectRef") &&
  !expectedExports["object-ref-navigation-v1"].includes("CurrentObjectRef"), "OBJECT_OWNER_SPLIT", "object contract");

const todoSchema = typeSchemas.get("Todo");
check(todoSchema.constants && todoSchema.constants.source_kind === "STANDALONE_USER_CREATED", "TODO_SOURCE_KIND", "constant");
check(attentionSchema.open_loop_todo_relation && attentionSchema.open_loop_todo_relation.no_auto_creation === true,
  "OPEN_LOOP_TODO_RULE", "no_auto_creation");
check(attentionSchema.open_loop_todo_relation && attentionSchema.open_loop_todo_relation.independent_lifecycle === true,
  "OPEN_LOOP_TODO_RULE", "independent_lifecycle");
for (const key of ["cardinality", "todo_creation_rule", "source_action_rule", "no_second_truth"]) {
  check(attentionSchema.open_loop_todo_relation && typeof attentionSchema.open_loop_todo_relation[key] === "string" &&
    attentionSchema.open_loop_todo_relation[key].trim() !== "", "OPEN_LOOP_TODO_RULE", key);
}
check(typeSchemas.get("DecisionRequest").domain_owner === "SOURCE_OWNER_REF" &&
  typeSchemas.get("DecisionRequest").required_fields.includes("source_owner_ref"), "DECISION_SOURCE_OWNER", "DecisionRequest");

for (const key of ["decision_type_invariant", "report_type_invariant"]) {
  check(typeof projectSchema[key] === "string" && projectSchema[key].trim() !== "", "PROJECT_INVARIANT", key);
}
check(projectSchema.orchestration_identity && projectSchema.orchestration_identity.relationship === "PARENT_CHILD",
  "ORCHESTRATION_IDENTITY", "relationship");
check(typeSchemas.get("AuthorizationDecision").constants.idempotency_namespace === "PLAN_AUTHORIZATION",
  "DECISION_NAMESPACE", "AuthorizationDecision");
check(typeSchemas.get("ResultUserDecision").constants.idempotency_namespace === "RESULT_ACCEPTANCE",
  "DECISION_NAMESPACE", "ResultUserDecision");
const manualForbidden = asArray(typeSchemas.get("ManualOfflineClaim").forbidden_fields);
for (const field of ["dispatch_id", "attempt_id", "grant_id", "worker_role_session_id", "authoritative_execution_receipt_ref"]) {
  check(manualForbidden.includes(field) && !typeSchemas.get("ManualOfflineClaim").required_fields.includes(field),
    "MANUAL_CLAIM_EXECUTION_JOIN", field);
}
check(sameArray(typeSchemas.get("ExecutedReport").constants.acceptable_attempt_states,
  ["SUCCEEDED", "FAILED", "CANCELLED", "TIMED_OUT", "UNKNOWN_READBACK"]), "REPORT_ACCEPTABLE_STATES", "ExecutedReport");
check(typeSchemas.get("CapabilityGrant").constants.grant_kind === "CONNECTOR_CAPABILITY", "CAPABILITY_GRANT_KIND", "constant");
check(sameArray(typeSchemas.get("CapabilityId").allowed_values, ["VIEW", "INDEX", "SYNC", "ACTION", "SECRET"]),
  "CAPABILITY_KIND_VALUES", "CapabilityId");

function actionCommands(contractId) {
  return asArray(actionsByContract.get(contractId)).map((item) => item.command);
}
for (const command of ["CreateHandoff", "AcceptHandoff", "RequestHandoffReturn", "RecordHandoffReturnResult"]) {
  check(actionCommands("handoff-v1").includes(command), "HANDOFF_ACTION_REQUIRED", command);
}
for (const action of asArray(actionsByContract.get("handoff-v1"))) {
  check(!/^ACCEPTED->.*EXPIRED/.test(action.state), "HANDOFF_ACCEPTED_EXPIRES", action.id);
  check(!/(?:ExecutionGrant|CapabilityGrant|Mint.*Grant)/.test(action.command), "HANDOFF_MINTS_GRANT", action.command);
}

const eventSchema = schemaByContract.get("event-audit-outbox-v1");
const eventActions = asArray(actionsByContract.get("event-audit-outbox-v1"));
check(!eventActions.some((item) => item.command === "ClaimOutboxItem"),
  "M1_OUTBOX_RUNTIME_COMMAND", "ClaimOutboxItem");
check(!eventActions.some((item) => /AVAILABLE|LEASED|RETRY_WAIT|DELIVERED/.test(item.state)),
  "M1_OUTBOX_RUNTIME_TRANSITION", "event actions");
check(exactObject(eventSchema.legal_states, {"OutboxItem.status":["DECLARED"]}),
  "OUTBOX_LEGAL_STATES", "exact");
check(eventSchema.outbox_boundary?.m2_owned_runtime_state_machine === true,
  "OUTBOX_M2_RUNTIME_OWNER", "missing");
check(sameArray(eventSchema.outbox_boundary?.forbidden_m1_fields,
  ["lease_state", "attempt_count", "next_retry_not_before"]), "OUTBOX_FORBIDDEN_M1_FIELDS", "exact");
for (const field of asArray(eventSchema.outbox_boundary?.forbidden_m1_fields)) {
  check(!typeSchemas.get("OutboxItem").required_fields.includes(field), "OUTBOX_RUNTIME_FIELD_LEAK", field);
}
const declareOutbox = actionByCommand("event-audit-outbox-v1", "DeclareExternalEffectIntent");
check(declareOutbox?.state_owner === "outbox_repository" &&
  declareOutbox?.state_target === "OutboxItem.status" && declareOutbox?.state === "NONE->DECLARED" &&
  allNonEmptyStrings(declareOutbox?.preconditions), "OUTBOX_DECLARATION_ACTION", "exact");
const expectedOutboxFields = ["outbox_item_id","owning_command_id","owning_command_receipt_ref","effect_id","capability_id","scope_ref","subject_ref","payload_ref","payload_hash","result_command_type","idempotency_key","correlation_id","status","created_at"];
check(sameArray(typeSchemas.get("OutboxItem").required_fields, expectedOutboxFields), "OUTBOX_SEMANTIC_FIELDS", "exact");
const expectedOwningBinding = {
  command_field:"owning_command_id",
  receipt_ref_field:"owning_command_receipt_ref",
  receipt_join:"CommandReceipt.command_id=OutboxItem.owning_command_id",
  matching_fields:["scope_ref","correlation_id","idempotency_key"],
  receipt_commit_status:"EXTERNAL_PENDING",
  commit_semantics:"DOMAIN_EVENT_AUDIT_RECEIPT_OUTBOX_ALL_OR_NONE"
};
check(eventSchema.outbox_boundary?.declaration_scope === "OWNING_COMMAND_UOW_FACET_ONLY",
  "OUTBOX_DECLARATION_SCOPE", "scope");
check(eventSchema.outbox_boundary?.standalone_admission === false,
  "OUTBOX_STANDALONE_ADMISSION", "must be false");
check(exactObject(eventSchema.outbox_boundary?.owning_command_binding, expectedOwningBinding),
  "OUTBOX_OWNING_BINDING", "exact");
check(declareOutbox?.command_scope === "OWNING_COMMAND_UOW_FACET_ONLY",
  "OUTBOX_DECLARATION_COMMAND_SCOPE", "exact");
for (const text of [
  "owning_command_receipt_ref resolves to CommandReceipt",
  "CommandReceipt.command_id equals owning_command_id",
  "receipt scope_ref, correlation_id, and idempotency_key equal OutboxItem",
  "domain state, event, audit, receipt, and OutboxItem commit in one unit of work"
]) {
  check(declareOutbox?.preconditions.includes(text), "OUTBOX_BINDING_PRECONDITION", text);
}

for (const [command, state] of Object.entries({
  SnoozeOpenLoop:"OPEN->SNOOZED",
  DismissOpenLoop:"OPEN|SNOOZED->DISMISSED",
  CloseOpenLoop:"OPEN|SNOOZED->CLOSED",
  ReopenOpenLoop:"CLOSED|DISMISSED->OPEN"
})) {
  expectAction("attention-decision-v1", command, {
    state_owner:"secretary_coordination_domain", state_target:"OpenLoop.coordination_state", state
  });
}
for (const [command, state] of Object.entries({
  CreateStandaloneTodo:"NONE->OPEN",
  CompleteStandaloneTodo:"OPEN->COMPLETED",
  CancelStandaloneTodo:"OPEN->CANCELLED",
  ArchiveStandaloneTodo:"COMPLETED|CANCELLED->ARCHIVED"
})) {
  expectAction("attention-decision-v1", command, {
    state_owner:"personal_action_aggregate", state_target:"Todo.status", state
  });
}
check(!expectedAttentionLegalStates["OpenLoop.coordination_state"].includes("REOPENED"),
  "ATTENTION_REOPENED_PERSISTED", "REOPENED");
for (const action of asArray(actionsByContract.get("attention-decision-v1"))) {
  check(action.command !== "AnswerDecisionRequest" && !/^PENDING->.*ANSWERED/.test(action.state),
    "ATTENTION_OWNS_SOURCE_DECISION", action.command);
  if (/OpenLoop/.test(action.command)) {
    check(action.state_target === "OpenLoop.coordination_state", "OPEN_LOOP_CROSS_OWNER", action.command);
  }
  if (/Todo/.test(action.command)) {
    check(action.state_target === "Todo.status", "TODO_CROSS_OWNER", action.command);
  }
}
for (const command of ["CreateProposal", "SubmitProposal", "RecordAuthorizationDecision", "CreatePlanAuthorization",
  "CreateAuthorizedRunAndPreparedAttempt", "BeginAttemptGrantBinding", "MintAttemptScopedGrant",
  "ConfirmGrantReadback", "ConfirmAttemptGrantBinding", "DispatchGrantedAttempt", "RecordDispatchReadback",
  "MarkAttemptDispatched", "RecordExecutionAttemptReadback", "RecordExecutedReportClaim",
  "RecordManualOfflineClaim", "ReviewExecutionClaim", "RecordResultUserDecision"]) {
  check(actionCommands("project-orchestration-v1").includes(command), "PROJECT_ACTION_REQUIRED", command);
}
check(!actionCommands("project-orchestration-v1").includes("BindWorkerReport"), "REPORT_MUTATES_ATTEMPT", "BindWorkerReport");
expectAction("project-orchestration-v1", "RecordExecutedReportClaim",
  {state_owner:"claim_ledger", state_target:"ExecutedReport.claim_status", state:"NONE->RECORDED_UNVERIFIED|QUARANTINED"});
expectAction("project-orchestration-v1", "RecordManualOfflineClaim",
  {state_owner:"claim_ledger", state_target:"ManualOfflineClaim.claim_status", state:"NONE->RECORDED_UNVERIFIED|QUARANTINED"});
expectAction("project-orchestration-v1", "ReviewExecutionClaim",
  {state_owner:"review_domain", state_target:"Review.review_outcome", state:"NONE->VERIFIED|REJECTED|NEEDS_READBACK|UNKNOWN"});
expectAction("project-orchestration-v1", "RecordResultUserDecision",
  {state_owner:"review_domain", state_target:"ResultUserDecision.decision", state:"NONE->ACCEPTED_RESULT|REJECTED_RESULT|NEEDS_FOLLOWUP"});
expectAction("project-orchestration-v1", "RecordExecutionAttemptReadback",
  {state_owner:"execution_aggregate", state_target:"PreparedAttempt.state", state:"DISPATCHED|RUNNING->RUNNING|SUCCEEDED|FAILED|CANCELLED|TIMED_OUT|UNKNOWN_READBACK"});
for (const action of asArray(actionsByContract.get("project-orchestration-v1"))) {
  if (["RecordExecutedReportClaim", "RecordManualOfflineClaim"].includes(action.command)) {
    check(action.state_target.endsWith("claim_status"), "CLAIM_MUTATES_FOREIGN_OWNER", action.command);
  }
  if (action.command === "ReviewExecutionClaim") {
    check(action.state.startsWith("NONE->"), "REVIEW_REWRITES_CLAIM", action.state);
  }
  if (action.command === "RecordResultUserDecision") {
    check(action.state.startsWith("NONE->"), "RESULT_DECISION_REWRITES_REVIEW", action.state);
  }
  if (/^(?:SUCCEEDED|FAILED|CANCELLED|TIMED_OUT|UNKNOWN_READBACK)->/.test(action.state)) {
    check(false, "TERMINAL_STATE_USED_AS_FOREIGN_SOURCE", action.command);
  }
}
for (const command of ["RequestConnectorRead", "RequestConnectorSync", "RequestConnectorAction",
  "RecordConnectorReadback", "RecordConnectorActionResult"]) {
  check(actionCommands("connector-capability-v1").includes(command), "CONNECTOR_ACTION_REQUIRED", command);
}
for (const action of asArray(actionsByContract.get("connector-capability-v1"))) {
  if (/^RequestConnector/.test(action.command)) {
    check(!action.state.includes("SUCCEEDED"), "CONNECTOR_REQUEST_DIRECT_SUCCESS", action.command);
  }
  check(action.command !== "RevokeConnectorGrant", "CONNECTOR_OWNS_GRANT_REVOKE", action.command);
}
const objectActions = asArray(actionsByContract.get("object-ref-navigation-v1"));
check(actionCommands("object-ref-navigation-v1").includes("RequestExternalObjectOpen"), "OBJECT_EXTERNAL_SPLIT", "request");
check(actionCommands("object-ref-navigation-v1").includes("RecordExternalObjectOpenResult"), "OBJECT_EXTERNAL_SPLIT", "result");
for (const action of objectActions.filter((item) => item.command === "RequestExternalObjectOpen")) {
  check(!action.state.includes("OPENED"), "OBJECT_REQUEST_DIRECT_OPEN", action.state);
}
for (const command of ["CreatePersonalFactVersion", "CreateModelAssertionVersion"]) {
  check(actionCommands("memory-personal-model-v1").includes(command), "MEMORY_TYPED_ACTION", command);
}

check(contractFixtures.schema === "syn.fixture.contract-cases.v1", "CONTRACT_FIXTURE_SCHEMA", contractFixtures.schema);
const contractFixtureCases = asArray(contractFixtures.cases);
check(unique(contractFixtureCases.map((item) => item.id)), "CONTRACT_FIXTURE_ID", "duplicate");
requireFixtureIds(contractFixtureCases, [
  "CF-EVENT-POS-002", "CF-EVENT-NEG-002", "CF-EVENT-NEG-003", "CF-EVENT-NEG-004",
  "CF-EVENT-NEG-005", "CF-EVENT-NEG-006",
  "CF-ATTENTION-POS-002", "CF-ATTENTION-POS-003", "CF-ATTENTION-POS-004",
  "CF-ORCHESTRATION-POS-002", "CF-ORCHESTRATION-POS-003", "CF-ORCHESTRATION-POS-004",
  "CF-ORCHESTRATION-NEG-002", "CF-ORCHESTRATION-NEG-003", "CF-ORCHESTRATION-NEG-004",
  "CF-ORCHESTRATION-NEG-005"
], "CONTRACT_FIXTURE_REQUIRED");

function ok(code, mutated_targets = []) { return { ok: true, code, mutated_targets }; }
function denied(code, mutated_targets = []) { return { ok: false, code, mutated_targets }; }
const contractRules = {
  "identity-scope-v1": {
    "resolve-identity": (input) => {
      if (input.identity_source === "PATH" || input.snapshot_state !== "CURRENT") return denied("DENIED_PATH_OR_STALE");
      if (!input.actor_id || !input.scope_ref || !input.permission_snapshot_ref) return denied("DENIED_MISSING_IDENTITY");
      return ok("RESOLVED");
    }
  },
  "command-v1": {
    "authorize-command": (input) => {
      if (!input.scope_ref) return denied("DENIED_MISSING_SCOPE");
      if (input.prior_request_hash && input.prior_request_hash !== input.request_hash) return denied("DENIED_IDEMPOTENCY_CONFLICT");
      if (input.expected_revision !== input.current_revision) return denied("DENIED_STALE_REVISION");
      return ok("ALLOWED");
    }
  },
  "event-audit-outbox-v1": {
    "external-effect-outbox": (input) => {
      if (input.command === "ClaimOutboxItem" || input.transition === "AVAILABLE->LEASED") {
        return denied("DEFERRED_M2_RUNTIME_OWNER");
      }
      if (["lease_state", "attempt_count", "next_retry_not_before"].some((key) => Object.hasOwn(input, key))) {
        return denied("FAILED_M1_RUNTIME_FIELD");
      }
      if (input.command === "DeclareExternalEffectIntent") {
        const missingBinding = !input.owning_command_id || !input.owning_command_receipt_ref ||
          !input.resolved_receipt_ref || !input.receipt_command_id;
        const mismatchedBinding = input.owning_command_receipt_ref !== input.resolved_receipt_ref ||
          input.owning_command_id !== input.receipt_command_id ||
          input.scope_ref !== input.receipt_scope_ref ||
          input.correlation_id !== input.receipt_correlation_id ||
          input.idempotency_key !== input.receipt_idempotency_key ||
          input.receipt_final_status !== "EXTERNAL_PENDING";
        if (missingBinding || mismatchedBinding) return denied("FAILED_OUTBOX_OWNING_COMMAND_BINDING");
        if (input.same_uow !== true) return denied("FAILED_OUTBOX_ATOMICITY");
        if (!input.effect_id || !input.payload_ref || !input.payload_hash ||
            !input.result_command_type || input.audit_state !== "SCRUBBED") {
          return denied("FAILED_OUTBOX_DECLARATION");
        }
        return ok("OUTBOX_DECLARED", ["OutboxItem.status"]);
      }
      if (input.effect_kind === "EXTERNAL" && input.outbox_status !== "DECLARED") return denied("FAILED_OUTBOX_REQUIRED");
      if (input.audit_state !== "SCRUBBED") return denied("FAILED_UNSCRUBBED_AUDIT");
      return ok("COMMITTED_INTERNAL");
    }
  },
  "role-session-v1": {
    "exact-session-binding": (input) => {
      if (![input.actor_match, input.role_match, input.session_match, input.thread_match, input.scope_match].every(Boolean) ||
          input.snapshot_state !== "CURRENT") return denied("DENIED_BINDING_MISMATCH");
      return ok("SESSION_BOUND");
    }
  },
  "handoff-v1": {
    "handoff-transition": (input) => {
      if (input.replay) return denied("REJECTED_REPLAY");
      if (input.expired) return denied("REJECTED_EXPIRED");
      if (!input.recipient_match) return denied("REJECTED_RECIPIENT");
      if (!input.permission_is_request) return denied("REJECTED_PERMISSION_NOT_REQUEST");
      if (input.state === "CREATED" && input.action === "ACCEPT") return ok("ACCEPTED");
      return denied("REJECTED_TRANSITION");
    }
  },
  "attention-decision-v1": {
    "attention-routing": (input) => {
      if (input.auto_create_todo) return denied("DENIED_AUTO_TODO");
      if (input.command === "CreateStandaloneTodo" && input.explicit_user_command &&
          input.source_kind === "STANDALONE_USER_CREATED") return ok("TODO_CREATED", ["Todo.status"]);
      if (input.command === "CompleteStandaloneTodo" && input.current_state === "OPEN" && input.revision_match) {
        return ok("TODO_COMPLETED", ["Todo.status"]);
      }
      if (input.command === "ReopenOpenLoop" && ["CLOSED", "DISMISSED"].includes(input.current_state) && input.revision_match) {
        return ok("OPEN_LOOP_REOPENED", ["OpenLoop.coordination_state"]);
      }
      if (input.command === "CloseOpenLoop" && ["OPEN", "SNOOZED"].includes(input.current_state) && input.revision_match) {
        return ok("OPEN_LOOP_CLOSED", ["OpenLoop.coordination_state"]);
      }
      if (!input.source_owner_ref || !input.source_revision_match) return denied("DENIED_SOURCE_OWNER_OR_REVISION");
      return ok("ROUTED_TO_SOURCE_OWNER", ["DecisionRequestProjection.projected_status"]);
    }
  },
  "project-orchestration-v1": {
    "dispatch-and-report-claim": (input) => {
      if (input.action === "CREATE_AUTHORIZED_RUN") {
        if (input.authorization_status !== "ACTIVE" || !input.authorization_revision_match || !input.joins_match) {
          return denied("AUTHORIZED_RUN_CREATE_DENIED");
        }
        if (input.failed_facet) return denied("ATOMIC_CREATE_ROLLED_BACK");
        return ok("AUTHORIZED_RUN_CREATED", ["WorkflowRun.status", "WorkItem.status", "PreparedAttempt.state"]);
      }
      if (input.action === "RECORD_REPORT") {
        const terminal = ["SUCCEEDED", "FAILED", "CANCELLED", "TIMED_OUT", "UNKNOWN_READBACK"].includes(input.attempt_state);
        if (input.report_kind === "MANUAL_OFFLINE" &&
            [input.dispatch_id, input.attempt_id, input.grant_id, input.worker_role_session_id,
              input.authoritative_execution_receipt_ref].some(Boolean)) {
          return denied("REJECTED_MANUAL_EXECUTION_JOIN");
        }
        if (input.report_kind === "EXECUTED" &&
            (!terminal || !input.joins_match || !input.authoritative_execution_receipt_ref || input.claim_only !== true)) {
          return denied("REJECTED_REPORT_AS_SUCCESS");
        }
        return ok("CLAIM_RECORDED", [input.report_kind === "MANUAL_OFFLINE" ?
          "ManualOfflineClaim.claim_status" : "ExecutedReport.claim_status"]);
      }
      if (input.action === "REVIEW") {
        if (input.claim_status !== "RECORDED_UNVERIFIED" || !input.readback_match) return denied("REVIEW_DENIED_READBACK");
        return ok("REVIEW_RECORDED", ["Review.review_outcome"]);
      }
      if (input.action === "RESULT_DECISION") {
        if (input.review_outcome !== "VERIFIED") return denied("RESULT_DECISION_DENIED_REVIEW");
        return ok("RESULT_DECISION_RECORDED", ["ResultUserDecision.decision"]);
      }
      if (input.action === "DISPATCH" && input.attempt_state === "GRANT_READY_NON_RUNNABLE" &&
          input.grant_present && input.grant_persisted && input.grant_readback && input.joins_match) {
        return ok("DISPATCH_ALLOWED", ["Dispatch.state"]);
      }
      return denied("DISPATCH_DENIED_GRANT_OR_JOIN");
    }
  },
  "memory-personal-model-v1": {
    "typed-promotion": (input) => {
      if (input.candidate_kind === "PERSONAL_FACT" && input.origin_kind === "MODEL_INFERENCE") {
        return denied("QUARANTINED_INFERENCE_AS_FACT");
      }
      if (input.policy_decision !== "ACCEPTED" || input.conflict) return denied("QUARANTINED_POLICY_OR_CONFLICT");
      if (input.candidate_kind === "PERSONAL_FACT" && ["EXPLICIT_USER", "RELIABLE_DETERMINISTIC_SOURCE"].includes(input.provenance_kind)) {
        return ok("PERSONAL_FACT_ALLOWED");
      }
      if (input.candidate_kind === "MODEL_ASSERTION") return ok("MODEL_ASSERTION_ALLOWED");
      return denied("QUARANTINED_UNKNOWN_MEMORY_KIND");
    }
  },
  "connector-capability-v1": {
    "capability-grant": (input) => {
      const sensitive = findSensitiveMaterial(input, forbiddenKeys, allowedKeys, sensitiveSentinels, "$", "FIXTURE");
      if (sensitive) return denied("DENIED_SENSITIVE_MATERIAL");
      if (input.grant_kind !== "CONNECTOR_CAPABILITY" || !input.grant_match || !input.scope_match ||
          !input.credential_ref || !input.confirmation_ref) return denied("DENIED_GRANT_OR_SCOPE");
      return ok("EXTERNAL_PENDING");
    }
  },
  "object-ref-navigation-v1": {
    "object-resolution": (input) => {
      if (!input.scope_match || !input.revision_match) return denied("DENIED_SCOPE_OR_REVISION");
      if (String(input.relative_path).startsWith("/") || String(input.relative_path).split("/").includes("..") ||
          input.external_uri && !input.external_policy) return denied("DENIED_PATH_OR_EXTERNAL_POLICY");
      return ok("RESOLVED");
    }
  }
};

for (const [contractId, rules] of Object.entries(contractRules)) {
  for (const ruleId of Object.keys(rules)) {
    const cases = contractFixtureCases.filter((item) => item.contract_id === contractId && item.rule_id === ruleId);
    check(cases.some((item) => item.polarity === "POSITIVE"), "CONTRACT_FIXTURE_RULE_POLARITY", contractId + ":" + ruleId + ":positive");
    check(cases.some((item) => item.polarity === "NEGATIVE"), "CONTRACT_FIXTURE_RULE_POLARITY", contractId + ":" + ruleId + ":negative");
  }
}
for (const fixture of contractFixtureCases) {
  const evaluator = contractRules[fixture.contract_id]?.[fixture.rule_id];
  check(typeof evaluator === "function", "CONTRACT_FIXTURE_EVALUATOR", fixture.id);
  if (typeof evaluator !== "function") continue;
  const observed = evaluator(fixture.input);
  check(observed.code === fixture.expected_code, "CONTRACT_FIXTURE_MISMATCH", fixture.id + ":" + observed.code + "!=" + fixture.expected_code);
  if (Object.hasOwn(fixture, "expected_mutated_targets")) {
    check(sameArray(observed.mutated_targets, fixture.expected_mutated_targets),
      "CONTRACT_FIXTURE_MUTATION", fixture.id + ":" + observed.mutated_targets.join(","));
  }
  check(fixture.polarity === "POSITIVE" ? observed.ok : !observed.ok,
    "CONTRACT_FIXTURE_POLARITY_RESULT", fixture.id);
  const body = sectionBody(readRepoText("docs/contracts/" + manifest.contracts.find((item) => item.id === fixture.contract_id).path), "contract.fixtures");
  check(body.includes(fixture.id), "CONTRACT_FIXTURE_REFERENCE", fixture.id);
}

const holds = asArray(holdRegistry.holds);
check(holdRegistry.schema === "syn.open-design-holds.v1", "HOLD_SCHEMA", holdRegistry.schema);
check(holdRegistry.status === "FROZEN_V1", "HOLD_STATUS", holdRegistry.status);
check(holds.length > 0 && unique(holds.map((item) => item.id)), "HOLD_COUNT_OR_DUPLICATE", holds.length);
const holdIds = new Set(holds.map((item) => item.id));
for (const hold of holds) {
  check(/^HOLD-[A-Z0-9-]+$/.test(hold.id), "HOLD_ID_FORMAT", hold.id);
  check(hold.status === "HOLD", "HOLD_ITEM_STATUS", hold.id);
  check(typeof hold.owner === "string" && hold.owner.trim() !== "", "HOLD_OWNER", hold.id);
  check(typeof hold.reason === "string" && hold.reason.trim() !== "", "HOLD_REASON", hold.id);
  check(typeof hold.unblock === "string" && hold.unblock.trim() !== "", "HOLD_UNBLOCK", hold.id);
  check(allNonEmptyStrings(hold.forbidden_to_decide_in_m1), "HOLD_FORBIDDEN_DECISION", hold.id);
}

check(sourceManifest.schema === "syn.source-opening-manifest.v1", "SOURCE_SCHEMA", sourceManifest.schema);
check(sourceManifest.task_id === "SYN-FND-001-R1", "SOURCE_TASK", sourceManifest.task_id);
check(sourceManifest.evidence_level === "STATIC_OPENING_ONLY", "SOURCE_EVIDENCE", sourceManifest.evidence_level);
check(sourceManifest.base_oid === expectedBaseOid, "SOURCE_BASE_OID", sourceManifest.base_oid);
const sourceFiles = asArray(sourceManifest.source_files);
check(sourceFiles.length === 30, "SOURCE_FILE_COUNT", sourceFiles.length);
check(sourceManifest.counts && sourceManifest.counts.source_files === 30, "SOURCE_DECLARED_COUNT", "source_files");
check(sourceManifest.counts && sourceManifest.counts.tauri_commands === 171, "SOURCE_DECLARED_COUNT", "tauri");
check(sourceManifest.counts && sourceManifest.counts.supervisor_mcp_capabilities === 8, "SOURCE_DECLARED_COUNT", "mcp");
check(sourceManifest.counts && sourceManifest.counts.sqlite_tables === 68, "SOURCE_DECLARED_COUNT", "sqlite");
check(unique(sourceFiles.map((item) => item.id)), "SOURCE_ID_UNIQUE", "ids");
check(unique(sourceFiles.map((item) => item.path)), "SOURCE_PATH_UNIQUE", "paths");
const sourceById = new Map(sourceFiles.map((item) => [item.id, item]));
const sourceTextById = new Map();
for (const source of sourceFiles) {
  const baseSpec = expectedBaseOid + ":" + source.path;
  const observedBlob = gitText(["rev-parse", baseSpec]);
  const baseBuffer = gitBuffer(["show", baseSpec]);
  sourceTextById.set(source.id, baseBuffer.toString("utf8"));
  check(observedBlob === source.blob_oid, "SOURCE_BLOB", source.id + ":" + observedBlob);
  check(sha256(baseBuffer) === source.sha256, "SOURCE_SHA256", source.id);
  const worktreePath = resolve(repoRoot, source.path);
  check(existsSync(worktreePath), "SOURCE_WORKTREE_MISSING", source.path);
  if (existsSync(worktreePath)) {
    check(sha256(readFileSync(worktreePath)) === source.sha256, "SOURCE_WORKTREE_DRIFT", source.path);
  }
}

check(entryInventory.schema === "syn.entrypoint-inventory.v1", "ENTRY_SCHEMA", entryInventory.schema);
check(entryInventory.task_id === "SYN-FND-001-R1", "ENTRY_TASK", entryInventory.task_id);
check(entryInventory.opening && entryInventory.opening.base_oid === expectedBaseOid, "ENTRY_BASE", "base oid");
const routingEnums = entryInventory.routing_enums || {};
check(sameArray(routingEnums.bypass_status, ["NONE_OBSERVED", "KNOWN_BYPASS", "STATICALLY_BLOCKED", "UNKNOWN"]), "ENTRY_ENUM", "bypass");
check(sameArray(routingEnums.migration_status, ["MIGRATED", "GUARDED_LEGACY", "BLOCKED", "NOT_IN_SCOPE"]), "ENTRY_ENUM", "migration");
check(sameArray(routingEnums.disposition, ["KEEP_ADAPTER", "REWRITE_LATER", "RETIRE_AFTER_PARITY", "RETAIN_BLOCK"]), "ENTRY_ENUM", "disposition");

const tauriSource = gitText(["show", expectedBaseOid + ":" + sourceById.get("tauri-command-registry").path]);
const handlerMatch = tauriSource.match(/tauri::generate_handler!\s*\[([\s\S]*?)\]\s*/);
check(Boolean(handlerMatch), "TAURI_REGISTRY_PARSE", "generate_handler");
const actualTauriCommands = handlerMatch ? handlerMatch[1]
  .replace(/\/\*[\s\S]*?\*\//g, "").replace(/\/\/.*$/gm, "")
  .split(",").map((item) => item.trim()).filter(Boolean) : [];
check(actualTauriCommands.length === 171 && unique(actualTauriCommands), "TAURI_SOURCE_COUNT_OR_DUPLICATE", actualTauriCommands.length);
const tauriRoutes = asArray(entryInventory.tauri_routes);
check(tauriRoutes.length >= 20 && unique(tauriRoutes.map((item) => item.id)), "TAURI_ROUTE_COUNT_OR_DUPLICATE", tauriRoutes.length);
const inventoriedTauriCommands = [];
for (const route of tauriRoutes) {
  check(sourceById.has(route.source_ref), "ENTRY_SOURCE_REF", route.id + ":" + route.source_ref);
  const result = inventoryRecordResult(route, contractIds, holdIds, routingEnums);
  check(result === "PASS", "ENTRY_ROUTE", route.id + ":" + result);
  check(allNonEmptyStrings(route.commands), "ENTRY_COMMANDS", route.id);
  inventoriedTauriCommands.push(...asArray(route.commands));
  for (const ref of asArray(route.guard_proof?.source_refs)) check(sourceRefLineResult(ref, sourceById) === "PASS", "ENTRY_GUARD_SOURCE", route.id + ":" + ref);
  for (const ref of asArray(route.block_proof?.source_refs)) check(sourceRefLineResult(ref, sourceById) === "PASS", "ENTRY_BLOCK_SOURCE", route.id + ":" + ref);
}
check(inventoriedTauriCommands.length === 171 && unique(inventoriedTauriCommands), "TAURI_INVENTORY_COUNT_OR_DUPLICATE", inventoriedTauriCommands.length);
check(sameArray(inventoriedTauriCommands, actualTauriCommands), "TAURI_INVENTORY_EXACT", "ordered registry differs");

const mcpSource = gitText(["show", expectedBaseOid + ":" + sourceById.get("mcp-capability-registry").path]);
const mcpRegistryMatch = mcpSource.match(/const REGISTRY:[\s\S]*?=\s*&\[([\s\S]*?)\n\];/);
check(Boolean(mcpRegistryMatch), "MCP_REGISTRY_PARSE", "REGISTRY");
const actualMcpNames = mcpRegistryMatch ? [...mcpRegistryMatch[1].matchAll(/name:\s*"([^"]+)"/g)].map((match) => match[1]) : [];
check(actualMcpNames.length === 8 && unique(actualMcpNames), "MCP_SOURCE_COUNT_OR_DUPLICATE", actualMcpNames.length);
const mcpEntries = asArray(entryInventory.supervisor_mcp_capabilities);
check(mcpEntries.length === 8 && unique(mcpEntries.map((item) => item.name)), "MCP_INVENTORY_COUNT_OR_DUPLICATE", mcpEntries.length);
for (const entry of mcpEntries) {
  const result = inventoryRecordResult(entry, contractIds, holdIds, routingEnums);
  check(result === "PASS", "MCP_ROUTE", entry.name + ":" + result);
  for (const ref of asArray(entry.guard_proof?.source_refs)) {
    check(sourceRefLineResult(ref, sourceById) === "PASS", "MCP_GUARD_SOURCE", entry.name + ":" + ref);
  }
}
check(sameArray(mcpEntries.map((item) => item.name), actualMcpNames), "MCP_INVENTORY_EXACT", "ordered registry differs");

const runnerEntries = asArray(entryInventory.runner_background_entrypoints);
check(runnerEntries.length === 19 && unique(runnerEntries.map((item) => item.id)), "RUNNER_COUNT_OR_DUPLICATE", runnerEntries.length);
check(runnerEntries.some((item) => item.kind === "RUNNER") && runnerEntries.some((item) => item.kind === "BACKGROUND_JOB"), "RUNNER_KIND", "coverage");
for (const entry of runnerEntries) {
  check(["RUNNER", "BACKGROUND_JOB"].includes(entry.kind), "RUNNER_KIND", entry.id);
  check(sourceById.has(entry.source_ref), "RUNNER_SOURCE_REF", entry.id + ":" + entry.source_ref);
  check(/^[A-Za-z_][A-Za-z0-9_]*$/.test(entry.source_symbol), "RUNNER_SYMBOL_IDENTIFIER", entry.id);
  check(Array.isArray(entry.source_definitions) && entry.source_definitions.length > 0, "RUNNER_DEFINITIONS", entry.id);
  const routeResult = inventoryRecordResult(entry, contractIds, holdIds, routingEnums);
  check(routeResult === "PASS", "RUNNER_ROUTE", entry.id + ":" + routeResult);
  if (sourceById.has(entry.source_ref)) {
    const source = sourceById.get(entry.source_ref);
    const sourceText = gitText(["show", expectedBaseOid + ":" + source.path]);
    const declaredLines = asArray(entry.source_definitions).map((item) => item.line);
    check(runnerDefinitionResult(sourceText, entry.source_symbol, declaredLines) === "PASS", "RUNNER_DEFINITION_VARIANTS", entry.id);
    const sourceLines = sourceText.split("\n");
    for (const definition of asArray(entry.source_definitions)) {
      const line = (sourceLines[definition.line - 1] || "").trim();
      check(Number.isInteger(definition.line) && definition.line > 0 &&
        typeof definition.signature_prefix === "string" && definition.signature_prefix.trim() !== "" &&
        line.startsWith(definition.signature_prefix),
        "RUNNER_LINE_NOT_DEFINITION", entry.id + ":" + definition.line);
    }
  }
}

check(migrationInventory.schema === "syn.legacy-migration-inventory.v1", "MIGRATION_SCHEMA", migrationInventory.schema);
check(migrationInventory.task_id === "SYN-FND-001-R1", "MIGRATION_TASK", migrationInventory.task_id);
check(migrationInventory.opening && migrationInventory.opening.base_oid === expectedBaseOid, "MIGRATION_BASE", "base oid");
const sqliteSource = gitText(["show", expectedBaseOid + ":" + sourceById.get("workbench-sqlite-schema").path]);
const actualSqliteTables = [...sqliteSource.matchAll(/CREATE TABLE IF NOT EXISTS\s+([A-Za-z0-9_]+)/g)].map((match) => match[1]);
check(actualSqliteTables.length === 68 && unique(actualSqliteTables), "SQLITE_SOURCE_COUNT_OR_DUPLICATE", actualSqliteTables.length);
const sqliteGroups = migrationInventory.opening?.sqlite_tables || {};
const inventoriedSqliteTables = Object.values(sqliteGroups).flatMap((items) => asArray(items));
check(migrationInventory.opening?.sqlite_table_count === 68, "SQLITE_DECLARED_COUNT", migrationInventory.opening?.sqlite_table_count);
check(inventoriedSqliteTables.length === 68 && unique(inventoriedSqliteTables), "SQLITE_INVENTORY_COUNT_OR_DUPLICATE", inventoriedSqliteTables.length);
check(sameSet(inventoriedSqliteTables, actualSqliteTables), "SQLITE_INVENTORY_EXACT", "table set differs");

const importerSource = sourceTextById.get("workbench-sqlite-importer") || "";
const processRegistrySource = sourceTextById.get("exec-process-registry") || "";
const primaryWorkflowState = rustStringConst(importerSource, "PRIMARY_WORKFLOW_STATE");
const canonicalRuntimeLog = rustStringConst(importerSource, "CANONICAL_RUNTIME_LOG");
const legacyRuntimeLogAlias = rustStringConst(importerSource, "LEGACY_RUNTIME_LOG_ALIAS");
const processRegistrySidecar = rustStringConst(processRegistrySource, "SIDECAR_NAME");
const optionalSidecarResult = rustStringArray(importerSource, "OPTIONAL_SIDECARS", {
  CANONICAL_RUNTIME_LOG: canonicalRuntimeLog,
  LEGACY_RUNTIME_LOG_ALIAS: legacyRuntimeLogAlias
});
check(optionalSidecarResult.code === "PASS", "SIDECAR_SOURCE_PARSE", optionalSidecarResult.code);
check([primaryWorkflowState, canonicalRuntimeLog, legacyRuntimeLogAlias, processRegistrySidecar]
  .every((value) => typeof value === "string" && value.trim() !== ""), "SIDECAR_SOURCE_CONST", "required constants");
check(optionalSidecarResult.values.includes(canonicalRuntimeLog), "SIDECAR_SOURCE_CANONICAL_REQUIRED", canonicalRuntimeLog);
check(optionalSidecarResult.values.includes(legacyRuntimeLogAlias), "SIDECAR_SOURCE_ALIAS_REQUIRED", legacyRuntimeLogAlias);
const sourceCanonicalSidecars = [
  primaryWorkflowState,
  ...optionalSidecarResult.values.filter((value) => value !== legacyRuntimeLogAlias),
  processRegistrySidecar
].filter((value) => typeof value === "string" && value.trim() !== "");
check(sourceCanonicalSidecars.length === 18 && unique(sourceCanonicalSidecars),
  "SIDECAR_SOURCE_COUNT_OR_DUPLICATE", sourceCanonicalSidecars.length);
check(sourceCanonicalSidecars.includes(canonicalRuntimeLog) && !sourceCanonicalSidecars.includes(legacyRuntimeLogAlias),
  "SIDECAR_SOURCE_CANONICAL_ALIAS", canonicalRuntimeLog + ":" + legacyRuntimeLogAlias);
const sidecars = Object.values(migrationInventory.opening?.sidecars || {}).flatMap((items) => asArray(items));
check(sidecars.length === 18 && unique(sidecars), "SIDECAR_INVENTORY", sidecars.length);
check(sameSet(sidecars, sourceCanonicalSidecars), "SIDECAR_SOURCE_EXACT", "opening sidecar set differs from base source");
check(migrationInventory.opening?.sidecar_evidence_kind === "FROZEN_BASE_SOURCE_DERIVED" &&
  sameArray(migrationInventory.opening?.sidecar_source_refs, ["workbench-sqlite-importer", "exec-process-registry"]),
  "SIDECAR_SOURCE_EVIDENCE", "exact");

const expectedProjectionAnchors = {
  sqlite_bridge: ["SOURCE::workbench-sqlite-schema::WORKBENCH_SQLITE_SCHEMA_DDL"],
  workflow: ["SOURCE::workflow-db-primary-wiring::write_m5b_batch1_workflow_state_db_primary"],
  page_read_model: ["SOURCE::page-read-model::query_page_read_model"],
  audit: ["SOURCE::audit-ledger-read-model::query_audit_ledger_read_model"],
  knowledge: ["SOURCE::knowledge-index::knowledge_workspace_snapshot"],
  secretary: ["SOURCE::secretary-agent::run_secretary_explain"],
  runtime: [
    "SOURCE::real-execution-db-primary::db_primary_projection_records",
    "SOURCE::session-continuation-db-primary::db_primary_projection_records",
    "SOURCE::runtime-log-db-primary::db_primary_projection_records"
  ]
};
const projectionNames = Object.keys(migrationInventory.opening?.projections || {});
check(sameArray(projectionNames, Object.keys(expectedProjectionAnchors)), "PROJECTION_INVENTORY", projectionNames.join(","));
check(migrationInventory.opening?.projection_evidence_kind === "CONTRACT_TAXONOMY_SOURCE_ANCHORED",
  "PROJECTION_EVIDENCE_KIND", migrationInventory.opening?.projection_evidence_kind);
check(exactObject(migrationInventory.opening?.projection_source_anchors, expectedProjectionAnchors),
  "PROJECTION_SOURCE_ANCHOR_EXACT", "taxonomy anchor map differs");
for (const [projection, anchors] of Object.entries(expectedProjectionAnchors)) {
  for (const reference of anchors) {
    const result = sourceDeclarationAnchorResult(reference, sourceById, sourceTextById);
    check(result === "PASS", result, projection + ":" + reference);
  }
}

check(storageInventory.schema === "syn.storage-opening-inventory.v1", "STORAGE_SCHEMA", storageInventory.schema);
check(storageInventory.task_id === "SYN-FND-001-R1", "STORAGE_TASK", storageInventory.task_id);
check(storageInventory.base_oid === expectedBaseOid, "STORAGE_BASE", storageInventory.base_oid);
check(sameArray(Object.keys(storageInventory.entry_ref_grammar || {}), ["SOURCE", "TAURI", "MCP", "HOLD"]) &&
  Object.values(storageInventory.entry_ref_grammar || {}).every((value) => typeof value === "string" && value.trim() !== ""),
  "STORAGE_ENTRY_REF_GRAMMAR", "exact keys");
const expectedStorageDispositions = {
  unknown_disposition: ["BLOCK_OR_QUARANTINE_REF_ONLY", "NO_AUTHORITY"],
  corrupt_disposition: ["BLOCK_OR_QUARANTINE_REF_ONLY", "DISCARD_AND_REBUILD"],
  sensitive_disposition: ["SCRUB_AND_STOP_BEFORE_ORDINARY_STORE", "SCRUB_OR_OMIT"]
};
check(exactObject(storageInventory.disposition_enum, expectedStorageDispositions), "STORAGE_DISPOSITION_ENUM", "exact");
const tauriCommandSet = new Set(actualTauriCommands);
const mcpCapabilitySet = new Set(actualMcpNames);
const resolvedStorage = [];
for (const family of asArray(storageInventory.families)) {
  check(typeof family.id === "string" && family.id.trim() !== "", "STORAGE_FAMILY_ID", JSON.stringify(family));
  check(family.defaults && typeof family.defaults === "object", "STORAGE_DEFAULTS", family.id);
  for (const member of asArray(family.members)) resolvedStorage.push({ ...family.defaults, ...member, family_id: family.id });
}
check(unique(resolvedStorage.map((item) => item.id)), "STORAGE_ID_UNIQUE", "records");
for (const record of resolvedStorage) {
  for (const field of asArray(storageInventory.required_resolved_fields)) {
    const value = record[field];
    check(value !== undefined && value !== null &&
      (typeof value !== "string" || value.trim() !== "") &&
      (!Array.isArray(value) || value.length > 0), "STORAGE_FIELD", record.id + ":" + field);
  }
  check(contractIds.has(record.owner_contract), "STORAGE_OWNER", record.id + ":" + record.owner_contract);
  check(sourceById.has(record.source_ref), "STORAGE_SOURCE", record.id + ":" + record.source_ref);
  check(storageInventory.truth_status_enum.includes(record.truth_status), "STORAGE_TRUTH_STATUS", record.id);
  check(storageInventory.migration_status_enum.includes(record.migration_status), "STORAGE_MIGRATION_STATUS", record.id);
  check(allNonEmptyStrings(record.read_entries), "STORAGE_READ_ENTRIES", record.id);
  check(allNonEmptyStrings(record.write_entries), "STORAGE_WRITE_ENTRIES", record.id);
  for (const field of Object.keys(expectedStorageDispositions)) {
    check(expectedStorageDispositions[field].includes(record[field]), "STORAGE_DISPOSITION", record.id + ":" + field);
  }
  for (const reference of [...asArray(record.read_entries), ...asArray(record.write_entries)]) {
    const result = storageEntryRefResult(reference, sourceById, sourceTextById, tauriCommandSet, mcpCapabilitySet, holdIds);
    check(result === "PASS", result, record.id + ":" + reference);
  }
  check(Array.isArray(record.hold_refs) && record.hold_refs.every((ref) => holdIds.has(ref)), "STORAGE_HOLDS", record.id);
  if (record.truth_status === "LEGACY_UNKNOWN") {
    check(record.hold_refs.length > 0, "STORAGE_UNKNOWN_HOLD", record.id);
  }
}
const processRegistry = resolvedStorage.find((item) => item.id === "sidecar.exec_process_registry");
check(processRegistry?.source_ref === "exec-process-registry", "PROCESS_REGISTRY_SOURCE", processRegistry?.source_ref);
check(!String(processRegistry?.natural_key).includes("store_id") &&
  ["run_id", "pid", "started_at", "process_group_id", "event_id"].every((token) => String(processRegistry?.natural_key).includes(token)),
  "PROCESS_REGISTRY_NATURAL_KEY", processRegistry?.natural_key);
check(String(processRegistry?.revision).includes("revision") && String(processRegistry?.revision).includes("increments"),
  "PROCESS_REGISTRY_REVISION", processRegistry?.revision);
const storageTables = resolvedStorage.filter((item) => item.kind === "SQLITE_TABLE").map((item) => item.physical_name);
const storageSidecars = resolvedStorage.filter((item) => item.kind === "SIDECAR").map((item) => item.physical_name);
const storageProjections = resolvedStorage.filter((item) => item.kind === "PROJECTION").map((item) => item.physical_name);
check(storageTables.length === 68 && unique(storageTables) && sameSet(storageTables, actualSqliteTables), "STORAGE_TABLE_COVERAGE", storageTables.length);
check(storageSidecars.length === 18 && unique(storageSidecars) && sameSet(storageSidecars, sidecars), "STORAGE_SIDECAR_COVERAGE", storageSidecars.length);
check(storageProjections.length === 7 && unique(storageProjections) && sameSet(storageProjections, projectionNames), "STORAGE_PROJECTION_COVERAGE", storageProjections.length);
check(storageInventory.coverage?.sqlite_tables === 68 && storageInventory.coverage?.canonical_sidecars === 18 &&
  storageInventory.coverage?.projections === 7, "STORAGE_DECLARED_COVERAGE", JSON.stringify(storageInventory.coverage));

const migrationStatusEnum = asArray(migrationInventory.migration_status_enum);
const dispositionEnum = asArray(migrationInventory.disposition_enum);
const classificationEnum = asArray(migrationInventory.source_classification_enum);
check(sameArray(migrationStatusEnum, ["BLOCKED", "GUARDED_LEGACY", "NOT_IN_SCOPE", "HOLD"]), "MIGRATION_STATUS_DECLARATION", migrationStatusEnum.join(","));
check(sameArray(dispositionEnum, ["KEEP", "EXTRACT", "REWRITE", "RETIRE", "HOLD"]), "MIGRATION_DISPOSITION_DECLARATION", dispositionEnum.join(","));
check(sameArray(classificationEnum, ["KNOWN_SCHEMA", "NOT_OBSERVED_STATIC", "UNKNOWN", "CORRUPT", "SENSITIVE"]), "MIGRATION_CLASSIFICATION_DECLARATION", classificationEnum.join(","));
const migrationItems = asArray(migrationInventory.items);
check(migrationItems.length === 31, "MIGRATION_COUNT", migrationItems.length);
const expectedMigrationIds = Array.from({ length: 31 }, (_, index) => "MIG-" + String(index + 1).padStart(3, "0"));
check(sameArray(migrationItems.map((item) => item.id), expectedMigrationIds), "MIGRATION_IDS", migrationItems.map((item) => item.id).join(","));
for (const rawItem of migrationItems) {
  const item = { ...migrationInventory.item_defaults, ...rawItem };
  for (const field of ["legacy", "target", "disposition", "bypass_status", "migration_status", "m1_owner", "next_stage", "hold_refs"]) {
    const value = item[field];
    check(value !== undefined && value !== null && !(typeof value === "string" && value.trim() === ""), "MIGRATION_FIELD", item.id + ":" + field);
  }
  check(contractIds.has(item.m1_owner), "MIGRATION_OWNER", item.id + ":" + item.m1_owner);
  check(Array.isArray(item.hold_refs) && item.hold_refs.every((ref) => holdIds.has(ref)), "MIGRATION_HOLDS", item.id);
  const result = migrationItemResult(item, migrationStatusEnum, dispositionEnum, classificationEnum);
  check(result === "PASS", result, item.id);
  if (["BLOCKED", "HOLD"].includes(item.migration_status) || item.disposition === "HOLD") {
    check(item.hold_refs.length > 0, "MIGRATION_HOLD_REQUIRED", item.id);
  }
  const openingRefs = item.opening_refs || {};
  check(asArray(openingRefs.tables).every((table) => inventoriedSqliteTables.includes(table)), "MIGRATION_TABLE_REF", item.id);
  check(asArray(openingRefs.sidecars).every((sidecar) => sidecars.includes(sidecar)), "MIGRATION_SIDECAR_REF", item.id);
  check(asArray(openingRefs.projections).every((projection) => projectionNames.includes(projection)), "MIGRATION_PROJECTION_REF", item.id);
}
check(allNonEmptyStrings(migrationInventory.retirement_protocol), "RETIREMENT_PROTOCOL", "missing");

const referencedHolds = new Set(contractHoldRefs);
collectHoldRefs(entryInventory, referencedHolds);
collectHoldRefs(migrationInventory, referencedHolds);
collectHoldRefs(storageInventory, referencedHolds);
for (const ref of referencedHolds) check(holdIds.has(ref), "UNKNOWN_HOLD_REFERENCE", ref);
for (const id of holdIds) check(referencedHolds.has(id), "UNUSED_HOLD", id);

check(m2Input.schema === "syn.m2-input.v1", "M2_SCHEMA", m2Input.schema);
check(m2Input.status === "FROZEN_V1", "M2_STATUS", m2Input.status);
check(m2Input.scope === "external_contracts_and_safety_invariants_only", "M2_SCOPE", m2Input.scope);
check(m2Input.ownership_boundary?.no_runtime_claim && m2Input.ownership_boundary?.m2_owns,
  "M2_OWNERSHIP_BOUNDARY", "missing");
check(allNonEmptyStrings(m2Input.storage_input_requirements) && m2Input.storage_input_requirements.length >= 5,
  "M2_STORAGE_INPUTS", "missing");
const expectedM2Interfaces = ["CommandReceipt", "WorkbenchEventEnvelope", "AuditRecord", "OutboxItem", "OutboxLease", "CurrentSnapshot", "ProjectionCheckpoint", "UnknownQuarantineRef"];
const m2Interfaces = asArray(m2Input.interfaces);
check(sameArray(m2Interfaces.map((item) => item.name), expectedM2Interfaces), "M2_INTERFACE_NAMES", m2Interfaces.map((item) => item.name).join(","));
for (const item of m2Interfaces) {
  check(contractIds.has(item.owner_contract), "M2_INTERFACE_OWNER", item.name);
  check(typeSchemas.has(item.name), "M2_INTERFACE_TYPE", item.name);
  check(allNonEmptyStrings(item.required_fields) && unique(item.required_fields), "M2_INTERFACE_FIELDS", item.name);
  if (typeSchemas.has(item.name)) {
    check(sameArray(item.required_fields, typeSchemas.get(item.name).required_fields), "M2_INTERFACE_SCHEMA_DRIFT", item.name);
    check(item.domain_owner === typeSchemas.get(item.name).domain_owner, "M2_INTERFACE_DOMAIN_OWNER", item.name);
  }
}
check(sameArray(m2Interfaces.find((item) => item.name === "OutboxItem")?.required_fields, expectedOutboxFields),
  "M2_OUTBOX_SEMANTIC_FIELDS", "exact");
check(m2Interfaces.find((item) => item.name === "OutboxLease")?.persistence_owner === "M2_HOLD", "M2_LEASE_OWNER_HOLD", "persistence");
check(m2Interfaces.find((item) => item.name === "UnknownQuarantineRef")?.runtime_state_machine === "M2_HOLD", "M2_QUARANTINE_HOLD", "runtime");
check(m2Input.outbox_runtime_hold?.runtime_owner === "M2_HOLD" &&
  m2Input.outbox_runtime_hold?.runtime_fields === "M2_HOLD" &&
  m2Input.outbox_runtime_hold?.runtime_state_machine === "M2_HOLD",
  "M2_OUTBOX_RUNTIME_HOLD", "runtime ownership");
check(sameArray(m2Input.outbox_runtime_hold?.forbidden_m1_commands, ["ClaimOutboxItem"]),
  "M2_OUTBOX_FORBIDDEN_COMMANDS", "exact");
check(sameArray(m2Input.outbox_runtime_hold?.forbidden_m1_transitions,
  ["AVAILABLE->LEASED", "LEASED->AVAILABLE", "LEASED->DELIVERED", "LEASED->RETRY_WAIT", "RETRY_WAIT->AVAILABLE"]),
  "M2_OUTBOX_FORBIDDEN_TRANSITIONS", "exact");
const requiredShadowStates = ["DISABLED", "OBSERVE_ONLY", "SHADOW_WRITING", "PARITY_HOLD", "ROLLBACK_READY"];
check(sameArray(m2Input.shadow_write?.states, requiredShadowStates), "M2_SHADOW_STATES", "states");
check(allNonEmptyStrings(m2Input.shadow_write?.invariants) &&
  m2Input.shadow_write.invariants.some((item) => item.includes("one declared primary truth owner")), "M2_SHADOW_INVARIANTS", "invariants");
const expectedParity = ["identity", "scope", "count", "key", "canonical_hash", "semantic", "state", "ordering", "idempotency", "audit", "redaction", "projection_readback", "recovery", "unknown", "corrupt", "sensitive"];
check(sameArray(m2Input.parity_dimensions, expectedParity), "M2_PARITY", asArray(m2Input.parity_dimensions).join(","));
for (const classification of ["unknown", "corrupt", "sensitive", "approved_difference", "bug"]) {
  check(typeof m2Input.classification_rules?.[classification] === "string" &&
    m2Input.classification_rules[classification].trim() !== "", "M2_CLASSIFICATION", classification);
}
check(allNonEmptyStrings(m2Input.rollback_guards) && m2Input.rollback_guards.length >= 5, "M2_ROLLBACK", "guards");
check(sameArray(asArray(m2Fixtures.forbidden_premature_keys).map(canonicalKey), expectedM2ForbiddenKeys),
  "M2_FORBIDDEN_LIST", "fixture drift");
check(sameArray(asArray(m2Input.premature_decisions_forbidden_in_m1).map(canonicalKey), expectedM2ForbiddenKeys),
  "M2_FORBIDDEN_LIST", "artifact drift");
const forbiddenM2Keys = new Set(expectedM2ForbiddenKeys);
const m2ArtifactResult = m2ArtifactShapeResult(m2Input, forbiddenM2Keys);
check(m2ArtifactResult === "PASS", m2ArtifactResult, "artifact");
const actualPremature = findForbiddenField(m2Input, forbiddenM2Keys);
check(actualPremature === null, "M2_PREMATURE_DECISION", actualPremature || "root");

for (const fixture of asArray(ownerFixtures.cases)) {
  const observed = dagResult(fixture.contracts);
  check(observed === fixture.expected, "FIXTURE_OWNER_DAG", fixture.id + ":" + observed + "!=" + fixture.expected);
}
for (const fixture of asArray(forbiddenFixtures.cases)) {
  const found = findSensitiveMaterial(fixture.payload, forbiddenKeys, allowedKeys, sensitiveSentinels, "$", "FIXTURE");
  const observed = found ? found.code : "PASS";
  check(observed === fixture.expected, "FIXTURE_FORBIDDEN", fixture.id + ":" + observed + "!=" + fixture.expected);
}
for (const fixture of asArray(inventoryFixtures.cases)) {
  const observed = inventoryRecordResult(fixture.record, contractIds, holdIds, routingEnums);
  check(observed === fixture.expected, "FIXTURE_INVENTORY", fixture.id + ":" + observed + "!=" + fixture.expected);
}
for (const fixture of asArray(m2Fixtures.cases)) {
  const observed = m2InputResult(fixture.input, forbiddenM2Keys);
  check(observed === fixture.expected, "FIXTURE_M2", fixture.id + ":" + observed + "!=" + fixture.expected);
}
for (const fixture of asArray(m2Fixtures.artifact_cases)) {
  const observed = m2ArtifactShapeResult(withFixtureMutation(m2Input, fixture.mutation), forbiddenM2Keys);
  check(observed === fixture.expected, "FIXTURE_M2_ARTIFACT", fixture.id + ":" + observed + "!=" + fixture.expected);
}
for (const fixture of asArray(migrationFixtures.cases)) {
  const observed = migrationItemResult(fixture.item, migrationStatusEnum, dispositionEnum, classificationEnum);
  check(observed === fixture.expected, "FIXTURE_MIGRATION", fixture.id + ":" + observed + "!=" + fixture.expected);
}
for (const fixture of asArray(documentFixtures.front_matter_cases)) {
  const observed = parseFrontMatterResult(fixture.text).code;
  check(observed === fixture.expected, "FIXTURE_FRONT_MATTER", fixture.id + ":" + observed + "!=" + fixture.expected);
}
for (const fixture of asArray(documentFixtures.heading_cases)) {
  const observed = headingResult(fixture.text, fixture.heading);
  check(observed === fixture.expected, "FIXTURE_HEADING", fixture.id + ":" + observed + "!=" + fixture.expected);
}
for (const fixture of asArray(runnerFixtures.cases)) {
  const observed = runnerDefinitionResult(fixture.source, fixture.symbol, fixture.declared_lines);
  check(observed === fixture.expected, "FIXTURE_RUNNER", fixture.id + ":" + observed + "!=" + fixture.expected);
}

for (const [name, artifact] of Object.entries({
  manifest,
  entryInventory,
  migrationInventory,
  storageInventory,
  holdRegistry,
  m2Input
})) {
  const sensitive = findSensitiveMaterial(artifact, forbiddenKeys, allowedKeys, sensitiveSentinels);
  check(sensitive === null, sensitive ? sensitive.code : "ARTIFACT_SENSITIVE", name + ":" + (sensitive ? sensitive.path : ""));
}

const matrixText = readRepoText("docs/contracts/m1-test-matrix-v1.md");
for (const matrixId of [
  "M1-C01", "M1-C02", "M1-C03", "M1-C04", "M1-C05", "M1-C06", "M1-C07", "M1-C08",
  "M1-I01", "M1-I02", "M1-I03", "M1-I04", "M1-S01", "M1-S02", "M1-S03", "M1-M01",
  "M1-H01", "M1-H02", "M1-M2-01", "M1-M2-02", "M1-M2-03", "M1-M2-04", "M1-G01"
]) {
  check(matrixText.includes("| " + matrixId + " |"), "MATRIX_CASE", matrixId);
}
check(matrixText.includes("| 171/171 |"), "MATRIX_COUNT", "tauri");
check(matrixText.includes("| 8/8 |"), "MATRIX_COUNT", "mcp");
check(matrixText.includes("| 68/68 |"), "MATRIX_COUNT", "sqlite");
check(matrixText.includes("| 18/18 |"), "MATRIX_COUNT", "sidecar");
check(matrixText.includes("| 7/7 |"), "MATRIX_COUNT", "projection");
check(matrixText.includes("| 10/10 contracts |"), "MATRIX_COUNT", "contract fixtures");

if (failures.length > 0) {
  failures.sort((left, right) => {
    const codeOrder = left.code.localeCompare(right.code);
    return codeOrder !== 0 ? codeOrder : left.detail.localeCompare(right.detail);
  });
  for (const item of failures) {
    process.stderr.write("SYN_FND_001_R1_VERIFY_FAIL " + item.code + " " + item.detail + "\n");
  }
  process.exit(1);
}

const fixtureCount = contractFixtureCases.length + asArray(ownerFixtures.cases).length +
  asArray(forbiddenFixtures.cases).length + asArray(inventoryFixtures.cases).length +
  asArray(m2Fixtures.cases).length + asArray(m2Fixtures.artifact_cases).length +
  asArray(migrationFixtures.cases).length +
  asArray(documentFixtures.front_matter_cases).length + asArray(documentFixtures.heading_cases).length +
  asArray(runnerFixtures.cases).length;
process.stdout.write(
  "SYN_FND_001_R1_VERIFY_PASS contracts=10 types=" + typeSchemas.size + " actions=" + actionCount +
  " tauri=171 mcp=8 runners=19 sqlite=68 sidecars=18 projections=7 migrations=31 fixtures=" +
  fixtureCount + " evidence=STATIC_OPENING_ONLY\n"
);
