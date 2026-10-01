// Links in markdown, and the rewrite applied when a repository document is
// copied into the package. A copied document keeps every link that resolves
// inside the package. A relative link to a repository file that does not ship
// is rewritten to that file on GitHub at the build's revision, so a reader of
// the installed package never meets a dead relative link. Nothing inside a
// fenced code block or an inline code span is a link, and nothing there
// changes. Used by kit/test/links.test.mjs (repository) and
// scripts/test-kit-package.mjs (packing and the installed package).
import path from 'node:path';

export const REPOSITORY = 'https://github.com/WSattazahn/caveat-lang';

// The markdown with every character inside code replaced by a space, so the
// offsets of everything else are unchanged.
function maskCode(markdown) {
  const lines = markdown.split('\n');
  let fence = null;
  const prose = lines.map(line => {
    const marker = /^\s*(`{3,}|~{3,})/.exec(line)?.[1];
    if (fence) {
      if (marker && marker[0] === fence[0] && marker.length >= fence.length && !line.trim().slice(marker.length).trim()) fence = null;
      return ' '.repeat(line.length);
    }
    if (marker) {
      fence = marker;
      return ' '.repeat(line.length);
    }
    return line;
  }).join('\n');
  // An inline code span opens with a run of backticks and closes at the next
  // run of the same length within its paragraph.
  let masked = '';
  let at = 0;
  for (const run of prose.matchAll(/`+/g)) {
    if (run.index < at) continue;
    const rest = prose.slice(run.index + run[0].length);
    const paragraph = rest.search(/\n[ \t]*\n/);
    const close = new RegExp(`(?<!\`)${run[0]}(?!\`)`).exec(paragraph < 0 ? rest : rest.slice(0, paragraph));
    if (!close) continue;
    const end = run.index + run[0].length + close.index + run[0].length;
    masked += prose.slice(at, run.index) + prose.slice(run.index, end).replace(/[^\n]/g, ' ');
    at = end;
  }
  return masked + prose.slice(at);
}

// Every link and image destination outside code, inline `](target)` and
// reference definitions `[label]: target`, with where its target starts and
// ends in the markdown.
export function markdownLinks(markdown) {
  const masked = maskCode(markdown);
  const patterns = [
    /\]\(\s*(<[^>\n]*>|[^)\s]+)(?:\s+(?:"[^"\n]*"|'[^'\n]*'|\([^)\n]*\)))?\s*\)/dg,
    /^ {0,3}\[[^\]\n]+\]:[ \t]*(<[^>\n]*>|\S+)/dgm,
  ];
  const links = [];
  for (const pattern of patterns) {
    for (const match of masked.matchAll(pattern)) {
      let [start, end] = match.indices[1];
      if (match[1].startsWith('<') && match[1].endsWith('>')) [start, end] = [start + 1, end - 1];
      links.push({ target: markdown.slice(start, end), start, end });
    }
  }
  return links.sort((a, b) => a.start - b.start);
}

// Absolute URLs (any scheme, such as https: or mailto:) and links within the
// same page are left alone; every other target is a relative path.
export function isRelative(target) {
  return !/^([a-z][a-z0-9+.-]*:|#|\/\/)/i.test(target);
}

// The repository path a relative link from the document at `from` names, and
// its anchor, if any.
export function resolveLink(from, target) {
  const hash = target.indexOf('#');
  const file = hash < 0 ? target : target.slice(0, hash);
  const anchor = hash < 0 ? '' : target.slice(hash);
  const resolved = file.startsWith('/') ? path.posix.normalize(file.slice(1)) : path.posix.normalize(path.posix.join(path.posix.dirname(from), file));
  return { path: resolved, anchor };
}

// The document at repository path `from`, as it is copied into the package.
// `shipped(path)` says whether a repository path is copied into the package
// beside it. Returns the rewritten markdown and each link that was rewritten.
export function rewriteLinks(markdown, { from, revision, shipped }) {
  if (!/^[0-9a-f]{40}$/.test(revision)) throw new Error(`revision ${JSON.stringify(revision)} is not a full commit SHA`);
  const rewritten = [];
  let text = '';
  let at = 0;
  for (const { target, start, end } of markdownLinks(markdown)) {
    if (!isRelative(target)) continue;
    const { path: file, anchor } = resolveLink(from, target);
    if (!file || file === '.' || file.startsWith('../')) throw new Error(`${from} links to ${target}, which is outside the repository`);
    if (shipped(file)) continue;
    const url = `${REPOSITORY}/blob/${revision}/${file}${anchor}`;
    rewritten.push({ target, url });
    text += markdown.slice(at, start) + url;
    at = end;
  }
  return { text: text + markdown.slice(at), rewritten };
}
