// The protocol compares evidence sets without order, but decision.basis,
// reopenedBy, history and each history entry's because retain observation order.
function isEvidenceSet(path) {
  const field = path.at(-1);
  if (path[0] === 'belief' && path.length === 2) {
    return ['supportedBy', 'contradictedBy', 'caveats'].includes(field);
  }
  if (path[0] === 'decision' && path.length === 2) return field === 'caveats';
  if (path[0] !== 'mushrooms' || !['because', 'caveats'].includes(field)) return false;
  return path.length === 3 || (path.length === 5 && path[2] === 'why'
    && ['absorb', 'taste'].includes(path[3]));
}

export function normaliseView(value, path = []) {
  if (Array.isArray(value)) {
    const items = value.map((item, index) => normaliseView(item, [...path, index]));
    return isEvidenceSet(path) ? items.sort() : items;
  }
  if (value && typeof value === 'object') {
    return Object.fromEntries(Object.keys(value).sort()
      .map((key) => [key, normaliseView(value[key], [...path, key])]));
  }
  return value;
}

function firstDifference(ts, caveat, at = '') {
  if (JSON.stringify(ts) === JSON.stringify(caveat)) return null;
  if (ts && caveat && typeof ts === 'object' && typeof caveat === 'object'
    && !Array.isArray(ts) && !Array.isArray(caveat)) {
    for (const key of new Set([...Object.keys(ts), ...Object.keys(caveat)])) {
      const found = firstDifference(ts[key], caveat[key], at ? `${at}.${key}` : key);
      if (found) return found;
    }
  }
  return { at, ts, caveat };
}

export function compareViews(ts, caveat) {
  return firstDifference(normaliseView(ts), normaliseView(caveat));
}
