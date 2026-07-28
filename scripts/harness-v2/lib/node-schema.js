'use strict';

const lifecycle = require('./lifecycle');

// Adaptive Harness v0.5 — canonical 节点模型（AH-050-02）
//
// 需求溯源：
//   EX-5  id 永不重用、历史按编号找得回
//   LY-1  parent-id 是唯一的容器关系；层数不钉死
//   LY-3  relations 承载分裂 / 取代
//   LY-4  已完成的事实不被后写内容覆盖（冻结由 store 执行）
//   LY-6  goal 单行——父节点摘要恒定长
//   PK-2  profile 只决定加多少保护，不决定进不进
//   PK-4  长度不决定合法性：本文件不含任何长度判据
//   GIT-4 开工五项（分支 / 可改路径 / 禁止路径 / 本地提交 / 推送）
//   KP-14 verification 的五档诚实分级
//
// 本文件是纯函数、零 IO：解析、校验、序列化。
// 一份节点 = 顶部小型 front matter（机读）+ 下面正文（人读）。不再有第二份 JSON 正文（§4.1）。

// ---------------------------------------------------------------------------
// 取值域
// ---------------------------------------------------------------------------

const NODE_KINDS = Object.freeze(['ROOT_PLAN', 'PHASE_PLAN', 'TASK']);

// profile：保护档位。它只决定要加多少保护，不决定这件事进不进 Harness（PK-2）。
const NODE_PROFILES = Object.freeze(['READ_ONLY', 'ORDINARY_LOCAL', 'STRICT_LOCAL']);

// 可声明的正向关系。SPLIT_INTO / SUPERSEDED_BY / DEPENDED_ON_BY 一律由索引反向生成，
// 不可手写——反向边写在旧节点上就要改一份已冻结的正文（LY-4）。
const RELATION_TYPES = Object.freeze(['SPLIT_FROM', 'REPLACES', 'DEPENDS_ON']);

// KP-14 的五档：有证据通过 / 只是提个醒 / 确实违规 / 证据不足不知道 / 知道该干嘛但被卡住。
const VERIFICATION_STATUSES = Object.freeze(['PASS', 'ADVISORY', 'VIOLATION', 'UNKNOWN', 'BLOCKED']);

// 不属于「完成」族的两档：任何把它们计入通过的实现违反 KP-14。
const VERIFICATION_NOT_DONE = Object.freeze(['UNKNOWN', 'BLOCKED']);

const GIT_DISPOSITIONS = Object.freeze([
  'RETAINED', 'TRANSFERRED', 'READY_FOR_CONFIRMED_REMOVAL', 'REMOVED',
]);

// front matter 上允许出现的全部键。多一个键就是多一个说法（EX-9 / DP-1）。
const NODE_FRONT_MATTER_KEYS = Object.freeze([
  'id',
  'kind',
  'parent-id',
  'lifecycle',
  'goal',
  'profile',
  'write-scope',
  'forbidden-scope',
  'exclusive-resources',
  'acceptance-criteria',
  'verification',
  'git',
  'relations',
  'result',
  'closed-at',
  'confirmations',
]);

const GIT_BINDING_KEYS = Object.freeze([
  'base-branch',
  'base-oid',
  'task-branch',
  'worktree',
  'local-commit-allowed',
  'push-allowed',
  'product-commit',
  'wip-commit',
  'no-product-change',
  'disposition',
  'integrated-observed',
]);

// 开工五项（GIT-4）：缺任一项开工失败，理由指名缺哪一项。
const OPENING_REQUIRED_KEYS = Object.freeze([
  'task-branch', 'write-scope', 'forbidden-scope', 'local-commit-allowed', 'push-allowed',
]);

// 字段矩阵。'required' = 必填；'forbidden' = 必须缺席（出现即失败）；'optional' = 可选。
// 'history-only' = 仅当 lifecycle 为 HISTORY 时必填，其余情形必须缺席。
const FIELD_MATRIX = Object.freeze({
  ROOT_PLAN: Object.freeze({
    id: 'required',
    kind: 'required',
    'parent-id': 'forbidden',
    lifecycle: 'required',
    goal: 'required',
    profile: 'forbidden',
    'write-scope': 'forbidden',
    'forbidden-scope': 'forbidden',
    'exclusive-resources': 'forbidden',
    'acceptance-criteria': 'optional',
    verification: 'forbidden',
    git: 'forbidden',
    relations: 'optional',
    result: 'history-only',
    'closed-at': 'history-only',
    confirmations: 'optional',
  }),
  PHASE_PLAN: Object.freeze({
    id: 'required',
    kind: 'required',
    'parent-id': 'required',
    lifecycle: 'required',
    goal: 'required',
    profile: 'forbidden',
    'write-scope': 'forbidden',
    'forbidden-scope': 'forbidden',
    'exclusive-resources': 'forbidden',
    'acceptance-criteria': 'required',
    verification: 'forbidden',
    git: 'forbidden',
    relations: 'optional',
    result: 'history-only',
    'closed-at': 'history-only',
    confirmations: 'optional',
  }),
  TASK: Object.freeze({
    id: 'required',
    kind: 'required',
    'parent-id': 'required',
    lifecycle: 'required',
    goal: 'required',
    profile: 'required',
    'write-scope': 'required',
    'forbidden-scope': 'required',
    'exclusive-resources': 'required',
    'acceptance-criteria': 'required',
    verification: 'required',
    git: 'conditional',
    relations: 'optional',
    result: 'history-only',
    'closed-at': 'history-only',
    confirmations: 'optional',
  }),
});

// 合法父类型：任务永远是叶子，谁也不许冒充谁（LY-1）。
const LEGAL_PARENT_KINDS = Object.freeze({
  ROOT_PLAN: Object.freeze([]),
  PHASE_PLAN: Object.freeze(['ROOT_PLAN', 'PHASE_PLAN']),
  TASK: Object.freeze(['ROOT_PLAN', 'PHASE_PLAN']),
});

// ---------------------------------------------------------------------------
// 极小 YAML 子集：标量 / 块序列 / 块映射。不引入运行时依赖。
// ---------------------------------------------------------------------------

function indentOf(line) {
  const match = /^[ ]*/.exec(line);
  return match[0].length;
}

function isSkippable(line) {
  const trimmed = line.trim();
  return trimmed === '' || trimmed.startsWith('#');
}

function toScalar(raw) {
  const text = String(raw).trim();
  if (text === '' || text === '~' || text === 'null') return null;
  if (text === 'true') return true;
  if (text === 'false') return false;
  if (text === '[]') return [];
  if (text === '{}') return {};
  if (/^-?\d+$/.test(text)) return Number(text);
  const quoted = (text.length >= 2)
    && ((text[0] === '"' && text[text.length - 1] === '"')
      || (text[0] === "'" && text[text.length - 1] === "'"));
  return quoted ? text.slice(1, -1) : text;
}

function readMapping(lines, state, indent) {
  const out = {};
  while (state.i < lines.length) {
    const line = lines[state.i];
    if (isSkippable(line)) { state.i += 1; continue; }
    const column = indentOf(line);
    if (column < indent) break;
    if (column > indent) { state.i += 1; continue; }
    const match = /^([A-Za-z_][A-Za-z0-9_-]*):(.*)$/.exec(line.trim());
    if (!match) { state.i += 1; continue; }
    const key = match[1];
    const inline = match[2];
    state.i += 1;
    if (inline.trim() !== '') { out[key] = toScalar(inline); continue; }
    let ahead = state.i;
    while (ahead < lines.length && isSkippable(lines[ahead])) ahead += 1;
    if (ahead >= lines.length) { out[key] = null; continue; }
    const nextColumn = indentOf(lines[ahead]);
    const nextTrimmed = lines[ahead].trim();
    const startsSequence = nextTrimmed === '-' || nextTrimmed.startsWith('- ');
    if (startsSequence && nextColumn >= indent) {
      state.i = ahead;
      out[key] = readSequence(lines, state, nextColumn);
      continue;
    }
    if (nextColumn <= indent) { out[key] = null; continue; }
    state.i = ahead;
    out[key] = readMapping(lines, state, nextColumn);
  }
  return out;
}

function readSequence(lines, state, indent) {
  const out = [];
  while (state.i < lines.length) {
    const line = lines[state.i];
    if (isSkippable(line)) { state.i += 1; continue; }
    const column = indentOf(line);
    if (column !== indent) break;
    const trimmed = line.trim();
    if (trimmed !== '-' && !trimmed.startsWith('- ')) break;
    const rest = trimmed.slice(1);
    state.i += 1;
    if (rest.trim() === '') {
      let ahead = state.i;
      while (ahead < lines.length && isSkippable(lines[ahead])) ahead += 1;
      if (ahead < lines.length && indentOf(lines[ahead]) > indent) {
        state.i = ahead;
        const nested = indentOf(lines[ahead]);
        const nestedTrimmed = lines[ahead].trim();
        out.push(nestedTrimmed === '-' || nestedTrimmed.startsWith('- ')
          ? readSequence(lines, state, nested)
          : readMapping(lines, state, nested));
      } else {
        out.push(null);
      }
      continue;
    }
    const inner = /^[ ]*([A-Za-z_][A-Za-z0-9_-]*):(.*)$/.exec(rest);
    if (inner) {
      const itemIndent = column + 1 + (rest.length - rest.trimStart().length);
      const collected = [' '.repeat(itemIndent) + rest.trimStart()];
      while (state.i < lines.length) {
        const follower = lines[state.i];
        if (isSkippable(follower)) { collected.push(follower); state.i += 1; continue; }
        if (indentOf(follower) < itemIndent) break;
        collected.push(follower);
        state.i += 1;
      }
      out.push(readMapping(collected, { i: 0 }, itemIndent));
      continue;
    }
    out.push(toScalar(rest));
  }
  return out;
}

function readFrontMatter(text) {
  const lines = String(text).replace(/\r\n/g, '\n').split('\n');
  if (lines[0] !== '---') return null;
  let end = -1;
  for (let cursor = 1; cursor < lines.length; cursor += 1) {
    if (lines[cursor] === '---') { end = cursor; break; }
  }
  if (end === -1) return null;
  return {
    block: lines.slice(1, end),
    body: lines.slice(end + 1).join('\n'),
  };
}

// ---------------------------------------------------------------------------
// 序列化：canonical 键序，原样往回写
// ---------------------------------------------------------------------------

function emitScalar(value) {
  if (value === null || value === undefined) return '';
  if (typeof value === 'boolean' || typeof value === 'number') return String(value);
  const text = String(value);
  if (text === '') return "''";
  // Git OID 恰好全是数字时，极小 YAML 解析器会把它读成 Number，精度随即丢失。
  // binding 身份必须字节级可回查，因此这类字符串也要作为标量字符串输出。
  if (/^-?\d+$/.test(text)) return `'${text}'`;
  return text;
}

function emitValue(key, value, indent, out) {
  const pad = ' '.repeat(indent);
  if (Array.isArray(value)) {
    if (value.length === 0) { out.push(`${pad}${key}: []`); return; }
    out.push(`${pad}${key}:`);
    for (const item of value) {
      if (item && typeof item === 'object' && !Array.isArray(item)) {
        const keys = Object.keys(item);
        keys.forEach((childKey, offset) => {
          const prefix = offset === 0 ? `${pad}  - ` : `${pad}    `;
          const childValue = item[childKey];
          if (childValue && typeof childValue === 'object' && !Array.isArray(childValue)) {
            out.push(`${prefix}${childKey}:`);
            for (const grandKey of Object.keys(childValue)) {
              out.push(`${pad}      ${grandKey}: ${emitScalar(childValue[grandKey])}`);
            }
          } else if (Array.isArray(childValue)) {
            out.push(`${prefix}${childKey}: ${childValue.length === 0 ? '[]' : ''}`.trimEnd());
            for (const element of childValue) out.push(`${pad}      - ${emitScalar(element)}`);
          } else {
            out.push(`${prefix}${childKey}: ${emitScalar(childValue)}`.trimEnd());
          }
        });
      } else {
        out.push(`${pad}  - ${emitScalar(item)}`);
      }
    }
    return;
  }
  if (value && typeof value === 'object') {
    out.push(`${pad}${key}:`);
    for (const childKey of Object.keys(value)) emitValue(childKey, value[childKey], indent + 2, out);
    return;
  }
  out.push(`${pad}${key}: ${emitScalar(value)}`.trimEnd());
}

function serializeNode(node, body) {
  const front = [];
  for (const key of NODE_FRONT_MATTER_KEYS) {
    if (!Object.prototype.hasOwnProperty.call(node || {}, key)) continue;
    const value = node[key];
    if (value === undefined) continue;
    emitValue(key, value, 0, front);
  }
  const text = String(body === undefined || body === null ? '' : body).replace(/\r\n/g, '\n');
  return `---\n${front.join('\n')}\n---\n${text.startsWith('\n') ? '' : '\n'}${text}`;
}

// ---------------------------------------------------------------------------
// 校验
// ---------------------------------------------------------------------------

function issue(code, field, message) {
  return { code, field, message };
}

function isStringList(value) {
  return Array.isArray(value) && value.every((item) => typeof item === 'string' && item.trim() !== '');
}

function validateGitBinding(binding, issues) {
  if (binding === null || binding === undefined) return;
  if (typeof binding !== 'object' || Array.isArray(binding)) {
    issues.push(issue('GIT_BINDING_MALFORMED', 'git', 'git 必须是一个子映射'));
    return;
  }
  for (const key of Object.keys(binding)) {
    if (!GIT_BINDING_KEYS.includes(key)) {
      issues.push(issue('GIT_BINDING_KEY_UNKNOWN', `git.${key}`, `git 下不接受未登记的键 ${key}`));
    }
  }
  for (const key of ['base-branch', 'base-oid', 'task-branch', 'worktree']) {
    const value = binding[key];
    if (typeof value !== 'string' || value.trim() === '') {
      issues.push(issue('GIT_BINDING_FIELD_MISSING', `git.${key}`, `开工即冻结的 ${key} 缺失`));
    }
  }
  if (typeof binding.worktree === 'string' && binding.worktree.trim() !== '' && !binding.worktree.startsWith('/')) {
    issues.push(issue('GIT_WORKTREE_NOT_ABSOLUTE', 'git.worktree', 'worktree 必须是绝对 realpath'));
  }
  for (const key of ['local-commit-allowed', 'push-allowed']) {
    if (typeof binding[key] !== 'boolean') {
      issues.push(issue('GIT_BINDING_FIELD_MISSING', `git.${key}`, `开工五项之一的 ${key} 必须显式写明`));
    }
  }
  for (const key of ['product-commit', 'wip-commit']) {
    const value = binding[key];
    if (value !== null && value !== undefined
      && (typeof value !== 'string' || !/^(?:[0-9a-f]{40}|[0-9a-f]{64})$/i.test(value))) {
      issues.push(issue(
        'GIT_COMMIT_OID_INVALID',
        `git.${key}`,
        `${key} 必须是不可移动的完整 commit OID，不能填写 HEAD、分支名或短 OID`,
      ));
    }
  }
  if (binding.disposition !== null && binding.disposition !== undefined
    && !GIT_DISPOSITIONS.includes(binding.disposition)) {
    issues.push(issue('GIT_DISPOSITION_UNKNOWN', 'git.disposition', `disposition 取值 ${binding.disposition} 不在四值之内`));
  }
}

function validateVerification(entries, issues) {
  if (entries === null || entries === undefined) return;
  if (!Array.isArray(entries)) {
    issues.push(issue('VERIFICATION_MALFORMED', 'verification', 'verification 必须是条目列表'));
    return;
  }
  const seen = new Set();
  for (const entry of entries) {
    if (!entry || typeof entry !== 'object' || Array.isArray(entry)) {
      issues.push(issue('VERIFICATION_ENTRY_MALFORMED', 'verification', 'verification 条目必须是映射'));
      continue;
    }
    if (typeof entry.id !== 'string' || entry.id.trim() === '') {
      issues.push(issue('VERIFICATION_ID_MISSING', 'verification.id', 'verification 条目缺 id'));
    } else if (seen.has(entry.id)) {
      issues.push(issue('VERIFICATION_ID_DUPLICATE', 'verification.id', `verification 条目 id 重复：${entry.id}`));
    } else {
      seen.add(entry.id);
    }
    if (typeof entry.command !== 'string' || entry.command.trim() === '') {
      issues.push(issue('VERIFICATION_COMMAND_MISSING', 'verification.command', 'verification 条目缺 command 原文'));
    }
    if (typeof entry.required !== 'boolean') {
      issues.push(issue('VERIFICATION_REQUIRED_MISSING', 'verification.required', 'verification 条目必须写明 required'));
    }
    if (!VERIFICATION_STATUSES.includes(entry.status)) {
      issues.push(issue('VERIFICATION_STATUS_UNKNOWN', 'verification.status', `status 取值 ${entry.status} 不在五档之内`));
    }
    if (entry.run !== null && entry.run !== undefined) {
      const run = entry.run;
      if (typeof run !== 'object' || Array.isArray(run)) {
        issues.push(issue('VERIFICATION_RUN_MALFORMED', 'verification.run', 'run 必须是映射'));
      } else {
        for (const key of ['head-oid', 'exit-code', 'output-ref']) {
          if (run[key] === null || run[key] === undefined || run[key] === '') {
            issues.push(issue('VERIFICATION_RUN_INCOMPLETE', `verification.run.${key}`, `run 有则四项齐全，缺 ${key}`));
          }
        }
      }
    }
  }
}

function validateRelations(relations, issues) {
  if (relations === null || relations === undefined) return;
  if (!Array.isArray(relations)) {
    issues.push(issue('RELATIONS_MALFORMED', 'relations', 'relations 必须是列表'));
    return;
  }
  for (const relation of relations) {
    if (!relation || typeof relation !== 'object' || Array.isArray(relation)) {
      issues.push(issue('RELATION_MALFORMED', 'relations', 'relations 条目必须是映射'));
      continue;
    }
    if (!RELATION_TYPES.includes(relation.type)) {
      issues.push(issue('RELATION_TYPE_UNKNOWN', 'relations.type', `只接受可声明的正向关系，收到 ${relation.type}`));
    }
    if (typeof relation['target-id'] !== 'string' || relation['target-id'].trim() === '') {
      issues.push(issue('RELATION_TARGET_MISSING', 'relations.target-id', 'relations 条目缺 target-id'));
    }
    for (const key of Object.keys(relation)) {
      if (!['type', 'target-id', 'note'].includes(key)) {
        issues.push(issue('RELATION_FIELD_UNKNOWN', `relations.${key}`, `relations 条目不存目标的标题 / 状态 / 摘要：${key}`));
      }
    }
  }
}

function presenceOf(node, key) {
  const value = node[key];
  if (value === undefined) return false;
  if (value === null) return false;
  if (Array.isArray(value)) return true;
  if (typeof value === 'string') return value.trim() !== '';
  return true;
}

function validateNode(node, options) {
  const settings = options || {};
  const issues = [];
  if (!node || typeof node !== 'object' || Array.isArray(node)) {
    return { issues: [issue('NODE_MALFORMED', '', '节点必须是一个映射')] };
  }

  for (const key of Object.keys(node)) {
    if (!NODE_FRONT_MATTER_KEYS.includes(key)) {
      issues.push(issue('FIELD_UNKNOWN', key, `front matter 上不接受未登记的键 ${key}`));
    }
  }

  if (typeof node.id !== 'string' || node.id.trim() === '') {
    issues.push(issue('ID_MISSING', 'id', 'id 必填且全局唯一、永不重用'));
  } else if (!/^[\x21-\x7e]+$/.test(node.id)) {
    issues.push(issue('ID_NOT_OPAQUE_ASCII', 'id', 'id 必须是不透明 ASCII 串'));
  }

  if (!NODE_KINDS.includes(node.kind)) {
    issues.push(issue('KIND_UNKNOWN', 'kind', `kind 取值 ${node.kind} 不在三值之内`));
    return { issues };
  }

  const matrix = FIELD_MATRIX[node.kind];
  const isHistory = node.lifecycle === 'HISTORY';
  for (const key of NODE_FRONT_MATTER_KEYS) {
    const rule = matrix[key];
    const present = presenceOf(node, key);
    if (rule === 'required' && !present) {
      issues.push(issue('FIELD_REQUIRED_MISSING', key, `${node.kind} 必填 ${key}`));
    }
    if (rule === 'forbidden' && present) {
      issues.push(issue('FIELD_MUST_BE_ABSENT', key, `${node.kind} 上 ${key} 必须缺席`));
    }
    if (rule === 'history-only') {
      if (isHistory && !present) {
        issues.push(issue('FIELD_REQUIRED_MISSING', key, `进入 HISTORY 时必填 ${key}`));
      }
      if (!isHistory && present) {
        issues.push(issue('FIELD_MUST_BE_ABSENT', key, `未进入 HISTORY 时 ${key} 必须缺席`));
      }
    }
  }

  if (node.kind === 'TASK') {
    const readOnly = node.profile === 'READ_ONLY';
    const hasBinding = presenceOf(node, 'git');
    if (readOnly && hasBinding) {
      issues.push(issue('FIELD_MUST_BE_ABSENT', 'git', 'READ_ONLY 任务不持有 git binding'));
    }
    // DRAFT / READY 尚未取得 branch/worktree binding 时仍可诚实 withdraw、
    // cancel 或 stop。它们没有产品/WIP 事实可冻结，强行要求 git 会把前置
    // 取消做成死路；其余 HISTORY 结果仍必须有 binding 承担产品/WIP 归属。
    const terminalWithoutBindingAllowed = node.lifecycle === 'HISTORY'
      && ['CANCELLED', 'STOPPED'].includes(node.result);
    const bindingRequired = !readOnly && !hasBinding && (
      ['ACTIVE', 'PARKED'].includes(node.lifecycle)
      || (node.lifecycle === 'HISTORY' && !terminalWithoutBindingAllowed)
    );
    if (bindingRequired) {
      issues.push(issue('FIELD_REQUIRED_MISSING', 'git', '已开工的任务必须持有 git binding'));
    }
    if (!NODE_PROFILES.includes(node.profile)) {
      issues.push(issue('PROFILE_UNKNOWN', 'profile', `profile 取值 ${node.profile} 不在三档之内`));
    }
    for (const key of ['write-scope', 'forbidden-scope', 'exclusive-resources']) {
      const value = node[key];
      if (value === undefined || value === null) continue;
      if (!Array.isArray(value)) {
        issues.push(issue('SCOPE_MALFORMED', key, `${key} 必须是列表（可显式为空）`));
      } else if (!isStringList(value)) {
        issues.push(issue('SCOPE_MALFORMED', key, `${key} 的每一项必须是非空字符串`));
      }
    }
    validateVerification(node.verification, issues);
    if (presenceOf(node, 'git')) validateGitBinding(node.git, issues);
  }

  if (presenceOf(node, 'goal')) {
    if (typeof node.goal !== 'string') {
      issues.push(issue('GOAL_MALFORMED', 'goal', 'goal 必须是单行文本'));
    } else if (node.goal.includes('\n')) {
      issues.push(issue('GOAL_NOT_SINGLE_LINE', 'goal', 'goal 禁含换行——父 / 根摘要靠它保持恒定长（LY-6）'));
    }
  }

  if (presenceOf(node, 'acceptance-criteria') && !isStringList(node['acceptance-criteria'])) {
    issues.push(issue('ACCEPTANCE_CRITERIA_MALFORMED', 'acceptance-criteria', 'acceptance-criteria 必须是短句列表'));
  }

  validateRelations(node.relations, issues);

  if (node.kind === 'TASK' && isHistory && Array.isArray(node.relations) && node.relations.length > 0) {
    issues.push(issue('HISTORY_OUTBOUND_RELATION_FORBIDDEN', 'relations', 'HISTORY TASK 不得保留任何正向关系'));
  }

  if (settings.lifecycleValues && !settings.lifecycleValues.includes(node.lifecycle)) {
    issues.push(issue('LIFECYCLE_UNKNOWN', 'lifecycle', `lifecycle 取值 ${node.lifecycle} 不在取值域内`));
  }

  if (isHistory && presenceOf(node, 'result') && !lifecycle.RESULT_VALUES.includes(node.result)) {
    issues.push(issue('RESULT_UNKNOWN', 'result', `HISTORY result 取值 ${node.result} 不在既有结果取值域内`));
  }

  return { issues };
}

// ---------------------------------------------------------------------------
// 解析入口
// ---------------------------------------------------------------------------

function extractTitle(body) {
  const lines = String(body || '').split('\n');
  for (const line of lines) {
    const match = /^#[^\S\r\n]+(.+)$/.exec(line);
    if (match) return match[1].trim();
  }
  return null;
}

function extractSections(body) {
  const sections = new Map();
  const lines = String(body || '').split('\n');
  let currentHeading = null;
  let buffer = [];
  const flush = () => {
    if (currentHeading !== null) sections.set(currentHeading, buffer.join('\n').trim());
  };
  for (const line of lines) {
    const match = /^##[^\S\r\n]+(.+)$/.exec(line);
    if (match) {
      flush();
      currentHeading = match[1].trim();
      buffer = [];
      continue;
    }
    if (currentHeading !== null) buffer.push(line);
  }
  flush();
  return sections;
}

/**
 * 解析一份 canonical 节点。永不抛出：坏输入以 issues 返回。
 * 中文标题、中文正文原样保留（KP-14 / §12 #15）。
 *
 * 长度不进任何判据（PK-4）：本函数不看行数、不看字节数，
 * 正文重复变长不会产生任何新的拒绝理由。
 */
function parseNode(text, options) {
  const settings = options || {};
  const source = typeof text === 'string' ? text : '';
  const front = readFrontMatter(source);
  if (!front) {
    return {
      ok: false,
      relativePath: settings.relativePath || null,
      node: null,
      title: extractTitle(source),
      body: source,
      sections: new Map(),
      issues: [issue('FRONT_MATTER_MISSING', '', '节点顶部必须有 --- 包住的 front matter')],
    };
  }
  let node;
  try {
    node = readMapping(front.block, { i: 0 }, 0);
  } catch (error) {
    return {
      ok: false,
      relativePath: settings.relativePath || null,
      node: null,
      title: extractTitle(front.body),
      body: front.body,
      sections: extractSections(front.body),
      issues: [issue('FRONT_MATTER_MALFORMED', '', 'front matter 解析失败')],
    };
  }
  const validation = validateNode(node, settings);
  const issues = validation.issues.slice();
  const title = extractTitle(front.body);
  if (!title) {
    issues.push(issue('TITLE_MISSING', '', '正文第一行需要一个 # 标题'));
  }
  return {
    ok: issues.length === 0,
    relativePath: settings.relativePath || null,
    node,
    title,
    body: front.body,
    sections: extractSections(front.body),
    issues,
  };
}

/** 只要 issues 的薄壳，方便被当作纯校验入口调用。 */
function validateNodeText(text, options) {
  return { issues: parseNode(text, options).issues };
}

module.exports = {
  NODE_KINDS,
  NODE_PROFILES,
  RELATION_TYPES,
  VERIFICATION_STATUSES,
  VERIFICATION_NOT_DONE,
  GIT_DISPOSITIONS,
  NODE_FRONT_MATTER_KEYS,
  GIT_BINDING_KEYS,
  OPENING_REQUIRED_KEYS,
  FIELD_MATRIX,
  LEGAL_PARENT_KINDS,
  parseNode,
  validateNodeText,
  validateNode,
  serializeNode,
  extractSections,
  extractTitle,
};
