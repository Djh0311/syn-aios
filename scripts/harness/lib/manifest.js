const crypto = require('crypto');
const fs = require('fs');
const path = require('path');

const manifestRelativePath = '.harness/manifest.json';

function sha256File(filePath) {
  return crypto.createHash('sha256').update(fs.readFileSync(filePath)).digest('hex');
}

function rel(root, filePath) {
  return path.relative(root, filePath).split(path.sep).join('/');
}

function manifestPath(targetRoot) {
  return path.join(targetRoot, manifestRelativePath);
}

function readManifest(targetRoot) {
  const file = manifestPath(targetRoot);
  if (!fs.existsSync(file)) return { path: file, data: null, error: null };
  try {
    return {
      path: file,
      data: JSON.parse(fs.readFileSync(file, 'utf8')),
      error: null
    };
  } catch (error) {
    return {
      path: file,
      data: null,
      error: error.message
    };
  }
}

function makeManifest(sourceRoot, targetRoot, items) {
  const files = {};
  for (const item of items) {
    if (!item.source || !item.target) continue;
    if (!fs.existsSync(item.source) || !fs.existsSync(item.target)) continue;
    const targetRelative = rel(targetRoot, item.target);
    files[targetRelative] = {
      source: rel(sourceRoot, item.source),
      sourceSha256: sha256File(item.source),
      installedSha256: sha256File(item.target)
    };
  }

  return {
    schemaVersion: 1,
    generatedAt: new Date().toISOString(),
    sourceRoot,
    files
  };
}

function writeManifest(sourceRoot, targetRoot, items) {
  const file = manifestPath(targetRoot);
  const manifest = makeManifest(sourceRoot, targetRoot, items);
  fs.mkdirSync(path.dirname(file), { recursive: true });
  fs.writeFileSync(file, `${JSON.stringify(manifest, null, 2)}\n`, 'utf8');
  return { path: file, data: manifest };
}

function manifestEntry(manifest, targetRelative) {
  return manifest && manifest.files ? manifest.files[targetRelative] : null;
}

function targetChangedSinceInstall(targetRoot, manifest, targetRelative) {
  const entry = manifestEntry(manifest, targetRelative);
  const target = path.join(targetRoot, targetRelative);
  if (!entry || !fs.existsSync(target)) return false;
  return sha256File(target) !== entry.installedSha256;
}

module.exports = {
  manifestRelativePath,
  manifestPath,
  readManifest,
  writeManifest,
  sha256File,
  targetChangedSinceInstall
};
