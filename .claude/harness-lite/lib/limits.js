'use strict';
// 上限检查。超了砍功能，代理不许自己抬这里的数字（需求 R15）。
// 2026-08-07 用户授权 stage-02，并确认本阶段所需的容量调整和扩展计数。
const fs = require('fs');
const path = require('path');

const CAPS = { impl: 2200, test: 2200, skills: 80, report: 8, entries: 8 };
const SKIP_DIRS = new Set(['node_modules', '.git', 'fixtures', 'sandbox']);

function lines(file) {
  let t;
  try { t = fs.readFileSync(file, 'utf8'); } catch { return 0; }
  if (t === '') return 0;
  return t.replace(/\n$/, '').split('\n').length;
}

function walk(dir, exts) {
  let out = [];
  let ents;
  try { ents = fs.readdirSync(dir, { withFileTypes: true }); } catch { return out; }
  for (const e of ents) {
    if (e.isDirectory()) {
      if (!SKIP_DIRS.has(e.name)) out = out.concat(walk(path.join(dir, e.name), exts));
    } else if (exts.includes(path.extname(e.name))) {
      out.push(path.join(dir, e.name));
    }
  }
  return out.sort();
}

function row(name, used, cap) {
  return { name, used, cap, over: used > cap };
}

// 五组数字。root 是仓库根，测试可以喂假目录树。
function check(root) {
  const at = (...p) => path.join(root, ...p);
  const entries = [...walk(at('bin'), ['.js']), ...walk(at('hooks'), ['.js'])];
  if (fs.existsSync(at('install.js'))) entries.push(at('install.js'));
  const extensions = walk(at('extensions'), ['.js']);
  const impl = [...walk(at('lib'), ['.js']), ...entries, ...extensions];
  const sum = (files) => files.reduce((n, f) => n + lines(f), 0);
  return [
    row('实现 lib+bin+hooks+install.js', sum(impl), CAPS.impl),
    row('测试 test/', sum(walk(at('test'), ['.js'])), CAPS.test),
    row('技能 skills/+extensions/', sum([...walk(at('skills'), ['.md']),
      ...walk(at('extensions'), ['.md'])]), CAPS.skills),
    row('报告模板 templates/report.md', lines(at('templates', 'report.md')), CAPS.report),
    row('命令入口', entries.length, CAPS.entries),
  ];
}

function format(rows) {
  return rows.map((r) => `${r.over ? '超限：' : ''}${r.name} ${r.used}/${r.cap}`).join('\n');
}

module.exports = { CAPS, check, format, lines };
