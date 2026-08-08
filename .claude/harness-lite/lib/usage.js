'use strict';
// 使用记录。钩子自动写，一行一轮，只追加（R12）。
// 这里故意不提供删除或改写的接口 —— 能删自己的记录，跑偏的证据就被擦了。
const fs = require('fs');
const path = require('path');

// 工具中文名。hl 的子命令加进来一个，这里加一行，`hl usage` 才数得到它。
const TOOLS = {
  chain: '读链', progress: '进度', facts: 'git现状', tests: '该跑哪些测试',
  map: '代码地图', mistake: '错题本', done: '叶子退场', report: '写报告',
  usage: '使用记录',
};

const dir = (root) => path.join(root, 'docs', 'harness', 'usage');
const month = (d) => `${(d || new Date()).getFullYear()}-${String((d || new Date()).getMonth() + 1).padStart(2, '0')}`;
const file = (root, d) => path.join(dir(root), `${month(d)}.log`);
const turnFile = (root) => path.join(dir(root), '.turn');

// 这一轮实际跑过的 hl 子命令。每次调用追加一行，stop 钩子读完就清。
// 只在已装 harness 的项目里记 —— 不然随手在别处跑一下 hl 就给人建目录。
function noteTool(root, cmd) {
  if (!TOOLS[cmd]) return;
  if (!fs.existsSync(path.join(root, 'docs', 'harness'))) return;
  try {
    fs.mkdirSync(dir(root), { recursive: true });
    fs.appendFileSync(turnFile(root), cmd + '\n');
  } catch { /* 记不上就算了，不许因为记账失败挡住主流程 */ }
}

function takeTools(root, opts) {
  let t = '';
  try { t = fs.readFileSync(turnFile(root), 'utf8'); } catch { return []; }
  if (opts && opts.write) { try { fs.unlinkSync(turnFile(root)); } catch { /* 已经没了 */ } }
  return [...new Set(t.split('\n').map((s) => s.trim()).filter((s) => TOOLS[s]))];
}

function line(rec) {
  const stamp = (rec.at || new Date()).toISOString().replace(/\.\d+Z$/, '');
  return [
    stamp,
    rec.stage || '-',
    rec.leaf || '-',
    `报告:${rec.report ? '有' : '无'}`,
    `文件:${rec.files || 0}`,
    `越界:${rec.outOfScope || 0}`,
    `测试:${rec.tests ? '有' : '无'}`,
    `工具:${rec.tools && rec.tools.length ? rec.tools.join(',') : '-'}`,
  ].join('  ');
}

function append(root, rec, opts) {
  const f = file(root, rec.at);
  const text = line(rec);
  if (!(opts && opts.write)) return { wrote: false, file: f, text };
  fs.mkdirSync(dir(root), { recursive: true });
  fs.appendFileSync(f, text + '\n');
  return { wrote: true, file: f, text };
}

function readAll(root) {
  let files;
  try { files = fs.readdirSync(dir(root)).filter((f) => f.endsWith('.log')).sort(); } catch { return []; }
  const out = [];
  for (const f of files) {
    for (const l of fs.readFileSync(path.join(dir(root), f), 'utf8').split('\n')) {
      const t = l.trim();
      if (t === '') continue;
      const cols = t.split(/\s{2,}/);
      const tools = (cols.find((c) => c.startsWith('工具:')) || '').slice(3);
      out.push({
        at: cols[0],
        tools: tools === '-' || tools === '' ? [] : tools.split(','),
        outOfScope: Number((cols.find((c) => c.startsWith('越界:')) || '越界:0').slice(3)) || 0,
        report: (cols.find((c) => c.startsWith('报告:')) || '') === '报告:有',
      });
    }
  }
  return out;
}

const days = (from, now) => Math.floor(((now || new Date()) - new Date(from)) / 86400000);

function ago(d) {
  if (d == null) return '从没用过';
  if (d <= 0) return '今天';
  return `${d}天前`;
}

// 每个工具上次什么时候用、一共几次。没用过或超 30 天没动的，建议删（R12）
function table(root, now) {
  const rows = readAll(root);
  const stat = {};
  for (const k of Object.keys(TOOLS)) stat[k] = { cmd: k, name: TOOLS[k], count: 0, last: null };
  for (const r of rows) {
    for (const t of r.tools) {
      if (!stat[t]) continue;
      stat[t].count++;
      if (!stat[t].last || r.at > stat[t].last) stat[t].last = r.at;
    }
  }
  return Object.values(stat).map((s) => {
    const d = s.last ? days(s.last, now) : null;
    return { ...s, days: d, ago: ago(d), suggestDrop: d == null || d > 30 };
  }).sort((a, b) => b.count - a.count || a.cmd.localeCompare(b.cmd));
}

function format(rows) {
  const w = Math.max(...rows.map((r) => [...r.name].reduce((n, c) => n + (c.charCodeAt(0) > 255 ? 2 : 1), 0)));
  return rows.map((r) => {
    const pad = ' '.repeat(w - [...r.name].reduce((n, c) => n + (c.charCodeAt(0) > 255 ? 2 : 1), 0) + 2);
    return `${r.name}${pad}${r.ago.padEnd(10)}用了 ${r.count} 次${r.suggestDrop ? '  ← 建议删' : ''}`;
  }).join('\n');
}

module.exports = { TOOLS, dir, month, file, turnFile, noteTool, takeTools, line, append, readAll, table, format, ago, days };
