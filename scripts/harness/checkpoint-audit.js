#!/usr/bin/env node

// checkpoint-audit.js — verify a completion report against git ground truth.
//
// This is NOT a re-run of verification-suite/shape-gate (those "run the checks").
// This tool answers a different question: "does what the completion report CLAIMS
// match the git/file ground truth?" It cross-checks the claimed commits, review
// status, declared write-scope, and CURRENT.md checkpoint against reality, and
// marks every mismatch red. Feed it a forged report (claimed commit absent, dirty
// tree, out-of-bounds files, missing review) and it must FAIL.
//
// HONEST BOUNDARY: this verifies MECHANICAL facts only — commit reachable, tree
// clean, files within the declared allow-list, review file + parseable STATUS,
// gates green. It CANNOT verify judgment: whether the diff truly has no behavior
// change, whether the approach has pitfalls. That remains human review.

const fs = require('fs');
const path = require('path');
const { execFileSync } = require('child_process');

const STATUS_TOKENS = ['CLEAR_WITH_P2', 'CLEAR', 'FINDINGS'];
const CURRENT_CONTEXT_PATH = 'docs/project-context.json';
const ALIGNMENT_FIELDS = [
  'authority_chain',
  'plan_anchor',
  'existing_before_new',
  'capabilities_touched',
  'forbidden_alternatives',
];
const CURRENT_ALIGNMENT_BOUNDARY = 'Checks only the declared alignment fields, referenced plan path/heading, and Code Map IDs. It does not prove semantic correctness, code completion, real execution, or product acceptance.';

function parseArgs(argv) {
  const args = {
    target: process.cwd(),
    package: null,
    commit: null,
    taskCommit: null,
    checkpointCommit: null,
    review: null,
    allow: null,
    allowDirty: false,
    skipGates: false,
    json: false,
    record: null,
    current: false,
  };
  for (let i = 0; i < argv.length; i += 1) {
    const a = argv[i];
    if (a === '--target') args.target = argv[++i];
    else if (a === '--package') args.package = argv[++i];
    else if (a === '--commit') args.commit = argv[++i];
    else if (a === '--task-commit') args.taskCommit = argv[++i];
    else if (a === '--checkpoint-commit') args.checkpointCommit = argv[++i];
    else if (a === '--review') args.review = argv[++i];
    else if (a === '--allow') args.allow = argv[++i];
    else if (a === '--allow-dirty') args.allowDirty = true;
    else if (a === '--skip-gates') args.skipGates = true;
    else if (a === '--record') args.record = argv[++i];
    else if (a === '--current') args.current = true;
    else if (a === '--json') args.json = true;
    else if (a === '--help' || a === '-h') { printHelp(); process.exit(0); }
    else throw new Error(`Unknown argument: ${a}`);
  }
  if (args.current && (args.package || args.commit || args.taskCommit || args.checkpointCommit || args.review || args.allow || args.record)) {
    throw new Error('--current cannot be combined with completion-claim arguments.');
  }
  if (!args.current && !args.package && !args.commit) {
    throw new Error('Provide --package <slug> or --commit <sha> (or both). See --help.');
  }
  args.target = path.resolve(args.target);
  return args;
}

function printHelp() {
  console.log(`Usage: node scripts/harness/checkpoint-audit.js (--package <slug> | --commit <sha> | --current) [options]

Verifies a completion report's CLAIMS against git/file ground truth (mechanical only).

Options:
  --current               Audit only the explicitly bound current important task in docs/project-context.json.
                          No binding returns NOT_APPLICABLE; this never scans tasks/* or uses taskPackage.
                          Only currentImportantTask.mode: "strict" marks a user-frozen strict execution entry.
  --package <slug>        Resolve claims from CURRENT.md block + tasks/*<slug>*.md + evidence review file.
  --commit <sha>          Impl commit to audit (overrides/ supplements the parsed one).
  --task-commit <sha>     Claimed task-package commit (else parsed from CURRENT.md).
  --checkpoint-commit <sha>  Claimed checkpoint commit (optional).
  --review <file>         Review evidence file (else parsed/globbed for the slug).
  --allow <globs>         Comma-separated write-scope globs for the file-boundary check (e.g. "docs/**,AGENTS.md").
  --allow-dirty           Do not fail on a dirty working tree (declare expected dirtiness).
  --skip-gates            Skip check (6) (do not spawn shape-gate / verification-suite).
  --record <p[,p...]>     Comma-separated evidence JSON files; check (7) verifies that
                          hash-named string fields are 64-lowercase-hex or a non-hex sentinel.
  --target <dir>          Repo root (default: cwd).
  --json                  Emit JSON.

Boundary: mechanical facts only (commit reachable / tree clean / files in allow-list /
review+STATUS present / gates green / evidence hash-field format). It does NOT judge whether
the diff is behaviorally safe or pitfall-free — that stays human review.`);
}

function git(target, gitArgs) {
  try {
    const out = execFileSync('git', gitArgs, { cwd: target, encoding: 'utf8', stdio: ['ignore', 'pipe', 'ignore'] });
    return { ok: true, out: out.replace(/\n$/, '') };
  } catch (error) {
    return { ok: false, out: (error.stdout || '').toString().replace(/\n$/, ''), code: error.status };
  }
}

function readIf(file) {
  try { return fs.readFileSync(file, 'utf8'); } catch (_e) { return null; }
}

function isSafeRelativePath(value) {
  if (typeof value !== 'string' || !value || /[\0\\\r\n]/.test(value) || path.isAbsolute(value)) return false;
  const segments = value.split('/');
  return segments.every((segment) => segment && segment !== '.' && segment !== '..');
}

function globToRegExp(glob) {
  let re = '';
  for (let i = 0; i < glob.length; i += 1) {
    const c = glob[i];
    if (c === '*') {
      if (glob[i + 1] === '*') { re += '.*'; i += 1; if (glob[i + 1] === '/') i += 1; }
      else re += '[^/]*';
    } else if ('.+?^${}()|[]\\'.includes(c)) re += `\\${c}`;
    else re += c;
  }
  return new RegExp(`^${re}$`);
}

function matchesAny(file, globs) {
  return globs.some((g) => globToRegExp(g).test(file));
}

// --- claim resolution ---------------------------------------------------------

function currentMdParagraphForSlug(target, slug) {
  const text = readIf(path.join(target, 'CURRENT.md'));
  if (!text) return null;
  // limit to the 当前结论 section if present
  const start = text.indexOf('## 当前结论');
  const scoped = start >= 0 ? text.slice(start) : text;
  const paras = scoped.split(/\n\s*\n/);
  const needle = slug.toLowerCase();
  return paras.find((p) => p.toLowerCase().includes(needle)) || null;
}

function extractFromParagraph(p) {
  if (!p) return {};
  const grab = (re) => { const m = p.match(re); return m ? m[1] : null; };
  return {
    implCommit: grab(/implementation commit[^`]*`([0-9a-f]{7,40})`/i),
    taskCommit: grab(/task package commit[^`]*`([0-9a-f]{7,40})`/i),
    status: grab(new RegExp(`STATUS:\\s*(${STATUS_TOKENS.join('|')})`)),
    reviewFile: grab(/`?(evidence\/[^\s`，。)]*review[^\s`，。)]*\.md)`?/i)
  };
}

function findTaskPackage(target, slug) {
  const dir = path.join(target, 'tasks');
  let names = [];
  try { names = fs.readdirSync(dir); } catch (_e) { return null; }
  const hits = names.filter((n) => n.endsWith('.md') && n.toLowerCase().includes(slug.toLowerCase()));
  hits.sort();
  return hits.length ? path.join('tasks', hits[hits.length - 1]) : null;
}

function findReviewFile(target, slug) {
  const dir = path.join(target, 'evidence');
  let names = [];
  try { names = fs.readdirSync(dir); } catch (_e) { return null; }
  const hits = names.filter((n) => n.endsWith('.md') && n.toLowerCase().includes(slug.toLowerCase()) && /review/i.test(n));
  hits.sort();
  return hits.length ? path.join('evidence', hits[hits.length - 1]) : null;
}

function parseAllowFromTaskPackage(target, taskRel) {
  if (!taskRel) return null;
  const text = readIf(path.join(target, taskRel));
  if (!text) return null;
  const m = text.match(/##\s*允许写入([\s\S]*?)(?:\n##\s|\n---|$)/);
  if (!m) return null;
  const section = m[1];
  const globs = [];
  const re = /`([^`]+)`/g;
  let g;
  while ((g = re.exec(section))) {
    let p = g[1].trim();
    if (!/[\/.]/.test(p) && !p.includes('*')) continue; // skip non-path tokens
    if (p.endsWith('/')) p += '**';
    globs.push(p);
  }
  return globs.length ? Array.from(new Set(globs)) : null;
}

// --- checks -------------------------------------------------------------------

function checkCommitsReachable(target, claims) {
  const items = [
    ['task', claims.taskCommit],
    ['impl', claims.implCommit],
    ['checkpoint', claims.checkpointCommit]
  ].filter(([, sha]) => sha);
  if (!items.length) {
    return { id: 'commits_reachable', status: 'na', detail: 'no commit claimed (pass --commit/--task-commit or use --package with a CURRENT.md block)' };
  }
  const results = [];
  let failed = false;
  for (const [role, sha] of items) {
    const exists = git(target, ['cat-file', '-e', `${sha}^{commit}`]).ok;
    const reachable = exists && git(target, ['merge-base', '--is-ancestor', sha, 'HEAD']).ok;
    const verdict = !exists ? 'MISSING' : !reachable ? 'NOT_REACHABLE' : 'ok';
    if (verdict !== 'ok') failed = true;
    results.push({ role, sha, verdict });
  }
  return { id: 'commits_reachable', status: failed ? 'fail' : 'pass', detail: results };
}

function checkTreeClean(target, allowDirty) {
  const r = git(target, ['status', '--porcelain']);
  if (!r.ok) return { id: 'tree_clean', status: 'na', detail: 'not a git repo / git unavailable' };
  const dirty = r.out.split('\n').filter(Boolean);
  if (dirty.length === 0) return { id: 'tree_clean', status: 'pass', detail: 'clean' };
  if (allowDirty) return { id: 'tree_clean', status: 'warn', detail: { declared_dirty: true, entries: dirty.slice(0, 20) } };
  return { id: 'tree_clean', status: 'fail', detail: { dirty_entries: dirty.slice(0, 20), count: dirty.length } };
}

function checkReviewStatus(target, reviewRel) {
  if (!reviewRel) return { id: 'review_status', status: 'fail', detail: 'no review evidence file found/claimed' };
  const text = readIf(path.join(target, reviewRel));
  if (text === null) return { id: 'review_status', status: 'fail', detail: `review file missing: ${reviewRel}` };
  const m = text.match(new RegExp(`STATUS:\\s*(${STATUS_TOKENS.join('|')})`));
  if (!m) return { id: 'review_status', status: 'fail', detail: `review file has no parseable STATUS line: ${reviewRel}` };
  return { id: 'review_status', status: 'pass', detail: { file: reviewRel, status: m[1] } };
}

function checkCurrentRefs(target, slug, claims, cmPara) {
  if (!slug) return { id: 'current_md_refs', status: 'na', detail: 'no --package slug; CURRENT.md cross-ref skipped' };
  if (!cmPara) return { id: 'current_md_refs', status: 'na', detail: 'package not referenced in CURRENT.md top (e.g. harness-line package)' };
  const hasImpl = claims.implCommit ? cmPara.includes(claims.implCommit) : false;
  const hasReview = !!claims.status || (claims.reviewFile && cmPara.includes(claims.reviewFile));
  if (claims.implCommit && hasImpl && hasReview) {
    return { id: 'current_md_refs', status: 'pass', detail: { impl_referenced: true, review_referenced: true } };
  }
  return {
    id: 'current_md_refs',
    status: 'fail',
    detail: { impl_commit: claims.implCommit, impl_referenced: hasImpl, review_referenced: hasReview }
  };
}

function checkFilesWithinAllow(target, claims, allowGlobs) {
  if (!claims.implCommit) return { id: 'files_within_allow', status: 'na', detail: 'no impl commit to inspect' };
  if (!allowGlobs || !allowGlobs.length) return { id: 'files_within_allow', status: 'na', detail: 'allow-list not provided/parseable; pass --allow to verify boundary' };
  const r = git(target, ['show', '--name-only', '--format=', claims.implCommit]);
  if (!r.ok) return { id: 'files_within_allow', status: 'fail', detail: `cannot read impl commit files: ${claims.implCommit}` };
  const files = r.out.split('\n').map((s) => s.trim()).filter(Boolean);
  const outOfBounds = files.filter((f) => !matchesAny(f, allowGlobs));
  if (outOfBounds.length) {
    return { id: 'files_within_allow', status: 'fail', detail: { allow: allowGlobs, out_of_bounds: outOfBounds, total_files: files.length } };
  }
  return { id: 'files_within_allow', status: 'pass', detail: { allow: allowGlobs, files_checked: files.length } };
}

function checkGates(target, skipGates) {
  if (skipGates) return { id: 'gates_green', status: 'na', detail: 'skipped (--skip-gates)' };
  const gate = path.join(target, 'scripts/harness/workbench-shape-gate.js');
  if (!fs.existsSync(gate)) return { id: 'gates_green', status: 'na', detail: 'shape-gate not found at target; nothing to delegate to' };
  // Delegate to the existing gate; do not reimplement verification.
  try {
    execFileSync(process.execPath, [gate, '--mode', 'check', '--target', target], { cwd: target, encoding: 'utf8', stdio: ['ignore', 'pipe', 'pipe'] });
    return { id: 'gates_green', status: 'pass', detail: 'workbench-shape-gate --mode check: pass (delegated)' };
  } catch (error) {
    return { id: 'gates_green', status: 'fail', detail: `workbench-shape-gate --mode check: fail (exit ${error.status})` };
  }
}

// --- orchestration ------------------------------------------------------------

// Check (7): mechanically classify hash-named string fields inside evidence JSON.
// A value is OK if it is 64-char lowercase hex (sha256) OR a non-hex sentinel string
// (e.g. "missing_before=true", "not_created"). A value that looks like a hash but is
// not 64 lowercase hex (wrong length like the real B1 59-char case, or uppercase) is a
// defect. Key heuristic is intentionally narrow: only keys matching /(sha256|hash)/i are
// inspected, so git_head_* (40-hex) and agent_id (uuid) are never picked up.
function classifyHashValue(value) {
  if (/^[0-9a-f]{64}$/.test(value)) return { ok: true };
  if (/^[0-9a-fA-F]{8,}$/.test(value)) return { ok: false, reason: 'not_64_lowercase_hex' };
  return { ok: true };
}

function collectHashFields(node, key, segs, out) {
  if (typeof node === 'string') {
    if (key && /(sha256|hash)/i.test(key)) out.push({ keyPath: segs.join('.'), key, value: node });
    return;
  }
  if (Array.isArray(node)) {
    node.forEach((el, i) => collectHashFields(el, key, segs.concat(`[${i}]`), out)); // keep enclosing key
    return;
  }
  if (node && typeof node === 'object') {
    for (const k of Object.keys(node)) collectHashFields(node[k], k, segs.concat(k), out);
  }
}

function checkEvidenceHashFormat(target, recordPaths) {
  if (!recordPaths || !recordPaths.length) {
    return { id: 'evidence_hash_format', status: 'na', detail: 'no --record; evidence JSON hash fields not inspected' };
  }
  const violations = [];
  const errors = [];
  let fieldsChecked = 0;
  for (const rel of recordPaths) {
    const abs = path.isAbsolute(rel) ? rel : path.join(target, rel);
    const text = readIf(abs);
    if (text === null) { errors.push({ file: rel, reason: 'record_unreadable' }); continue; }
    let data;
    try { data = JSON.parse(text); } catch (_e) { errors.push({ file: rel, reason: 'record_unparseable' }); continue; }
    const fields = [];
    collectHashFields(data, null, [], fields);
    for (const f of fields) {
      fieldsChecked += 1;
      const cls = classifyHashValue(f.value);
      if (!cls.ok) violations.push({ file: rel, keyPath: f.keyPath, value: f.value, reason: cls.reason });
    }
  }
  if (violations.length || errors.length) {
    return { id: 'evidence_hash_format', status: 'fail', detail: { violations, errors, fields_checked: fieldsChecked } };
  }
  return { id: 'evidence_hash_format', status: 'pass', detail: { files_checked: recordPaths.length, hash_fields_checked: fieldsChecked } };
}

// --- current important-task alignment ----------------------------------------

function parseJsonText(text) {
  try { return { value: JSON.parse(text) }; }
  catch (_e) { return { error: 'invalid_json' }; }
}

function readCurrentImportantTaskBinding(target) {
  const contextText = readIf(path.join(target, CURRENT_CONTEXT_PATH));
  if (contextText === null) return { applicable: false, reason: 'NO_CURRENT_IMPORTANT_TASK_PACKAGE' };
  const parsed = parseJsonText(contextText);
  if (parsed.error || !parsed.value || typeof parsed.value !== 'object' || Array.isArray(parsed.value)) {
    return { applicable: true, issue: { code: 'CURRENT_IMPORTANT_TASK_CONTEXT_INVALID', path: CURRENT_CONTEXT_PATH } };
  }
  const checkpoint = parsed.value.checkpoint;
  const current = checkpoint && typeof checkpoint === 'object' && !Array.isArray(checkpoint)
    ? checkpoint.currentImportantTask
    : null;
  if (current === null || current === undefined) return { applicable: false, reason: 'NO_CURRENT_IMPORTANT_TASK_PACKAGE' };
  if (!current || typeof current !== 'object' || Array.isArray(current)) {
    return { applicable: true, issue: { code: 'CURRENT_IMPORTANT_TASK_BINDING_INVALID', path: CURRENT_CONTEXT_PATH } };
  }
  if (!isSafeRelativePath(current.path) || !/^tasks\/.+\.md$/.test(current.path)) {
    return {
      applicable: true,
      issue: { code: 'CURRENT_IMPORTANT_TASK_PATH_INVALID', path: current.path || null, contextPath: CURRENT_CONTEXT_PATH },
      mode: current.mode === 'strict' ? 'strict' : 'advisory',
    };
  }
  return {
    applicable: true,
    binding: { path: current.path, mode: current.mode === 'strict' ? 'strict' : 'advisory' },
  };
}

function parseAuthorityPlanAlignment(taskText) {
  const heading = /^##[ \t]+Authority and plan alignment[ \t]*$/im.exec(taskText);
  const fields = Object.fromEntries(ALIGNMENT_FIELDS.map((field) => [field, null]));
  if (!heading) return { found: false, fields };
  const afterHeading = taskText.slice(heading.index + heading[0].length);
  const nextHeading = afterHeading.search(/^##\s+/m);
  const section = nextHeading >= 0 ? afterHeading.slice(0, nextHeading) : afterHeading;
  for (const field of ALIGNMENT_FIELDS) {
    const line = new RegExp(`^[ \\t]*[-*][ \\t]+${field}:[ \\t]*(.*)$`, 'mi').exec(section);
    fields[field] = line ? line[1].trim() : null;
  }
  return { found: true, fields };
}

function normalizeHeading(value) {
  let decoded = value;
  try { decoded = decodeURIComponent(value); } catch (_e) { /* keep raw anchor */ }
  return decoded
    .normalize('NFKC')
    .replace(/^#+/, '')
    .trim()
    .toLocaleLowerCase()
    .replace(/[^\p{L}\p{N}]+/gu, '-')
    .replace(/^-+|-+$/g, '');
}

function parsePlanAnchor(value) {
  if (typeof value !== 'string') return null;
  const link = /\]\(([^)]+)\)/.exec(value);
  const direct = /(?:^|[\s`])([^\s`]+\.md#[^\s`]+)/.exec(value);
  const target = link ? link[1] : direct ? direct[1] : null;
  if (!target) return null;
  const hashAt = target.indexOf('#');
  if (hashAt <= 0 || hashAt === target.length - 1) return null;
  const planPath = target.slice(0, hashAt);
  const anchor = target.slice(hashAt + 1);
  if (!isSafeRelativePath(planPath) || !anchor || !normalizeHeading(anchor)) return null;
  return { path: planPath, anchor };
}

function planHasHeading(text, anchor) {
  const expected = normalizeHeading(anchor);
  return text.split(/\r?\n/).some((line) => {
    const match = /^#{1,6}\s+(.+?)\s*#*\s*$/.exec(line);
    return !!match && normalizeHeading(match[1]) === expected;
  });
}

function loadCodeMapCapabilities(target) {
  const indexRel = 'docs/code-map/index.json';
  const indexText = readIf(path.join(target, indexRel));
  if (indexText === null) return { issue: { code: 'CODE_MAP_UNAVAILABLE', path: indexRel } };
  const indexParsed = parseJsonText(indexText);
  if (indexParsed.error || !Array.isArray(indexParsed.value.domains)) {
    return { issue: { code: 'CODE_MAP_UNAVAILABLE', path: indexRel } };
  }
  const capabilities = new Map();
  for (const domain of indexParsed.value.domains) {
    if (!domain || !isSafeRelativePath(domain.path)) {
      return { issue: { code: 'CODE_MAP_UNAVAILABLE', path: indexRel } };
    }
    const domainText = readIf(path.join(target, domain.path));
    const domainParsed = domainText === null ? { error: 'unreadable' } : parseJsonText(domainText);
    if (domainParsed.error || !Array.isArray(domainParsed.value.capabilities)) {
      return { issue: { code: 'CODE_MAP_UNAVAILABLE', path: domain.path } };
    }
    for (const capability of domainParsed.value.capabilities) {
      if (capability && typeof capability.id === 'string' && typeof capability.status === 'string') {
        capabilities.set(capability.id, capability.status);
      }
    }
  }
  return { capabilities };
}

function mapIdsIn(value) {
  if (typeof value !== 'string') return [];
  return Array.from(new Set(value.match(/\b[a-z][a-z0-9-]*\.[a-z0-9-]+\b/g) || []));
}

function isNoneWithExplanation(value) {
  if (typeof value !== 'string' || !/^none(?:\s|$)/i.test(value)) return null;
  const explanation = value.slice(4).replace(/^[\s:—-]+/, '').trim();
  return { explained: explanation.length > 0, explanation };
}

function currentReport(target, binding, status, fields, warnings, errors, extra = {}) {
  return {
    target,
    mode: 'current',
    boundary: CURRENT_ALIGNMENT_BOUNDARY,
    status,
    verdict: status,
    currentImportantTask: binding ? { path: binding.path, mode: binding.mode } : null,
    fields,
    warnings,
    errors,
    ...extra,
  };
}

function auditCurrentAlignment(args) {
  const bindingResult = readCurrentImportantTaskBinding(args.target);
  if (!bindingResult.applicable) {
    return currentReport(args.target, null, 'NOT_APPLICABLE', {}, [], [], { reason: bindingResult.reason });
  }

  const binding = bindingResult.binding || { path: null, mode: bindingResult.mode || 'advisory' };
  const warnings = [];
  const errors = [];
  const requiredIssues = [];
  const add = (issue, required = false) => {
    if (required) requiredIssues.push(issue);
    if (required && binding.mode === 'strict') errors.push(issue);
    else warnings.push(issue);
  };

  if (bindingResult.issue) {
    add(bindingResult.issue, true);
    return currentReport(args.target, binding, errors.length ? 'STRICT_ALIGNMENT_ERRORS' : 'FIELDS_INCOMPLETE', {}, warnings, errors);
  }

  const taskText = readIf(path.join(args.target, binding.path));
  if (taskText === null) {
    add({ code: 'CURRENT_IMPORTANT_TASK_PACKAGE_MISSING', path: binding.path }, true);
    return currentReport(args.target, binding, errors.length ? 'STRICT_ALIGNMENT_ERRORS' : 'FIELDS_INCOMPLETE', {}, warnings, errors);
  }

  const alignment = parseAuthorityPlanAlignment(taskText);
  if (!alignment.found) add({ code: 'ALIGNMENT_BLOCK_MISSING', path: binding.path }, true);
  for (const field of ALIGNMENT_FIELDS) {
    if (!alignment.fields[field]) add({ code: 'ALIGNMENT_FIELD_MISSING', field, path: binding.path }, true);
  }

  const planAnchor = alignment.fields.plan_anchor ? parsePlanAnchor(alignment.fields.plan_anchor) : null;
  if (alignment.fields.plan_anchor) {
    if (!planAnchor) {
      add({ code: 'PLAN_ANCHOR_INVALID', field: 'plan_anchor', value: alignment.fields.plan_anchor }, false);
    } else {
      const planText = readIf(path.join(args.target, planAnchor.path));
      if (planText === null) add({ code: 'PLAN_ANCHOR_FILE_MISSING', path: planAnchor.path, anchor: planAnchor.anchor }, false);
      else if (!planHasHeading(planText, planAnchor.anchor)) add({ code: 'PLAN_ANCHOR_HEADING_MISSING', path: planAnchor.path, anchor: planAnchor.anchor }, false);
    }
  }

  const capabilitiesValue = alignment.fields.capabilities_touched;
  const none = isNoneWithExplanation(capabilitiesValue);
  if (none && !none.explained) add({ code: 'CAPABILITIES_TOUCHED_NONE_NEEDS_EXPLANATION', field: 'capabilities_touched' }, true);

  const touchedIds = none ? [] : mapIdsIn(capabilitiesValue);
  if (capabilitiesValue && !none && touchedIds.length === 0) {
    add({ code: 'CAPABILITY_MAP_ID_MISSING', field: 'capabilities_touched', value: capabilitiesValue }, false);
  }
  const reusedIds = mapIdsIn(alignment.fields.existing_before_new);
  if (alignment.fields.existing_before_new && /(?:\breuse\b|复用)/i.test(alignment.fields.existing_before_new) && reusedIds.length === 0) {
    add({ code: 'EXISTING_BEFORE_NEW_MAP_ID_MISSING', field: 'existing_before_new' }, false);
  }

  const declaredIds = [
    ...touchedIds.map((id) => ({ id, field: 'capabilities_touched' })),
    ...reusedIds.map((id) => ({ id, field: 'existing_before_new' })),
  ];
  if (declaredIds.length) {
    const loadedMap = loadCodeMapCapabilities(args.target);
    if (loadedMap.issue) add(loadedMap.issue, false);
    else {
      for (const { id, field } of declaredIds) {
        const status = loadedMap.capabilities.get(id);
        if (!status) add({ code: 'MAP_CAPABILITY_NOT_FOUND', id, field }, false);
        else if (status === 'legacy') add({ code: 'MAP_CAPABILITY_LEGACY', id, field, status }, false);
        else if (status === 'needs-confirmation') add({ code: 'MAP_CAPABILITY_NEEDS_CONFIRMATION', id, field, status }, false);
      }
    }
  }

  const fieldsPresent = requiredIssues.length === 0;
  const status = errors.length ? 'STRICT_ALIGNMENT_ERRORS' : fieldsPresent ? 'FIELDS_PRESENT' : 'FIELDS_INCOMPLETE';
  return currentReport(args.target, binding, status, alignment.fields, warnings, errors, {
    alignmentBlockFound: alignment.found,
    capabilityIds: declaredIds.map(({ id, field }) => ({ id, field })),
  });
}

function audit(args) {
  const slug = args.package;
  const cmPara = slug ? currentMdParagraphForSlug(args.target, slug) : null;
  const parsed = extractFromParagraph(cmPara);
  const taskRel = slug ? findTaskPackage(args.target, slug) : null;

  const claims = {
    implCommit: args.commit || parsed.implCommit || null,
    taskCommit: args.taskCommit || parsed.taskCommit || null,
    checkpointCommit: args.checkpointCommit || null,
    status: parsed.status || null,
    reviewFile: args.review || parsed.reviewFile || (slug ? findReviewFile(args.target, slug) : null)
  };
  const allowGlobs = args.allow ? args.allow.split(',').map((s) => s.trim()).filter(Boolean)
    : parseAllowFromTaskPackage(args.target, taskRel);

  const recordPaths = args.record ? args.record.split(',').map((s) => s.trim()).filter(Boolean) : null;
  const checks = [
    checkCommitsReachable(args.target, claims),
    checkTreeClean(args.target, args.allowDirty),
    checkReviewStatus(args.target, claims.reviewFile),
    checkCurrentRefs(args.target, slug, claims, cmPara),
    checkFilesWithinAllow(args.target, claims, allowGlobs),
    checkGates(args.target, args.skipGates),
    checkEvidenceHashFormat(args.target, recordPaths)
  ];

  const failed = checks.filter((c) => c.status === 'fail');
  return {
    target: args.target,
    boundary: 'MECHANICAL facts only (commit reachable / tree clean / files in allow-list / review+STATUS present / gates green / evidence hash-field format). Does NOT judge behavior-change or pitfalls — human review still required.',
    package: slug,
    resolved: { claims, allow: allowGlobs, record: recordPaths, task_package: taskRel, current_md_block_found: !!cmPara },
    checks,
    verdict: failed.length ? 'FAIL' : 'PASS',
    failed_checks: failed.map((c) => c.id)
  };
}

function printReport(r) {
  console.log(`checkpoint-audit: ${r.target}`);
  console.log(`Package: ${r.package || '(commit-only)'}`);
  console.log(`Boundary: ${r.boundary}`);
  console.log('');
  console.log('Resolved claims:');
  console.log(`- impl commit:   ${r.resolved.claims.implCommit || '(none)'}`);
  console.log(`- task commit:   ${r.resolved.claims.taskCommit || '(none)'}`);
  console.log(`- review file:   ${r.resolved.claims.reviewFile || '(none)'}`);
  console.log(`- review STATUS: ${r.resolved.claims.status || '(parsed at check)'}`);
  console.log(`- allow-list:    ${r.resolved.allow ? r.resolved.allow.join(', ') : '(none)'}`);
  console.log(`- record files:  ${r.resolved.record ? r.resolved.record.join(', ') : '(none)'}`);
  console.log(`- CURRENT.md block found: ${r.resolved.current_md_block_found}`);
  console.log('');
  console.log('Checks:');
  for (const c of r.checks) {
    const mark = c.status === 'pass' ? 'PASS' : c.status === 'fail' ? 'FAIL' : c.status.toUpperCase();
    const detail = typeof c.detail === 'string' ? c.detail : JSON.stringify(c.detail);
    console.log(`- [${mark}] ${c.id}: ${detail}`);
  }
  console.log('');
  console.log(`VERDICT: ${r.verdict}${r.failed_checks.length ? ` (failed: ${r.failed_checks.join(', ')})` : ''}`);
}

function printCurrentReport(r) {
  console.log(`checkpoint-audit current: ${r.target}`);
  console.log(`Status: ${r.status}`);
  console.log(`Boundary: ${r.boundary}`);
  if (r.reason) console.log(`Reason: ${r.reason}`);
  if (r.currentImportantTask) {
    console.log(`Current important task: ${r.currentImportantTask.path} (${r.currentImportantTask.mode})`);
  }
  for (const [field, value] of Object.entries(r.fields || {})) {
    console.log(`- ${field}: ${value || '(missing)'}`);
  }
  for (const warning of r.warnings || []) console.log(`- [WARN] ${warning.code}: ${JSON.stringify(warning)}`);
  for (const error of r.errors || []) console.log(`- [FAIL] ${error.code}: ${JSON.stringify(error)}`);
}

try {
  const args = parseArgs(process.argv.slice(2));
  const report = args.current ? auditCurrentAlignment(args) : audit(args);
  if (args.json) console.log(JSON.stringify(report, null, 2));
  else if (args.current) printCurrentReport(report);
  else printReport(report);
  process.exit(args.current ? (report.errors.length ? 1 : 0) : (report.verdict === 'PASS' ? 0 : 1));
} catch (error) {
  console.error(`ERROR: ${error.message}`);
  process.exit(2);
}
