//! Holds 0.1 under random histories: a hold keeps the current revision in
//! force and defers every cause that would have reopened it, once each; a
//! release reopens it because of exactly the deferred causes, in order; a
//! revision resting on withdrawn evidence is never held; the journal records
//! each hold, deferral and release; and a session restored from the live save
//! runs on exactly as the live one. See spec/caveat-holds-0.1.md.
use caveat_runtime::reactive::ReactiveSession;
use serde_json::Value;

const SOURCE: &str = r#"
claim passing;
evidence tool from "a tool result";
evidence correction from "a tool result found wrong";
evidence team from "the team's instruction";
evidence lead from "the lead's instruction";
readings tests from tool limit 64;
decisions gate limit 64 reopened by tests;
state h = 0;
state r = 0;
event report outcome in pass fail;
event decide;
event freeze;
event freeze_lead;
event unfreeze;
event retract;
event revisit;
event ask;
on report when outcome == outcome.pass sample tests = 1 supports passing;
on report when outcome == outcome.fail sample tests = 0 opposes passing;
on decide when not committed(gate) or reopened(gate) commit gate because enough using latest(tests);
on freeze when not observed(team) reveal team;
on freeze hold gate because team;
on freeze_lead when not observed(lead) reveal lead;
on freeze_lead hold gate because lead;
on unfreeze when observed(team) release gate because team;
on retract when not observed(correction) reveal correction;
on retract when has_sample(tests) and not withdrawn(latest(tests)) withdraw latest(tests) because correction;
on retract when committed(gate) and not reopened(gate) and in_basis(gate, latest(tests)) reopen gate because latest(tests);
on revisit when committed(gate) and not reopened(gate) and has_sample(tests) reopen gate because latest(tests);
on ask set h = if(on_hold(gate), 1, 0);
on ask set r = if(reopened(gate), 1, 0);
"#;

const EVENTS: [(&str, &str); 16] = [
    ("report", r#"{"outcome": "pass"}"#),
    ("report", r#"{"outcome": "pass"}"#),
    ("report", r#"{"outcome": "fail"}"#),
    ("decide", "{}"),
    ("decide", "{}"),
    ("decide", "{}"),
    ("freeze", "{}"),
    ("freeze", "{}"),
    ("freeze_lead", "{}"),
    ("unfreeze", "{}"),
    ("unfreeze", "{}"),
    ("retract", "{}"),
    ("revisit", "{}"),
    ("revisit", "{}"),
    ("ask", "{}"),
    ("ask", "{}"),
];

fn snapshot(game: &ReactiveSession) -> Value {
    serde_json::to_value(game.snapshot()).unwrap()
}

fn strings(list: &Value) -> Vec<String> {
    list.as_array()
        .map(|items| {
            items
                .iter()
                .map(|item| item.as_str().unwrap().to_string())
                .collect()
        })
        .unwrap_or_default()
}

fn readings(shown: &Value) -> Vec<String> {
    shown["reading_streams"]["tests"]["occurrences"]
        .as_array()
        .map(|list| {
            list.iter()
                .map(|item| item["id"].as_str().unwrap().to_string())
                .collect()
        })
        .unwrap_or_default()
}

fn current(shown: &Value) -> Option<String> {
    shown["decision_series"]["gate"]["current"]
        .as_str()
        .map(str::to_string)
}

fn open(shown: &Value, revision: &str) -> bool {
    shown["commitments"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["action"] == revision)
        .map(|item| item["open"] == true)
        .unwrap_or(true)
}

fn on_hold(shown: &Value) -> Vec<String> {
    strings(&shown["decision_series"]["gate"]["on_hold"])
}

fn deferred(shown: &Value) -> Vec<String> {
    strings(&shown["decision_series"]["gate"]["deferred"])
}

fn withdrawn(shown: &Value, id: &str) -> bool {
    shown["withdrawals"]
        .as_array()
        .into_iter()
        .flatten()
        .any(|record| record["evidence"] == id)
}

fn grounds(shown: &Value, revision: &str) -> Vec<String> {
    strings(&shown["commitment_grounds"][revision]["evidence"])
}

fn rests_on_withdrawn(shown: &Value, revision: &str) -> bool {
    grounds(shown, revision)
        .iter()
        .any(|evidence| withdrawn(shown, evidence))
}

/// The journal as (revision, change, because) triples.
fn journal(shown: &Value) -> Vec<(String, String, Vec<String>)> {
    shown["decision_journal"]
        .as_array()
        .unwrap()
        .iter()
        .map(|entry| {
            (
                entry["commitment"].as_str().unwrap().to_string(),
                entry["change"].as_str().unwrap().to_string(),
                strings(&entry["because"]),
            )
        })
        .collect()
}

fn value(shown: &Value, state: &str) -> f64 {
    shown["qualified_values"][state]["value"].as_f64().unwrap()
}

/// The model of the series: what the current revision is, whether it is in
/// force, what holds it and what waits on the hold.
#[derive(Default, Clone, Debug)]
struct Model {
    current: Option<String>,
    open: bool,
    on_hold: Vec<String>,
    deferred: Vec<String>,
    journal: Vec<(String, String, Vec<String>)>,
}

impl Model {
    fn in_force(&self) -> bool {
        self.current.is_some() && !self.open
    }

    fn entry(&mut self, change: &str, because: Vec<String>) {
        let revision = self.current.clone().unwrap();
        self.journal.push((revision, change.to_string(), because));
    }

    /// A reopening because of `cause`, as a rule or the trigger asks it of
    /// a revision in force (Holds 0.1, section 2).
    fn reopen(&mut self, cause: String, through: bool, counters: &mut Counters) {
        if self.on_hold.is_empty() {
            self.open = true;
            self.entry("reopened", vec![cause]);
            counters.plain_reopen += 1;
        } else if through {
            let mut causes = vec![cause];
            for waited in std::mem::take(&mut self.deferred) {
                if !causes.contains(&waited) {
                    causes.push(waited);
                }
            }
            self.on_hold.clear();
            self.open = true;
            self.entry("reopened", causes);
            counters.through += 1;
        } else if !self.deferred.contains(&cause) {
            self.deferred.push(cause.clone());
            self.entry("deferred", vec![cause]);
            counters.deferred += 1;
        }
    }

    fn hold(&mut self, reason: &str, counters: &mut Counters) {
        if !self.in_force() || self.on_hold.iter().any(|held| held == reason) {
            return;
        }
        self.on_hold.push(reason.to_string());
        self.entry("held", vec![reason.to_string()]);
        counters.held += 1;
    }

    fn release(&mut self, reason: &str, counters: &mut Counters) {
        if !self.in_force() || self.on_hold.is_empty() {
            return;
        }
        self.on_hold.clear();
        self.entry("released", vec![reason.to_string()]);
        let waited = std::mem::take(&mut self.deferred);
        if waited.is_empty() {
            counters.released_empty += 1;
        } else {
            self.open = true;
            self.entry("reopened", waited);
            counters.released_reopen += 1;
        }
    }
}

#[derive(Default)]
struct Counters {
    held: usize,
    deferred: usize,
    released_reopen: usize,
    released_empty: usize,
    through: usize,
    plain_reopen: usize,
    decided: usize,
    refused: usize,
    round_trips: usize,
}

#[test]
fn the_program_loads() {
    ReactiveSession::from_source(SOURCE).unwrap();
}

#[test]
fn a_hold_defers_every_reopening_and_a_release_applies_them() {
    let mut seed: u64 = 0x401d_5a7e_c0de_f00d;
    let mut next = move || {
        seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (seed >> 33) as usize
    };
    let mut counters = Counters::default();
    let mut accepted = 0usize;
    for run in 0..200 {
        let mut game = ReactiveSession::from_source(SOURCE).unwrap();
        let mut model = Model::default();
        let (mut team_seen, mut lead_seen, mut correction_seen) = (false, false, false);
        let mut before = snapshot(&game);
        for step in 0..60 {
            let at = format!("run {run} step {step}");
            let pick = next() % (EVENTS.len() + 2);
            if pick >= EVENTS.len() {
                let save = game.save_json().unwrap();
                let restored = ReactiveSession::restore_json(SOURCE, &save)
                    .unwrap_or_else(|error| panic!("{at}: restore: {error}"));
                assert_eq!(restored.save_json().unwrap(), save, "{at}: save round trip");
                assert_eq!(snapshot(&restored), before, "{at}: restored snapshot");
                game = restored;
                counters.round_trips += 1;
                continue;
            }
            let (event, payload) = EVENTS[pick];
            let save_before = game.save_json().unwrap();
            let mut mirror = ReactiveSession::restore_json(SOURCE, &save_before).unwrap();
            let outcome: Value =
                serde_json::to_value(game.dispatch_outcome_json(event, payload).unwrap()).unwrap();
            let mirrored: Value =
                serde_json::to_value(mirror.dispatch_outcome_json(event, payload).unwrap())
                    .unwrap();
            assert_eq!(
                mirrored, outcome,
                "{at}: the restored session ran differently"
            );

            let latest = readings(&before).last().cloned();
            let decide_fires = !model.current.is_some() || model.open;
            let expected: Option<&str> = match event {
                "decide" if decide_fires && latest.is_none() => Some("expression"),
                "freeze" | "freeze_lead" if model.current.is_none() => Some("not_committed"),
                _ => None,
            };
            if let Some(code) = expected {
                assert_eq!(outcome["outcome"], "rejected", "{at}: {event}: {outcome}");
                assert_eq!(outcome["code"], code, "{at}: {event}: {outcome}");
                assert_eq!(
                    game.save_json().unwrap(),
                    save_before,
                    "{at}: a refusal changed the save"
                );
                counters.refused += 1;
                continue;
            }
            assert_eq!(
                outcome["outcome"], "accepted",
                "{at}: {event} {payload}: {outcome}"
            );
            accepted += 1;
            let shown = snapshot(&game);
            assert_eq!(
                snapshot(&mirror),
                shown,
                "{at}: the restored session's snapshot differs"
            );
            // Whether a reopening breaks through a hold: the revision's
            // grounds hold withdrawn evidence, as of this event.
            let through = |model: &Model, shown: &Value| {
                model
                    .current
                    .as_deref()
                    .is_some_and(|revision| rests_on_withdrawn(shown, revision))
            };
            match event {
                "report" => {
                    let taken = readings(&shown).last().cloned().unwrap();
                    assert_ne!(Some(&taken), latest.as_ref(), "{at}: no new reading");
                    if model.in_force() {
                        let through = through(&model, &shown);
                        model.reopen(taken, through, &mut counters);
                    }
                }
                "decide" if decide_fires => {
                    let revision = current(&shown).unwrap();
                    assert_ne!(
                        Some(&revision),
                        model.current.as_ref(),
                        "{at}: no new revision"
                    );
                    model = Model {
                        current: Some(revision.clone()),
                        open: false,
                        on_hold: Vec::new(),
                        deferred: Vec::new(),
                        journal: model.journal,
                    };
                    model.entry("committed", grounds(&shown, &revision));
                    counters.decided += 1;
                }
                "freeze" => {
                    team_seen = true;
                    model.hold("team", &mut counters);
                }
                "freeze_lead" => {
                    lead_seen = true;
                    model.hold("lead", &mut counters);
                }
                "unfreeze" if team_seen => model.release("team", &mut counters),
                "retract" => {
                    correction_seen = true;
                    if let Some(latest) = latest.clone() {
                        let was_withdrawn = withdrawn(&before, &latest);
                        assert!(withdrawn(&shown, &latest), "{at}: {latest} not withdrawn");
                        let _ = was_withdrawn;
                        let in_basis = model
                            .current
                            .as_deref()
                            .is_some_and(|revision| grounds(&shown, revision).contains(&latest));
                        if model.in_force() && in_basis {
                            // The grounds now hold a withdrawn reading, so
                            // the reopening is never held back.
                            assert!(through(&model, &shown), "{at}: basis withdrawn");
                            model.reopen(latest, true, &mut counters);
                        }
                    }
                }
                "revisit" => {
                    if let Some(latest) = latest.clone() {
                        if model.in_force() {
                            let through = through(&model, &shown);
                            model.reopen(latest, through, &mut counters);
                        }
                    }
                }
                "ask" => {
                    assert_eq!(
                        value(&shown, "h"),
                        if model.in_force() && !model.on_hold.is_empty() {
                            1.0
                        } else {
                            0.0
                        },
                        "{at}: on_hold(gate)"
                    );
                    assert_eq!(
                        value(&shown, "r"),
                        if model.open { 1.0 } else { 0.0 },
                        "{at}: reopened(gate)"
                    );
                }
                _ => {}
            }
            let _ = (lead_seen, correction_seen);
            // The series as the snapshot shows it.
            assert_eq!(current(&shown), model.current, "{at}: current revision");
            if let Some(revision) = &model.current {
                assert_eq!(open(&shown, revision), model.open, "{at}: {revision} open");
            }
            assert_eq!(on_hold(&shown), model.on_hold, "{at}: on_hold");
            assert_eq!(deferred(&shown), model.deferred, "{at}: deferred");
            assert!(
                model.on_hold.is_empty() || !model.open,
                "{at}: a reopened revision is held"
            );
            assert!(
                model.deferred.is_empty() || !model.on_hold.is_empty(),
                "{at}: deferred causes with no hold"
            );
            assert_eq!(journal(&shown), model.journal, "{at}: journal");
            before = shown;
        }
    }
    let Counters {
        held,
        deferred,
        released_reopen,
        released_empty,
        through,
        plain_reopen,
        decided,
        refused,
        round_trips,
    } = counters;
    eprintln!("{accepted} accepted events, {refused} refused, {decided} decisions, {held} holds, {deferred} deferrals, {released_reopen} releases that reopened, {released_empty} with nothing deferred, {through} reopenings through a hold, {plain_reopen} plain reopenings, {round_trips} round trips");
    assert!(
        accepted > 5000
            && refused > 300
            && decided > 500
            && held > 230
            && deferred > 170
            && released_reopen > 60
            && released_empty > 40
            && through > 50
            && plain_reopen > 380
            && round_trips > 650,
        "coverage"
    );
}
