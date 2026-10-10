// Market props and Miso the cat for The Missing Dumpling, drawn in the same flat
// style as the cast in mr-caveat.html (soft fills, small dark eyes, a ground
// shadow), so no emoji stands in for a character or object in play.

const shadow = (cx, cy, rx) => `<ellipse cx="${cx}" cy="${cy}" rx="${rx}" ry="${(rx / 4.6).toFixed(1)}" fill="#120d0e" opacity=".38"/>`;

export const ART = {
  steamer: `<svg viewBox="0 0 60 50" aria-hidden="true">${shadow(30, 46, 24)}
    <path d="M9 22h42v14q0 8-8 8H17q-8 0-8-8z" fill="#b98a4e"/><path d="M9 29h42" stroke="#8a6234" stroke-width="2.4"/>
    <ellipse cx="30" cy="22" rx="21" ry="6" fill="#d8ad6e"/><ellipse cx="30" cy="21" rx="15" ry="3.6" fill="#c39457"/>
    <circle cx="23" cy="19.6" r="4.6" fill="#fff4e2"/><circle cx="33" cy="19" r="4.6" fill="#fff7ea"/><circle cx="28" cy="16.4" r="4.2" fill="#fffaf2"/>
    <path class="steam" d="M20 12q-4-5 0-9M30 10q-4-5 0-9M40 12q-4-5 0-9" stroke="#fff3df" stroke-width="2.2" fill="none" stroke-linecap="round" opacity=".55"/></svg>`,
  bell: `<svg viewBox="0 0 50 58" aria-hidden="true">${shadow(25, 54, 14)}
    <path d="M25 4v6" stroke="#6b5132" stroke-width="3" stroke-linecap="round"/>
    <path d="M25 9c-10 0-14 8-14 18v9l-5 6h38l-5-6v-9c0-10-4-18-14-18z" fill="#e7b64c"/>
    <path d="M17 18q-3 6-3 16" stroke="#fff0b8" stroke-width="2.6" fill="none" stroke-linecap="round" opacity=".75"/>
    <path d="M6 42h38" stroke="#b5862d" stroke-width="2.4"/><circle cx="25" cy="46" r="4" fill="#9a6e22"/></svg>`,
  basket: `<svg viewBox="0 0 60 50" aria-hidden="true">${shadow(30, 46, 23)}
    <path d="M14 22q16-22 32 0" stroke="#9a6a3a" stroke-width="3.4" fill="none"/>
    <path d="M11 13q5-8 9 0q4-7 8 1q4-8 9 0q4-7 8 1" stroke="#6f9e58" stroke-width="5" fill="none" stroke-linecap="round"/>
    <path d="M8 22h44l-5 20q-1 4-5 4H18q-4 0-5-4z" fill="#c9924f"/>
    <path d="M10 29h40M12 36h36M20 22l2 24M30 22v24M40 22l-2 24" stroke="#a47139" stroke-width="2"/></svg>`,
  crate: `<svg viewBox="0 0 56 52" aria-hidden="true">${shadow(28, 48, 23)}
    <rect x="7" y="12" width="42" height="34" rx="3" fill="#b07b48"/><rect x="7" y="12" width="42" height="7" rx="2" fill="#c99360"/>
    <path d="M7 27h42M7 38h42M12 19l32 27M44 19L12 46" stroke="#8a5a30" stroke-width="2.4"/>
    <rect x="7" y="12" width="42" height="34" rx="3" fill="none" stroke="#7a4d27" stroke-width="2"/></svg>`,
  // Miso: a round black cat with amber eyes and a white-tipped tail.
  miso: `<svg viewBox="0 0 110 92" aria-hidden="true">${shadow(54, 87, 38)}
    <path class="tail" d="M86 66q22-6 18-30q-2-9-8-6q4 16-12 26" fill="#1e1a1f"/><circle cx="97" cy="31" r="4.6" fill="#efe6da"/>
    <ellipse cx="55" cy="66" rx="34" ry="21" fill="#221d23"/>
    <path d="M30 80q-2 6 6 6h8q4 0 3-6M62 80q-2 6 6 6h8q4 0 3-6" fill="#1a161b"/>
    <circle cx="36" cy="42" r="23" fill="#262128"/>
    <path d="M17 32l-3-22 19 12M55 32l3-22-19 12" fill="#262128"/><path d="M18 27l-1-11 9 6M54 27l1-11-9 6" fill="#6b4a55"/>
    <ellipse class="blink" cx="28" cy="41" rx="4.4" ry="5.6" fill="#f2b84b"/><ellipse class="blink" cx="45" cy="41" rx="4.4" ry="5.6" fill="#f2b84b"/>
    <ellipse cx="28.6" cy="41.6" rx="1.6" ry="3.6" fill="#141015"/><ellipse cx="45.6" cy="41.6" rx="1.6" ry="3.6" fill="#141015"/>
    <path d="M34 50l2.4 2 2.4-2" stroke="#e8a1a8" stroke-width="2" fill="none" stroke-linecap="round"/>
    <path d="M13 49l-9-2M13 53l-9 2M59 49l9-2M59 53l9 2" stroke="#8f8790" stroke-width="1.2" stroke-linecap="round"/></svg>`,
  dumpling: `<svg viewBox="0 0 40 30" aria-hidden="true"><path d="M5 22q0-16 15-16t15 16q-15 6-30 0z" fill="#fff4e0"/>
    <path d="M12 10q3 5 3 10M20 7v12M28 10q-3 5-3 10" stroke="#e7d2b0" stroke-width="1.6" fill="none" stroke-linecap="round"/>
    <path d="M10 6q-3-3 0-5M30 6q3-3 0-5" stroke="#fff3df" stroke-width="1.4" fill="none" opacity=".6"/></svg>`,
};
