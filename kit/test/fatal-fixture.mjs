// A fault no dispatch catalog code classifies, so the event that reaches it is
// fatal `unclassified`: under `rule`, two 32,768-byte evidence names and one
// more fill value provenance past its 65,536 name bytes. `id_text` of a number
// that is no handle served as this example until it became a refusal (F309).
const left = 'a'.repeat(32768);
const right = 'b'.repeat(32768);

export const provenanceOverflow = rule => `
evidence ${left} from left_sensor; evidence ${right} from right_sensor;
evidence extra from extra_sensor;
state wide_left = 0; state wide_right = 0; state wide = 0;
${rule} reveal ${left}; ${rule} reveal ${right}; ${rule} reveal extra;
${rule} set wide_left = qualified(1, ${left});
${rule} set wide_right = qualified(2, ${right});
${rule} set wide = wide_left + wide_right + qualified(0, extra);
`;
