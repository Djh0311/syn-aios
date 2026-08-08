'use strict';
// leaves/ 只放唯一 current；unfinished/ 放未开始、暂停或受阻；done/ 只放完成。
const fs = require('fs');
const path = require('path');
const tree = require('./tree.js');
const auth = require('./authorization.js');

const dirs = (root) => ({
  current: path.join(tree.hdir(root), 'leaves'),
  unfinished: path.join(tree.hdir(root), 'unfinished'),
});
const idOf = (f) => path.basename(f, '.md').split('-')[0];

function findIn(dir, name) {
  const key = String(name || '').replace(/\.md$/, '').toLowerCase();
  return tree.mdFiles(dir).find((f) => path.basename(f, '.md').toLowerCase() === key)
    || tree.mdFiles(dir).find((f) => path.basename(f, '.md').toLowerCase().startsWith(key + '-'))
    || null;
}

function next(root, stageId) {
  return tree.mdFiles(dirs(root).unfinished).map((f) => ({ f, leaf: require('./leaf.js').parse(f) }))
    .find((x) => x.leaf && (!stageId || x.leaf.stageId === stageId));
}

function promoteNext(root, opts) {
  const o = opts || {};
  const c = tree.readChain(root);
  if (tree.mdFiles(dirs(root).current).length) return { ok: false, msg: '已经有当前叶子' };
  if (!auth.stageMayContinue(root, c)) return { ok: false, msg: '没有整阶段授权，不能自动进入下一叶子' };
  const n = next(root, c.stage && c.stage.id);
  if (!n) return { ok: true, file: null, msg: '没有未完成叶子' };
  const dest = path.join(dirs(root).current, path.basename(n.f));
  if (o.write) {
    fs.mkdirSync(path.dirname(dest), { recursive: true });
    fs.renameSync(n.f, dest);
  }
  return { ok: true, file: dest, msg: `${o.write ? '已' : '会'}进入 ${path.basename(dest)}` };
}

function park(root, name, reason, opts) {
  const o = opts || {};
  const c = tree.readChain(root);
  if (!c.lifecycle.ok || !c.leaf) return { ok: false, msg: '当前叶子状态不正常，不能暂停' };
  if (idOf(c.leaf.file).toLowerCase() !== String(name || '').toLowerCase()) {
    return { ok: false, msg: `${name} 不是当前叶子` };
  }
  if (!reason || !String(reason).trim()) return { ok: false, msg: '暂停要留下原因' };
  if (!auth.active(root, c).ok) return { ok: false, msg: '没有当前任务授权' };
  const dest = path.join(dirs(root).unfinished, path.basename(c.leaf.file));
  if (o.write) {
    fs.appendFileSync(c.leaf.file, `\n未完成原因：${String(reason).trim()}\n`);
    fs.mkdirSync(path.dirname(dest), { recursive: true });
    fs.renameSync(c.leaf.file, dest);
  }
  return { ok: true, file: dest, msg: `${o.write ? '已' : '会'}把 ${idOf(c.leaf.file)} 放回未完成区` };
}

function resume(root, name, opts) {
  const o = opts || {};
  const c = tree.readChain(root);
  if (tree.mdFiles(dirs(root).current).length) return { ok: false, msg: '先完成或暂停当前叶子' };
  if (!auth.stageMayContinue(root, c)) return { ok: false, msg: '没有整阶段授权，不能切换当前工作' };
  const src = findIn(dirs(root).unfinished, name);
  if (!src) return { ok: false, msg: `unfinished/ 里没找到 ${name}` };
  const dest = path.join(dirs(root).current, path.basename(src));
  if (o.write) fs.renameSync(src, dest);
  return { ok: true, file: dest, msg: `${o.write ? '已' : '会'}恢复 ${path.basename(dest)}` };
}

module.exports = { dirs, findIn, next, promoteNext, park, resume };
