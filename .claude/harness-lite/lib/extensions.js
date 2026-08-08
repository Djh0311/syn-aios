'use strict';
// 扩展只是仓内一张文件清单；生命周期、授权、检查和报告仍全部归 Lite。
const fs = require('fs');
const path = require('path');
const ownership = require('./ownership.js');

function load(dir) {
  const manifest = JSON.parse(fs.readFileSync(path.join(dir, 'manifest.json'), 'utf8'));
  if (manifest.version !== 1 || !/^[a-z0-9][a-z0-9-]*$/.test(manifest.id || '')
      || typeof manifest.packageVersion !== 'string' || !manifest.packageVersion
      || !manifest.files || Array.isArray(manifest.files)) {
    throw new Error('扩展 manifest 不合规');
  }
  const base = fs.realpathSync(dir);
  const items = Object.entries(manifest.files).map(([to, from]) => {
    const target = ownership.relative(to);
    const source = ownership.relative(from);
    const abs = fs.realpathSync(path.join(dir, source));
    if (!abs.startsWith(`${base}${path.sep}`) || !fs.lstatSync(abs).isFile()) {
      throw new Error(`扩展源文件不在扩展目录内：${from}`);
    }
    return [target, fs.readFileSync(abs)];
  });
  return { ...manifest, items };
}

function install(root, dir, opts) {
  const extension = load(dir);
  if (opts && opts.expectedId && extension.id !== opts.expectedId) {
    throw new Error(`扩展 id 不一致：要 ${opts.expectedId}，实际 ${extension.id}`);
  }
  return ownership.installPackage(root, `extension:${extension.id}`,
    extension.packageVersion, extension.items, opts);
}

function uninstall(root, id, opts) {
  if (!/^[a-z0-9][a-z0-9-]*$/.test(id || '')) throw new Error('扩展 id 不合规');
  return ownership.uninstallPackage(root, `extension:${id}`, opts);
}

module.exports = { load, install, uninstall };
