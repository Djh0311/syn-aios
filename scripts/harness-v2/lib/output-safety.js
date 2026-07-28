'use strict';

const SECRET_TOKEN = /\b(?:AKIA[0-9A-Z]{16}|gh[pousr]_[A-Za-z0-9]{20,}|xox[baprs]-[A-Za-z0-9-]{20,}|sk-(?:proj-|live-)?[A-Za-z0-9_-]{20,})\b/g;

function sanitizeOutputText(value, maxChars = 240) {
  let text = String(value || '')
    .replace(/\x1B(?:\[[0-?]*[ -/]*[@-~]|\][^\x07]*(?:\x07|\x1B\\)?)/g, '')
    .replace(/[\u0000-\u0008\u000B\u000C\u000E-\u001F\u007F-\u009F]/g, ' ')
    .replace(/\b(?:Bearer|Basic)\s+[A-Za-z0-9._~+/=-]+/gi, '<authorization-redacted>')
    .replace(
      /\b(?:Authorization|Cookie|Set-Cookie)\s*[:=]\s*[^\s,;]+/gi,
      '<authorization-redacted>'
    )
    .replace(SECRET_TOKEN, '<secret-token-redacted>')
    .replace(
      /\b([a-z][a-z0-9+.-]*:\/\/)[^/\s:@]+:[^@\s/]+@/gi,
      '$1<credentials-redacted>@'
    )
    .replace(/\/(?:Users|home|private|tmp|var\/folders)\/[^\s`'"<>]+/g, '<absolute-path-redacted>')
    .replace(/[A-Za-z]:\\[^\s`'"<>]+/g, '<absolute-path-redacted>')
    .replace(
      /\b(?:TOKEN|SECRET|PASSWORD|PASSWD|CREDENTIAL|COOKIE|API_KEY|ACCESS_KEY|PRIVATE_KEY)\s*[:=]\s*(?:"[^"]*"|'[^']*'|[^\s,;]+)/gi,
      '<secret-assignment-redacted>'
    )
    .replace(/\s+/g, ' ')
    .trim();
  if (text.length > maxChars) text = `${text.slice(0, Math.max(0, maxChars - 1))}…`;
  return text;
}

function safeOutputRepoPath(value, maxChars = 240) {
  const raw = String(value || '');
  if (
    !raw ||
    raw.includes('\0') ||
    raw.includes('\\') ||
    raw.startsWith('/') ||
    raw.split('/').includes('..')
  ) {
    return '<unsafe-path-redacted>';
  }
  return sanitizeOutputText(raw, maxChars);
}

module.exports = {
  safeOutputRepoPath,
  sanitizeOutputText,
};
