'use strict';

// Adaptive Harness v0.5 — 唯一允许起子进程的模块（AH-050-03）
//
// 需求溯源：
//   EX-3  「改一个状态词、挪一下文件都不算退场」——退场那一刻必须真去问一次版本库，
//         读回来的现实要参与「能不能进入 HISTORY / 收尾」的判定，而不是读完扔掉。
//   WK-1  实物与记录冲突时报冲突；不得把记录静默改写成现实值。
//   GIT-7 验证跑的是不是当前这版，靠 HEAD OID 现场核对。
//   KP-16 把 execFile 收敛到一处，安全边界可审计。
//
// 普通事实入口 runGit 只读。AH-050-06 另提供一个只供 task start 使用的
// 固定语法写入口；它不接受任意 argv，也不扩张 runGit 的只读白名单。

const { execFileSync } = require('node:child_process');

const READ_ONLY_SUBCOMMANDS = new Set([
  'rev-parse', 'status', 'diff', 'merge-base', 'cat-file', 'show', 'log', 'ls-files', 'ls-tree',
  'for-each-ref',
]);

const START_GIT_ACTIONS = Object.freeze([
  'CREATE_BRANCH',
  'ADD_WORKTREE',
  'STAGE_OPENING_PACKAGE',
  'COMMIT_OPENING',
]);

class GitFactError extends Error {
  constructor(code, message) {
    super(message);
    this.name = 'GitFactError';
    this.code = code;
  }
}

/**
 * 跑一条只读 git 查询。退场（closeout / 进入历史）判定就靠它取现实。
 * 拒绝任何不在只读白名单里的子命令：本模块永远不写版本库。
 */
function runGit(args, options) {
  const settings = options || {};
  if (!Array.isArray(args) || args.length === 0) {
    throw new GitFactError('GIT_ARGS_INVALID', 'git 查询至少要有一个子命令');
  }
  const subcommand = args.find((item) => !String(item).startsWith('-'));
  if (!READ_ONLY_SUBCOMMANDS.has(subcommand)) {
    throw new GitFactError('GIT_SUBCOMMAND_NOT_READ_ONLY', `拒绝执行非只读子命令 ${subcommand}`);
  }
  try {
    const stdout = execFileSync('git', args, {
      cwd: settings.cwd || process.cwd(),
      encoding: 'utf8',
      stdio: ['ignore', 'pipe', 'pipe'],
      maxBuffer: 32 * 1024 * 1024,
    });
    return { ok: true, code: 0, stdout };
  } catch (error) {
    return {
      ok: false,
      code: typeof error.status === 'number' ? error.status : 1,
      stdout: typeof error.stdout === 'string' ? error.stdout : '',
      stderr: typeof error.stderr === 'string' ? error.stderr : String(error.message || ''),
    };
  }
}

/**
 * 全仓所有 linked worktree 共享的那一个目录。在办控制面挂在它下面，
 * 于是「不切分支即可读到全部在册声明」（EX-9 第 1 条）物理上成立。
 * 非常规布局探测失败时报冲突并停，不猜（WK-1）。
 */
function gitCommonDir(cwd) {
  const result = runGit(['rev-parse', '--path-format=absolute', '--git-common-dir'], { cwd });
  if (!result.ok) throw new GitFactError('GIT_COMMON_DIR_UNRESOLVED', '无法解析 git common dir，报冲突并停');
  const value = result.stdout.trim();
  if (!value) throw new GitFactError('GIT_COMMON_DIR_UNRESOLVED', 'git common dir 为空，报冲突并停');
  return value;
}

function repoRoot(cwd) {
  const result = runGit(['rev-parse', '--show-toplevel'], { cwd });
  if (!result.ok) throw new GitFactError('GIT_REPO_ROOT_UNRESOLVED', '无法解析仓库根');
  return result.stdout.trim();
}

function headOid(cwd) {
  const result = runGit(['rev-parse', 'HEAD'], { cwd });
  return result.ok ? result.stdout.trim() : null;
}

/** 整个工作副本的现场状态。退场第 6 项拿它与开工时的 inspect 快照对账。 */
function porcelainStatus(cwd) {
  const result = runGit(['status', '--porcelain'], { cwd });
  if (!result.ok) throw new GitFactError('GIT_STATUS_UNAVAILABLE', '读不到工作副本状态');
  return result.stdout.split('\n').map((line) => line.trimEnd()).filter((line) => line !== '');
}

/** base..head 的改动路径清单。退场第 2 / 3 / 5 项的现实输入。 */
function changedPaths(cwd, baseOid, head) {
  const target = head || 'HEAD';
  const result = runGit(['diff', '--name-only', `${baseOid}..${target}`], { cwd });
  if (!result.ok) throw new GitFactError('GIT_DIFF_UNAVAILABLE', `读不到 ${baseOid}..${target} 的改动清单`);
  return result.stdout.split('\n').map((line) => line.trim()).filter((line) => line !== '');
}

/**
 * 解析 `git diff --name-status -z`。
 *
 * 不能按行切：合法路径可以含空格、tab、换行和非 ASCII 字符。`-z` 的字段边界
 * 才是 Git 给出的真实边界。A/M/D 各带一个路径；R/C 带 oldPath 与 newPath，
 * 并保留 Git 给出的相似度分数。
 */
function parseNameStatusZ(stdout) {
  const fields = String(stdout || '').split('\0');
  if (fields[fields.length - 1] === '') fields.pop();
  const entries = [];
  let cursor = 0;

  while (cursor < fields.length) {
    const token = fields[cursor++];
    const status = token.slice(0, 1);
    if (!['A', 'M', 'D', 'R', 'C'].includes(status)) {
      throw new GitFactError('GIT_DIFF_UNAVAILABLE', `不支持的 Git delta 状态 ${token || '（空）'}`);
    }

    if (status === 'R' || status === 'C') {
      if (cursor + 1 >= fields.length) {
        throw new GitFactError('GIT_DIFF_UNAVAILABLE', `${token} 缺 oldPath/newPath`);
      }
      const oldPath = fields[cursor++];
      const newPath = fields[cursor++];
      entries.push({
        status,
        oldPath,
        newPath,
        score: /^\d+$/.test(token.slice(1)) ? Number(token.slice(1)) : null,
      });
      continue;
    }

    if (cursor >= fields.length) {
      throw new GitFactError('GIT_DIFF_UNAVAILABLE', `${token} 缺路径`);
    }
    const filePath = fields[cursor++];
    entries.push({
      status,
      oldPath: status === 'A' ? null : filePath,
      newPath: status === 'D' ? null : filePath,
    });
  }
  return entries;
}

function parseNulPaths(stdout) {
  const fields = String(stdout || '').split('\0');
  if (fields[fields.length - 1] === '') fields.pop();
  return fields;
}

/**
 * base..head 的真实增量事实。rename/copy 不压成一个含糊的 path，而是显式保留
 * oldPath/newPath，供测试资产退场审计判断“新增、删除、替代”。
 */
function changedEntries(cwd, baseOid, head) {
  const target = head || 'HEAD';
  const result = runGit([
    'diff',
    '--name-status',
    '-z',
    '--find-renames',
    '--find-copies',
    `${baseOid}..${target}`,
  ], { cwd });
  if (!result.ok) throw new GitFactError('GIT_DIFF_UNAVAILABLE', `读不到 ${baseOid}..${target} 的增量事实`);
  return parseNameStatusZ(result.stdout);
}

/** 读取某个真实 ref 下全部 tracked 路径；同样只认 NUL 字段边界。 */
function trackedPaths(cwd, ref) {
  const target = typeof ref === 'string' && ref.trim() !== '' ? ref : 'HEAD';
  const result = runGit(['ls-tree', '-r', '--name-only', '-z', target], { cwd });
  if (!result.ok) throw new GitFactError('GIT_DIFF_UNAVAILABLE', `读不到 ${target} 的 tracked 路径`);
  return parseNulPaths(result.stdout);
}

/**
 * 收尾时的完整未跟踪事实：普通 untracked 与被 ignore 规则藏起来的 untracked
 * 必须同时枚举。两次查询都用 `-z`，合法路径中的空格、tab、换行和 Unicode
 * 不会被拆坏。调用方仍负责按产品依赖 / 永不提交 / 其他产物做三分法。
 */
function untrackedPathsIncludingIgnored(cwd, prefixes) {
  const scope = (Array.isArray(prefixes) ? prefixes : [])
    .filter((item) => typeof item === 'string' && item.trim() !== '');
  const suffix = scope.length > 0 ? ['--', ...scope] : [];
  const ordinary = runGit([
    'ls-files',
    '--others',
    '--exclude-standard',
    '-z',
    ...suffix,
  ], { cwd });
  const ignored = runGit([
    'ls-files',
    '--others',
    '--ignored',
    '--exclude-standard',
    '-z',
    ...suffix,
  ], { cwd });
  if (!ordinary.ok || !ignored.ok) {
    throw new GitFactError('GIT_DIFF_UNAVAILABLE', '读不到普通与 ignored untracked 的完整清单');
  }
  return [...new Set([
    ...parseNulPaths(ordinary.stdout),
    ...parseNulPaths(ignored.stdout),
  ])].sort();
}

/** 路径级零 diff 核验：写面之内一条改动都没有，才允许 no-product-change 为真。 */
function scopedChangedPaths(cwd, baseOid, head, prefixes) {
  const target = head || 'HEAD';
  const scope = Array.isArray(prefixes) ? prefixes : [];
  if (scope.length === 0) return [];
  const result = runGit(['diff', '--name-only', `${baseOid}..${target}`, '--', ...scope], { cwd });
  if (!result.ok) throw new GitFactError('GIT_DIFF_UNAVAILABLE', '读不到写面内的改动清单');
  return result.stdout.split('\n').map((line) => line.trim()).filter((line) => line !== '');
}

/** 集成事实必须在退场那一刻现跑，不能拿记录里的 integrated-observed 顶替（WK-1）。 */
function isAncestor(cwd, ancestorOid, descendantRef) {
  const result = runGit(['merge-base', '--is-ancestor', ancestorOid, descendantRef], { cwd });
  return result.ok;
}

function objectExists(cwd, oid) {
  if (typeof oid !== 'string' || oid.trim() === '') return false;
  const result = runGit(['cat-file', '-e', `${oid}^{commit}`], { cwd });
  return result.ok;
}

/** 历史平面只经固定的 integration ref 读，不切分支、不检出。 */
function showFromRef(cwd, ref, repoRelativePath) {
  const result = runGit(['show', `${ref}:${repoRelativePath}`], { cwd });
  return result.ok ? result.stdout : null;
}

function runExactGit(args, options) {
  const settings = options || {};
  const execute = typeof settings.execFileSync === 'function'
    ? settings.execFileSync
    : execFileSync;
  try {
    const stdout = execute('git', args, {
      cwd: settings.cwd || process.cwd(),
      encoding: 'utf8',
      stdio: ['ignore', 'pipe', 'pipe'],
      maxBuffer: 32 * 1024 * 1024,
    });
    return { ok: true, code: 0, stdout: typeof stdout === 'string' ? stdout : '' };
  } catch (error) {
    return {
      ok: false,
      code: typeof error.status === 'number' ? error.status : 1,
      stdout: typeof error.stdout === 'string' ? error.stdout : '',
      stderr: typeof error.stderr === 'string' ? error.stderr : String(error.message || ''),
    };
  }
}

function safeBranchName(value) {
  const text = typeof value === 'string' ? value.trim() : '';
  return text !== ''
    && !text.startsWith('-')
    && !text.startsWith('/')
    && !text.endsWith('/')
    && !text.endsWith('.')
    && !text.endsWith('.lock')
    && !text.includes('..')
    && !text.includes('@{')
    && !/[\s~^:?*[\]\\\x00-\x1f\x7f]/.test(text)
    && text.split('/').every((part) => part !== '' && part !== '.' && part !== '..');
}

function safeOpeningPath(value) {
  const text = typeof value === 'string' ? value.trim().replace(/\\/g, '/') : '';
  return text !== ''
    && !text.startsWith('/')
    && !text.includes('\0')
    && text.split('/').every((part) => part !== '' && part !== '.' && part !== '..');
}

function canonicalCommitOid(value) {
  const text = typeof value === 'string' ? value.trim() : '';
  return /^(?:[0-9a-f]{40}|[0-9a-f]{64})$/i.test(text) ? text : null;
}

/**
 * `git worktree list --porcelain` 是查询，但 `worktree` 这个顶层动词也包含
 * add/remove。为避免把整个动词塞进 runGit 白名单，这里只暴露这一条固定查询。
 */
function worktreeList(cwd, options) {
  const result = runExactGit(['worktree', 'list', '--porcelain'], {
    ...(options || {}),
    cwd,
  });
  if (!result.ok) {
    throw new GitFactError('GIT_WORKTREE_LIST_UNAVAILABLE', '读不到 git worktree list --porcelain');
  }
  const entries = [];
  let current = null;
  for (const line of result.stdout.replace(/\r\n/g, '\n').split('\n')) {
    if (line.startsWith('worktree ')) {
      if (current) entries.push(current);
      current = {
        worktree: line.slice('worktree '.length),
        head: null,
        branch: null,
        detached: false,
        bare: false,
      };
      continue;
    }
    if (!current || line === '') continue;
    if (line.startsWith('HEAD ')) current.head = line.slice('HEAD '.length);
    else if (line.startsWith('branch ')) current.branch = line.slice('branch '.length);
    else if (line === 'detached') current.detached = true;
    else if (line === 'bare') current.bare = true;
  }
  if (current) entries.push(current);
  return entries;
}

/**
 * task start 的唯一 Git 写入口。调用者只能选四个固定动作；参数逐字段校验后，
 * 本函数自己拼 argv。push/reset/clean/stash/rebase/delete 没有表示方式。
 */
function runStartGit(action, input, options) {
  const operation = String(action || '');
  const fields = input && typeof input === 'object' ? input : {};
  const settings = options || {};
  if (!START_GIT_ACTIONS.includes(operation)) {
    throw new GitFactError('START_GIT_ACTION_FORBIDDEN', `task start 不允许 Git 动作 ${operation || '（空）'}`);
  }
  if (settings.authorized !== true) {
    throw new GitFactError('START_GIT_WRITE_NOT_AUTHORIZED', 'task start Git 写入缺少本次显式 --write 授权');
  }

  let args;
  let cwd = settings.cwd || process.cwd();
  if (operation === 'CREATE_BRANCH') {
    const branch = fields.branch;
    const baseOid = canonicalCommitOid(fields.baseOid);
    if (!safeBranchName(branch) || !baseOid) {
      throw new GitFactError('START_GIT_IDENTITY_INVALID', '创建 task branch 需要安全 branch 名与完整 base OID');
    }
    args = ['branch', '--no-track', branch, baseOid];
  } else if (operation === 'ADD_WORKTREE') {
    const branch = fields.branch;
    const worktree = fields.worktree;
    if (!safeBranchName(branch) || typeof worktree !== 'string' || !worktree.startsWith('/') || worktree.includes('\0')) {
      throw new GitFactError('START_GIT_IDENTITY_INVALID', '创建 linked worktree 需要安全 branch 名与绝对 worktree 路径');
    }
    args = ['worktree', 'add', worktree, branch];
  } else if (operation === 'STAGE_OPENING_PACKAGE') {
    if (!safeOpeningPath(fields.packagePath)) {
      throw new GitFactError('START_GIT_OPENING_PATH_INVALID', 'opening package 必须是仓库内单一路径，不得越界');
    }
    cwd = fields.worktree;
    args = ['add', '--', fields.packagePath];
  } else {
    if (!safeOpeningPath(fields.packagePath)) {
      throw new GitFactError('START_GIT_OPENING_PATH_INVALID', 'opening commit 必须绑定精确 package 路径');
    }
    const subject = typeof fields.subject === 'string' ? fields.subject.trim() : '';
    const why = typeof fields.why === 'string' ? fields.why.trim() : '';
    const what = typeof fields.what === 'string' ? fields.what.trim() : '';
    const verification = typeof fields.verification === 'string' ? fields.verification.trim() : '';
    if (!subject || !why || !what || !verification) {
      throw new GitFactError('START_GIT_COMMIT_MESSAGE_INCOMPLETE', 'opening commit 必须有 subject、Why、What、Verification');
    }
    cwd = fields.worktree;
    const staged = runGit(['diff', '--cached', '--name-only'], { cwd });
    const stagedPaths = staged.ok
      ? staged.stdout.split('\n').map((line) => line.trim()).filter(Boolean)
      : [];
    if (!staged.ok || stagedPaths.length !== 1 || stagedPaths[0] !== fields.packagePath) {
      throw new GitFactError(
        'START_GIT_STAGED_SCOPE_MISMATCH',
        `opening commit 只能暂存 ${fields.packagePath}`,
      );
    }
    args = [
      'commit',
      '-m', subject,
      '-m', `Why: ${why}`,
      '-m', `What: ${what}`,
      '-m', `Verification: ${verification}`,
    ];
  }
  return runExactGit(args, { ...settings, cwd });
}

module.exports = {
  GitFactError,
  READ_ONLY_SUBCOMMANDS,
  START_GIT_ACTIONS,
  runGit,
  runStartGit,
  worktreeList,
  gitCommonDir,
  repoRoot,
  headOid,
  porcelainStatus,
  changedPaths,
  parseNameStatusZ,
  parseNulPaths,
  changedEntries,
  trackedPaths,
  untrackedPathsIncludingIgnored,
  scopedChangedPaths,
  isAncestor,
  objectExists,
  showFromRef,
};
