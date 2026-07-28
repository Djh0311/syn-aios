'use strict';

// Adaptive Harness v0.5 — 精确提交与提交自述（AH-050-07）
//
// 需求溯源：
//   GIT-5  每个提交只装本次任务范围内的改动，不把不明来源的东西一把捞进去；
//          一个提交要能自己说清为什么改、改了什么、怎么验证。
//   GIT-4  可改路径 / 禁止路径是提交的判据基准，不是提示语。
//   §6.2   不得整树暂存、不得覆盖用户已有 staged 内容、不得把别的任务或
//          用户原有未提交内容混进提交、不得自动 reset / stash / clean / rebase / push。
//   §6.4   越界三行表：落在禁区拒绝；与他人相交拒绝并停机；都不相交则回填。
//
// 本模块**只出计划与判定，不执行任何 Git 写操作**。真正的 commit 由调用方
// 在取得授权后执行；这里跑的全是只读查询。

const gitFacts = require('./git-facts');
const scope = require('./scope');

// 提交信息的三段。缺一段就拒绝提交——「一个提交要能自己说清楚」是判据，不是风格建议。
const COMMIT_MESSAGE_SECTIONS = Object.freeze([
  { key: 'why', label: '为什么改', aliases: ['为什么改', '为什么', 'why', 'rationale', 'reason'] },
  { key: 'what', label: '改了什么', aliases: ['改了什么', 'what', 'changes', 'what changed'] },
  { key: 'verify', label: '怎么验证', aliases: ['怎么验证', '如何验证', 'verify', 'verification', 'how verified'] },
]);

const TASK_ID_PATTERN = /(?:^|\s)(?:task|任务)[：: ]\s*([A-Za-z0-9][A-Za-z0-9._/-]{1,60})/i;

// ---------------------------------------------------------------------------
// 提交信息
// ---------------------------------------------------------------------------

/**
 * 解析一条提交信息。要求任务标识 + 三段自述（为什么改 / 改了什么 / 怎么验证）。
 * 三段用 `段名：正文` 的行首标记，中英文皆可；缺哪一段就报哪一段。
 */
function parseCommitMessage(text) {
  const raw = String(text === null || text === undefined ? '' : text).replace(/\r\n/g, '\n');
  const lines = raw.split('\n');
  const sections = {};
  let current = null;
  for (const line of lines) {
    const match = /^\s*([^：:]{1,24})[：:]\s*(.*)$/.exec(line);
    if (match) {
      const heading = match[1].trim().toLowerCase();
      const spec = COMMIT_MESSAGE_SECTIONS
        .find((entry) => entry.aliases.some((alias) => heading === alias.toLowerCase()));
      if (spec) {
        current = spec.key;
        sections[current] = match[2].trim();
        continue;
      }
    }
    if (current && line.trim() !== '') {
      sections[current] = `${sections[current] || ''}\n${line.trim()}`.trim();
    }
  }
  const taskMatch = TASK_ID_PATTERN.exec(raw);
  const issues = [];
  const taskId = taskMatch ? taskMatch[1] : null;
  if (!taskId) {
    issues.push({ code: 'COMMIT_MESSAGE_TASK_ID_MISSING', field: 'task', message: '提交信息里没有任务标识，说不清这笔改动属于谁' });
  }
  for (const spec of COMMIT_MESSAGE_SECTIONS) {
    const value = sections[spec.key];
    if (typeof value !== 'string' || value.trim() === '') {
      issues.push({
        code: 'COMMIT_MESSAGE_SECTION_MISSING',
        field: spec.key,
        message: `提交信息缺「${spec.label}」这一段；三段缺一即拒绝提交`,
      });
    }
  }
  return { taskId, sections, issues, ok: issues.length === 0 };
}

/** 按三段格式渲染一条合格的提交信息。 */
function renderCommitMessage(input) {
  const settings = input || {};
  const lines = [String(settings.subject || '').trim() || '（缺标题）', ''];
  lines.push(`任务：${settings.taskId || '（缺任务标识）'}`);
  for (const spec of COMMIT_MESSAGE_SECTIONS) {
    lines.push(`${spec.label}：${String(settings[spec.key] || '').trim()}`);
  }
  return lines.join('\n');
}

// ---------------------------------------------------------------------------
// 暂存现实
// ---------------------------------------------------------------------------

/** 当前已暂存的路径清单。只读查询。 */
function stagedPaths(cwd) {
  const result = gitFacts.runGit(['diff', '--cached', '--name-only'], { cwd: cwd || process.cwd() });
  if (!result.ok) throw new gitFacts.GitFactError('GIT_STAGED_UNREADABLE', '读不到已暂存的路径清单');
  return result.stdout.split('\n').map((line) => line.trim()).filter((line) => line !== '');
}

function unique(list) {
  return [...new Set(list)].sort();
}

// ---------------------------------------------------------------------------
// 精确提交计划
// ---------------------------------------------------------------------------

/**
 * 提交前的完整判定。四道检查，全过才给计划：
 *
 *   1. 只装点名的那一批：暂存内容与点名的**精确路径集合**必须完全一致。
 *      多出来的一律视为不明来源，拒绝；点了名却没暂存的同样拒绝。
 *      整树暂存在这里根本表达不出来——计划里只有逐条点名的路径。
 *   2. 点名的每条路径都要落在**任务声明的写面**之内。这一步的基准是开工时冻结的
 *      write-scope / forbidden-scope，不是命令行上重复报一遍的那份清单。
 *   3. 越界按 §6.4 的三行表处理：落在禁区拒绝且不回填；与他人在册声明相交则拒绝
 *      并显式停机；两者都不是才允许并回填。
 *   4. 提交信息三段齐备。
 */
function planExactCommit(input) {
  const settings = input || {};
  const cwd = settings.cwd || process.cwd();
  const named = unique(Array.isArray(settings.paths) ? settings.paths.map((item) => scope.normalizePath(item)) : []);
  const declaration = scope.readDeclaration(settings.declaration);
  const exempt = settings.controlPlaneExempt || scope.CONTROL_PLANE_EXEMPT_PREFIXES;
  const refusals = [];

  if (named.length === 0) {
    refusals.push({ code: 'COMMIT_PATHS_REQUIRED', message: '提交必须逐条点名路径；不接受整树暂存，也不接受空清单' });
  }
  for (const candidate of named) {
    if (candidate === '.' || candidate === '' || candidate.includes('*')) {
      refusals.push({ code: 'COMMIT_PATH_NOT_EXACT', path: candidate, message: `点名路径「${candidate}」不是一条精确路径` });
    }
  }

  // 第 1 道：暂存内容必须与点名的路径集合逐条相符。
  let staged = [];
  let stagedReadable = true;
  try {
    staged = unique(settings.stagedPaths ? settings.stagedPaths.map((item) => scope.normalizePath(item)) : stagedPaths(cwd));
  } catch (error) {
    stagedReadable = false;
    refusals.push({ code: 'COMMIT_STAGED_UNREADABLE', message: `读不到暂存现实，报冲突并停：${error.message}` });
  }
  if (stagedReadable) {
    const extra = staged.filter((item) => !named.includes(item));
    const missing = named.filter((item) => !staged.includes(item));
    if (extra.length > 0) {
      refusals.push({
        code: 'COMMIT_STAGED_PATHS_MISMATCH',
        message: `暂存里有点名之外的路径，来源不明，拒绝提交：${extra.join('，')}`,
        paths: extra,
      });
    }
    if (missing.length > 0) {
      refusals.push({
        code: 'COMMIT_STAGED_PATHS_MISMATCH',
        message: `点名了但没有暂存的路径：${missing.join('，')}`,
        paths: missing,
      });
    }
  }

  // 第 2、3 道：点名路径与任务声明的写面比对，越界按三行表处置。
  const outOfScope = scope.classifyOutOfScopePaths({
    changedPaths: named,
    declaration,
    registered: settings.registered || [],
    controlPlaneExempt: exempt,
  });
  if (outOfScope.inForbidden.length > 0) {
    refusals.push({
      code: 'COMMIT_PATH_IN_FORBIDDEN_SCOPE',
      message: '提交路径落在本任务显式禁区内，拒绝提交，不得回填，也不因无人占用而放行',
      paths: outOfScope.inForbidden.map((entry) => entry.path),
      // 理由指向路径边界本身，不看这些文件的内容是什么。
      basis: 'PATH_BOUNDARY',
    });
  }
  if (outOfScope.collidingWithOthers.length > 0) {
    refusals.push({
      code: 'COMMIT_PATH_COLLIDES_WITH_OTHER_TASK',
      message: '提交路径越出本任务写面且与他人在册声明相交，拒绝提交并显式停机，不产生提交对象',
      items: outOfScope.collidingWithOthers,
      basis: 'PATH_BOUNDARY',
    });
  }

  // 第 4 道：提交自述三段。
  const message = parseCommitMessage(settings.message);
  refusals.push(...message.issues.map((issue) => ({ code: issue.code, message: issue.message, field: issue.field })));

  const allowed = refusals.length === 0;
  return {
    allowed,
    refusals,
    named,
    staged,
    message,
    // 越界但既不在禁区、又与谁都不相交的那批：允许，并把实际范围回填进声明。
    backfill: allowed && outOfScope.backfillable.length > 0
      ? scope.backfillWriteScope(declaration, outOfScope.backfillable)
      : null,
    // 计划里只有逐条点名的路径。执行由调用方在取得授权后完成，本模块不跑写命令。
    plan: allowed
      ? {
        stage: named.map((item) => ({ intent: 'STAGE_EXACT_PATH', path: item })),
        commit: { intent: 'COMMIT_NAMED_PATHS_ONLY', paths: named, message: settings.message },
      }
      : null,
    executedHere: false,
  };
}

module.exports = {
  COMMIT_MESSAGE_SECTIONS,
  parseCommitMessage,
  renderCommitMessage,
  stagedPaths,
  planExactCommit,
};
