'use strict';
// 错题本。一句话追加，一句话搜（R11）。
// 不填表：没有栏目、没有非空要求、没有格式校验。
// 前一版要填九项、满足四个条件，结果 0 条记录 —— 要填的那一刻正是活刚干完最不想动的时候。
const fs = require('fs');
const path = require('path');

const file = (root) => path.join(root, 'docs', 'harness', 'MISTAKES.md');

function read(root) {
  try { return fs.readFileSync(file(root), 'utf8'); } catch { return ''; }
}

// 一行一条，条目就是 `- ` 开头那些。标题和说明行不算。
function all(root) {
  return read(root).split('\n')
    .map((l, i) => ({ n: i + 1, text: l.trim() }))
    .filter((l) => l.text.startsWith('- '));
}

function search(root, word) {
  const key = String(word || '').toLowerCase();
  if (key === '') return all(root);
  return all(root).filter((l) => l.text.toLowerCase().includes(key));
}

// 追加一行，带日期。校验只有一条：话不能是空的。
function add(root, text, opts) {
  const o = opts || {};
  const say = String(text || '').replace(/\s+/g, ' ').trim();
  if (say === '') return { ok: false, msg: '要说一句话：hl mistake add "一句话" [--write]' };
  const d = o.at || new Date();
  const stamp = `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate()).padStart(2, '0')}`;
  const line = `- ${stamp} ${say}`;
  const f = file(root);
  if (!o.write) return { ok: true, wrote: false, file: f, line, msg: `会追加到 ${f}：\n${line}` };
  fs.mkdirSync(path.dirname(f), { recursive: true });
  const prev = read(root);
  const head = prev === '' ? '# 错题本\n\n一行一条，一句话。查：hl mistake <关键词>\n\n' : (prev.endsWith('\n') ? '' : '\n');
  fs.appendFileSync(f, head + line + '\n');
  return { ok: true, wrote: true, file: f, line, msg: `已追加：${line}` };
}

function format(hits, word) {
  if (!hits.length) return word ? `错题本里没有跟"${word}"相关的` : '错题本还是空的';
  return hits.map((h) => h.text).join('\n');
}

module.exports = { file, read, all, search, add, format };
