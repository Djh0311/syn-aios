'use strict';
// 解析叶子文件。允许动的每一条要么能指回阶段文件，要么标 [新增]（需求 R3）。
const fs = require('fs');
const path = require('path');

function read(file) {
  try { return fs.readFileSync(file, 'utf8'); } catch { return null; }
}

// `字段：值` 取值，中英文冒号都认
function field(text, name) {
  const m = text.match(new RegExp('^\\s*' + name + '[:：]\\s*(.+)$', 'm'));
  return m ? m[1].trim() : null;
}

// `名字：` 单独一行，之后连续的 `- xxx`
function bullets(text, name) {
  const ls = text.split('\n');
  const i = ls.findIndex((l) => new RegExp('^' + name + '[:：]\\s*$').test(l.trim()));
  if (i === -1) return [];
  const out = [];
  for (const l of ls.slice(i + 1)) {
    const t = l.trim();
    if (t === '') { if (out.length) break; continue; }
    if (!t.startsWith('- ')) break;
    out.push(t.slice(2).trim());
  }
  return out;
}

// `## 标题` 之后到下一个 # 之前的行
function after(text, re) {
  const ls = text.split('\n');
  const i = ls.findIndex((l) => re.test(l.trim()));
  if (i === -1) return [];
  const out = [];
  for (const l of ls.slice(i + 1)) {
    if (l.trim().startsWith('#')) break;
    out.push(l);
  }
  return out;
}

function title(text, fallback) {
  const m = text.match(/^#\s*(.+)$/m);
  return m ? m[1].trim() : fallback;
}

function parse(file) {
  const text = read(file);
  if (text == null) return null;
  const stage = field(text, '阶段') || '';
  return {
    file,
    name: path.basename(file, '.md'),
    title: title(text, path.basename(file, '.md')),
    stage,
    stageId: stage.split(/\s+/)[0] || null,
    goal: field(text, '目标'),
    doneWhen: field(text, '干完的标准'),
    allowed: bullets(text, '允许动').map((raw) => ({
      path: raw.replace(/\[新增\]/g, '').trim(),
      marked: /\[新增\]/.test(raw),
    })),
    steps: after(text, /^##\s*步骤/)
      .map((l) => l.trim().match(/^\d+\.\s*(.+)$/))
      .filter(Boolean)
      .map((m) => m[1]),
  };
}

// 一条允许动的路径落在某个允许范围里吗
function inScope(file, allowed) {
  return (allowed || []).some((a) => {
    const s = a.replace(/\/+$/, '');
    return file === s || file.startsWith(s + '/');
  });
}

// R3：叶子的允许动能不能指回阶段文件。指不回来又没标 [新增] 的，是要用户重点看的那行。
function checkAllowed(leaf, stageAllowed) {
  return leaf.allowed.map((a) => {
    const from = (stageAllowed || []).find(
      (s) => a.path === s.replace(/\/+$/, '') || inScope(a.path, [s]) || s === a.path
    );
    return { path: a.path, marked: a.marked, tracedTo: from || null, isNew: !from };
  });
}

module.exports = { read, field, bullets, after, title, parse, inScope, checkAllowed };
