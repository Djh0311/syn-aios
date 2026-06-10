const fs = require('fs');
const path = require('path');
const { detectProjectKind } = require('./project-kind');

function exists(root, relativePath) {
  return fs.existsSync(path.join(root, relativePath));
}

function readJson(filePath) {
  try {
    return { data: JSON.parse(fs.readFileSync(filePath, 'utf8')), error: null };
  } catch (error) {
    return { data: null, error: error.message };
  }
}

function listDirectories(root, relativePath) {
  const full = path.join(root, relativePath);
  if (!fs.existsSync(full)) return [];
  try {
    return fs.readdirSync(full, { withFileTypes: true })
      .filter((entry) => entry.isDirectory())
      .map((entry) => path.join(relativePath, entry.name).split(path.sep).join('/'));
  } catch (_error) {
    return [];
  }
}

function packageJson(root) {
  const file = path.join(root, 'package.json');
  if (!fs.existsSync(file)) return { exists: false, data: null, error: null };
  const parsed = readJson(file);
  return { exists: true, data: parsed.data, error: parsed.error };
}

function dependencyNames(pkg) {
  const deps = Object.assign(
    {},
    pkg && pkg.dependencies,
    pkg && pkg.devDependencies,
    pkg && pkg.optionalDependencies
  );
  return Object.keys(deps);
}

function addSignal(signals, name, weight, evidence) {
  signals.push({ name, weight, evidence: Array.isArray(evidence) ? evidence : [evidence] });
}

function hasAny(root, relativePaths) {
  return relativePaths.some((relativePath) => exists(root, relativePath));
}

function detectProjectSignals(targetRoot) {
  const kind = detectProjectKind(targetRoot);
  const pkg = packageJson(targetRoot);
  const deps = dependencyNames(pkg.data);
  const signals = [];

  if (kind.isSourcePackage) addSignal(signals, 'source-package', 1, 'standard harness source package');
  if (kind.isInstalledProject) addSignal(signals, 'installed-project', 1, 'installed harness runtime docs or AGENTS.md present');
  if (pkg.error) addSignal(signals, 'package-json-error', 2, `package.json parse error: ${pkg.error}`);

  if (exists(targetRoot, '.git')) addSignal(signals, 'git-repository', 1, '.git');
  if (hasAny(targetRoot, ['.github/workflows', '.gitlab-ci.yml', 'Jenkinsfile', 'circle.yml', '.circleci/config.yml'])) {
    addSignal(signals, 'ci', 2, 'CI configuration detected');
  }
  if (exists(targetRoot, 'pnpm-workspace.yaml') || exists(targetRoot, 'turbo.json') || exists(targetRoot, 'nx.json') || exists(targetRoot, 'lerna.json') || (pkg.data && pkg.data.workspaces)) {
    addSignal(signals, 'monorepo', 3, 'workspace/monorepo signal');
  }

  const appDirs = [
    ...listDirectories(targetRoot, 'apps'),
    ...listDirectories(targetRoot, 'packages'),
    ...listDirectories(targetRoot, 'services')
  ];
  if (appDirs.length >= 3) addSignal(signals, 'multi-package', 2, `${appDirs.length} app/package/service directories`);

  if (hasAny(targetRoot, ['prisma/schema.prisma', 'drizzle.config.ts', 'drizzle.config.js', 'migrations', 'db/migrations', 'database/migrations'])) {
    addSignal(signals, 'database-schema', 4, 'database schema or migrations detected');
  }
  if (deps.some((name) => /prisma|drizzle|sequelize|typeorm|mongoose|knex|pg|mysql|sqlite|redis/i.test(name))) {
    addSignal(signals, 'database-dependencies', 3, 'database dependency detected');
  }
  if (deps.some((name) => /next-auth|auth0|passport|jsonwebtoken|jose|bcrypt|argon2|lucia|clerk|supabase/i.test(name))) {
    addSignal(signals, 'auth-security', 4, 'auth/security dependency detected');
  }
  if (deps.some((name) => /stripe|paypal|braintree|paddle|checkout/i.test(name))) {
    addSignal(signals, 'payment', 5, 'payment dependency detected');
  }
  if (deps.some((name) => /next|vite|react|vue|svelte|astro|@remix-run\//.test(name)) || hasAny(targetRoot, ['vite.config.ts', 'vite.config.js', 'next.config.js', 'next.config.mjs'])) {
    addSignal(signals, 'frontend', 2, 'frontend framework signal');
  }
  if (deps.some((name) => /express|fastify|hono|koa|@nestjs\//.test(name)) || hasAny(targetRoot, ['server.js', 'server.ts', 'src/server.js', 'src/server.ts'])) {
    addSignal(signals, 'backend-api', 3, 'backend/API signal');
  }
  if (hasAny(targetRoot, ['Dockerfile', 'docker-compose.yml', 'docker-compose.yaml', 'compose.yml', 'compose.yaml', 'k8s', 'helm', 'infra', 'terraform'])) {
    addSignal(signals, 'deployment-infra', 3, 'deployment or infrastructure signal');
  }

  const scripts = pkg.data && pkg.data.scripts && typeof pkg.data.scripts === 'object' ? Object.keys(pkg.data.scripts) : [];
  const verificationScripts = scripts.filter((name) => /lint|type|test|spec|e2e|build|check/i.test(name));
  if (verificationScripts.length >= 4) addSignal(signals, 'broad-verification-surface', 2, `${verificationScripts.length} verification-like scripts`);
  else if (verificationScripts.length > 0) addSignal(signals, 'verification-surface', 1, `${verificationScripts.length} verification-like scripts`);

  if (hasAny(targetRoot, ['pyproject.toml', 'requirements.txt', 'go.mod', 'Cargo.toml', 'pom.xml', 'build.gradle', 'build.gradle.kts'])) {
    addSignal(signals, 'non-node-ecosystem', 1, 'non-Node project entrypoint detected');
  }

  return {
    projectKind: kind.kind,
    packageJson: {
      exists: pkg.exists,
      error: pkg.error
    },
    signals
  };
}

function projectPresetRecommendation(targetRoot) {
  const profile = detectProjectSignals(targetRoot);
  const score = profile.signals.reduce((total, signal) => total + signal.weight, 0);
  const signalNames = new Set(profile.signals.map((signal) => signal.name));
  const strictReasons = ['database-schema', 'auth-security', 'payment', 'deployment-infra'];
  const hasStrictReason = strictReasons.some((name) => signalNames.has(name));
  const hasLargeProjectReason = signalNames.has('monorepo') || signalNames.has('multi-package');
  let preset = 'balanced';

  if (profile.projectKind === 'source-package') preset = 'balanced';
  else if (hasStrictReason || (hasLargeProjectReason && (signalNames.has('ci') || signalNames.has('backend-api') || signalNames.has('frontend'))) || score >= 9) preset = 'strict';
  else if (score <= 1 && !signalNames.has('installed-project')) preset = 'advisory';

  return {
    preset,
    score,
    rationale: explainProjectPreset(preset, profile, score),
    profile
  };
}

function explainProjectPreset(preset, profile, score) {
  if (profile.projectKind === 'source-package') return 'Source package context uses balanced so source checks stay useful without installed-project runtime requirements.';
  if (preset === 'strict') return `Project risk score ${score} includes high-impact or large-project signals; strict gates are recommended.`;
  if (preset === 'advisory') return `Project risk score ${score} has very few concrete project signals; advisory mode avoids premature blocking.`;
  return `Project risk score ${score} fits normal project work; balanced gates are recommended.`;
}

const strictTaskPatterns = [
  ['database-schema', /\b(db|database|schema|migration|migrate|prisma|drizzle|sql|data integrity)\b/i],
  ['auth-security', /\b(auth|permission|rbac|security|secret|token|session|login|password|oauth)\b/i],
  ['payment', /\b(payment|billing|checkout|stripe|invoice|subscription)\b/i],
  ['public-api', /\b(api|contract|endpoint|public interface|breaking change)\b/i],
  ['deployment', /\b(deploy|deployment|ci|cd|release|production|infra|docker|kubernetes)\b/i],
  ['multi-agent', /\b(multi-agent|subagent|parallel agent|cross-module|monorepo|large refactor)\b/i],
  ['failed-fix', /\b(regression|failed fix|retry|again|still broken|production bug)\b/i]
];

const standardTaskPatterns = [
  ['feature', /\b(feature|implement|add|create|build|support)\b/i],
  ['bugfix', /\b(bug|fix|error|failure|failing|broken|crash)\b/i],
  ['refactor', /\b(refactor|restructure|cleanup|rename)\b/i],
  ['test-change', /\b(test|spec|coverage|e2e)\b/i],
  ['ui', /\b(ui|frontend|layout|responsive|browser|visual|page|component)\b/i],
  ['behavior', /\b(behavior|logic|flow|state|workflow)\b/i]
];

function taskPathRecommendation(targetRoot, options) {
  const title = options && options.title ? String(options.title) : '';
  const description = options && options.description ? String(options.description) : '';
  const text = `${title}\n${description}`.trim();
  const project = projectPresetRecommendation(targetRoot);
  const evidence = [];
  let pathName = 'fast';

  for (const [name, pattern] of strictTaskPatterns) {
    if (pattern.test(text)) evidence.push({ name, path: 'strict', evidence: `Task text matched ${name}` });
  }
  for (const [name, pattern] of standardTaskPatterns) {
    if (pattern.test(text)) evidence.push({ name, path: 'standard', evidence: `Task text matched ${name}` });
  }

  if (evidence.some((item) => item.path === 'strict')) pathName = 'strict';
  else if (evidence.some((item) => item.path === 'standard')) pathName = 'standard';

  if (pathName === 'fast' && project.preset === 'strict' && /change|update|modify|edit/i.test(text)) {
    pathName = 'standard';
    evidence.push({ name: 'strict-project-change', path: 'standard', evidence: 'Strict project preset plus change-like task text' });
  }

  if (!text) {
    evidence.push({ name: 'no-task-text', path: pathName, evidence: 'No task title or description supplied; defaulting to lightest path until scope is known' });
  }

  return {
    path: pathName,
    projectPreset: project.preset,
    handling: handlingForPath(pathName),
    rationale: explainTaskPath(pathName, evidence, project),
    evidence,
    project
  };
}

function explainTaskPath(pathName, evidence, project) {
  if (pathName === 'strict') return 'Task text includes high-impact or coordination signals that require Strict Path handling.';
  if (pathName === 'standard') return 'Task text includes normal implementation, bugfix, UI, refactor, test, or behavior-change signals.';
  if (project.preset === 'strict') return 'No task-risk keywords were found; project is strict but this task can start Fast until scope expands.';
  return 'No task-risk keywords were found; Fast Path is enough unless investigation expands scope.';
}

function handlingForPath(pathName) {
  if (pathName === 'strict') {
    return [
      'Read Strict Path skills and affected control files before implementation.',
      'Use durable evidence and evaluator acceptance before completion.',
      'Use multi-agent protocol when delegating or crossing module boundaries.'
    ];
  }
  if (pathName === 'standard') {
    return [
      'Read task-specific skills before implementation.',
      'Use systematic debugging for bugs and browser verification for UI.',
      'Run relevant verification before completion.'
    ];
  }
  return [
    'Keep read/write scope narrow.',
    'Inspect changed lines or run the smallest direct check before completion.',
    'Re-route to Standard or Strict if behavior, UI, API, data, or cross-file risk appears.'
  ];
}

module.exports = {
  detectProjectSignals,
  projectPresetRecommendation,
  taskPathRecommendation
};
