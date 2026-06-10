#!/usr/bin/env node

const fs = require('fs');
const os = require('os');
const path = require('path');
const zlib = require('zlib');
const { spawnSync } = require('child_process');
const { loadSkillIndex, recommendSkills } = require('./lib/skill-index');
const { queryMistakes } = require('./lib/mistake-retrieval');
const { buildReport: buildContextPackReport } = require('./lib/context-pack');
const { scanSecurityFindings } = require('./lib/security');
const {
  classifyMemoryTrust,
  isMemoryStale,
  shouldPromoteMemory,
  shouldQuarantineMemory,
  validateMemoryCandidate
} = require('./lib/memory-governance');

const suites = {
  smoke: [
    {
      id: 'security-redaction',
      command: ['security-scan.js', '--target', 'scripts/harness/fixtures/security-redaction', '--file', 'input.txt', '--source', 'web', '--json'],
      metrics: ['security-scan', 'prompt-injection', 'secret-redaction']
    },
    {
      id: 'fixture-check',
      command: ['fixture-check.js', '--json'],
      metrics: ['fixtures']
    },
    {
      id: 'skill-recommend',
      command: ['skill-recommend.js', '--text', 'Fix failing browser UI test before completion', '--json'],
      metrics: ['skill-recommendation']
    },
    {
      id: 'context-pack-source-skip',
      command: ['context-pack.js', '--target', '.', '--json'],
      metrics: ['context-pack']
    },
    {
      id: 'task-package-lint-source-skip',
      command: ['task-package-lint.js', '--target', '.', '--json'],
      metrics: ['task-package-schema']
    }
  ]
};

const securityCaseFiles = [
  'scripts/harness/eval/cases/security/prompt-injection.json'
];
const skillCaseFiles = [
  'scripts/harness/eval/cases/skill-recommend.json'
];
const mistakeCaseFiles = [
  'scripts/harness/eval/cases/mistake-retrieval.json'
];
const contextCaseFiles = [
  'scripts/harness/eval/cases/context-pack.json'
];
const memoryCaseFiles = [
  'scripts/harness/eval/cases/memory-governance.json'
];

function parseArgs(argv) {
  const args = {
    target: process.cwd(),
    suite: 'smoke',
    write: false,
    json: false
  };

  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    if (arg === '--target') args.target = argv[++i];
    else if (arg === '--suite') args.suite = argv[++i];
    else if (arg === '--write') args.write = true;
    else if (arg === '--json') args.json = true;
    else throw new Error(`Unknown argument: ${arg}`);
  }

  args.target = path.resolve(args.target);
  return args;
}

function rel(root, filePath) {
  return path.relative(root, filePath).split(path.sep).join('/') || '.';
}

function runTask(targetRoot, task) {
  const script = path.join(targetRoot, 'scripts', 'harness', task.command[0]);
  const args = task.command.slice(1);
  const started = Date.now();
  const result = spawnSync(process.execPath, [script, ...args], {
    cwd: targetRoot,
    encoding: 'utf8',
    shell: false,
    timeout: 120000,
    maxBuffer: 1024 * 1024 * 10
  });

  const durationMs = Date.now() - started;
  return {
    id: task.id,
    command: `node ${rel(targetRoot, script)} ${args.join(' ')}`.trim(),
    metrics: task.metrics,
    exitCode: typeof result.status === 'number' ? result.status : null,
    durationMs,
    stdoutBytes: Buffer.byteLength(result.stdout || '', 'utf8'),
    stderrBytes: Buffer.byteLength(result.stderr || '', 'utf8'),
    timedOut: Boolean(result.error && result.error.code === 'ETIMEDOUT'),
    error: result.error ? result.error.message : null
  };
}

function loadJson(filePath) {
  return JSON.parse(fs.readFileSync(filePath, 'utf8'));
}

function loadCases(targetRoot, relativePaths) {
  const cases = [];
  for (const relativePath of relativePaths) {
    const filePath = path.join(targetRoot, relativePath);
    if (!fs.existsSync(filePath)) {
      return {
        cases: [],
        error: `Eval case file missing: ${relativePath}`
      };
    }
    for (const testCase of loadJson(filePath)) cases.push(Object.assign({ caseFile: relativePath }, testCase));
  }
  return { cases, error: null };
}

function unique(values) {
  return Array.from(new Set(values.filter(Boolean)));
}

function rankMetrics(results) {
  const perCase = results.map((result) => {
    const expected = unique(result.expected || []);
    const actual = unique(result.actual || []);
    const k = result.k || actual.length || 1;
    const topK = actual.slice(0, k);
    const hits = expected.filter((item) => topK.includes(item));
    const firstRanks = expected
      .map((item) => actual.indexOf(item))
      .filter((index) => index >= 0)
      .map((index) => index + 1);
    const reciprocalRank = firstRanks.length === 0 ? 0 : 1 / Math.min(...firstRanks);
    return Object.assign({}, result, {
      k,
      hits,
      missing: expected.filter((item) => !topK.includes(item)),
      recallAtK: expected.length === 0 ? 1 : hits.length / expected.length,
      precisionAtK: k === 0 ? 1 : hits.length / k,
      reciprocalRank,
      passed: expected.every((item) => topK.includes(item))
    });
  });

  const count = perCase.length || 1;
  const mean = (field) => perCase.reduce((sum, item) => sum + item[field], 0) / count;
  return {
    caseCount: perCase.length,
    pass: perCase.every((item) => item.passed),
    recallAtK: mean('recallAtK'),
    precisionAtK: mean('precisionAtK'),
    mrr: mean('reciprocalRank'),
    failures: perCase.filter((item) => !item.passed),
    results: perCase
  };
}

function scoreBinary(results, field) {
  const counts = {
    truePositive: 0,
    falsePositive: 0,
    trueNegative: 0,
    falseNegative: 0
  };

  for (const result of results) {
    const expected = Boolean(result.expected[field]);
    const actual = Boolean(result.actual[field]);
    if (expected && actual) counts.truePositive += 1;
    else if (!expected && actual) counts.falsePositive += 1;
    else if (!expected && !actual) counts.trueNegative += 1;
    else counts.falseNegative += 1;
  }

  const precisionDenominator = counts.truePositive + counts.falsePositive;
  const recallDenominator = counts.truePositive + counts.falseNegative;
  const precision = precisionDenominator === 0 ? 1 : counts.truePositive / precisionDenominator;
  const recall = recallDenominator === 0 ? 1 : counts.truePositive / recallDenominator;
  const f1 = precision + recall === 0 ? 0 : (2 * precision * recall) / (precision + recall);

  return Object.assign(counts, {
    precision,
    recall,
    f1
  });
}

function scoreExpectedBoolean(results, field) {
  const mapped = results.map((result) => ({
    expected: { value: Boolean(result.expected[field]) },
    actual: { value: Boolean(result.actual[field]) }
  }));
  return scoreBinary(mapped, 'value');
}

function runSecuritySuite(targetRoot) {
  const loaded = loadCases(targetRoot, securityCaseFiles);
  if (loaded.error) {
    return {
      suite: 'security',
      cases: [],
      metrics: {},
      pass: false,
      error: loaded.error
    };
  }

  const cases = expandSecurityCases(loaded.cases);
  const results = cases.map((testCase) => {
    const scan = scanSecurityFindings(testCase.text, {
      source: testCase.source,
      path: testCase.path || null,
      url: testCase.url || null
    });
    const actual = {
      promptInjection: scan.promptInjectionDetected,
      secret: scan.redacted
    };
    return {
      id: testCase.id,
      source: testCase.source,
      expected: testCase.expected,
      actual,
      risk: scan.risk,
      findings: scan.findings.map((finding) => ({
        type: finding.type,
        name: finding.name || null,
        encoding: finding.encoding || null,
        match: finding.match || null
      })),
      passed: actual.promptInjection === Boolean(testCase.expected.promptInjection)
        && actual.secret === Boolean(testCase.expected.secret)
    };
  });

  const metrics = {
    promptInjection: scoreBinary(results, 'promptInjection'),
    secret: scoreBinary(results, 'secret')
  };
  const failures = results.filter((result) => !result.passed);
  const pass = failures.length === 0
    && metrics.promptInjection.recall === 1
    && metrics.secret.recall === 1;

  return {
    suite: 'security',
    caseCount: results.length,
    pass,
    metrics,
    failures,
    results
  };
}

function leetVariant(text) {
  return String(text || '').replace(/[ioeast]/gi, (char) => ({
    i: '1',
    I: '1',
    o: '0',
    O: '0',
    e: '3',
    E: '3',
    a: '4',
    A: '4',
    s: '5',
    S: '5',
    t: '7',
    T: '7'
  })[char] || char);
}

function homoglyphVariant(text) {
  return String(text || '').replace(/[ioeapcxy]/gi, (char) => ({
    i: 'і',
    I: 'І',
    o: 'о',
    O: 'О',
    e: 'е',
    E: 'Е',
    a: 'а',
    A: 'А',
    p: 'р',
    P: 'Р',
    c: 'с',
    C: 'С',
    x: 'х',
    X: 'Х',
    y: 'у',
    Y: 'У'
  })[char] || char);
}

function generatedSecurityVariants(text) {
  const source = String(text || '');
  const variants = [
    ['url', encodeURIComponent(source)],
    ['hex', Buffer.from(source, 'utf8').toString('hex')],
    ['base64', Buffer.from(source, 'utf8').toString('base64')],
    ['gzip-base64', zlib.gzipSync(Buffer.from(source, 'utf8')).toString('base64')],
    ['leetspeak', leetVariant(source)],
    ['homoglyph', homoglyphVariant(source)]
  ];
  return variants.filter(([, value]) => value && value !== source);
}

function shouldGenerateSecurityVariants(testCase) {
  if (testCase.generateVariants === false) return false;
  if (!testCase.expected || !testCase.expected.promptInjection || testCase.expected.secret) return false;
  if (testCase.variantOf || testCase.generatedFrom || testCase.preEncoded) return false;
  return true;
}

function expandSecurityCases(cases) {
  const expanded = [];
  const positiveInjection = cases.filter(shouldGenerateSecurityVariants);
  for (const testCase of cases) expanded.push(testCase);
  for (const testCase of positiveInjection) {
    for (const [encoding, text] of generatedSecurityVariants(testCase.text)) {
      expanded.push(Object.assign({}, testCase, {
        id: `${testCase.id}-${encoding}`,
        text,
        generatedFrom: testCase.id,
        generatedEncoding: encoding
      }));
    }
  }
  return expanded;
}

function runSkillSuite(targetRoot) {
  const loaded = loadCases(targetRoot, skillCaseFiles);
  if (loaded.error) return { suite: 'skill', pass: false, error: loaded.error, caseCount: 0, metrics: {}, failures: [], results: [] };
  const skills = loadSkillIndex(path.join(targetRoot, 'skills'));
  const results = loaded.cases.map((testCase) => {
    const recommendations = recommendSkills(skills, testCase.query, { limit: testCase.k || 8 });
    return {
      id: testCase.id,
      query: testCase.query,
      expected: testCase.expected,
      actual: recommendations.map((skill) => skill.name),
      k: testCase.k || 8
    };
  });
  const metrics = rankMetrics(results);
  return Object.assign({ suite: 'skill', metrics }, metrics);
}

function writeTempLedger(targetRoot, ledgerText) {
  const tempRoot = fs.mkdtempSync(path.join(os.tmpdir(), 'harness-eval-mistake-'));
  fs.mkdirSync(path.join(tempRoot, 'docs'), { recursive: true });
  fs.writeFileSync(path.join(tempRoot, 'docs/agent-mistake-ledger.md'), ledgerText, 'utf8');
  return tempRoot;
}

function runMistakeSuite(targetRoot) {
  const loaded = loadCases(targetRoot, mistakeCaseFiles);
  if (loaded.error) return { suite: 'mistake', pass: false, error: loaded.error, caseCount: 0, metrics: {}, failures: [], results: [] };
  const results = [];
  for (const testCase of loaded.cases) {
    const tempRoot = writeTempLedger(targetRoot, testCase.ledger || '');
    try {
      const related = queryMistakes(tempRoot, testCase.query || {}, { limit: testCase.k || 3 });
      results.push({
        id: testCase.id,
        query: testCase.query,
        expected: testCase.expected,
        actual: related.matches.map((entry) => entry.id),
        k: testCase.k || 3,
        matches: related.matches
      });
    } finally {
      fs.rmSync(tempRoot, { recursive: true, force: true });
    }
  }
  const metrics = rankMetrics(results);
  return Object.assign({ suite: 'mistake', metrics }, metrics);
}

function runContextSuite(targetRoot) {
  const loaded = loadCases(targetRoot, contextCaseFiles);
  if (loaded.error) return { suite: 'context', pass: false, error: loaded.error, caseCount: 0, metrics: {}, failures: [], results: [] };
  const fixtureTarget = path.join(targetRoot, 'scripts/harness/fixtures/context-pack-runtime-docs');
  const results = loaded.cases.map((testCase) => {
    const context = buildContextPackReport({
      target: fixtureTarget,
      taskId: testCase.taskId,
      slug: testCase.slug,
      aliases: testCase.aliases || []
    });
    const snippets = context.details && Array.isArray(context.details.snippets) ? context.details.snippets : [];
    const actualFiles = snippets.map((snippet) => snippet.file);
    const actualTerms = snippets.map((snippet) => snippet.value);
    const expected = unique((testCase.expectedFiles || []).concat(testCase.expectedTerms || []));
    const actual = unique(actualFiles.concat(actualTerms));
    return {
      id: testCase.id,
      expected,
      actual,
      k: testCase.k || 12,
      snippetCount: snippets.length,
      snippets
    };
  });
  const metrics = rankMetrics(results);
  return Object.assign({ suite: 'context', metrics }, metrics);
}

function runMemorySuite(targetRoot) {
  const loaded = loadCases(targetRoot, memoryCaseFiles);
  if (loaded.error) return { suite: 'memory', pass: false, error: loaded.error, caseCount: 0, metrics: {}, failures: [], results: [] };
  const defaultProjectContext = {
    targetRoot,
    staleAfterDays: 30,
    now: '2026-05-20T00:00:00.000Z',
    authorityText: fs.existsSync(path.join(targetRoot, 'AGENTS.md')) ? fs.readFileSync(path.join(targetRoot, 'AGENTS.md'), 'utf8') : ''
  };
  const results = loaded.cases.map((testCase) => {
    const projectContext = Object.assign({}, defaultProjectContext, testCase.projectContext || {});
    const validation = validateMemoryCandidate(testCase.candidate, { projectContext });
    const classification = classifyMemoryTrust(testCase.candidate, projectContext);
    const promote = shouldPromoteMemory(testCase.candidate, projectContext);
    const quarantine = shouldQuarantineMemory(testCase.candidate, projectContext);
    const stale = isMemoryStale(testCase.candidate, projectContext);
    const actual = {
      promote: promote.promote,
      quarantine: quarantine.quarantine,
      stale: stale.stale,
      trustedForContext: classification.trustedForContext,
      recommendedStatus: classification.recommendedStatus
    };
    const expected = testCase.expected || {};
    const passed = actual.promote === Boolean(expected.promote)
      && actual.quarantine === Boolean(expected.quarantine)
      && actual.stale === Boolean(expected.stale)
      && actual.trustedForContext === Boolean(expected.trustedForContext)
      && actual.recommendedStatus === expected.recommendedStatus;
    return {
      id: testCase.id,
      expected,
      actual,
      valid: validation.valid,
      errors: validation.errors,
      warnings: validation.warnings,
      reasons: {
        promote: promote.reasons,
        quarantine: quarantine.reasons,
        stale: stale.reasons,
        classification: classification.reasons
      },
      passed
    };
  });

  const failures = results.filter((result) => !result.passed);
  const metrics = {
    promote: scoreExpectedBoolean(results, 'promote'),
    quarantine: scoreExpectedBoolean(results, 'quarantine'),
    stale: scoreExpectedBoolean(results, 'stale'),
    trustedForContext: scoreExpectedBoolean(results, 'trustedForContext'),
    falseAuthorityRate: results.length === 0
      ? 0
      : results.filter((result) => result.actual.trustedForContext && !result.expected.trustedForContext).length / results.length
  };
  const pass = failures.length === 0
    && metrics.promote.f1 === 1
    && metrics.quarantine.f1 === 1
    && metrics.stale.f1 === 1
    && metrics.falseAuthorityRate === 0;

  return {
    suite: 'memory',
    caseCount: results.length,
    pass,
    metrics,
    failures,
    results
  };
}

function buildMarkdown(report) {
  const lines = [
    '# Harness Eval Report',
    '',
    `- target: ${report.target}`,
    `- suite: ${report.suite}`,
    `- generatedAt: ${report.generatedAt}`,
    `- pass: ${report.summary.pass}`,
    `- fail: ${report.summary.fail}`,
    `- durationMs: ${report.summary.durationMs}`,
    '',
    '## Tasks',
    ''
  ];

  for (const task of report.tasks) {
    lines.push(`- ${task.id}: exit ${task.exitCode === null ? 'unknown' : task.exitCode}; durationMs=${task.durationMs}; metrics=${task.metrics.join(', ')}`);
  }

  if (report.security && report.security.metrics) {
    lines.push('');
    lines.push('## Security Metrics');
    for (const [name, metric] of Object.entries(report.security.metrics)) {
      lines.push(`- ${name}: precision=${metric.precision.toFixed(3)}; recall=${metric.recall.toFixed(3)}; f1=${metric.f1.toFixed(3)}; tp=${metric.truePositive}; fp=${metric.falsePositive}; fn=${metric.falseNegative}; tn=${metric.trueNegative}`);
    }
    if (report.security.failures && report.security.failures.length > 0) {
      lines.push('');
      lines.push('## Security Failures');
      for (const failure of report.security.failures) {
        lines.push(`- ${failure.id}: expected ${JSON.stringify(failure.expected)} actual ${JSON.stringify(failure.actual)}`);
      }
    }
  }

  const rankingSuites = [
    ['Skill Metrics', report.skill],
    ['Mistake Metrics', report.mistake],
    ['Context Metrics', report.context]
  ];
  for (const [title, suiteReport] of rankingSuites) {
    if (!suiteReport || !suiteReport.metrics) continue;
    lines.push('');
    lines.push(`## ${title}`);
    lines.push(`- recallAtK=${suiteReport.metrics.recallAtK.toFixed(3)}; precisionAtK=${suiteReport.metrics.precisionAtK.toFixed(3)}; mrr=${suiteReport.metrics.mrr.toFixed(3)}; cases=${suiteReport.caseCount}`);
    if (suiteReport.failures && suiteReport.failures.length > 0) {
      for (const failure of suiteReport.failures) {
        lines.push(`- failure ${failure.id}: missing ${failure.missing.join(', ') || 'none'}`);
      }
    }
  }

  if (report.memory && report.memory.metrics) {
    lines.push('');
    lines.push('## Memory Governance Metrics');
    for (const name of ['promote', 'quarantine', 'stale', 'trustedForContext']) {
      const metric = report.memory.metrics[name];
      if (!metric) continue;
      lines.push(`- ${name}: precision=${metric.precision.toFixed(3)}; recall=${metric.recall.toFixed(3)}; f1=${metric.f1.toFixed(3)}; tp=${metric.truePositive}; fp=${metric.falsePositive}; fn=${metric.falseNegative}; tn=${metric.trueNegative}`);
    }
    lines.push(`- falseAuthorityRate=${report.memory.metrics.falseAuthorityRate.toFixed(3)}; cases=${report.memory.caseCount}`);
    if (report.memory.failures && report.memory.failures.length > 0) {
      for (const failure of report.memory.failures) {
        lines.push(`- failure ${failure.id}: expected ${JSON.stringify(failure.expected)} actual ${JSON.stringify(failure.actual)}`);
      }
    }
  }

  lines.push('');
  lines.push('## Notes');
  lines.push('- This eval runner records harness command outcomes and instrumentation metrics; it does not claim real-world false-completion-rate reduction by itself.');
  if (report.security) {
    lines.push('- Security suite reports fixture-level recall, precision, and F1 for prompt-injection and secret detection; it is still a regression harness, not a proof of adversarial robustness.');
  }
  if (report.skill || report.mistake || report.context) {
    lines.push('- Ranking suites report recall@k, precision@k, and MRR for fixture expectations; they measure harness regressions, not universal retrieval quality.');
  }
  if (report.memory) {
    lines.push('- Memory governance suite measures promotion, quarantine, staleness, and false-authority regression fixtures; it does not prove all memory poisoning attacks are caught.');
  }
  return `${lines.join('\n')}\n`;
}

function assignSuiteReport(report, name, suiteReport, started) {
  report[name] = suiteReport;
  report.summary.taskCount = suiteReport.caseCount || 0;
  report.summary.pass = suiteReport.pass ? 1 : 0;
  report.summary.fail = suiteReport.pass ? 0 : 1;
  report.summary.durationMs = Date.now() - started;
  if (suiteReport.pass) report.pass.push(`${name} eval suite passed`);
  else report.fail.push(suiteReport.error || `${name} eval suite failed case(s): ${(suiteReport.failures || []).length}`);
  report.markdown = buildMarkdown(report);
}

function buildReport(args) {
  const suite = suites[args.suite];
  const report = {
    target: args.target,
    suite: args.suite,
    generatedAt: new Date().toISOString(),
    pass: [],
    warn: [],
    fail: [],
    summary: {
      taskCount: 0,
      pass: 0,
      fail: 0,
      durationMs: 0
    },
    tasks: [],
    files: {}
  };

  if (!fs.existsSync(args.target) || !fs.statSync(args.target).isDirectory()) {
    report.fail.push(`Target must be an existing directory: ${args.target}`);
    report.markdown = buildMarkdown(report);
    return report;
  }

  const caseSuites = new Set(['security', 'skill', 'mistake', 'context', 'memory']);
  if (!caseSuites.has(args.suite) && !suite) {
    report.fail.push(`Unknown eval suite: ${args.suite}`);
    report.warn.push(`Available suites: ${Object.keys(suites).concat(Array.from(caseSuites)).join(', ')}`);
    report.markdown = buildMarkdown(report);
    return report;
  }

  if (args.suite === 'security') {
    const started = Date.now();
    assignSuiteReport(report, 'security', runSecuritySuite(args.target), started);
  } else if (args.suite === 'skill') {
    const started = Date.now();
    assignSuiteReport(report, 'skill', runSkillSuite(args.target), started);
  } else if (args.suite === 'mistake') {
    const started = Date.now();
    assignSuiteReport(report, 'mistake', runMistakeSuite(args.target), started);
  } else if (args.suite === 'context') {
    const started = Date.now();
    assignSuiteReport(report, 'context', runContextSuite(args.target), started);
  } else if (args.suite === 'memory') {
    const started = Date.now();
    assignSuiteReport(report, 'memory', runMemorySuite(args.target), started);
  } else {
    report.tasks = suite.map((task) => runTask(args.target, task));
    report.summary.taskCount = report.tasks.length;
    report.summary.pass = report.tasks.filter((task) => task.exitCode === 0).length;
    report.summary.fail = report.tasks.length - report.summary.pass;
    report.summary.durationMs = report.tasks.reduce((sum, task) => sum + task.durationMs, 0);

    if (report.summary.fail === 0) report.pass.push(`Eval suite passed: ${args.suite}`);
    else report.fail.push(`Eval suite had failing task(s): ${report.summary.fail}`);

    report.markdown = buildMarkdown(report);
  }

  if (args.write) {
    const reportDir = path.join(args.target, 'reports', 'eval');
    fs.mkdirSync(reportDir, { recursive: true });
    const jsonPath = path.join(reportDir, 'latest.json');
    const mdPath = path.join(reportDir, 'latest.md');
    const serializable = Object.assign({}, report);
    delete serializable.markdown;
    fs.writeFileSync(jsonPath, `${JSON.stringify(serializable, null, 2)}\n`, 'utf8');
    fs.writeFileSync(mdPath, report.markdown, 'utf8');
    report.files.json = rel(args.target, jsonPath);
    report.files.markdown = rel(args.target, mdPath);
  }

  return report;
}

function printSection(title, items) {
  console.log(`\n${title} (${items.length})`);
  if (items.length === 0) console.log('  None');
  else for (const item of items) console.log(`  - ${item}`);
}

function printReport(report) {
  console.log(report.markdown);
  printSection('PASS', report.pass);
  printSection('WARN', report.warn);
  printSection('FAIL', report.fail);
}

try {
  const args = parseArgs(process.argv.slice(2));
  const report = buildReport(args);
  if (args.json) console.log(JSON.stringify(report, null, 2));
  else printReport(report);
  if (report.fail.length > 0) process.exit(1);
} catch (error) {
  console.error(`ERROR: ${error.message}`);
  process.exit(1);
}
