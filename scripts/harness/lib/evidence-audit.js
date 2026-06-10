const fs = require('fs');
const path = require('path');
const { redactSecrets } = require('./security');

const ledgerRelativePath = path.join('docs', 'agent-mistake-ledger.md');

function appendSeparator(text) {
  if (!text) return '';
  if (text.endsWith('\n\n')) return '';
  if (text.endsWith('\n')) return '\n';
  return '\n\n';
}

function fence(value) {
  return String(value || '').replace(/```/g, '`` `');
}

function relativePath(root, filePath) {
  return path.relative(root, filePath).split(path.sep).join('/') || '.';
}

function normalizeReason(reason) {
  const value = String(reason || '').trim();
  if (!value) return '';
  return redactSecrets(value).text;
}

function appendSecretEscapeHatchAudit(options) {
  const target = path.resolve(options.target);
  const ledgerPath = path.join(target, ledgerRelativePath);
  const result = {
    attempted: true,
    wrote: false,
    ledgerPath,
    relativeLedgerPath: relativePath(target, ledgerPath),
    reason: normalizeReason(options.reason),
    warning: null
  };

  if (!fs.existsSync(ledgerPath)) {
    result.warning = `Mistake ledger not found; audit entry was not written: ${ledgerRelativePath}`;
    return result;
  }

  const stat = fs.statSync(ledgerPath);
  if (!stat.isFile()) {
    result.warning = `Mistake ledger is not a file; audit entry was not written: ${ledgerRelativePath}`;
    return result;
  }

  const now = new Date().toISOString();
  const findings = Array.isArray(options.findings) ? options.findings : [];
  const findingNames = findings
    .map((finding) => finding && finding.name ? finding.name : finding && finding.type ? finding.type : null)
    .filter(Boolean)
    .join(', ') || 'secret-like content';
  const command = String(options.command || 'TBD');
  const slug = String(options.slug || 'TBD');
  const evidenceFile = String(options.evidenceFile || 'TBD');

  const entry = `## Secret Evidence Escape Hatch - ${now}

- recordedAt: ${now}
- source: ${options.source || 'evidence-writer'}
- evidenceSlug: ${slug}
- evidenceFile: ${evidenceFile}
- command: \`${command.replace(/`/g, '\\`')}\`
- redactionFindings: ${findingNames}
- userReason: ${result.reason || 'TBD'}

### Root Cause

The user explicitly allowed writing evidence after secret-like content was detected and redacted.

### Evidence

\`\`\`text
${fence(options.summary || 'Secret-like content was redacted before evidence was written.')}
\`\`\`

### Prevention

Keep secret redaction as a hard write gate. Use \`--allow-redacted-secrets --redaction-reason <reason>\` only when a human explicitly accepts the audit trail.
`;

  const current = fs.readFileSync(ledgerPath, 'utf8');
  fs.appendFileSync(ledgerPath, `${appendSeparator(current)}${entry}`, 'utf8');
  result.wrote = true;
  return result;
}

module.exports = {
  appendSecretEscapeHatchAudit,
  ledgerRelativePath,
  normalizeReason
};
