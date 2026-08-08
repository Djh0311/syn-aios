'use strict';
// 五类硬边界共用一个判断器；普通开发动作不进门。
const crypto = require('crypto');
const fs = require('fs');
const path = require('path');
const auth = require('./authorization.js');

const HARD = new Set(['external', 'destructive', 'context', 'control', 'project-sensitive']);
const CONTROL = /(^|[\/\s])(docs\/harness|\.claude\/harness-lite|\.codex|\.githooks)(\/|$)|(^|[\/\s])AGENTS\.md$/;
const cmdOf = (i) => String(i && i.tool_input && i.tool_input.command || '');
const fileOf = (i) => String(i && i.tool_input
  && (i.tool_input.file_path || i.tool_input.path || i.tool_input.target_file) || '');
const patchOf = (i) => String(i && i.tool_input && i.tool_input.patch || '');

function classify(input, policy) {
  const explicit = input && input.harnessCategory;
  if (HARD.has(explicit)) return { category: explicit, operation: input.operation || 'act', target: input.target || '*' };
  const tool = String(input && input.tool_name || '');
  const cmd = /bash|shell|terminal/i.test(tool) ? cmdOf(input) : '';
  const file = fileOf(input);
  const patch = patchOf(input);
  if (/\bgit\s+push\b|\b(ssh|scp|sftp)\b|\b(kubectl|terraform)\s+(apply|destroy)\b|\b(deploy|release)\b/i.test(cmd)) {
    return { category: 'external', operation: /git\s+push/i.test(cmd) ? 'push' : 'act', target: /git\s+push/i.test(cmd) ? 'origin' : '*' };
  }
  if (/\b(rm|rmdir|unlink)\b|\bgit\s+(clean|reset\s+--hard|branch\s+-[dD]|worktree\s+(remove|prune))\b/i.test(cmd)
      || /delete/i.test(tool)) {
    const named = cmd.match(/\bgit\s+branch\s+-[dD]\s+([^\s;&|]+)/i)
      || cmd.match(/\bgit\s+worktree\s+remove\s+([^\s;&|]+)/i)
      || cmd.match(/\b(?:rm|rmdir|unlink)\s+(?:-[^\s]+\s+)*([^\s;&|]+)/i);
    return { category: 'destructive', operation: 'delete', target: file || (named && named[1]) || '*' };
  }
  if (/^\s*(?:(?:[^\s;&|]+\/)?node\s+)?(?:"[^"$`\r\n]*hl\.js"|'[^'\r\n]*hl\.js'|[^\s;&|$`]*hl\.js|hl)\s+close-stage(?:\s+--(?:write|json)|\s+--target\s+(?:"[^"$`\r\n]*"|'[^'\r\n]*'|[^\s;&|$`]+))*\s*$/i.test(cmd)) {
    return { category: 'context', operation: 'close-stage', target: 'docs/harness/stages/' };
  }
  if (/\bgit\s+(switch|checkout|merge|rebase|cherry-pick)\b|\bhl\s+(done|park|resume)\b/i.test(cmd)) {
    return { category: 'context', operation: 'change', target: '*' };
  }
  if ((file && CONTROL.test(file) && /write|edit|delete|move|rename/i.test(tool))
      || (patch && CONTROL.test(patch) && /apply.?patch/i.test(tool))
      || (/\b(mv|cp|sed|perl|apply_patch)\b/.test(cmd) && CONTROL.test(cmd))) {
    return { category: 'control', operation: 'modify', target: file || '*' };
  }
  const p = policy || {};
  if ((p.sensitivePathPatterns || []).some((x) => new RegExp(x).test(file))
      || (p.sensitiveCommandPatterns || []).some((x) => new RegExp(x).test(cmd))) {
    return { category: 'project-sensitive', operation: 'act', target: file || '*' };
  }
  return { category: 'ordinary', operation: 'act', target: file || '*' };
}

const month = (d) => `${d.getUTCFullYear()}-${String(d.getUTCMonth() + 1).padStart(2, '0')}`;
const digest = (x) => crypto.createHash('sha256').update(String(x || '')).digest('hex');

function audit(root, request, result, opts) {
  const o = opts || {};
  if (!o.write || request.category === 'ordinary') return null;
  const at = o.at || new Date();
  const dir = path.join(root, 'docs', 'harness', 'audit');
  const file = path.join(dir, `${month(at)}.jsonl`);
  const row = {
    at: at.toISOString(), event: result.breakGlass ? 'BREAK_GLASS' : 'GATE', decision: result.decision,
    category: request.category, operation: request.operation, targetHash: digest(request.target),
    authorizationId: result.authorizationId || null, grantId: result.grantId || null,
    reason: result.breakGlass ? String(o.reason || '').slice(0, 240) : undefined,
  };
  fs.mkdirSync(dir, { recursive: true });
  fs.appendFileSync(file, JSON.stringify(row) + '\n');
  return file;
}

function evaluate(root, request, opts) {
  const o = opts || {};
  const q = { category: request.category, operation: request.operation || 'act', target: request.target || '*' };
  if (!HARD.has(q.category)) return { decision: 'allow', reason: '普通动作，不进硬门' };
  if (q.category === 'context' && q.operation === 'close-stage') {
    const a = auth.canCloseStage(root);
    const targetOk = a.ok && (q.target === a.target || q.target === 'docs/harness/stages/');
    const r = targetOk
      ? { decision: 'allow', reason: '只允许归档这个已完成阶段', authorizationId: a.record.id, grantId: a.grant.id }
      : { decision: 'ask', reason: a.ok ? '收尾目标不是当前阶段' : a.why };
    r.auditFile = audit(root, targetOk ? { ...q, target: a.target } : q, r, o);
    return r;
  }
  const a = auth.active(root);
  let result = { decision: 'ask', reason: a.ok ? '当前授权没有覆盖这个硬门' : a.why };
  if (a.ok && o.breakGlass) {
    const g = (a.record.grants || []).find((x) => x && x.id === o.breakGlass && x.breakGlass === true
      && x.category === q.category && (x.operations || []).includes('break-glass')
      && (x.operations || []).some((op) => op === '*' || op === q.operation)
      && (x.targets || []).some((t) => auth.targetMatches(t, q.target)));
    if (g && o.reason) result = { decision: 'allow', reason: '已授权误拦的 break-glass', breakGlass: true,
      authorizationId: a.record.id, grantId: g.id };
  } else if (a.ok) {
    const g = auth.grantFor(a.record, q);
    if (g) result = { decision: 'allow', reason: '用户已有明确授权', authorizationId: a.record.id, grantId: g.id };
  }
  result.auditFile = audit(root, q, result, o);
  return result;
}

function policy(root) {
  try { return JSON.parse(fs.readFileSync(path.join(root, 'docs/harness/policy.json'), 'utf8')); } catch { return {}; }
}

module.exports = { HARD, classify, evaluate, policy, audit };
