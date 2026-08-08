'use strict';
// done --write 是模型声明“当前 leaf 已按干完标准完成”，然后归档并退出当前工作。
const fs = require('fs');
const path = require('path');
const tree = require('./tree.js');
const leaf = require('./leaf.js');

const monthDir = (d) => {
  const t = d || new Date();
  return `${t.getFullYear()}-${String(t.getMonth() + 1).padStart(2, '0')}`;
};

// 按名字找 leaves/ 里的叶子：全名、去掉 .md、或者 L3 这样的前缀
function find(root, name) {
  const files = tree.mdFiles(path.join(tree.hdir(root), 'leaves'));
  const key = String(name || '').replace(/\.md$/, '').toLowerCase();
  const base = (f) => path.basename(f, '.md').toLowerCase();
  return files.find((f) => base(f) === key)
    || files.find((f) => base(f).startsWith(key + '-'))
    || files.find((f) => base(f).startsWith(key))
    || null;
}

// 阶段文件里对应那行勾上。只给人看，机器不拿它算数（判据只有 done/ 里有没有文件）
function tick(stageFile, leafName, opts) {
  const text = leaf.read(stageFile);
  if (text == null) return { ticked: false, why: '没有阶段文件' };
  const id = leafName.split('-')[0];
  const re = new RegExp(`^(\\s*-\\s*)\\[ \\](\\s*${id}\\b.*)$`, 'm');
  if (!re.test(text)) return { ticked: false, why: `阶段文件里没找到 ${id} 那行` };
  if (opts && opts.write) fs.writeFileSync(stageFile, text.replace(re, '$1[x]$2'));
  return { ticked: true };
}

// 叶子这次的轻交代：四样（R9）。只说"移好了"等于没交代 ——
// 用户不知道这块干出什么。四样说完立刻开下一个，不等人点头。
const FOUR = ['做出什么', '验证跑了什么', '改了哪些文件', '遗留'];

function handoff(root, r, mayContinue) {
  const p = tree.progress(root);
  const id = path.basename(r.dest, '.md').split('-')[0];
  const left = p.remaining.filter((t) => !t.startsWith(id + ' '));
  return `这个叶子交代四样：${FOUR.join('、')}。`
    + (mayContinue && left.length ? `说完立刻开下一个：${left[0]}。` : '答那九件并停止。');
}

function done(root, name, opts) {
  const o = opts || {};
  const currentFiles = tree.mdFiles(path.join(tree.hdir(root), 'leaves'));
  if (currentFiles.length !== 1) return { ok: false, msg: `当前叶子有 ${currentFiles.length} 个，必须恰好一个` };
  const allowed = require('./gate.js').evaluate(root,
    { category: 'context', operation: 'change', target: path.basename(currentFiles[0]) }, { write: !!o.write });
  if (allowed.decision !== 'allow') return { ok: false, msg: `不能结束当前工作：${allowed.reason}` };
  const src = find(root, name);
  if (!src) {
    const pending = require('./lifecycle.js').findIn(path.join(tree.hdir(root), 'unfinished'), name);
    return { ok: false, msg: pending ? `${name} 不是当前叶子，不能归档` : `leaves/ 里没找到 ${name}` };
  }
  const destDir = path.join(tree.hdir(root), 'done', monthDir(o.at));
  const dest = path.join(destDir, path.basename(src));
  if (fs.existsSync(dest)) return { ok: false, msg: `${dest} 已经在了，没动` };

  const chain = tree.readChain(root);
  let t = chain.stage ? tick(chain.stage.file, path.basename(src, '.md'), { write: false }) : { ticked: false, why: '没有阶段文件' };
  const c = tree.readChain(root);
  const mayContinue = require('./authorization.js').stageMayContinue(root, c);
  if (o.write) {
    fs.mkdirSync(destDir, { recursive: true });
    fs.renameSync(src, dest);
    t = chain.stage ? tick(chain.stage.file, path.basename(src, '.md'), { write: true }) : t;
  }
  const promoted = mayContinue ? require('./lifecycle.js').promoteNext(root, { write: o.write }) : null;
  const r = {
    ok: true, wrote: !!o.write, src, dest, tick: t,
    msg: `${o.write ? '已把' : '会把'} ${path.relative(root, src)} 移到 ${path.relative(root, dest)}`
      + (t.ticked ? '，阶段文件里那行勾上' : `（阶段文件没勾：${t.why}）`),
  };
  r.promoted = promoted && promoted.ok ? promoted.file : null;
  r.handoff = handoff(root, r, mayContinue);
  return r;
}

module.exports = { monthDir, find, tick, handoff, done, FOUR };
