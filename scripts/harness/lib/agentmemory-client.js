const http = require('http');
const https = require('https');

function memoryConfig(config) {
  const memory = config && config.memoryIntegration ? config.memoryIntegration : {};
  const auth = memory.auth || {};
  return {
    enabled: memory.enabled === true,
    provider: memory.provider || 'agentmemory',
    endpoint: memory.endpoint || 'http://127.0.0.1:3111',
    secretEnv: auth.secretEnv || 'AGENTMEMORY_SECRET',
    allowRemote: memory.allowRemote === true,
    readPolicy: memory.readPolicy || {},
    writePolicy: memory.writePolicy || {}
  };
}

function isLocalEndpoint(endpoint) {
  return /^https?:\/\/(?:127\.0\.0\.1|localhost|\[::1\])(?::\d+)?(?:\/|$)/i.test(String(endpoint || ''));
}

function requestJson(endpoint, route, options = {}) {
  return new Promise((resolve) => {
    let url;
    try {
      url = new URL(route, endpoint.endsWith('/') ? endpoint : `${endpoint}/`);
    } catch (error) {
      resolve({ ok: false, statusCode: null, data: null, error: `Invalid agentmemory endpoint: ${error.message}` });
      return;
    }

    const body = options.body === undefined ? null : JSON.stringify(options.body);
    const headers = Object.assign({}, options.headers || {});
    if (body !== null) {
      headers['content-type'] = 'application/json';
      headers['content-length'] = Buffer.byteLength(body);
    }

    const transport = url.protocol === 'https:' ? https : http;
    const req = transport.request({
      method: options.method || (body === null ? 'GET' : 'POST'),
      hostname: url.hostname,
      port: url.port || (url.protocol === 'https:' ? 443 : 80),
      path: `${url.pathname}${url.search}`,
      headers,
      timeout: options.timeoutMs || 5000
    }, (res) => {
      const chunks = [];
      res.on('data', (chunk) => chunks.push(chunk));
      res.on('end', () => {
        const text = Buffer.concat(chunks).toString('utf8');
        let data = null;
        if (text.trim()) {
          try {
            data = JSON.parse(text);
          } catch (error) {
            resolve({ ok: false, statusCode: res.statusCode, data: null, error: `Invalid JSON response: ${error.message}` });
            return;
          }
        }
        resolve({ ok: res.statusCode >= 200 && res.statusCode < 300, statusCode: res.statusCode, data, error: null });
      });
    });
    req.on('timeout', () => {
      req.destroy(new Error('agentmemory request timed out'));
    });
    req.on('error', (error) => {
      resolve({ ok: false, statusCode: null, data: null, error: error.message });
    });
    if (body !== null) req.write(body);
    req.end();
  });
}

function authHeaders(config) {
  const resolved = memoryConfig(config);
  const secret = resolved.secretEnv ? process.env[resolved.secretEnv] : '';
  return secret ? { authorization: `Bearer ${secret}` } : {};
}

function validateEndpoint(config) {
  const resolved = memoryConfig(config);
  if (resolved.provider !== 'agentmemory') {
    return { ok: false, config: resolved, error: `Unsupported memory provider: ${resolved.provider}` };
  }
  if (!isLocalEndpoint(resolved.endpoint) && !resolved.allowRemote) {
    return { ok: false, config: resolved, error: 'Remote agentmemory endpoint requires memoryIntegration.allowRemote=true' };
  }
  return { ok: true, config: resolved, error: null };
}

async function agentmemoryHealth(config) {
  const validation = validateEndpoint(config);
  if (!validation.ok) return { ok: false, data: null, error: validation.error };
  return requestJson(validation.config.endpoint, '/agentmemory/health', {
    method: 'GET',
    headers: authHeaders(config)
  });
}

async function agentmemorySmartSearch(config, query, options = {}) {
  const validation = validateEndpoint(config);
  if (!validation.ok) return { ok: false, data: null, error: validation.error };
  return requestJson(validation.config.endpoint, '/agentmemory/smart-search', {
    method: 'POST',
    headers: authHeaders(config),
    body: {
      query,
      limit: options.limit || validation.config.readPolicy.maxMemoriesPerTask || 5
    },
    timeoutMs: options.timeoutMs || 5000
  });
}

async function agentmemoryRemember(config, candidate, options = {}) {
  const validation = validateEndpoint(config);
  if (!validation.ok) return { ok: false, data: null, error: validation.error };
  return requestJson(validation.config.endpoint, '/agentmemory/remember', {
    method: 'POST',
    headers: authHeaders(config),
    body: {
      text: candidate.claim,
      metadata: Object.assign({}, candidate, { claim: undefined })
    },
    timeoutMs: options.timeoutMs || 5000
  });
}

module.exports = {
  agentmemoryHealth,
  agentmemoryRemember,
  agentmemorySmartSearch,
  isLocalEndpoint,
  memoryConfig
};
