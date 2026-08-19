#!/usr/bin/env node
const { readFileSync } = require("node:fs");
const { dirname, resolve } = require("node:path");

const here = dirname(__filename);
const fixturePath = resolve(here, "contract-cases-v1.json");
const sourcePath = resolve(
  here,
  "../../../../prototypes/productized-desktop-shell/src-tauri/src/f2_shell_core_bridge.rs",
);

const fixture = JSON.parse(readFileSync(fixturePath, "utf8"));
const source = readFileSync(sourcePath, "utf8");
const cases = fixture.cases;
if (!Array.isArray(cases) || cases.length === 0) {
  console.error("coverage-audit: no cases");
  process.exit(1);
}

const required = fixture.required_keys;
const missing = [];
let behavior = 0;
let document = 0;

function findMatchingBrace(text, openIndex) {
  let depth = 0;
  let quote = null;
  let rawHashCount = null;
  let lineComment = false;
  let blockCommentDepth = 0;

  for (let index = openIndex; index < text.length; index += 1) {
    const char = text[index];
    const next = text[index + 1];

    if (lineComment) {
      if (char === "\n") lineComment = false;
      continue;
    }
    if (blockCommentDepth > 0) {
      if (char === "/" && next === "*") {
        blockCommentDepth += 1;
        index += 1;
      } else if (char === "*" && next === "/") {
        blockCommentDepth -= 1;
        index += 1;
      }
      continue;
    }
    if (rawHashCount !== null) {
      if (char === '"' && text.slice(index + 1, index + 1 + rawHashCount) === "#".repeat(rawHashCount)) {
        const closingHashCount = rawHashCount;
        rawHashCount = null;
        index += closingHashCount;
      }
      continue;
    }
    if (quote !== null) {
      if (char === "\\") {
        index += 1;
      } else if (char === quote) {
        quote = null;
      }
      continue;
    }
    if (char === "/" && next === "/") {
      lineComment = true;
      index += 1;
      continue;
    }
    if (char === "/" && next === "*") {
      blockCommentDepth = 1;
      index += 1;
      continue;
    }
    if (char === "r") {
      const rawStart = text.slice(index).match(/^r(#+)?"/);
      if (rawStart) {
        rawHashCount = rawStart[1]?.length ?? 0;
        index += rawStart[0].length - 1;
        continue;
      }
    }
    if (char === '"' || char === "'") {
      quote = char;
      continue;
    }
    if (char === "{") {
      depth += 1;
    } else if (char === "}") {
      depth -= 1;
      if (depth === 0) return index;
    }
  }
  return -1;
}

function parseRustTestFunctions(text) {
  const functions = new Map();
  const testAttribute = /#\[test\]\s*fn\s+([A-Za-z_][A-Za-z0-9_]*)\s*\(/g;
  let match;
  while ((match = testAttribute.exec(text)) !== null) {
    const signatureStart = match.index;
    const bodyOpen = text.indexOf("{", testAttribute.lastIndex);
    if (bodyOpen === -1) continue;
    const bodyClose = findMatchingBrace(text, bodyOpen);
    if (bodyClose === -1) continue;
    functions.set(match[1], {
      name: match[1],
      signatureStart,
      bodyStart: bodyOpen + 1,
      bodyEnd: bodyClose,
      body: text.slice(bodyOpen + 1, bodyClose),
    });
    testAttribute.lastIndex = bodyClose + 1;
  }
  return functions;
}

const testFunctions = parseRustTestFunctions(source);
const caseToFunction = {};
const assertionCaseIds = new Map();

for (const item of cases) {
  for (const key of required) {
    if (item[key] === undefined || item[key] === null || item[key] === "") {
      missing.push({ id: item.id, reason: `missing ${key}` });
    }
  }
  if (!["BEHAVIOR", "DOCUMENT"].includes(item.case_class)) {
    missing.push({ id: item.id, reason: "case_class must be BEHAVIOR or DOCUMENT" });
    continue;
  }
  if (item.case_class === "BEHAVIOR") {
    behavior += 1;
  } else {
    document += 1;
  }
  const testFunction = testFunctions.get(item.precise_assertion);
  const caseIdLiteral = JSON.stringify(item.id);
  const bodyContainsCaseId = testFunction?.body.includes(caseIdLiteral) ?? false;
  caseToFunction[item.id] = {
    precise_assertion: item.precise_assertion,
    test_function: testFunction?.name ?? null,
    case_id_in_body: bodyContainsCaseId,
  };
  if (!testFunction) {
    missing.push({
      id: item.id,
      reason: `precise_assertion ${item.precise_assertion} not found as #[test] fn`,
    });
  } else if (!bodyContainsCaseId) {
    missing.push({
      id: item.id,
      reason: `#[test] fn ${testFunction.name} body does not contain case id ${item.id}`,
    });
  }
  if (!assertionCaseIds.has(item.precise_assertion)) assertionCaseIds.set(item.precise_assertion, []);
  assertionCaseIds.get(item.precise_assertion).push(item.id);
}

const coveredIds = new Set(cases.map((item) => item.id));
for (const entry of missing) {
  coveredIds.delete(entry.id);
}
const covered = coveredIds.size;
const preciseAssertionManyToOne = [...assertionCaseIds.entries()]
  .map(([preciseAssertion, caseIds]) => ({
    precise_assertion: preciseAssertion,
    test_function: testFunctions.get(preciseAssertion)?.name ?? null,
    case_count: caseIds.length,
    case_ids: caseIds,
  }))
  .filter((entry) => entry.case_count > 1);
console.log(
  JSON.stringify(
    {
      fixture: fixturePath,
      cases: cases.length,
      behavior,
      document,
      covered_with_precise_assertion: covered,
      required_keys_only_does_not_count: true,
      missing,
      percent: `${((covered / cases.length) * 100).toFixed(1)}%`,
      case_to_function: caseToFunction,
      precise_assertion_many_to_one: {
        groups: preciseAssertionManyToOne,
        group_count: preciseAssertionManyToOne.length,
      },
    },
    null,
    2,
  ),
);
process.exit(missing.length === 0 && covered === cases.length ? 0 : 1);
