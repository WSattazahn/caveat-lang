// Transport and projection shared by the page and verification harness.
// Every gameplay decision, observation and explanation comes from the source.
export function createPolicyFromSession(Session, source, saved) {
  const session = saved === undefined ? new Session(source) : Session.restore(source, JSON.stringify(saved));
  const entities = JSON.parse(session.snapshot()).world.entities;
  const tunnels = entities.filter(({ kind }) => kind === 'tunnel').map(({ id }) => id);
  const observations = entities.filter(({ kind }) => kind === 'observation').map(({ id }) => id);
  let shown = JSON.parse(session.view());
  const named = name => name.replace(/^seen_/, '');
  const caveats = values => ['secondhand', 'stale'].filter(value => values.includes(value));

  function dispatch(event) {
    if (!event || typeof event !== 'object' || Array.isArray(event)) throw new Error('Expected an event object.');
    const tunnel = () => {
      if (!tunnels.includes(event.tunnel)) throw new Error('Unknown tunnel.');
      return event.tunnel;
    };
    const condition = () => {
      if (!['clear', 'blocked'].includes(event.condition)) throw new Error('Expected clear or blocked.');
      return event.condition;
    };
    let type = event.type;
    let payload;
    switch (type) {
      case 'observe': {
        const target = tunnel();
        if (event.method !== 'report' && event.method !== 'scout') throw new Error('Unknown observation method.');
        if (event.method === 'scout' && 'condition' in event) throw new Error('A scout reads the tunnel itself.');
        payload = { target, method: event.method, condition: event.method === 'scout' ? 0 : ['clear', 'blocked'].indexOf(condition()) + 1 };
        break;
      }
      case 'change': payload = { target: tunnel(), condition: condition() }; break;
      case 'plan': payload = { target: tunnel() }; break;
      case 'tick':
        if (typeof event.dt !== 'number' || !Number.isFinite(event.dt)) throw new Error('Expected a finite time step.');
        type = 'advance';
        payload = { dt: event.dt };
        break;
      case 'rescue': payload = {}; break;
      default: throw new Error('Unknown event type.');
    }
    shown = JSON.parse(session.dispatch_view(type, JSON.stringify(payload)));
  }

  function view() {
    const { bindings, binding_explanations: cites, decision_journal: journal, decision_series: series } = shown;
    const ordered = observations.filter(id => bindings[id].observed)
      .sort((a, b) => bindings[a].order - bindings[b].order);
    // Grounds are sets; present their members in the source's acquisition order.
    const evidenceOrder = names => ordered.filter(id => names.some(name => named(name) === id));
    const current = series.route.current;
    const basis = journal.find(entry => entry.commitment === current && entry.change === 'committed');
    const reopening = journal.find(entry => entry.commitment === current && entry.change === 'reopened');
    return {
      ...bindings.hud,
      evidence: ordered.map(id => {
        const { tunnel, method, condition, at } = bindings[id];
        return { id, tunnel, method, condition, at, caveats: caveats(cites[id].condition.caveats) };
      }),
      tunnels: Object.fromEntries(tunnels.map(id => {
        const { status, canScout, canReport, canPlan } = bindings[id];
        return [id, { status,
          clearBy: evidenceOrder(cites[id].clearCount.evidence),
          blockedBy: evidenceOrder(cites[id].blockedCount.evidence),
          because: evidenceOrder(cites[id].status.evidence),
          caveats: caveats(cites[id].status.caveats), canScout, canReport, canPlan }];
      })),
      decision: {
        state: bindings.decision.state, tunnel: basis ? tunnels[basis.value - 1] : null,
        basis: basis?.because.map(named) ?? [], caveats: caveats(basis?.caveats ?? []),
        reopenedBy: reopening?.because.map(named) ?? [],
        history: journal.filter(entry => entry.decision === 'route').map(entry => ({
          change: entry.change, tunnel: tunnels[entry.value - 1], at: entry.elapsed,
          because: entry.because.map(named), caveats: caveats(entry.caveats),
        })),
      },
    };
  }

  return { dispatch, view, save: () => JSON.parse(session.save()), free: () => session.free() };
}
