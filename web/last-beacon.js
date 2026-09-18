import init, { WebGameSession } from './pkg/caveat_runtime.js';
import { createBeaconWorld } from './beacon-world.js';

const $ = selector => document.querySelector(selector);
const app = $('#app');
const saveKey = 'caveat:last-beacon:watch:v1';
const escape = value => String(value ?? '').replace(/[&<>"']/g, c => ({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]));
const human = id => String(id).replaceAll('_', ' ');
let source, session, snapshot, world, saved = null, busy = false, started = false, feedback = null;
let sound = null, soundEnabled = false, audioContext = null, journalFocus = null, storageAvailable = true;
const label = id => snapshot?.labels[id] || human(id);
const text = (id, fallback = '') => snapshot?.labels[id] || fallback;
const announce = message => { $('#announcer').textContent = message; };

function setBusy(value) {
  busy = value;
  app.dataset.busy = String(value);
  $('#journey-status').hidden = !value;
  $('#journal-toggle').disabled = value || !session;
  $('#restart').disabled = value;
  document.querySelectorAll('[data-selection], [data-continue]').forEach(button => { button.disabled = value; });
}

function persist() {
  try { localStorage.setItem(saveKey, session.save()); saved = session.save(); }
  catch { storageAvailable = false; }
  $('#save-status').textContent = storageAvailable ? 'Watch saved on this device' : 'Saving unavailable · keep this tab open';
}

function button(html, className, onClick) {
  const node = document.createElement('button');
  node.className = className;
  node.innerHTML = html;
  node.addEventListener('click', onClick);
  return node;
}

function renderTitle() {
  started = false;
  feedback = null;
  app.dataset.screen = 'title';
  $('#intro').hidden = false;
  $('#intro-footer').hidden = false;
  $('#decision-panel').hidden = true;
  $('#watch-hud').hidden = true;
  $('#place-name').textContent = 'SAINT ORIN · BEFORE DAWN';
  const actions = $('#start-actions');
  actions.replaceChildren();
  if (saved) {
    actions.append(button('Resume your watch <span class="arrow">↗</span>', 'primary', () => begin(true)));
    actions.append(button('Start a new watch', 'secondary', () => begin(false)));
  } else actions.append(button('Begin the watch <span class="arrow">↗</span>', 'primary', () => begin(false)));
  world?.reset();
}

async function begin(resume) {
  if (busy) return;
  let next;
  try {
    next = resume && saved ? WebGameSession.restore(source, saved) : new WebGameSession(source);
  } catch (error) {
    saved = null;
    renderTitle();
    const warning = document.createElement('p');
    warning.className = 'empty';
    warning.textContent = 'This saved watch belongs to an older version. Start a new watch to play this story.';
    $('#start-actions').append(warning);
    return;
  }
  session?.free();
  session = next;
  snapshot = JSON.parse(session.snapshot());
  feedback = null;
  started = true;
  app.dataset.screen = 'playing';
  $('#intro').hidden = true;
  $('#intro-footer').hidden = true;
  $('#watch-hud').hidden = false;
  $('#decision-panel').hidden = false;
  world?.reset();
  setBusy(true);
  try {
    if (resume && snapshot.selections.length) {
      const replay = new WebGameSession(source);
      try {
        for (const selection of snapshot.selections) {
          const turn = JSON.parse(replay.apply(selection));
          await world?.play(turn.last_execution, { immediate: true });
        }
      } finally { replay.free(); }
    }
    world?.setWorld(snapshot.state);
    if (world?.focus) await world.focus(snapshot.state.current_place);
    persist();
  } finally { setBusy(false); }
  render();
  window.scrollTo({ top: 0, behavior: 'instant' });
  announce(resume ? 'Your watch has resumed.' : 'Your watch has begun. Choose one investigation.');
}

function render() {
  if (!started || !snapshot) return;
  const pending = snapshot.pending;
  $('#place-name').textContent = label(snapshot.state.current_place);
  $('#chapter-count').textContent = `${String(Math.min(snapshot.turn + (feedback ? 0 : 1), 6)).padStart(2,'0')} / 06`;
  $('#progress').innerHTML = Array.from({length:6}, (_, i) => `<li class="${i < snapshot.turn ? 'done' : i === snapshot.turn ? 'current' : ''}" aria-label="Decision ${i+1}${i < snapshot.turn ? ' completed' : ''}"></li>`).join('');
  const budget = snapshot.budget;
  $('#budget').textContent = budget ? `${budget.remaining} / ${budget.initial}` : '—';
  $('#budget-dots').innerHTML = budget ? Array.from({length:budget.initial}, (_,i) => `<i class="${i >= budget.remaining ? 'spent' : ''}"></i>`).join('') : '';
  if (feedback) renderFeedback();
  else if (pending.kind === 'complete') renderEnding();
  else if (pending.kind === 'blocked') renderBlocked();
  else renderChoices();
  renderJournal();
}

function renderChoices() {
  const pending = snapshot.pending;
  app.dataset.screen = 'playing';
  $('#decision-panel').classList.remove('ending');
  $('#phase-label').textContent = pending.kind === 'investigate' ? 'A QUESTION WORTH ASKING' : 'ACT WITH UNCERTAINTY';
  $('#phase-meta').textContent = pending.kind === 'investigate' ? `Costs ${pending.cost} attention` : 'A provisional commitment';
  $('#decision-title').textContent = label(pending.name).replace(/^[IVX]+\s*\/\s*/, '');
  $('#decision-body').textContent = text(`${pending.name}_body`);
  const blocked = snapshot.blocked_actions || [];
  const unavailable = blocked.length ? `<details class="unavailable"><summary>${blocked.length} other ${blocked.length === 1 ? 'plan needs' : 'plans need'} more investigation</summary>${blocked.map(item => `<div class="unavailable-plan" data-locked-action="${escape(item.action)}"><b>${escape(label(item.action))}</b>${item.reasons.map(reason => `<small>${reason.kind === 'examined' ? 'Examine the uncertainty' : 'Find the observation'}: ${escape(label(reason.symbol))}</small>`).join('')}</div>`).join('')}<p>You can still act with the knowledge you have.</p></details>` : '';
  $('#decision-content').innerHTML = `<div class="choices">${pending.options.map((id, i) => `<button class="choice" data-selection="${escape(id)}"><span class="number">0${i+1}</span><span class="choice-copy"><b>${escape(label(id))}</b><small>${escape(text(`${id}_hint`))}</small></span><span class="arrow" aria-hidden="true">↗</span></button>`).join('')}</div>${unavailable}`;
  document.querySelectorAll('[data-selection]').forEach(node => node.addEventListener('click', () => choose(node.dataset.selection)));
}

async function choose(selection) {
  if (busy || feedback || !started || !snapshot.pending.options?.includes(selection)) return;
  const previous = snapshot;
  setBusy(true);
  $('#journey-text').textContent = label(selection);
  try {
    snapshot = JSON.parse(session.apply(selection));
    persist();
    const commitment = snapshot.commitments.find(item => item.action === selection);
    feedback = {selection, kind:previous.pending.kind, commitment, discoveries:snapshot.discoveries.slice(previous.discoveries.length)};
    try { await world?.play(snapshot.last_execution); }
    catch (error) { console.warn('The island animation was interrupted.', error); world?.setWorld(snapshot.state); }
    ping(commitment?.reopened_by?.length ? 'doubt' : 'discovery');
    render();
    announce(text(`${snapshot.outcome?.action === selection ? snapshot.outcome.id : selection}_result`, label(selection)));
  } catch (error) {
    const errorBox = document.createElement('p');
    errorBox.className = 'error-box';
    errorBox.textContent = `That action could not complete: ${error.message || error}`;
    $('#decision-content').append(errorBox);
  } finally { setBusy(false); }
}

function renderFeedback() {
  const {selection, commitment, discoveries} = feedback;
  const reopened = commitment?.reopened_by?.length > 0;
  $('#phase-label').textContent = reopened ? 'NEW EVIDENCE · COMMITMENT REOPENED' : commitment ? 'DECISION RECORDED' : 'FIELD OBSERVATION';
  $('#phase-meta').textContent = `Watch ${Math.ceil(snapshot.turn / 2)} of 3`;
  $('#decision-title').textContent = reopened ? 'A reason to reconsider.' : commitment ? 'The choice is yours.' : 'The signal has a catch.';
  $('#decision-body').textContent = text(`${snapshot.outcome?.action === selection ? snapshot.outcome.id : selection}_result`, label(selection));
  let details = '';
  if (reopened) details = `<div class="feedback reopened"><span class="eyebrow">YOUR EARLIER REASONING IS STILL HERE</span>${commitment.reopened_by.map(id => `<span class="doubt">${escape(text(`${id}_uncertainty`, label(id)))}</span>`).join('')}<span class="doubt">The next decision starts with what you learned.</span></div>`;
  else if (discoveries.length) details = `<div class="feedback"><span class="eyebrow">ADDED TO YOUR FIELD JOURNAL</span>${[...new Set(discoveries.map(item => item.evidence))].map(id => `<div>${escape(label(id))}</div>`).join('')}</div>`;
  $('#decision-content').innerHTML = `${details}<button class="primary continue-button" data-continue>${snapshot.pending.kind === 'complete' ? 'See the dawn' : 'Continue the watch'}<span class="arrow" aria-hidden="true">→</span></button>`;
  $('[data-continue]').addEventListener('click', () => { if (busy) return; feedback = null; render(); announce($('#decision-title').textContent); });
}

function renderEnding() {
  app.dataset.screen = 'ending';
  $('#decision-panel').classList.add('ending');
  const final = snapshot.selections.at(-1);
  const outcome = snapshot.outcome?.action === final ? snapshot.outcome : null;
  const ending = outcome?.id || final;
  const examined = snapshot.symbols.filter(symbol => symbol.kind === 'caveat' && symbol.attention === 'examined').length;
  const reopened = snapshot.commitments.filter(commitment => commitment.reopened_by.length).length;
  $('#phase-label').textContent = 'DAWN AT SAINT ORIN';
  $('#phase-meta').textContent = 'Your watch is complete';
  $('#decision-title').textContent = text(`${ending}_title`, label(ending));
  $('#decision-body').textContent = text(`${ending}_result`);
  const basis = outcome?.basis?.length ? `<div class="feedback"><span class="eyebrow">THE KNOWLEDGE THAT CHANGED THIS CROSSING</span>${outcome.basis.map(reason => `<div>${escape(label(reason.symbol))}</div>`).join('')}</div>` : '';
  const epilogue = text(`${ending}_epilogue`);
  $('#decision-content').innerHTML = `${basis}${epilogue ? `<p class="decision-body epilogue">${escape(epilogue)}</p>` : ''}<div class="end-summary"><div><strong>${examined}</strong><span>UNCERTAINTIES EXAMINED</span></div><div><strong>${reopened}</strong><span>COMMITMENTS REOPENED</span></div><div><strong>${snapshot.commitments.find(commitment => commitment.action === final)?.retained.length || 0}</strong><span>CAVEATS REMEMBERED</span></div></div><div class="ending-actions"><button class="primary" id="ending-journal">Read your record <span class="arrow">↗</span></button><button class="secondary" id="ending-replay">Take another watch</button></div>`;
  $('#ending-journal').addEventListener('click', () => openJournal('decisions'));
  $('#ending-replay').addEventListener('click', () => begin(false));
  ping('ending');
}

function renderBlocked() {
  $('#phase-label').textContent = 'WATCH INTERRUPTED';
  $('#phase-meta').textContent = '';
  $('#decision-title').textContent = 'No available action.';
  $('#decision-body').textContent = snapshot.pending.reason;
  $('#decision-content').innerHTML = '<button class="secondary" id="blocked-restart">Start a new watch</button>';
  $('#blocked-restart').addEventListener('click', () => begin(false));
}

function renderJournal() {
  if (!snapshot) return;
  const claims = snapshot.symbols.filter(symbol => symbol.kind === 'claim');
  const symbolById = new Map(snapshot.symbols.map(symbol => [symbol.name, symbol]));
  const evidence = claims.map(claim => {
    const relations = snapshot.relations.filter(relation => relation.to === claim.name && ['supports','opposes'].includes(relation.relation));
    if (!relations.length) return '';
    const disputed = relations.some(r => r.relation === 'supports') && relations.some(r => r.relation === 'opposes');
    return `<article class="claim-card">${disputed ? '<div class="claim-status">Conflicting evidence retained</div>' : ''}<h3>${escape(label(claim.name))}</h3>${relations.map(relation => `<div class="evidence-line ${relation.relation}">${escape(label(relation.from))}<small>${relation.relation === 'supports' ? 'Supports' : 'Challenges'} this claim${symbolById.get(relation.from)?.source ? ` · ${escape(symbolById.get(relation.from).source)}` : ''}</small></div>`).join('')}</article>`;
  }).join('');
  $('#page-evidence').innerHTML = evidence || '<p class="empty">No observations recorded yet. Start the watch to examine the island’s first signals.</p>';
  const commitments = snapshot.selections.map(selection => snapshot.commitments.find(commitment => commitment.action === selection)).filter(Boolean);
  $('#page-decisions').innerHTML = commitments.length ? commitments.map(commitment => `<article class="decision-entry"><h3>${escape(label(commitment.action))}</h3>${commitment.reopened_by.length ? '<span class="reopened-label">REOPENED BY NEW EVIDENCE</span>' : '<span class="reopened-label">PROVISIONAL COMMITMENT</span>'}<p>${escape(text(`${snapshot.outcome?.action === commitment.action ? snapshot.outcome.id : commitment.action}_result`))}</p><p>Uncertainty carried forward:</p><ul>${commitment.retained.map(id => `<li>${escape(text(`${id}_uncertainty`, label(id)))}</li>`).join('')}</ul></article>`).join('') : '<p class="empty">No commitments yet. When you act, your reasons and unresolved caveats stay in this record.</p>';
  const places = snapshot.world.places;
  const nodes = new Map(places.map((place,i) => [place.id, {x:150 + Math.cos(i / places.length * Math.PI * 2 - Math.PI / 2) * 115, y:88 + Math.sin(i / places.length * Math.PI * 2 - Math.PI / 2) * 67}]));
  $('#page-chart').innerHTML = `<svg class="chart-svg" viewBox="0 0 300 176" role="img" aria-label="Declared routes connecting the island's seven locations">${snapshot.world.connections.map(link => {const a=nodes.get(link.from),b=nodes.get(link.to);return `<line x1="${a.x}" y1="${a.y}" x2="${b.x}" y2="${b.y}" stroke="#7fb3b466" stroke-width="1"/>`;}).join('')}${places.map((place,i)=>{const p=nodes.get(place.id);return `<circle cx="${p.x}" cy="${p.y}" r="${place.id===snapshot.state.current_place?7:4}" fill="${place.id===snapshot.state.current_place?'#ead3a0':'#8bbaba'}"/><text x="${p.x+9}" y="${p.y+4}" font-size="9" fill="#c9d9d3">${i+1}</text>`;}).join('')}</svg><div class="chart-list">${places.map((place,i) => `<button class="chart-button" data-focus="${escape(place.id)}" aria-current="${place.id === snapshot.state.current_place}"><span>0${i+1} &nbsp; ${escape(label(place.id))}</span><small>${place.id === snapshot.state.current_place ? 'YOU ARE HERE' : 'LOOK ↗'}</small></button>`).join('')}</div><p class="chart-legend">Look around without spending attention. Travel and investigations happen when you choose an action.</p>`;
  document.querySelectorAll('[data-focus]').forEach(node => node.addEventListener('click', () => { world?.focus(node.dataset.focus); closeJournal(); }));
}

function setTab(tab) {
  for (const node of document.querySelectorAll('[data-tab]')) {
    const selected = node.dataset.tab === tab;
    node.setAttribute('aria-selected', String(selected));
    node.tabIndex = selected ? 0 : -1;
    $(`#page-${node.dataset.tab}`).hidden = !selected;
  }
}
function openJournal(tab='evidence') {
  if (busy || !session) return;
  journalFocus = document.activeElement;
  renderJournal();
  setTab(tab);
  $('#journal').hidden = false;
  $('#journal-backdrop').hidden = false;
  $('#journal-toggle').setAttribute('aria-expanded', 'true');
  for (const node of app.children) if (node.id !== 'journal' && node.id !== 'journal-backdrop') node.inert = true;
  $('#journal-close').focus();
}
function closeJournal() {
  $('#journal').hidden = true;
  $('#journal-backdrop').hidden = true;
  $('#journal-toggle').setAttribute('aria-expanded', 'false');
  for (const node of app.children) node.inert = false;
  journalFocus?.focus();
}

function toggleSound() {
  soundEnabled = !soundEnabled;
  if (soundEnabled && !audioContext) {
    const Audio = window.AudioContext || window.webkitAudioContext;
    if (!Audio) { soundEnabled = false; return; }
    audioContext = new Audio();
    sound = audioContext.createGain();
    sound.gain.value = 0;
    sound.connect(audioContext.destination);
    const buffer = audioContext.createBuffer(1, audioContext.sampleRate*4, audioContext.sampleRate);
    const data = buffer.getChannelData(0);
    let last = 0;
    for(let i=0;i<data.length;i++){last=(last+Math.random()*.03-.015)/1.007;data[i]=last;}
    const wind = audioContext.createBufferSource();
    wind.buffer=buffer;wind.loop=true;
    const filter=audioContext.createBiquadFilter();filter.type='lowpass';filter.frequency.value=700;
    wind.connect(filter).connect(sound);wind.start();
    for(const frequency of [55,82.4,110]){const oscillator=audioContext.createOscillator(),gain=audioContext.createGain();oscillator.type='sine';oscillator.frequency.value=frequency;gain.gain.value=.027;oscillator.connect(gain).connect(sound);oscillator.start();}
  }
  if(audioContext){audioContext.resume().catch(()=>{});sound.gain.setTargetAtTime(soundEnabled?.34:0,audioContext.currentTime,.4);}
  $('#sound').textContent = soundEnabled ? 'Sound on' : 'Sound off';
  $('#sound').setAttribute('aria-pressed',String(soundEnabled));
  $('#sound').setAttribute('aria-label',`Turn ambient sound ${soundEnabled?'off':'on'}`);
}
function ping(kind) {
  if (!soundEnabled || !audioContext) return;
  const time=audioContext.currentTime;
  const notes=kind==='doubt'?[196,207.65]:kind==='ending'?[261.63,329.63,392]:[329.63,493.88];
  notes.forEach((frequency,i)=>{const oscillator=audioContext.createOscillator(),gain=audioContext.createGain();oscillator.type='sine';oscillator.frequency.value=frequency;gain.gain.setValueAtTime(0,time+i*.15);gain.gain.linearRampToValueAtTime(.11,time+i*.15+.02);gain.gain.exponentialRampToValueAtTime(.0001,time+i*.15+2.2);oscillator.connect(gain).connect(sound);oscillator.start(time+i*.15);oscillator.stop(time+i*.15+2.3);});
}

$('#sound').addEventListener('click',toggleSound);
$('#journal-toggle').addEventListener('click',()=>openJournal());
$('#journal-close').addEventListener('click',closeJournal);
$('#journal-backdrop').addEventListener('click',closeJournal);
$('#restart').addEventListener('click',()=>{if(!busy)renderTitle();});
document.querySelectorAll('[data-tab]').forEach(node=>node.addEventListener('click',()=>setTab(node.dataset.tab)));
document.addEventListener('keydown',event=>{
  if (event.ctrlKey || event.altKey || event.metaKey || event.repeat) return;
  if (!$('#journal').hidden) {
    if(event.key==='Escape'){event.preventDefault();closeJournal();}
    if(event.key==='Tab'){const nodes=[...$('#journal').querySelectorAll('button:not([tabindex="-1"])')].filter(node=>!node.closest('[hidden]'));const first=nodes[0],last=nodes.at(-1);if(event.shiftKey&&document.activeElement===first){event.preventDefault();last.focus();}else if(!event.shiftKey&&document.activeElement===last){event.preventDefault();first.focus();}}
    if(['ArrowLeft','ArrowRight'].includes(event.key)&&document.activeElement.matches('[data-tab]')){const tabs=[...document.querySelectorAll('[data-tab]')],i=tabs.indexOf(document.activeElement),next=tabs[(i+(event.key==='ArrowRight'?1:2))%3];setTab(next.dataset.tab);next.focus();event.preventDefault();}
    return;
  }
  if(event.key.toLowerCase()==='j'){event.preventDefault();openJournal();}
  if(event.key.toLowerCase()==='m'){event.preventDefault();toggleSound();}
  if(!busy&&started&&!feedback&&['1','2','3'].includes(event.key)){const selection=snapshot.pending.options?.[Number(event.key)-1];if(selection){event.preventDefault();choose(selection);}}
});
document.addEventListener('visibilitychange',()=>{if(audioContext){if(document.hidden)audioContext.suspend().catch(()=>{});else if(soundEnabled)audioContext.resume().catch(()=>{});}});

async function boot() {
  const response = await fetch('./the_last_beacon.cav');
  if(!response.ok)throw new Error('The story source could not be loaded.');
  source=await response.text();
  await init();
  session=new WebGameSession(source);
  snapshot=JSON.parse(session.snapshot());
  try {
    world=createBeaconWorld($('#world'),{model:snapshot.world,labels:snapshot.labels,onPlace:id=>{if(started)$('#place-name').textContent=label(id);},onStatus:status=>{if(!status)return;const verbs={move:'Walking to',inspect:'Examining',operate:'Operating',open:'Opening',observe:'Observing',stay:'Waiting at'};$('#journey-text').textContent=typeof status==='string'?status:`${verbs[status.kind]||'Following'} ${label(status.to||status.entity||status.symbol||status.place||'')}`;}});
  } catch(error) {
    console.warn('3D view unavailable; story mode is still playable.',error);
    $('#intro-description').textContent='The island view is unavailable on this device. You can still play the complete story and follow every decision in the field journal.';
  }
  try { saved=localStorage.getItem(saveKey);if(saved){const check=WebGameSession.restore(source,saved);check.free();} }
  catch {saved=null;}
  $('#journal-toggle').disabled=false;
  renderTitle();
  // Read-only diagnostics for smoke tests; actions still go through actual UI + Rust.
  Object.defineProperty(window,'__beacon',{value:Object.freeze({snapshot:()=>JSON.parse(session.snapshot()),get busy(){return busy;},get screen(){return app.dataset.screen;},get rendered(){return Boolean(world);}})});
}

boot().catch(error=>{
  console.error(error);
  $('#loading-status')?.remove();
  $('#start-actions').innerHTML=`<div class="error-box">The watch could not load. ${escape(error.message||error)}<br><button class="secondary" id="retry">Try again</button></div>`;
  $('#retry').addEventListener('click',()=>location.reload());
});
