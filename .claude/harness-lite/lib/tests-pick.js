'use strict';
// 给改动路径 → 该跑哪几个测试 + 一条能直接复制的命令（R11）。
// 只正着做。这里永远不做"判断代理选得对不对" —— 前一版 850 行只干了那个，方向是反的。
const fs = require('fs');
const path = require('path');

const SKIP = new Set(['node_modules', '.git', 'dist', 'build', 'coverage', '.next', 'vendor', 'done']);
const TEST_RE = /\.(test|spec)\.[cm]?[jt]sx?$/;
const TEST_DIR = new Set(['test', 'tests', '__tests__', 'spec']);

function walk(root, rel, out) {
  let ents;
  try { ents = fs.readdirSync(path.join(root, rel), { withFileTypes: true }); } catch { return out; }
  for (const e of ents) {
    if (e.name.startsWith('.') || SKIP.has(e.name)) continue;
    const r = rel ? `${rel}/${e.name}` : e.name;
    if (e.isDirectory()) walk(root, r, out);
    else if (TEST_RE.test(e.name)) out.push(r);
  }
  return out;
}

// 去掉 .test.ts / .ts，只留名字本身
const stem = (p) => path.basename(p).replace(TEST_RE, '').replace(/\.[^.]+$/, '');

// 人工覆盖：{ "src/order/": ["test/order/"] }，前缀也算命中
function mapFor(root, file) {
  let m;
  try { m = JSON.parse(fs.readFileSync(path.join(root, 'docs', 'harness', 'test-map.json'), 'utf8')); } catch { return []; }
  const hit = [];
  for (const [k, v] of Object.entries(m || {})) {
    const key = k.replace(/\/+$/, '');
    if (file === key || file.startsWith(key + '/')) hit.push(...[].concat(v));
  }
  return hit;
}

// 一个源文件对应哪几个测试
function forFile(root, file, tests) {
  const f = String(file).replace(/^\.\//, '').replace(/\\/g, '/');
  if (TEST_RE.test(f)) return { file: f, tests: [f], why: '本身就是测试文件' };

  const manual = mapFor(root, f);
  if (manual.length) return { file: f, tests: manual, why: 'test-map.json' };

  const s = stem(f);
  const dir = path.posix.dirname(f);
  const segs = dir.split('/').filter((x) => x && x !== '.');
  const hit = new Set();

  for (const t of tests) {
    const tsegs = path.posix.dirname(t).split('/').filter(Boolean);
    if (stem(t) === s) { hit.add(t); continue; }                      // 同名
    if (path.posix.dirname(t) === dir) { hit.add(t); continue; }      // 同目录
    // 同目录下的 test/、__tests__/，或 test/ 里镜像着源码的目录
    const shared = segs.filter((x) => tsegs.includes(x));
    if (shared.length && tsegs.some((x) => TEST_DIR.has(x))) hit.add(t);
  }
  return { file: f, tests: [...hit].sort(), why: hit.size ? '同名 / 同目录 / 镜像目录' : null };
}

// 用什么跑：有 npm test 就用它
function runner(root) {
  let pkg;
  try { pkg = JSON.parse(fs.readFileSync(path.join(root, 'package.json'), 'utf8')); } catch { pkg = null; }
  return pkg && pkg.scripts && pkg.scripts.test
    ? { one: 'npm test --', all: 'npm test' }
    : { one: 'node --test', all: 'node --test' };
}

function pick(root, files) {
  const tests = walk(root, '', []).sort();
  const rows = (files || []).map((f) => forFile(root, f, tests));
  const all = [...new Set(rows.flatMap((r) => r.tests))].sort();
  const r = runner(root);
  return {
    rows,
    tests: all,
    // 找不到不报错，给一条兜底的（R11）
    cmd: all.length ? `${r.one} ${all.join(' ')}` : r.all,
    found: all.length > 0,
  };
}

function format(p) {
  const lines = p.rows.map((r) => `${r.file} → ${r.tests.length ? r.tests.join('、') : '没找到对应测试'}`);
  lines.push('');
  lines.push(p.found ? `跑这条：${p.cmd}` : `没找到对应测试，建议跑 ${p.cmd}`);
  return lines.join('\n');
}

module.exports = { walk, stem, forFile, runner, pick, format, TEST_RE };
