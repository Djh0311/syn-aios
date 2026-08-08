'use strict';
// 安装归属只记 Lite 真正创建的文件；已有文件不认领，改过的文件不覆盖、不删除。
const crypto = require('crypto');
const fs = require('fs');
const path = require('path');
const MANIFEST = '.claude/harness-lite/ownership.json';
const empty = () => ({ version: 1, managedBy: 'harness-lite', packages: {} });
const digest = (body) => crypto.createHash('sha256').update(body).digest('hex');
function stat(file) {
  try { return fs.lstatSync(file); } catch (e) { if (e.code === 'ENOENT') return null; throw e; }
}
function relative(rel) {
  if (typeof rel !== 'string' || !rel || path.isAbsolute(rel)
      || rel.split(/[\\/]/).includes('..')) {
    throw new Error(`必须是项目内相对路径：${rel}`);
  }
  return rel.replaceAll('\\', '/').replace(/^\.\//, '');
}
function target(root, rel) {
  const clean = relative(rel);
  const abs = path.resolve(root, clean);
  const base = path.resolve(root);
  if (abs !== base && !abs.startsWith(`${base}${path.sep}`)) {
    throw new Error(`必须是项目内相对路径：${rel}`);
  }
  let cursor = base;
  const parts = clean.split('/');
  for (const [i, part] of parts.entries()) {
    cursor = path.join(cursor, part);
    const s = stat(cursor);
    if (s && s.isSymbolicLink()) throw new Error(`安装路径不能经过符号链接：${clean}`);
    if (s && i < parts.length - 1 && !s.isDirectory()) throw new Error(`安装路径的父级不是目录：${clean}`);
    if (s && i === parts.length - 1 && !s.isFile()) throw new Error(`安装目标不是普通文件：${clean}`);
  }
  return { rel: clean, abs, stat: stat(abs) };
}
function read(root) {
  const { abs, stat: present } = target(root, MANIFEST);
  if (!present) return empty();
  const data = JSON.parse(fs.readFileSync(abs, 'utf8'));
  if (data.version !== 1 || data.managedBy !== 'harness-lite'
      || !data.packages || Array.isArray(data.packages)) {
    throw new Error('不认识这个安装归属清单');
  }
  return data;
}
function save(root, data) {
  const { abs, stat: present } = target(root, MANIFEST);
  if (Object.keys(data.packages).length === 0) {
    if (present) fs.unlinkSync(abs);
    return;
  }
  fs.mkdirSync(path.dirname(abs), { recursive: true });
  fs.writeFileSync(abs, `${JSON.stringify(data, null, 2)}\n`);
}
function installPackage(root, packageId, packageVersion, items, opts) {
  const o = opts || {};
  const manifest = read(root);
  const prior = manifest.packages[packageId];
  if (o.upgrade && !prior) throw new Error(`${packageId} 还没有安装，不能升级`);
  const prepared = items.map(([rel, body]) => {
    const item = target(root, rel);
    return { ...item, body, hash: digest(body), current: item.stat ? digest(fs.readFileSync(item.abs)) : null };
  });
  target(root, MANIFEST); // 写任何文件前，也把账本路径完整预检掉
  const sourceFiles = new Set(prepared.map((x) => x.rel));
  const sourceChanged = prior && (prepared.some((x) => prior.files[x.rel] ? prior.files[x.rel] !== x.hash : !x.stat)
    || Object.keys(prior.files).some((x) => !sourceFiles.has(x)));
  if (prior && !o.upgrade && (prior.packageVersion !== packageVersion || sourceChanged)) {
    throw new Error(`${packageId} 已安装内容和当前源不同；请显式使用 --upgrade`);
  }
  const files = { ...(prior ? prior.files : {}) };
  const roots = (o.upgradeRoots || []).map((x) => `${relative(x).replace(/\/$/, '')}/`);
  const changed = [];
  let rows;
  try { rows = prepared.map((item) => {
    const recorded = files[item.rel];
    const allowed = !roots.length || roots.some((x) => item.rel.startsWith(x));
    if (!item.stat) {
      if (prior && (recorded || (o.upgrade && !allowed))) {
        return { rel: item.rel, wrote: false, skipped: true, protected: !!recorded, preserved: !allowed };
      }
      if (o.write) {
        changed.push([item.abs, null]);
        fs.mkdirSync(path.dirname(item.abs), { recursive: true });
        fs.writeFileSync(item.abs, item.body);
        files[item.rel] = item.hash;
      }
      return { rel: item.rel, wrote: !!o.write, skipped: false, protected: false };
    }
    if (o.upgrade && allowed && recorded && item.current === recorded && item.hash !== recorded) {
      if (o.write) { changed.push([item.abs, fs.readFileSync(item.abs)]); fs.writeFileSync(item.abs, item.body); files[item.rel] = item.hash; }
      return { rel: item.rel, wrote: !!o.write, skipped: false, protected: false, updated: true };
    }
    const protectedFile = !!recorded && (item.current !== recorded || item.hash !== recorded);
    return { rel: item.rel, wrote: false, skipped: true, protected: protectedFile, preserved: !!o.upgrade && !allowed };
  });
  const versionChanged = !!prior && prior.packageVersion !== packageVersion;
  if (o.write && (versionChanged || rows.some((row) => row.wrote))) {
    const ledger = path.join(root, MANIFEST);
    changed.push([ledger, stat(ledger) ? fs.readFileSync(ledger) : null]);
    manifest.packages[packageId] = { packageVersion, files };
    save(root, manifest);
  }
  } catch (e) {
    for (const [file, before] of changed.reverse()) try { before == null ? fs.rmSync(file, { force: true }) : fs.writeFileSync(file, before); } catch { /* 原错误优先 */ }
    throw e;
  }
  return {
    root, packageId, wrote: !!o.write, rows,
    written: rows.filter((row) => row.wrote).length,
    skipped: rows.filter((row) => row.skipped).map((row) => row.rel),
    protected: rows.filter((row) => row.protected).map((row) => row.rel),
    total: rows.length,
  };
}
function uninstallPackage(root, packageId, opts) {
  const o = opts || {};
  const manifest = read(root);
  const pkg = manifest.packages[packageId];
  const rows = Object.entries(pkg ? pkg.files : {}).map(([rel, recorded]) => {
    const item = target(root, rel);
    if (!item.stat) return { ...item, missing: true, protected: false, removed: false };
    const protectedFile = digest(fs.readFileSync(item.abs)) !== recorded;
    return { ...item, missing: false, protected: protectedFile, removed: false };
  });
  if (o.write && pkg) {
    for (const row of rows) {
      if (!row.missing && !row.protected) { fs.unlinkSync(row.abs); row.removed = true; }
    }
    const keep = Object.fromEntries(rows.filter((row) => row.protected)
      .map((row) => [row.rel, pkg.files[row.rel]]));
    if (Object.keys(keep).length) manifest.packages[packageId] = { ...pkg, files: keep };
    else delete manifest.packages[packageId];
    save(root, manifest);
  }
  return {
    root, packageId, wrote: !!o.write, rows,
    removed: rows.filter((row) => row.removed).map((row) => row.rel),
    protected: rows.filter((row) => row.protected).map((row) => row.rel),
    missing: rows.filter((row) => row.missing).map((row) => row.rel),
  };
}
module.exports = { MANIFEST, digest, read, relative, installPackage, uninstallPackage };
