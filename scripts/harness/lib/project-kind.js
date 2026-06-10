const fs = require('fs');
const path = require('path');

function exists(targetRoot, relativePath) {
  return fs.existsSync(path.join(targetRoot, relativePath));
}

function hasAll(targetRoot, relativePaths) {
  return relativePaths.every((relativePath) => exists(targetRoot, relativePath));
}

function detectProjectKind(targetRoot) {
  const hasRuntimeDocs = exists(targetRoot, 'docs');
  const hasRequiredSourceSignals = hasAll(targetRoot, [
    'plans',
    'AGENTS.md',
    'codex-multi-agent-safe-collaboration.md',
    'skills/using-superpowers/SKILL.md',
    'templates/docs/current-state.md',
    'templates/docs/evidence/README.md',
    'scripts/harness/rules-lint.js',
    'scripts/harness/install-harness.js',
    'scripts/harness/sync-harness.js'
  ]);
  const hasSourcePlanSignals = exists(targetRoot, 'plans/2026-05-12-harness-upgrade-phase-1.md');
  const hasSourceSignals = hasRequiredSourceSignals;
  const hasInstalledSignals = hasAll(targetRoot, [
    'AGENTS.md',
    'codex-multi-agent-safe-collaboration.md',
    'skills/using-superpowers/SKILL.md',
    'docs/current-state.md'
  ]);
  const isSourcePackage = hasSourceSignals && (!hasInstalledSignals || hasSourcePlanSignals);

  if (isSourcePackage) {
    return {
      kind: 'source-package',
      isSourcePackage: true,
      isInstalledProject: false,
      hasRuntimeDocs,
      hasSourceSignals,
      hasInstalledSignals,
      hasSourcePlanSignals
    };
  }

  if (hasInstalledSignals || hasRuntimeDocs) {
    return {
      kind: 'installed-project',
      isSourcePackage: false,
      isInstalledProject: true,
      hasRuntimeDocs,
      hasSourceSignals,
      hasInstalledSignals,
      hasSourcePlanSignals
    };
  }

  return {
    kind: 'unknown',
    isSourcePackage: false,
    isInstalledProject: false,
    hasRuntimeDocs,
    hasSourceSignals,
    hasInstalledSignals,
    hasSourcePlanSignals
  };
}

module.exports = {
  detectProjectKind
};
