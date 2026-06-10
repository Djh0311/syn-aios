const zlib = require('zlib');

const secretPatterns = [
  {
    name: 'github-token',
    pattern: /\bgh[pousr]_[A-Za-z0-9_]{20,}\b/g,
    replacement: '[REDACTED:github-token]'
  },
  {
    name: 'openai-api-key',
    pattern: /\bsk-[A-Za-z0-9_-]{20,}\b/g,
    replacement: '[REDACTED:openai-api-key]'
  },
  {
    name: 'aws-access-key',
    pattern: /\bAKIA[0-9A-Z]{16}\b/g,
    replacement: '[REDACTED:aws-access-key]'
  },
  {
    name: 'jwt',
    pattern: /\beyJ[A-Za-z0-9_-]{8,}\.[A-Za-z0-9_-]{8,}\.[A-Za-z0-9_-]{8,}\b/g,
    replacement: '[REDACTED:jwt]'
  },
  {
    name: 'stripe-secret-key',
    pattern: /\b(?:sk|rk)_(?:live|test)_[A-Za-z0-9]{16,}\b/g,
    replacement: '[REDACTED:stripe-secret-key]'
  },
  {
    name: 'slack-token',
    pattern: /\bxox[baprs]-[A-Za-z0-9-]{10,}\b/g,
    replacement: '[REDACTED:slack-token]'
  },
  {
    name: 'google-api-key',
    pattern: /\bAIza[0-9A-Za-z_-]{30,}\b/g,
    replacement: '[REDACTED:google-api-key]'
  },
  {
    name: 'npm-token',
    pattern: /\bnpm_[A-Za-z0-9]{30,}\b/g,
    replacement: '[REDACTED:npm-token]'
  },
  {
    name: 'private-key-block',
    pattern: /-----BEGIN [A-Z ]*PRIVATE KEY-----[\s\S]*?-----END [A-Z ]*PRIVATE KEY-----/g,
    replacement: '[REDACTED:private-key-block]'
  },
  {
    name: 'json-private-key',
    pattern: /"private_key"\s*:\s*"[^"]{20,}"/g,
    replacement: '"private_key":"[REDACTED:json-private-key]"'
  },
  {
    name: 'generic-secret-assignment',
    pattern: /\b(api[_-]?key|secret|token|password)\s*[:=]\s*["']?[^"'\s]{12,}["']?/gi,
    replacement: '$1=[REDACTED:secret]'
  }
];

const promptInjectionPatterns = [
  /ignore (all )?(previous|prior|above) (instructions|rules|messages)/i,
  /ign[o0]re (all )?(previous|prior|above) (instructions|rules|messages)/i,
  /disregard (all )?(previous|prior|above) (instructions|rules|messages)/i,
  /reveal (your )?(system|developer) (prompt|message|instructions)/i,
  /(?:print|show|dump|display).{0,80}(?:system|developer).{0,40}(?:prompt|message|instructions)/i,
  /exfiltrate|send .*secret|upload .*secret/i,
  /you are now (in|running) developer mode/i,
  /override (the )?(safety|security|policy|protocol)/i,
  /忽(?:略|视)|无视|不要遵守|覆盖.*(?:指令|规则|协议)|公开.*(?:系统提示|开发者消息)/i,
  /泄露|透露.*(?:系统提示|开发者消息|指令)/i,
  /(?:ignorar|ignora|ignore|ignorer|ignorez|ignori|ignoriere|ignora).{0,80}(?:instrucciones|istruzioni|instructions|consignes|anweisungen|regras|instru[cç][oõ]es)/i,
  /(?:revela|revele|r[eé]v[eè]le|rivela|enth[uü]lle).{0,80}(?:sistema|syst[eè]me|system|sistema|prompt|nachricht)/i,
  /(?:ignoriere|ignoriere alle|missachte).{0,80}(?:anweisungen|regeln|vorgaben)/i,
  /(?:игнорируй|проигнорируй).*(?:инструкции|правила)/i,
  /(?:раскрой|покажи).{0,80}(?:системн|промпт|сообщени)/i,
  /(?:無視|무시).{0,80}(?:指示|命令|지시|규칙)|(?:指示|命令|지시|규칙).{0,80}(?:無視|무시)/i,
  /(?:公開|開示|공개|노출).{0,80}(?:システム|시스템|プロンプト|프롬프트)/i,
  /(?:تجاهل).{0,80}(?:التعليمات|القواعد)/i,
  /(?:اكشف|أظهر).{0,80}(?:النظام|رسالة|التعليمات)/i,
  /(?:^|\n)\s*(?:tool|function)[ _-]?(?:result|output)\s*:/i,
  /(?:^|\n)\s*observation\s*:/i,
  /begin tool output/i,
  /(?:urgent|supervisor requires|admin requires|must comply).{0,120}(?:bypass|override|ignore|reveal|exfiltrate|policy|protocol|secret)/i
];

function benignPromptContext(value, match) {
  const text = String(value || '');
  const index = typeof match.index === 'number' ? match.index : text.indexOf(match[0]);
  const start = Math.max(0, index - 100);
  const end = Math.min(text.length, index + match[0].length + 100);
  const window = text.slice(start, end).toLowerCase();

  if (/\b(?:not|never|do not|don't)\s+(?:to\s+)?(?:follow|obey|execute|ask|asks|asking)\b/.test(window)) return true;
  if (/\b(?:does not|doesn't)\s+(?:ask|tell|instruct|request)\b/.test(window)) return true;
  if (/\b(?:documentation|docs|guide|training|defense|defence)\b/.test(window)
    && /\b(?:explains|example|quote|quoted|describes|mentions)\b/.test(window)) return true;
  if (/\b(?:test name|fixture name|function name|parser handles)\b/.test(window)) return true;
  return false;
}

function stripControlObfuscation(text) {
  return String(text || '').normalize('NFKC').replace(/[\u200B-\u200D\uFEFF]/g, '');
}

function plausibleDecodedText(text) {
  const value = stripControlObfuscation(text);
  if (value.trim().length < 8) return false;
  if (value.includes('\uFFFD')) return false;
  const controlMatches = value.match(/[\x00-\x08\x0b\x0c\x0e-\x1f\x7f]/g) || [];
  return controlMatches.length <= Math.max(1, Math.floor(value.length * 0.02));
}

function rot13(text) {
  return String(text || '').replace(/[A-Za-z]/g, (char) => {
    const base = char <= 'Z' ? 65 : 97;
    return String.fromCharCode(((char.charCodeAt(0) - base + 13) % 26) + base);
  });
}

function decodeBase64Candidates(text) {
  const candidates = [];
  const matches = String(text || '').match(/\b[A-Za-z0-9+/]{24,}={0,2}\b/g) || [];
  for (const match of matches.slice(0, 12)) {
    try {
      const decoded = stripControlObfuscation(Buffer.from(match, 'base64').toString('utf8'));
      if (plausibleDecodedText(decoded)) {
        candidates.push(decoded);
      }
    } catch (error) {
      // Ignore non-base64-looking values that Buffer tolerated poorly.
    }
  }
  return candidates;
}

function decodeUrlCandidates(text) {
  const candidates = [];
  const input = String(text || '');
  if (/%[0-9A-Fa-f]{2}/.test(input)) {
    try {
      const decoded = stripControlObfuscation(decodeURIComponent(input.replace(/\+/g, ' ')));
      if (plausibleDecodedText(decoded)) candidates.push(decoded);
    } catch (error) {
      // Keep looking for smaller URL-encoded spans below.
    }
  }
  const matches = input.match(/\b(?:[A-Za-z0-9._~-]|%[0-9A-Fa-f]{2})*%[0-9A-Fa-f]{2}(?:[A-Za-z0-9._~-]|%[0-9A-Fa-f]{2})*\b/g) || [];
  for (const match of matches.slice(0, 12)) {
    try {
      const decoded = stripControlObfuscation(decodeURIComponent(match));
      if (plausibleDecodedText(decoded)) candidates.push(decoded);
    } catch (error) {
      // Ignore malformed percent-encoding.
    }
  }
  return candidates;
}

function decodeHtmlEntities(text) {
  return String(text || '')
    .replace(/&#x([0-9a-f]+);/gi, (_, hex) => String.fromCodePoint(Number.parseInt(hex, 16)))
    .replace(/&#([0-9]+);/g, (_, decimal) => String.fromCodePoint(Number.parseInt(decimal, 10)))
    .replace(/&(amp|lt|gt|quot|apos);/gi, (_, name) => {
      const entities = { amp: '&', lt: '<', gt: '>', quot: '"', apos: "'" };
      return entities[String(name).toLowerCase()] || _;
    });
}

function decodeHexCandidates(text) {
  const candidates = [];
  const matches = String(text || '').match(/\b(?:[0-9a-fA-F]{2}){12,}\b/g) || [];
  for (const match of matches.slice(0, 12)) {
    try {
      const decoded = stripControlObfuscation(Buffer.from(match, 'hex').toString('utf8'));
      if (plausibleDecodedText(decoded)) {
        candidates.push(decoded);
      }
    } catch (error) {
      // Ignore non-text hex-looking values.
    }
  }
  return candidates;
}

function decodeGzipBase64Candidates(text) {
  const candidates = [];
  const matches = String(text || '').match(/\bH4sI[A-Za-z0-9+/]{20,}={0,2}\b/g) || [];
  for (const match of matches.slice(0, 6)) {
    try {
      const input = Buffer.from(match, 'base64');
      if (input.length > 4096) continue;
      const decoded = zlib.gunzipSync(input, { finishFlush: zlib.constants.Z_SYNC_FLUSH });
      if (decoded.length > 8192) continue;
      const textValue = stripControlObfuscation(decoded.toString('utf8'));
      if (plausibleDecodedText(textValue)) candidates.push(textValue);
    } catch (error) {
      // Ignore malformed or oversized gzip/base64 candidates.
    }
  }
  return candidates;
}

function decodeLeetspeak(text) {
  return String(text || '').replace(/[013457@]/g, (char) => ({
    0: 'o',
    1: 'i',
    3: 'e',
    4: 'a',
    5: 's',
    7: 't',
    '@': 'a'
  })[char] || char);
}

function normalizeHomoglyphs(text) {
  return String(text || '').replace(/[іІоОеЕаАрРсСхХуУ]/g, (char) => ({
    'і': 'i',
    'І': 'I',
    'о': 'o',
    'О': 'O',
    'е': 'e',
    'Е': 'E',
    'а': 'a',
    'А': 'A',
    'р': 'p',
    'Р': 'P',
    'с': 'c',
    'С': 'C',
    'х': 'x',
    'Х': 'X',
    'у': 'y',
    'У': 'Y'
  })[char] || char);
}

function securityVariants(text) {
  const normalized = stripControlObfuscation(text);
  const variants = [
    { encoding: 'plain', value: normalized },
    { encoding: 'html-entities', value: decodeHtmlEntities(normalized) },
    { encoding: 'homoglyph', value: normalizeHomoglyphs(normalized) },
    { encoding: 'leetspeak', value: decodeLeetspeak(normalized) },
    { encoding: 'rot13', value: rot13(normalized) },
    ...decodeUrlCandidates(normalized).map((value) => ({ encoding: 'url', value })),
    ...decodeHexCandidates(normalized).map((value) => ({ encoding: 'hex', value })),
    ...decodeBase64Candidates(normalized).map((value) => ({ encoding: 'base64', value })),
    ...decodeGzipBase64Candidates(normalized).map((value) => ({ encoding: 'gzip-base64', value }))
  ];
  const seen = new Set();
  return variants.filter((variant) => {
    const key = `${variant.encoding}:${variant.value}`;
    if (seen.has(key)) return false;
    seen.add(key);
    return variant.value && variant.value.trim().length > 0;
  });
}

function classifyInputTrust(input) {
  const source = String(input && input.source ? input.source : '').toLowerCase();
  const url = String(input && input.url ? input.url : '');
  const filePath = String(input && input.path ? input.path : '');

  if (source === 'user' || source === 'project-protocol' || source === 'local-protocol') {
    return {
      trust: 'trusted',
      reason: `Trusted source: ${source}`
    };
  }

  if (/^https?:\/\//i.test(url) || source === 'web' || source === 'issue' || source === 'pull-request') {
    return {
      trust: 'untrusted',
      reason: 'External web, issue, or pull-request content is untrusted input'
    };
  }

  if (filePath && /(^|\/)(README|AGENTS|SKILL)\.md$/i.test(filePath)) {
    return {
      trust: 'project-controlled',
      reason: 'Project-controlled local file'
    };
  }

  return {
    trust: 'unknown',
    reason: 'No explicit trust source was provided'
  };
}

function redactSecrets(text) {
  let redacted = stripControlObfuscation(text);
  const findings = [];

  for (const rule of secretPatterns) {
    let count = 0;
    redacted = redacted.replace(rule.pattern, () => {
      count += 1;
      return rule.replacement;
    });
    if (count > 0) findings.push({ type: 'secret', name: rule.name, count });
  }

  return {
    text: redacted,
    findings,
    redacted: findings.length > 0
  };
}

function detectPromptInjection(text) {
  const variants = securityVariants(text);
  const findings = [];

  for (const variant of variants) {
    for (const rule of promptInjectionPatterns) {
      const match = variant.value.match(rule);
      if (match) {
        if (benignPromptContext(variant.value, match)) continue;
        findings.push({
          type: 'prompt-injection',
          encoding: variant.encoding,
          pattern: rule.source,
          match: match[0].slice(0, 160)
        });
      }
    }
  }

  return {
    detected: findings.length > 0,
    findings
  };
}

function scanSecurityFindings(text, input) {
  const trust = classifyInputTrust(input || {});
  const redaction = redactSecrets(text);
  const injection = detectPromptInjection(text);
  const findings = [
    ...redaction.findings,
    ...injection.findings
  ];

  return {
    trust,
    redactedText: redaction.text,
    redacted: redaction.redacted,
    promptInjectionDetected: injection.detected,
    findings,
    risk: riskLevel({ trust, findings })
  };
}

function riskLevel(scan) {
  const hasSecret = scan.findings.some((finding) => finding.type === 'secret');
  const hasInjection = scan.findings.some((finding) => finding.type === 'prompt-injection');
  if (hasSecret && hasInjection) return 'high';
  if (hasSecret) return 'medium';
  if (hasInjection && scan.trust.trust === 'untrusted') return 'medium';
  if (hasInjection) return 'low';
  return 'none';
}

module.exports = {
  classifyInputTrust,
  detectPromptInjection,
  decodeBase64Candidates,
  decodeGzipBase64Candidates,
  decodeHexCandidates,
  decodeUrlCandidates,
  redactSecrets,
  scanSecurityFindings,
  securityVariants,
  stripControlObfuscation
};
