#!/usr/bin/env node

const fs = require('fs');
const path = require('path');
const { spawnSync } = require('child_process');
const { detectPromptInjection, redactSecrets } = require('./lib/security');
const { appendSecretEscapeHatchAudit, normalizeReason } = require('./lib/evidence-audit');

const runnableKeys = new Set([
  'lint',
  'typecheck',
  'test',
  'testUnit',
  'testIntegration',
  'testE2E',
  'e2e',
  'build'
]);
const strictKeys = new Set(['lint', 'typecheck', 'test', 'build']);
const packageManagers = new Set(['pnpm', 'npm', 'yarn', 'bun']);
const defaultTimeoutMs = 120000;

function parseArgs(argv) {
  const args = {
    target: process.cwd(),
    targetProvided: false,
    slug: null,
    commandKey: null,
    command: null,
    write: false,
    json: false,
    strict: false,
    allowRedactedSecrets: false,
    redactionReason: null,
    timeoutMs: defaultTimeoutMs
  };

  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    if (arg === '--target') {
      args.target = argv[++i];
      args.targetProvided = true;
    } else if (arg === '--slug') args.slug = argv[++i];
    else if (arg === '--command-key') args.commandKey = argv[++i];
    else if (arg === '--command') args.command = argv[++i];
    else if (arg === '--write') args.write = true;
    else if (arg === '--json') args.json = true;
    else if (arg === '--strict') args.strict = true;
    else if (arg === '--allow-redacted-secrets') args.allowRedactedSecrets = true;
    else if (arg === '--redaction-reason') args.redactionReason = argv[++i];
    else if (arg === '--timeout-ms') args.timeoutMs = Number(argv[++i]);
    else throw new Error(`Unknown argument: ${arg}`);
  }

  if (args.write && !args.targetProvided) {
    throw new Error('--write requires an explicit --target installed-project directory');
  }
  if (args.write && !args.slug) {
    throw new Error('--write requires an explicit --slug evidence archive');
  }
  if (!Number.isInteger(args.timeoutMs) || args.timeoutMs <= 0) {
    throw new Error('--timeout-ms must be a positive integer');
  }

  args.target = path.resolve(args.target);
  if (args.commandKey) args.commandKey = String(args.commandKey).trim();
  return args;
}

function add(report, kind, message) {
  report[kind].push(message);
}

function rel(root, filePath) {
  return path.relative(root, filePath).split(path.sep).join('/') || '.';
}

function statSafe(filePath) {
  try {
    return fs.statSync(filePath);
  } catch (error) {
    return null;
  }
}

function readJson(filePath) {
  try {
    return { data: JSON.parse(fs.readFileSync(filePath, 'utf8')), error: null };
  } catch (error) {
    return { data: null, error };
  }
}

function loadConfig(targetRoot) {
  const candidates = [
    path.join(targetRoot, 'harness.config.json'),
    path.join(targetRoot, 'harness.config.example.json')
  ];

  for (const candidate of candidates) {
    if (!fs.existsSync(candidate)) continue;
    const parsed = readJson(candidate);
    return {
      path: candidate,
      data: parsed.data,
      error: parsed.error ? parsed.error.message : null
    };
  }

  return { path: null, data: null, error: null };
}

function loadPackage(targetRoot) {
  const packagePath = path.join(targetRoot, 'package.json');
  if (!fs.existsSync(packagePath)) return { path: packagePath, data: null, error: null, exists: false };
  const parsed = readJson(packagePath);
  return {
    path: packagePath,
    data: parsed.data,
    error: parsed.error ? parsed.error.message : null,
    exists: true
  };
}

function safeSlug(value) {
  const slug = String(value || '').trim();
  if (!slug) throw new Error('--slug is required');
  if (/[\\/]/.test(slug) || slug.includes('..')) {
    throw new Error('Slug must be a single safe path segment without traversal');
  }
  if (!/^[A-Za-z0-9][A-Za-z0-9._-]*$/.test(slug)) {
    throw new Error('Slug may contain only letters, numbers, dots, underscores, and hyphens');
  }
  return slug;
}

function normalizeCommand(value) {
  return typeof value === 'string' ? value.trim() : '';
}

function configCommand(configData, key) {
  if (!configData || !configData.commands || !key) return '';
  return normalizeCommand(configData.commands[key]);
}

function isConcretePackageManager(value) {
  return packageManagers.has(String(value || '').trim());
}

function packageManagerFromPackageField(pkg) {
  if (!pkg || typeof pkg.packageManager !== 'string') return null;
  const name = pkg.packageManager.trim().split('@')[0];
  return isConcretePackageManager(name) ? name : null;
}

function exists(targetRoot, relativePath) {
  return fs.existsSync(path.join(targetRoot, relativePath));
}

function detectPackageManager(targetRoot, configData, pkg) {
  const configured = configData && configData.commands ? configData.commands.packageManager : null;
  if (isConcretePackageManager(configured)) return configured.trim();

  const packageField = packageManagerFromPackageField(pkg);
  if (packageField) return packageField;

  const locks = [
    ['pnpm', 'pnpm-lock.yaml'],
    ['npm', 'package-lock.json'],
    ['yarn', 'yarn.lock'],
    ['bun', 'bun.lockb'],
    ['bun', 'bun.lock']
  ];
  for (const [name, lockfile] of locks) {
    if (exists(targetRoot, lockfile)) return name;
  }

  return 'npm';
}

function scriptCommand(packageManager, scriptName) {
  if (packageManager === 'yarn') return `yarn ${scriptName}`;
  if (packageManager === 'bun') return `bun run ${scriptName}`;
  return `${packageManager} run ${scriptName}`;
}

function scriptNamesForKey(key) {
  if (key === 'testUnit') return ['test:unit', 'unit'];
  if (key === 'testIntegration') return ['test:integration', 'integration'];
  if (key === 'testE2E' || key === 'e2e') return ['test:e2e', 'e2e'];
  if (key === 'typecheck') return ['typecheck', 'type-check', 'tsc'];
  if (key === 'lint') return ['lint'];
  if (key === 'test') return ['test'];
  if (key === 'build') return ['build'];
  return [];
}

function findPackageScript(scripts, key) {
  for (const name of scriptNamesForKey(key)) {
    if (Object.prototype.hasOwnProperty.call(scripts, name)) return name;
  }
  return null;
}

function resolveCommand(args, report, config, pkg) {
  if (args.commandKey === 'dev') {
    add(report, 'fail', 'Dev command key is refused; verification-runner will not run dev commands');
    return null;
  }
  if (args.commandKey && !runnableKeys.has(args.commandKey)) {
    add(report, 'fail', `Unsupported command key: ${args.commandKey}`);
    return null;
  }
  if (args.strict && args.commandKey && !strictKeys.has(args.commandKey)) {
    add(report, 'warn', `Strict mode usually expects one of: ${Array.from(strictKeys).join(', ')}`);
  }

  const directCommand = normalizeCommand(args.command);
  if (directCommand) {
    return {
      command: directCommand,
      source: '--command',
      key: args.commandKey || null
    };
  }

  if (!args.commandKey) {
    add(report, 'fail', 'Provide --command or --command-key');
    return null;
  }

  const configured = configCommand(config.data, args.commandKey);
  if (configured) {
    return {
      command: configured,
      source: `harness config commands.${args.commandKey}`,
      key: args.commandKey
    };
  }

  const packageManager = detectPackageManager(args.target, config.data, pkg.data);
  const scripts = pkg.data && pkg.data.scripts && typeof pkg.data.scripts === 'object' ? pkg.data.scripts : {};
  const scriptName = findPackageScript(scripts, args.commandKey);
  if (scriptName) {
    return {
      command: scriptCommand(packageManager, scriptName),
      source: `package.json scripts.${scriptName}`,
      key: args.commandKey
    };
  }

  const message = `No command found for --command-key ${args.commandKey}`;
  if (!pkg.exists) add(report, 'warn', `${message}; no package.json found`);
  else add(report, args.strict ? 'fail' : 'warn', message);
  return null;
}

function splitCommand(command) {
  const input = String(command || '').trim();
  if (!input) throw new Error('Command is empty');
  if (/[|;&<>`$()]/.test(input)) {
    throw new Error('Command contains shell control syntax');
  }

  const tokens = [];
  let current = '';
  let quote = null;

  for (let i = 0; i < input.length; i += 1) {
    const char = input[i];

    if (quote) {
      if (char === quote) quote = null;
      else current += char;
      continue;
    }

    if (char === '"' || char === "'") {
      quote = char;
      continue;
    }

    if (/\s/.test(char)) {
      if (current) {
        tokens.push(current);
        current = '';
      }
      continue;
    }

    current += char;
  }

  if (quote) throw new Error('Command has an unterminated quote');
  if (current) tokens.push(current);
  if (tokens.length === 0) throw new Error('Command is empty');
  return tokens;
}

function appendSeparator(text) {
  if (!text) return '';
  if (text.endsWith('\n\n')) return '';
  if (text.endsWith('\n')) return '\n';
  return '\n\n';
}

function fence(value) {
  return String(value || '').replace(/```/g, '`` `');
}

function buildEvidenceEntry(report, result, now) {
  const command = report.details.command || 'TBD';
  const resultText = result.exitCode === 0 ? 'exit 0' : `exit ${result.exitCode === null ? 'unknown' : result.exitCode}`;
  const timeoutText = result.timedOut ? 'yes' : 'no';

  const entry = `## Command Evidence

- recordedAt: ${now.toISOString()}
- command: \`${command.replace(/`/g, '\\`')}\`
- result: ${resultText}
- notes: source=${report.details.commandSource || 'unknown'}; cwd=${rel(report.target, report.target)}; timeoutMs=${report.timeoutMs}; timedOut=${timeoutText}; durationMs=${result.durationMs}

### Output

\`\`\`text
$ ${command}

exitCode: ${result.exitCode === null ? 'unknown' : result.exitCode}
signal: ${result.signal || 'none'}
error: ${result.error || 'none'}

--- stdout ---
${fence(result.stdout || '')}

--- stderr ---
${fence(result.stderr || '')}
\`\`\`
`;
  const redaction = redactSecrets(entry);
  const injection = detectPromptInjection(entry);
  report.details.redaction = {
    redacted: redaction.redacted,
    findings: redaction.findings
  };
  report.details.promptInjection = {
    detected: injection.detected,
    findings: injection.findings
  };
  if (redaction.redacted) add(report, 'warn', 'Secret-like content was redacted from verification evidence');
  if (injection.detected) add(report, 'warn', 'Prompt-injection-like content was recorded as untrusted evidence, not instructions');
  return redaction.text;
}

function writeEvidence(report, result, args) {
  const file = report.details.evidenceFile;
  const entry = buildEvidenceEntry(report, result, new Date());
  report.details.evidenceEntry = entry;

  if (report.details.redaction && report.details.redaction.redacted && !args.allowRedactedSecrets) {
    add(report, 'fail', 'Secret-like content detected in verification evidence; refusing to write without --allow-redacted-secrets');
    return;
  }
  if (report.details.redaction && report.details.redaction.redacted && args.allowRedactedSecrets && !normalizeReason(args.redactionReason)) {
    add(report, 'fail', 'Secret-like content detected in verification evidence; --allow-redacted-secrets requires --redaction-reason');
    return;
  }

  if (fs.existsSync(file)) {
    const current = fs.readFileSync(file, 'utf8');
    fs.appendFileSync(file, `${appendSeparator(current)}${entry}`, 'utf8');
    add(report, 'pass', `Appended command evidence to ${report.details.relativeEvidenceFile}`);
  } else {
    fs.writeFileSync(file, `# Test Output: ${report.details.slug}\n\n${entry}`, { encoding: 'utf8', flag: 'wx' });
    add(report, 'pass', `Created command evidence: ${report.details.relativeEvidenceFile}`);
  }
  report.wrote = true;
  if (report.details.redaction && report.details.redaction.redacted && args.allowRedactedSecrets) {
    const audit = appendSecretEscapeHatchAudit({
      target: report.target,
      source: 'verification-runner',
      slug: report.details.slug,
      evidenceFile: report.details.relativeEvidenceFile,
      command: report.details.command || 'TBD',
      reason: args.redactionReason,
      findings: report.details.redaction.findings,
      summary: `verification-runner wrote redacted evidence after explicit override; result exitCode=${result.exitCode === null ? 'unknown' : result.exitCode}.`
    });
    report.details.redactionOverride.audit = audit;
    if (audit.wrote) add(report, 'warn', `Secret evidence escape hatch audited in ${audit.relativeLedgerPath}`);
    else if (audit.warning) add(report, 'warn', audit.warning);
  }
}

function runCommand(report, tokens) {
  const started = Date.now();
  const child = spawnSync(tokens[0], tokens.slice(1), {
    cwd: report.target,
    encoding: 'utf8',
    shell: false,
    timeout: report.timeoutMs,
    maxBuffer: 1024 * 1024 * 10
  });

  const durationMs = Date.now() - started;
  const timedOut = child.error && child.error.code === 'ETIMEDOUT';
  return {
    exitCode: typeof child.status === 'number' ? child.status : null,
    signal: child.signal || null,
    error: child.error ? child.error.message : null,
    timedOut: Boolean(timedOut),
    durationMs,
    stdout: child.stdout || '',
    stderr: child.stderr || ''
  };
}

function buildReport(args) {
  const report = {
    target: args.target,
    mode: args.write ? 'write' : 'dry-run',
    strict: args.strict,
    timeoutMs: args.timeoutMs,
    wrote: false,
    pass: [],
    warn: [],
    fail: [],
    details: {
      slug: null,
      evidenceDir: null,
      evidenceFile: null,
      relativeEvidenceFile: null,
      configPath: null,
      packageJsonPath: null,
      commandKey: args.commandKey || null,
      command: null,
      commandSource: null,
      tokens: [],
      result: null,
      evidenceEntry: null,
      redaction: null,
      promptInjection: null,
      redactionOverride: {
        allowed: args.allowRedactedSecrets,
        reason: normalizeReason(args.redactionReason),
        audit: null
      }
    }
  };

  const targetStat = statSafe(args.target);
  if (!targetStat || !targetStat.isDirectory()) {
    add(report, 'fail', `Target must be an existing directory: ${args.target}`);
    return report;
  }

  if (args.slug) {
    const slug = safeSlug(args.slug);
    const evidenceDir = path.join(args.target, 'docs', 'evidence', slug);
    const evidenceFile = path.join(evidenceDir, 'test-output.md');
    report.details.slug = slug;
    report.details.evidenceDir = evidenceDir;
    report.details.evidenceFile = evidenceFile;
    report.details.relativeEvidenceFile = rel(args.target, evidenceFile);

    if (!fs.existsSync(evidenceDir) || !fs.statSync(evidenceDir).isDirectory()) {
      add(report, 'fail', `Evidence directory must already exist: docs/evidence/${slug}/`);
    }
  } else if (args.write) {
    add(report, 'fail', '--write requires --slug');
  } else {
    add(report, 'warn', 'No --slug provided; dry-run will not plan an evidence file');
  }

  const config = loadConfig(args.target);
  report.details.configPath = config.path ? rel(args.target, config.path) : null;
  if (config.error) add(report, 'fail', `Harness config could not be loaded: ${config.error}`);

  const pkg = loadPackage(args.target);
  report.details.packageJsonPath = pkg.exists ? rel(args.target, pkg.path) : null;
  if (pkg.error) add(report, 'fail', `package.json could not be parsed: ${pkg.error}`);

  const resolved = resolveCommand(args, report, config, pkg);
  if (resolved) {
    report.details.command = resolved.command;
    report.details.commandSource = resolved.source;
    add(report, 'pass', `Command selected from ${resolved.source}: ${resolved.command}`);
    try {
      report.details.tokens = splitCommand(resolved.command);
      add(report, 'pass', `Command is safe for shell:false spawn: ${report.details.tokens[0]}`);
    } catch (error) {
      add(report, 'fail', `Unsafe command refused: ${error.message}`);
    }
  }

  if (report.fail.length > 0) return report;

  if (!resolved && args.write) {
    add(report, 'fail', 'Cannot execute verification because no command was resolved');
    return report;
  }

  if (!args.write) {
    add(report, 'warn', 'PLAN ONLY: dry run did not execute the command and did not write evidence');
    return report;
  }

  const result = runCommand(report, report.details.tokens);
  report.details.result = result;
  if (result.exitCode === 0) add(report, 'pass', 'Verification command exited 0');
  else add(report, 'fail', `Verification command exited ${result.exitCode === null ? 'unknown' : result.exitCode}`);
  if (result.timedOut) add(report, 'fail', `Verification command timed out after ${args.timeoutMs}ms`);

  writeEvidence(report, result, args);
  return report;
}

function printSection(title, items) {
  console.log(`\n${title} (${items.length})`);
  if (items.length === 0) {
    console.log('  None');
    return;
  }
  for (const item of items) console.log(`  - ${item}`);
}

function printDetails(report) {
  console.log('\nDETAILS');
  console.log(`Config: ${report.details.configPath || 'None'}`);
  console.log(`package.json: ${report.details.packageJsonPath || 'None'}`);
  console.log(`Command key: ${report.details.commandKey || 'None'}`);
  console.log(`Command source: ${report.details.commandSource || 'None'}`);
  console.log(`Command: ${report.details.command || 'None'}`);
  console.log(`Tokens: ${report.details.tokens.length > 0 ? report.details.tokens.join(' | ') : 'None'}`);
  console.log(`Evidence file: ${report.details.relativeEvidenceFile || 'None'}`);
  if (report.details.result) {
    console.log(`Exit code: ${report.details.result.exitCode === null ? 'unknown' : report.details.result.exitCode}`);
    console.log(`Duration ms: ${report.details.result.durationMs}`);
  }
}

function printReport(report) {
  console.log(`Harness verification runner: ${report.target}`);
  console.log(`Mode: ${report.mode}`);
  if (report.strict) console.log('Strict: true');
  printSection('PASS', report.pass);
  printSection('WARN', report.warn);
  printSection('FAIL', report.fail);
  printDetails(report);
}

try {
  const args = parseArgs(process.argv.slice(2));
  const report = buildReport(args);
  if (args.json) console.log(JSON.stringify(report, null, 2));
  else printReport(report);
  const commandFailed = report.details.result && report.details.result.exitCode !== 0;
  if (report.fail.length > 0 || commandFailed) process.exit(1);
} catch (error) {
  console.error(`ERROR: ${error.message}`);
  process.exit(1);
}
