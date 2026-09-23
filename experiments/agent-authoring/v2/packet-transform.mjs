// Preregister this exact transformation; never expose earlier outcomes to authors.
export const GUIDE_TRANSFORM = {
  file: 'docs/AI_AUTHORING.md',
  marker: '## Evidence from fresh authors',
  operation: 'Remove the final section beginning with the exact heading, including its heading; preserve every preceding byte.',
};

export function transformReference(name, contents) {
  if (name !== GUIDE_TRANSFORM.file) return contents;
  const source = contents.toString('utf8');
  const marker = `${GUIDE_TRANSFORM.marker}\n`;
  const offset = source.indexOf(marker);
  if (offset < 0 || (offset > 0 && source[offset - 1] !== '\n')) throw new Error('Expected historical-results heading is missing');
  if (/^## /m.test(source.slice(offset + marker.length))) throw new Error('Historical-results section must be the final section');
  return Buffer.from(source.slice(0, offset));
}
