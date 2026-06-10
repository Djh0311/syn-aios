const fs = require('fs');
const path = require('path');

function readJson(filePath) {
  try {
    return { data: JSON.parse(fs.readFileSync(filePath, 'utf8')), error: null };
  } catch (error) {
    return { data: null, error: error.message };
  }
}

function loadHarnessConfig(targetRoot, explicitConfig) {
  const candidates = explicitConfig
    ? [path.resolve(explicitConfig)]
    : [
        path.join(targetRoot, 'harness.config.json'),
        path.join(targetRoot, 'harness.config.example.json')
      ];

  for (const candidate of candidates) {
    if (!fs.existsSync(candidate)) continue;
    const parsed = readJson(candidate);
    return {
      path: candidate,
      data: parsed.data,
      error: parsed.error
    };
  }

  return {
    path: null,
    data: null,
    error: null
  };
}

module.exports = {
  loadHarnessConfig
};
