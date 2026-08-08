'use strict';
// git 只读事实。只跑不会改仓库状态的命令。
const { spawnSync } = require('child_process');

function git(root, args) {
  const r = spawnSync('git', args, { cwd: root, encoding: 'utf8' });
  if (r.error || r.status !== 0) return null;
  return r.stdout;
}

function isRepo(root) {
  const out = git(root, ['rev-parse', '--is-inside-work-tree']);
  return out != null && out.trim() === 'true';
}

// git status --porcelain 一行：XY 路径。改名那种带 ->，取箭头后面那个。
function parseStatus(text) {
  const files = [];
  for (const line of text.split('\n')) {
    if (line.trim() === '') continue;
    const xy = line.slice(0, 2);
    let p = line.slice(3).trim();
    const arrow = p.indexOf(' -> ');
    if (arrow !== -1) p = p.slice(arrow + 4).trim();
    if (p.startsWith('"') && p.endsWith('"')) p = p.slice(1, -1);
    files.push({ path: p, x: xy[0], y: xy[1], staged: xy[0] !== ' ' && xy[0] !== '?' });
  }
  return files;
}

function lastLine(text) {
  const ls = (text || '').trim().split('\n').filter((l) => l !== '');
  return ls.length ? ls[ls.length - 1].trim() : null;
}

function facts(root) {
  if (!isRepo(root)) return { repo: false, note: '不是 git 仓', files: [], paths: [], staged: [], clean: null };
  // -uall：未跟踪的整个目录默认会被折叠成 `src/`，那样就没法拿文件路径跟允许清单比
  const files = parseStatus(git(root, ['status', '--porcelain', '-uall']) || '');
  return {
    repo: true,
    root,
    branch: (git(root, ['rev-parse', '--abbrev-ref', 'HEAD']) || '').trim() || null,
    files,
    paths: files.map((f) => f.path),
    staged: files.filter((f) => f.staged).map((f) => f.path),
    clean: files.length === 0,
    diffstat: lastLine(git(root, ['diff', '--stat'])) || '无改动',
    cachedstat: lastLine(git(root, ['diff', '--cached', '--stat'])) || '无暂存',
  };
}

function format(f) {
  if (!f.repo) return f.note;
  return [
    `分支：${f.branch || '(无)'}`,
    `工作区：${f.clean ? '干净' : '有改动'}`,
    `改动文件：${f.paths.length ? f.paths.join('、') : '无'}`,
    `暂存：${f.staged.length ? f.staged.join('、') : '无'}`,
    `未暂存统计：${f.diffstat}`,
    `暂存统计：${f.cachedstat}`,
  ].join('\n');
}

const topLevel = (dir) => {
  const out = git(dir, ['rev-parse', '--show-toplevel']);
  return out == null ? null : out.trim() || null;
};

// git worktree list --porcelain 里 `worktree <路径>` 那些行
function worktrees(dir) {
  const out = git(dir, ['worktree', 'list', '--porcelain']);
  if (out == null) return [];
  return out.split('\n').filter((l) => l.startsWith('worktree ')).map((l) => l.slice(9).trim());
}

// 你指的目录和你人在的仓库不是一个 → 说一句，绝不拦（R7）。
// 2026-08-06 出过这事：报告写进一个工作树、提交在另一个，两边都不报错，
// 另一边 git status 也不提那些文件，于是记录没进任何提交。
function splitNote(targetRoot, cwdRoot, list) {
  const out = [];
  if (targetRoot && cwdRoot && targetRoot !== cwdRoot) {
    out.push(`你指的是 ${targetRoot}，人在 ${cwdRoot}：写文件和提交会分叉`);
  }
  if ((list || []).length > 1) out.push(`这个仓库开着 ${list.length} 个工作树`);
  return out.length ? out.join('；') + '。提交完核对记录在不在这个提交里' : null;
}

// 现场取两边的仓库根和工作树个数，交给上面那个纯函数判
const splitCheck = (targetDir, cwdDir) => splitNote(topLevel(targetDir), topLevel(cwdDir), worktrees(targetDir));

// harness 自己的东西（报告、使用记录、叶子挪窝、装进去的钩子和 lib）不算代理越界 ——
// 不排掉的话这行永远非 0，用户就学会忽略它，那就等于没有（R7、R14）
const OWN = ['docs/harness/', '.claude/harness-lite/'];

const isOwn = (p) => OWN.some((d) => String(p).startsWith(d));

// 这轮动过的文件里，几个不在允许清单里（报告"范围外"那行，纯机器算）
function outOfScope(paths, allowed) {
  const { inScope } = require('./leaf.js');
  return (paths || []).filter((p) => !isOwn(p) && !inScope(p, allowed));
}

module.exports = { git, isRepo, parseStatus, facts, format, outOfScope, isOwn, OWN,
  topLevel, worktrees, splitNote, splitCheck };
