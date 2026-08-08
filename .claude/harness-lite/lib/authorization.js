'use strict';
// 轻量授权账：只认用户来源、只在当前 stage/leaf 有效。
// 同一系统账号能改这个文件，所以它是支持执行面的流程边界，不是假装成 OS 安全边界。
const fs = require('fs');
const path = require('path');

const file = (root) => path.join(root, 'docs', 'harness', 'authorization.json');

function read(root) {
  try {
    const value = JSON.parse(fs.readFileSync(file(root), 'utf8'));
    return value && typeof value === 'object' ? value : null;
  } catch { return null; }
}

const leafId = (c) => c && c.leaf ? c.leaf.name.split('-')[0] : null;

function active(root, chain) {
  const record = read(root);
  if (!record) return { ok: false, why: '没有有效授权记录', record: null };
  if (record.version !== 1 || record.issuedBy !== 'user' || !record.id) {
    return { ok: false, why: '授权记录不是用户来源或格式不对', record };
  }
  const c = chain || require('./tree.js').readChain(root);
  if (c.lifecycle && !c.lifecycle.ok) return { ok: false, why: '当前生命周期状态不合法', record };
  const scope = record.scope || {};
  const stageMatches = scope.kind === 'stage' && !!c.stage && scope.id === c.stage.id;
  if (stageMatches && require('./tree.js').progress(root, c).allDone) {
    return { ok: false, why: '阶段已经完成，授权自动失效', record };
  }
  const matches = scope.kind === 'stage'
    ? stageMatches
    : scope.kind === 'leaf' && scope.id === leafId(c);
  if (!matches) return { ok: false, why: '授权不属于当前工作', record };
  return { ok: true, why: null, record };
}

const stageMayContinue = (root, chain) => {
  const a = active(root, chain);
  return !!(a.ok && a.record.scope && a.record.scope.kind === 'stage');
};

// 阶段完成后普通授权仍失效；只给 close-stage 一张窄的终态归档票。
function canCloseStage(root, chain) {
  const record = read(root);
  if (!record || record.version !== 1 || record.issuedBy !== 'user' || !record.id) {
    return { ok: false, why: '没有用户来源的阶段授权', record };
  }
  const c = chain || require('./tree.js').readChain(root);
  if (!c.lifecycle.ok || !c.stage || c.extraStages.length) return { ok: false, why: '当前阶段状态不唯一', record };
  if (c.lifecycle.currentCount || c.lifecycle.unfinishedCount) return { ok: false, why: '阶段还有未完成 leaf', record };
  const p = require('./tree.js').progress(root, c);
  if (!p.total || !p.allDone) return { ok: false, why: '阶段还没有完整完成记录', record };
  if (!record.scope || record.scope.kind !== 'stage' || record.scope.id !== c.stage.id) {
    return { ok: false, why: '授权不属于这个已完成阶段', record };
  }
  const target = path.relative(root, c.stage.file).split(path.sep).join('/');
  const grant = grantFor(record, { category: 'context', operation: 'change', target });
  return grant ? { ok: true, record, grant, chain: c, progress: p, target }
    : { ok: false, why: '阶段授权没有覆盖收尾动作', record };
}

function targetMatches(pattern, target) {
  const p = String(pattern || '');
  const t = String(target || '');
  if (p === '*') return true;
  if (p.endsWith('*')) return t.startsWith(p.slice(0, -1));
  if (p.endsWith('/')) return t === p.slice(0, -1) || t.startsWith(p);
  return t === p;
}

function grantFor(record, request) {
  return (record && Array.isArray(record.grants) ? record.grants : []).find((g) =>
    g && g.id && g.category === request.category
    && (g.operations || []).some((x) => x === '*' || x === request.operation)
    && (g.targets || []).some((x) => targetMatches(x, request.target)));
}

module.exports = { file, read, active, stageMayContinue, canCloseStage, targetMatches, grantFor };
