'use strict';
// 拼报告并追加到 reports/YYYY-MM-DD.md。
// 两栏分开（R4）：分隔线以上代理写，以下从 git 和命令输出取，代理写不进去。
const fs = require('fs');
const path = require('path');
const gitfacts = require('./gitfacts.js');

const CAP = 8;                        // 整份报告行数上限（R15），含分隔线
const SEP = '--- 以下机器取 ---';
const MACHINE_PREFIX = ['动了文件：', '范围外：', '测试：'];
const MACHINE = MACHINE_PREFIX.length; // 机器栏固定三行，截断时不动它
const HEAD_CAP = CAP - MACHINE - 1;   // 上半截最多几行

const dir = (root) => path.join(root, 'docs', 'harness', 'reports');

function today(d) {
  const t = d || new Date();
  const p = (n) => String(n).padStart(2, '0');
  return `${t.getFullYear()}-${p(t.getMonth() + 1)}-${p(t.getDate())}`;
}

const file = (root, date) => path.join(dir(root), `${date || today()}.md`);

// 上半截：进度两行是从文件算的，代理只写"这轮"、"判断"这类
function head(progress, agentLines) {
  const tree = require('./tree.js');
  const lines = tree.formatProgress(progress).split('\n');
  // 分隔线以下代理不许写，带进来的剥掉
  const clean = (agentLines || [])
    .map((l) => String(l).trim())
    .filter((l) => l !== '' && !l.startsWith('---') && l !== SEP);
  const all = lines.concat(clean);
  return { lines: all.slice(0, HEAD_CAP), truncated: all.length > HEAD_CAP };
}

// 一行里最多列几个文件。行数够了但一行拖到屏幕外，一样是没法看
const LIST = 6;
const list = (a) => (a.length > LIST ? a.slice(0, LIST).join('、') + `…等 ${a.length} 个` : a.join('、'));

// 下半截：全部来自 git 和命令输出（R4）
// harness 自己在 docs/harness/ 下的账不算代理动的文件，不然每轮都是一片噪音
function machine(facts, allowed, tests) {
  const all = facts && facts.repo ? facts.paths : [];
  const paths = all.filter((p) => !gitfacts.isOwn(p));
  const out = gitfacts.outOfScope(all, allowed);
  return [
    `动了文件：${paths.length ? list(paths) : '无'}`,
    `范围外：${out.length}${out.length ? '（' + list(out) + '）' : ''}`,
    `测试：${tests || '没跑'}`,
  ];
}

// 一轮报告 = 上半截 + 分隔线 + 机器三行，中间不留空行（不然一份就超 8 行）。
// 空行只隔开轮次，所以只在新一轮的上半截之前加。
function append(root, text, opts) {
  const o = opts || {};
  const f = file(root, o.date);
  if (!o.write) return { wrote: false, file: f, text, note: `会追加到 ${f}` };
  fs.mkdirSync(path.dirname(f), { recursive: true });
  let head = '';
  if (o.newRound) {
    let prev = '';
    try { prev = fs.readFileSync(f, 'utf8'); } catch { /* 还没有这个文件 */ }
    if (prev.trim() !== '') head = '\n';
  }
  fs.appendFileSync(f, head + text.replace(/\n*$/, '\n'));
  return { wrote: true, file: f, text };
}

// 代理这轮写没写报告：最后一轮有上半截、还没配上机器栏
function hasPendingHead(root, date) {
  let t;
  try { t = fs.readFileSync(file(root, date), 'utf8'); } catch { return false; }
  const round = t.split(/\n\s*\n/).map((s) => s.trim()).filter((s) => s !== '').pop();
  if (!round) return false;
  return !MACHINE_PREFIX.every((p) => round.includes('\n' + p) || round.startsWith(p));
}

// 代理那栏落盘。一轮从这里开头，所以空行加在它前面
function writeHead(root, progress, agentLines, opts) {
  const h = head(progress, agentLines);
  const r = append(root, h.lines.join('\n'), { ...(opts || {}), newRound: true });
  if (h.truncated) r.warn = `报告超 ${CAP} 行，已截断`;
  return r;
}

// 机器那栏落盘
function writeMachine(root, info, opts) {
  const lines = machine(info.facts, info.allowed, info.tests);
  const r = append(root, [SEP].concat(lines).join('\n'), opts);
  r.lines = lines;
  r.outOfScope = gitfacts.outOfScope(info.facts && info.facts.repo ? info.facts.paths : [], info.allowed).length;
  return r;
}

module.exports = { CAP, SEP, HEAD_CAP, dir, today, file, head, machine, append, hasPendingHead, writeHead, writeMachine };
