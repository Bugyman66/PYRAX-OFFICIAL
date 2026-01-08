const fs = require('fs');
const path = require('path');

const messagesDir = path.join(__dirname, '../src/messages');
const enPath = path.join(messagesDir, 'en.json');

// Read English (source of truth)
const enContent = JSON.parse(fs.readFileSync(enPath, 'utf8'));

// Get all locale files
const localeFiles = fs.readdirSync(messagesDir).filter(f => f.endsWith('.json') && f !== 'en.json');

// Deep merge function - source values override target, but keeps target structure
function deepMerge(source, target) {
  const result = { ...source };
  
  for (const key of Object.keys(target)) {
    if (target[key] && typeof target[key] === 'object' && !Array.isArray(target[key])) {
      if (source[key] && typeof source[key] === 'object') {
        result[key] = deepMerge(source[key], target[key]);
      } else {
        result[key] = target[key];
      }
    } else if (target[key] !== undefined) {
      result[key] = target[key];
    }
  }
  
  return result;
}

console.log('Syncing locale files with en.json...\n');

for (const file of localeFiles) {
  const localePath = path.join(messagesDir, file);
  const localeContent = JSON.parse(fs.readFileSync(localePath, 'utf8'));
  
  // Merge: English is base, locale-specific values override
  const merged = deepMerge(enContent, localeContent);
  
  // Write back
  fs.writeFileSync(localePath, JSON.stringify(merged, null, 2) + '\n', 'utf8');
  
  console.log(`✓ Synced ${file}`);
}

console.log('\nAll locale files synced successfully!');
