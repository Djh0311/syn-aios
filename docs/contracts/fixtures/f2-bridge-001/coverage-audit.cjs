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
  const fn = `fn ${item.precise_assertion}(`;
  if (!source.includes(fn)) {
    missing.push({
      id: item.id,
      reason: `precise_assertion ${item.precise_assertion} not found as a test fn`,
    });
  }
}

const coveredIds = new Set(cases.map((item) => item.id));
for (const entry of missing) {
  coveredIds.delete(entry.id);
}
const covered = coveredIds.size;
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
    },
    null,
    2,
  ),
);
process.exit(missing.length === 0 && covered === cases.length ? 0 : 1);
