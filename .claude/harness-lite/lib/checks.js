'use strict';
// 项目自己登记命令；Lite 只在明确调用时选择、执行并记一行，不做缓存或自动套档。
const fs = require('fs');
const path = require('path');
const { spawnSync } = require('child_process');
const ownership = require('./ownership.js');
const gate = require('./gate.js');

const CONFIG = 'docs/harness/checks.json';
const RESULTS = 'docs/harness/check-results.jsonl';
const PROFILES = new Set(['quick', 'task', 'full', 'manual']);
const TIMEOUT = { quick: 30000, task: 120000, full: 3600000 };
const bad = (why) => { throw new Error(`检查登记不合规：${why}`); };

function clean(file) {
  const value = ownership.relative(file).replace(/\/+$/, '');
  if (!value) bad('路径不能为空');
  return value;
}

function validate(config) {
  if (!config || config.version !== 1 || !Array.isArray(config.checks)) bad('需要 version 1 和 checks 数组');
  const ids = new Set();
  const list = config.checks.map((item) => {
    if (!item || !/^[a-z0-9][a-z0-9-]*$/.test(item.id || '') || ids.has(item.id)) bad('id 缺失、重复或不合规');
    ids.add(item.id);
    if (!PROFILES.has(item.profile)) bad(`${item.id} 的 profile 未知`);
    if (item.profile === 'manual') {
      if (typeof item.note !== 'string' || !item.note.trim() || item.argv !== undefined) bad(`${item.id} 的 manual 只能写 note`);
      return { ...item, note: item.note.trim() };
    }
    if (!Array.isArray(item.argv) || !item.argv.length
        || item.argv.some((arg) => typeof arg !== 'string' || !arg || arg.includes('\0'))) bad(`${item.id} 的 argv 必须是非空字符串数组`);
    if (!Number.isInteger(item.timeoutMs) || item.timeoutMs < 1 || item.timeoutMs > TIMEOUT[item.profile]) {
      bad(`${item.id} 的 timeoutMs 超出 ${item.profile} 上限`);
    }
    if (item.profile === 'task' && (!Array.isArray(item.paths) || !item.paths.length)) bad(`${item.id} 的 task 必须写 paths`);
    if (item.profile !== 'task' && item.paths !== undefined) bad(`${item.id} 只有 task 能写 paths`);
    return { ...item, paths: item.paths ? item.paths.map(clean) : undefined };
  });
  return { version: 1, checks: list };
}

function load(root) {
  try {
    return validate(JSON.parse(fs.readFileSync(path.join(root, CONFIG), 'utf8')));
  } catch (e) {
    if (e.code === 'ENOENT') return { version: 1, checks: [] };
    throw e;
  }
}

function select(config, profile, files) {
  if (!PROFILES.has(profile)) bad(`未知 profile ${profile}`);
  const changed = (files || []).map(clean);
  const list = validate(config).checks.filter((item) => {
    if (item.profile !== profile) return false;
    if (profile !== 'task') return true;
    return changed.some((file) => item.paths.some((prefix) => file === prefix || file.startsWith(`${prefix}/`)));
  });
  const commands = new Set();
  return list.filter((item) => {
    const key = item.argv ? JSON.stringify(item.argv) : item.id;
    if (commands.has(key)) return false;
    commands.add(key);
    return true;
  });
}

function safeResult(root) {
  const abs = path.resolve(root, RESULTS);
  let cursor = path.resolve(root);
  const parts = RESULTS.split('/');
  for (const [i, part] of parts.entries()) {
    cursor = path.join(cursor, part);
    let s;
    try { s = fs.lstatSync(cursor); } catch (e) { if (e.code === 'ENOENT') continue; throw e; }
    if (s.isSymbolicLink()) throw new Error(`检查结果路径不能经过符号链接：${RESULTS}`);
    if (i < parts.length - 1 && !s.isDirectory()) throw new Error(`检查结果路径父级不是目录：${RESULTS}`);
    if (i === parts.length - 1 && !s.isFile()) throw new Error(`检查结果路径不是普通文件：${RESULTS}`);
  }
  return abs;
}

function run(root, profile, files, opts) {
  const selected = select(load(root), profile, files);
  if (profile === 'manual') {
    return { profile, selected, notes: selected.map((x) => ({ id: x.id, note: x.note })), results: [], executed: 0, ok: true };
  }
  if (!selected.length) return { profile, selected, results: [], executed: 0, ok: true };
  const prepared = selected.map((item) => ({ item, permission: gate.evaluate(root,
    gate.classify({ tool_name: 'Bash', tool_input: { command: item.argv.join(' ') } }, gate.policy(root)),
    { write: true }) }));
  let file = safeResult(root);
  fs.mkdirSync(path.dirname(file), { recursive: true });
  file = safeResult(root);
  const fd = fs.openSync(file, fs.constants.O_WRONLY | fs.constants.O_CREAT
    | fs.constants.O_APPEND | (fs.constants.O_NOFOLLOW || 0), 0o600);
  const results = prepared.map(({ item, permission }) => {
    if (permission.decision !== 'allow') {
      return { id: item.id, ok: false, blocked: true, timedOut: false, status: null };
    }
    let p;
    try {
      p = spawnSync(item.argv[0], item.argv.slice(1), {
        cwd: root, shell: false, stdio: 'ignore', timeout: item.timeoutMs,
      });
    } catch { return { id: item.id, ok: false, timedOut: false, status: null }; }
    return { id: item.id, ok: !p.error && p.status === 0, timedOut: !!p.error && p.error.code === 'ETIMEDOUT', status: p.status };
  });
  const record = {
    at: ((opts && opts.at) || new Date()).toISOString(), profile,
    ids: results.map((x) => x.id), ok: results.every((x) => x.ok),
    timedOut: results.some((x) => x.timedOut), blocked: results.some((x) => x.blocked),
  };
  try { fs.writeSync(fd, `${JSON.stringify(record)}\n`); } finally { fs.closeSync(fd); }
  return { profile, selected, results, executed: results.length, ok: record.ok, record };
}

function format(result) {
  if (result.notes) return result.notes.length
    ? result.notes.map((x) => `需人工 ${x.id}：${x.note}`).join('\n') : '没有人工检查';
  if (!result.executed) return '没有相关检查（未执行）';
  return result.results.map((x) => `${x.ok ? '通过' : x.blocked ? '硬门拦下' : x.timedOut ? '超时' : '失败'}  ${x.id}`).join('\n');
}

module.exports = { CONFIG, RESULTS, TIMEOUT, validate, load, select, run, format };
