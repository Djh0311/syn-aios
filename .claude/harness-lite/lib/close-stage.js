'use strict';
// 最后一个 leaf 完成后的窄收口：归档唯一 stage，并把总计划对应行勾上。
const fs = require('fs');
const path = require('path');
const tree = require('./tree.js');
const gate = require('./gate.js');
const { monthDir } = require('./done.js');

function planEdit(root, title) {
  const file = path.join(tree.hdir(root), 'plan.md');
  let text;
  try { text = fs.readFileSync(file, 'utf8'); } catch { return { ok: false, msg: '没有可更新的总计划' }; }
  const eol = text.includes('\r\n') ? '\r\n' : '\n';
  const lines = text.split(eol);
  const hits = [];
  for (let i = 0; i < lines.length; i++) {
    const m = lines[i].match(/^(\s*)-\s+(?:\[ \]\s+)?(.+?)\s*$/);
    if (m && m[2] === title) hits.push({ i, indent: m[1] });
  }
  if (hits.length !== 1) return { ok: false, msg: `总计划里没有唯一的未完成阶段：${title}` };
  lines[hits[0].i] = `${hits[0].indent}- [x] ${title}`;
  return { ok: true, file, text: lines.join(eol) };
}

function prepare(root, opts) {
  const c = tree.readChain(root);
  if (!c.installed || !c.stage) return { ok: false, msg: '没有当前阶段可归档' };
  if (!c.lifecycle.ok || c.extraStages.length) return { ok: false, msg: 'stages/ 必须恰好一个当前阶段' };
  const p = tree.progress(root, c);
  if (!p.total || !p.allDone || c.lifecycle.currentCount || c.lifecycle.unfinishedCount) {
    return { ok: false, msg: '阶段还有未完成 leaf，不能归档' };
  }
  const plan = planEdit(root, c.stage.title);
  if (!plan.ok) return plan;
  const dest = path.join(tree.hdir(root), 'done', monthDir(opts && opts.at), path.basename(c.stage.file));
  if (fs.existsSync(dest)) return { ok: false, msg: `${path.relative(root, dest)} 已经存在` };
  const request = { category: 'context', operation: 'close-stage',
    target: path.relative(root, c.stage.file).split(path.sep).join('/') };
  const permission = gate.evaluate(root, request);
  if (permission.decision !== 'allow') return { ok: false, msg: `不能收阶段：${permission.reason}` };
  return { ok: true, src: c.stage.file, dest, plan, request };
}

function closeStage(root, opts) {
  const o = opts || {};
  const r = prepare(root, o);
  if (!r.ok) return r;
  const msg = `${o.write ? '已把' : '会把'} ${path.relative(root, r.src)} 归档到 ${path.relative(root, r.dest)}，总计划同步勾上`;
  if (!o.write) return { ...r, wrote: false, msg };
  const permission = gate.evaluate(root, r.request, { write: true });
  if (permission.decision !== 'allow') return { ok: false, msg: `不能收阶段：${permission.reason}` };
  fs.mkdirSync(path.dirname(r.dest), { recursive: true });
  const temp = path.join(path.dirname(r.plan.file), `.${path.basename(r.plan.file)}.close-${process.pid}-${Date.now()}`);
  let copied = false, removed = false;
  try {
    fs.writeFileSync(temp, r.plan.text, { flag: 'wx', mode: fs.statSync(r.plan.file).mode });
    fs.copyFileSync(r.src, r.dest, fs.constants.COPYFILE_EXCL);
    copied = true;
    fs.unlinkSync(r.src);
    removed = true;
    fs.renameSync(temp, r.plan.file);
  } catch (e) {
    try { if (copied && fs.existsSync(r.dest)) removed && !fs.existsSync(r.src) ? fs.renameSync(r.dest, r.src) : fs.unlinkSync(r.dest); } catch { /* 报下面的稳定错误 */ }
    try { fs.unlinkSync(temp); } catch { /* 已不存在 */ }
    return { ok: false, msg: `阶段收尾写入失败：${e.message}` };
  }
  return { ...r, permission, wrote: true, msg };
}

module.exports = { planEdit, prepare, closeStage };
