// A fault no dispatch catalog code classifies, so the event that reaches it is
// fatal `unclassified`: under `rule`, a number whose text adds 17 bytes to a
// 65,520-byte bound text takes the binding past its 65,536 bytes, while the
// expression's own source stays within its 65,536. Value
// provenance past its name bytes served as this example until it became a
// refusal (D022, `limit/provenance_limit`); `id_text` of a number that is no
// handle served before that (F309).
export const textOverflow = rule => `
state big = 0 min 0 max 2000000;
${rule} set big = 1234567.123456789;
bind wide.text = "${'A'.repeat(65520)}" + text(big);
`;
