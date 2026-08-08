'use strict';
// 代码地图：有哪些模块、每个几行、导出了什么。一屏以内（R11）。
// 只读，不建缓存文件 —— 需要刷新的索引就是需要用户定期维护的东西（R14）。
const fs = require('fs');
const path = require('path');

const SKIP = new Set(['node_modules', '.git', 'dist', 'build', 'coverage', '.next', 'vendor', 'fixtures', 'sandbox']);
const SRC_RE = /\.[cm]?[jt]sx?$/;
const TEST_RE = /\.(test|spec)\.[cm]?[jt]sx?$/;
const CAP = 40; // 一屏

function walk(root, rel, out) {
  let ents;
  try { ents = fs.readdirSync(path.join(root, rel), { withFileTypes: true }); } catch { return out; }
  for (const e of ents) {
    if (e.name.startsWith('.') || SKIP.has(e.name)) continue;
    const r = rel ? `${rel}/${e.name}` : e.name;
    if (e.isDirectory()) walk(root, r, out);
    else if (SRC_RE.test(e.name) && !TEST_RE.test(e.name)) out.push(r);
  }
  return out;
}

// 导出了什么。CommonJS 和 ESM 都认，认不出就空着，不猜。
function exportsOf(text) {
  const names = new Set();
  const m = text.match(/module\.exports\s*=\s*\{([^}]*)\}/);
  if (m) {
    for (const part of m[1].split(',')) {
      const n = part.split(':')[0].trim();
      if (/^[A-Za-z_$][\w$]*$/.test(n)) names.add(n);
    }
  }
  for (const re of [
    /module\.exports\.([A-Za-z_$][\w$]*)/g,
    /exports\.([A-Za-z_$][\w$]*)\s*=/g,
    /export\s+(?:async\s+)?(?:function|const|let|var|class)\s+([A-Za-z_$][\w$]*)/g,
  ]) {
    let x;
    while ((x = re.exec(text)) !== null) names.add(x[1]);
  }
  const named = text.match(/export\s*\{([^}]*)\}/);
  if (named) {
    for (const part of named[1].split(',')) {
      const n = part.split(/\s+as\s+/)[0].trim();
      if (/^[A-Za-z_$][\w$]*$/.test(n)) names.add(n);
    }
  }
  return [...names];
}

function build(root, opts) {
  const o = opts || {};
  const dirs = o.dirs && o.dirs.length ? o.dirs : ['lib', 'src', 'bin', 'hooks', 'scripts', 'app', 'packages'];
  const seen = new Set();
  const files = [];
  for (const d of dirs) {
    for (const f of walk(root, d, [])) if (!seen.has(f)) { seen.add(f); files.push(f); }
  }
  // 根目录那几个单文件（install.js 这类）
  for (const f of walk(root, '', []).filter((x) => !x.includes('/'))) {
    if (!seen.has(f)) { seen.add(f); files.push(f); }
  }

  const rows = files.map((f) => {
    let text = '';
    try { text = fs.readFileSync(path.join(root, f), 'utf8'); } catch { /* 读不到就当空的 */ }
    const first = text.split('\n').find((l) => /^\s*\/\//.test(l));
    return {
      file: f,
      lines: text === '' ? 0 : text.replace(/\n$/, '').split('\n').length,
      exports: exportsOf(text),
      note: first ? first.replace(/^\s*\/\/\s*/, '').slice(0, 40) : null,
    };
  }).sort((a, b) => b.lines - a.lines);

  return { root, total: rows.length, totalLines: rows.reduce((n, r) => n + r.lines, 0), rows: rows.slice(0, CAP), cut: Math.max(0, rows.length - CAP) };
}

function format(m) {
  if (!m.total) return '没找到源码文件（看过 lib/ src/ bin/ hooks/ scripts/ app/ packages/ 和根目录）';
  const w = Math.min(46, Math.max(...m.rows.map((r) => r.file.length)));
  const lines = m.rows.map((r) => {
    const ex = r.exports.length ? r.exports.slice(0, 6).join(' ') + (r.exports.length > 6 ? ' …' : '') : (r.note || '');
    return `${r.file.padEnd(w)}  ${String(r.lines).padStart(4)} 行  ${ex}`;
  });
  lines.push(`共 ${m.total} 个文件 ${m.totalLines} 行` + (m.cut ? `，还有 ${m.cut} 个没列` : ''));
  return lines.join('\n');
}

module.exports = { walk, exportsOf, build, format, CAP };
