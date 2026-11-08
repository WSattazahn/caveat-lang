//! D054: restore accepted a departure marker in a provenance that no rule or
//! declaration of the program can carry its history's records into. When a
//! record of a windowed history departs, each provenance that named it keeps
//! a marker of that history in place of the name, so a marker can only stand
//! where some rule, procedure, reopening trigger or declaration carries that
//! history's records. Restore checked a marker's range against what had
//! departed, but not where it stood: a program whose `b` readings are
//! sampled without reading `a` restored a commitment on `latest(b)` holding a
//! marker of `a`'s departed readings, copied from the save of a program whose
//! sample guard reads `a`. With that save's grounds and journal caveats too,
//! which D049 accepts with the grounds' marker, the restored commitment
//! carried a caveat no event of its own program could give it.
//!
//! Restore now bounds each part of what a session holds by the histories the
//! program's rules and declarations can carry into that part: a state's
//! lineage, and apart from it its grounds, which take what its initializer's
//! and its sets' values or citations ground on, never the guard of the rule
//! that sets it; a stream's readings, their observation and `observed`
//! records, and its selection, which only a skipped sample writes; an
//! evidence's observation and `observed` records; the `withdrawn` records of
//! an evidence or of a stream's readings, which take the guards of skipped
//! withdrawals; a commitment's or a series' basis, its grounds, which take
//! what its commits use, cite and give as reasons as they ground on, and its
//! `committed` and `reopened` records, which take the guards of skipped
//! commits and reopenings; a series' outcomes and the evidence they name,
//! its reopening records and the evidence that reopened it or that a hold
//! defers, the reasons of its hold, and its selection, which only a skipped
//! commit writes; the guards scheduled on each evidence, which take what its
//! `qualify ... after` rules evaluate; the withdrawals' reasons; and each
//! cue. A read carries the parts the runtime reads for it and no other part
//! of what it names: of a series or a commitment, a value read its bases,
//! `grounds_of` its grounds, `outcome` and `resolved` its outcomes,
//! `committed` and `in_basis` its bases, its reopening records and its
//! `committed` records, and `reopened` and `on_hold` those and its
//! `reopened` records, each with its selection; and a withdrawal predicate
//! the `withdrawn` records of what it asks about and the withdrawals'
//! reasons. Two reads take a name the session holds and read it through its
//! records: a `because caveated(STATE, CAVEAT)` each evidence the state's
//! grounds name, and `reopened` and `on_hold` each evidence that reopened
//! the commitment. A qualification's guard joins only the live records that
//! list what it qualifies. In a program with a window, a marker, a record's
//! name, or a cause a series or a withdrawal keeps is refused outside its
//! part's bound. Guards are not evaluated, so what some rule could carry
//! restores whether or not one did, and saves the runtime writes restore as
//! before. Each part's content moves only into parts whose bound contains
//! its own, so a session restored from a save within the bound stays within
//! it. See spec/caveat-save-0.1.md ("Changes").
use caveat_runtime::reactive::ReactiveSession;
use serde_json::{json, Value};

fn send(session: &mut ReactiveSession, event: &str, payload: &str) -> Value {
    let outcome = session
        .dispatch_outcome_json(event, payload)
        .unwrap_or_else(|fatal| panic!("{event} was fatal: {}", fatal.message));
    let outcome = serde_json::to_value(outcome).unwrap();
    assert_eq!(outcome["outcome"], "accepted", "{event}: {outcome}");
    outcome
}

/// Plays `events`. After each, the save restores and saves the same bytes
/// again, and every session restored so far answers the events still to
/// come as the live one does. Returns the last save.
fn plays_and_restores(source: &str, events: &[(&str, &str)]) -> Value {
    let live = ReactiveSession::from_source(source).unwrap();
    plays_from(source, live, events)
}

/// Restores `save` and plays `events` on, as `plays_and_restores` does.
fn plays_on(source: &str, save: &Value, events: &[(&str, &str)]) -> Value {
    let restored = ReactiveSession::restore_json(source, &save.to_string())
        .unwrap_or_else(|error| panic!("refused: {error}"));
    plays_from(source, restored, events)
}

fn plays_from(source: &str, mut live: ReactiveSession, events: &[(&str, &str)]) -> Value {
    let mut restored: Vec<ReactiveSession> = Vec::new();
    for (index, (event, payload)) in events.iter().enumerate() {
        let answer = send(&mut live, event, payload);
        for run in &mut restored {
            assert_eq!(send(run, event, payload), answer, "{index}: {event}");
        }
        let save = live.save_json().unwrap();
        let again = ReactiveSession::restore_json(source, &save)
            .unwrap_or_else(|error| panic!("{index}: {event}: {error}"));
        assert_eq!(again.save_json().unwrap(), save, "{index}: {event}");
        restored.push(again);
    }
    let save = live.save_json().unwrap();
    for run in &restored {
        assert_eq!(run.save_json().unwrap(), save);
    }
    serde_json::from_str(&save).unwrap()
}

fn refused(source: &str, save: &Value, expected: &str) {
    match ReactiveSession::restore_json(source, &save.to_string()) {
        Err(error) => assert_eq!(error, format!("cannot restore save: {expected}")),
        Ok(_) => panic!("accepted {save}"),
    }
}

fn no_rule_carries(place: &str, history: &str) -> String {
    format!("{place}: no rule or declaration of this program carries records of {history} into it")
}

/// `save` with `causes`, in observation order, as the causes of the last
/// `change` entry (`held` or `deferred`) of series `series`' current
/// revision, and with the series' `on_hold` or `deferred` and the revision's
/// reopening record made to agree. Restore holds the three to each other
/// (D062), so a cause only the series lists is refused there first ("decision
/// series r holds a@2, which its journal does not"), and one only the entry
/// cites is refused for the record that leaves it out.
fn with_causes(save: &Value, series: &str, change: &str, causes: &[&str]) -> Value {
    let mut forged = save.clone();
    let revision = forged["decision_series"][series]["current"]
        .as_str()
        .unwrap()
        .to_string();
    let entry = forged["decision_journal"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .rev()
        .find(|entry| entry["change"] == change && entry["commitment"] == revision.as_str())
        .unwrap();
    entry["because"] = json!(causes);
    let list = if change == "held" {
        "on_hold"
    } else {
        "deferred"
    };
    forged["decision_series"][series][list] = json!(causes);
    for cause in causes {
        add_to_record(&mut forged, &revision, cause);
    }
    forged
}

/// Adds `cause` to the evidence of `revision`'s reopening record, which
/// holds every cause a `held`, `deferred`, `released` or `reopened` entry of
/// the revision cites (D062).
fn add_to_record(save: &mut Value, revision: &str, cause: &str) {
    let record = &mut save["reopening_qualifications"][revision];
    let mut evidence = record["evidence"]
        .as_array()
        .map(|names| {
            names
                .iter()
                .map(|name| name.as_str().unwrap().to_string())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    evidence.push(cause.to_string());
    evidence.sort();
    evidence.dedup();
    record["evidence"] = json!(evidence);
}

/// Every provenance of `save`, as restore names it: the 14 holders of
/// spec/caveat-lineage-compaction-0.1.md.
fn places(save: &Value) -> Vec<(String, Value)> {
    let entries = |value: &Value| value.as_object().cloned().unwrap_or_default();
    let mut held: Vec<(String, Value)> = Vec::new();
    for (name, state) in entries(&save["states"]) {
        held.push((format!("state {name}"), state["lineage"].clone()));
        held.push((format!("state {name} grounds"), state["grounds"].clone()));
    }
    for (name, basis) in entries(&save["commitment_bases"]) {
        held.push((format!("commitment {name}"), basis["provenance"].clone()));
    }
    for (name, grounds) in entries(&save["commitment_grounds"]) {
        held.push((format!("commitment {name} grounds"), grounds));
    }
    for (name, stream) in entries(&save["reading_streams"]) {
        held.push((
            format!("reading stream {name}"),
            stream["selection_qualifications"].clone(),
        ));
        for reading in stream["occurrences"].as_array().unwrap() {
            held.push((
                format!("reading {}", reading["id"].as_str().unwrap()),
                reading["provenance"].clone(),
            ));
        }
    }
    for (name, series) in entries(&save["decision_series"]) {
        held.push((
            format!("decision series {name}"),
            series["selection_qualifications"].clone(),
        ));
        for revision in series["revisions"].as_array().unwrap() {
            held.push((
                format!("outcome of {}", revision["id"].as_str().unwrap()),
                revision["outcome"]["lineage"].clone(),
            ));
        }
    }
    for scheduled in save["scheduled_qualifications"]
        .as_array()
        .into_iter()
        .flatten()
    {
        held.push((
            format!(
                "scheduled qualification of {}",
                scheduled["evidence"].as_str().unwrap()
            ),
            scheduled["guard"].clone(),
        ));
    }
    for what in ["observation", "examination", "reopening"] {
        for (name, record) in entries(&save[format!("{what}_qualifications")]) {
            held.push((format!("{what} of {name}"), record));
        }
    }
    for (kind, records) in entries(&save["predicate_qualifications"]) {
        for (name, record) in entries(&records) {
            held.push((format!("{kind}({name})"), record));
        }
    }
    for (index, cue) in save["cues"].as_array().into_iter().flatten().enumerate() {
        held.push((
            format!("cue {}", cue.as_str().unwrap()),
            save["cue_qualifications"][index].clone(),
        ));
    }
    held
}

/// Each provenance of `save` that holds a marker, with the marker's history.
fn markers(save: &Value) -> Vec<(String, String)> {
    let mut found = Vec::new();
    for (place, provenance) in places(save) {
        for marker in provenance["departed"].as_array().into_iter().flatten() {
            found.push((
                place.clone(),
                marker["history"].as_str().unwrap().to_string(),
            ));
        }
    }
    found
}

fn holding(markers: &[(&str, &str)]) -> Vec<(String, String)> {
    markers
        .iter()
        .map(|(place, history)| (place.to_string(), history.to_string()))
        .collect()
}

/// Whether `name` is `history` or one of its records.
fn of_history(name: &str, history: &str) -> bool {
    name == history
        || name
            .strip_prefix(history)
            .is_some_and(|rest| rest.starts_with('@'))
}

/// Whether `provenance` names a record of `history` or holds a marker of it.
fn holds(provenance: &Value, history: &str) -> bool {
    provenance["evidence"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .any(|name| of_history(name, history))
        || provenance["departed"]
            .as_array()
            .into_iter()
            .flatten()
            .any(|marker| marker["history"] == history)
}

/// The places of `save` whose provenance names a record of `history` or
/// holds a marker of it.
fn holders(save: &Value, history: &str) -> Vec<String> {
    places(save)
        .into_iter()
        .filter(|(_, provenance)| holds(provenance, history))
        .map(|(place, _)| place)
        .collect()
}

/// Whether the provenance at `place` names a record of `history` or holds a
/// marker of it.
fn carries(save: &Value, place: &str, history: &str) -> bool {
    holders(save, history).iter().any(|at| at == place)
}

/// The grounds of state `name` as restore reads them: its lineage when the
/// save leaves them out, as it does when they are equal.
fn grounds_of(save: &Value, name: &str) -> Value {
    let state = &save["states"][name];
    if state["grounds"].is_null() {
        state["lineage"].clone()
    } else {
        state["grounds"].clone()
    }
}

/// `save` with what state `name`'s lineage holds of `history`, its records'
/// names and its markers, added to the state's grounds.
fn grounds_given(save: &Value, name: &str, history: &str) -> Value {
    let lineage = &save["states"][name]["lineage"];
    let mut grounds = grounds_of(save, name);
    let mut evidence = grounds["evidence"].as_array().cloned().unwrap_or_default();
    for record in lineage["evidence"].as_array().into_iter().flatten() {
        if of_history(record.as_str().unwrap(), history) && !evidence.contains(record) {
            evidence.push(record.clone());
        }
    }
    let mut departed = grounds["departed"].as_array().cloned().unwrap_or_default();
    for marker in lineage["departed"].as_array().into_iter().flatten() {
        if marker["history"] == history && !departed.contains(marker) {
            departed.push(marker.clone());
        }
    }
    if !evidence.is_empty() {
        grounds["evidence"] = json!(evidence);
    }
    if !departed.is_empty() {
        grounds["departed"] = json!(departed);
    }
    let mut forged = save.clone();
    forged["states"][name]["grounds"] = grounds;
    forged
}

/// What a dispatch answers: its outcome, or a fatal error's message.
fn reply(session: &mut ReactiveSession, event: &str, payload: &str) -> Result<Value, String> {
    session
        .dispatch_outcome_json(event, payload)
        .map(|outcome| serde_json::to_value(outcome).unwrap())
        .map_err(|fatal| fatal.message)
}

/// Plays 40 sessions of `source`, the same ones on every run: each is 4 to
/// 13 events drawn from `events`, each with one of its payloads, which the
/// program may reject. After every event the save restores and saves the
/// same bytes again, and the session restored from it answers the next
/// event as the live one does. `check` reads every save.
fn sessions(source: &str, events: &[(&str, &[&str])], mut check: impl FnMut(&Value)) {
    let mut seed = 0x9e37_79b9_7f4a_7c15_u64;
    let mut pick = |count: usize| {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        (seed % count as u64) as usize
    };
    for session in 0..40 {
        let mut live = ReactiveSession::from_source(source).unwrap();
        let mut restored: Option<ReactiveSession> = None;
        for step in 0..4 + pick(10) {
            let (event, payloads) = events[pick(events.len())];
            let payload = payloads[pick(payloads.len())];
            let answer = reply(&mut live, event, payload);
            if let Some(restored) = &mut restored {
                let again = reply(restored, event, payload);
                assert_eq!(again, answer, "{session}.{step}: {event}");
            }
            if answer.is_err() {
                break;
            }
            let save = live.save_json().unwrap();
            let again = ReactiveSession::restore_json(source, &save)
                .unwrap_or_else(|error| panic!("{session}.{step}: {event}: {error}"));
            assert_eq!(
                again.save_json().unwrap(),
                save,
                "{session}.{step}: {event}"
            );
            check(&serde_json::from_str(&save).unwrap());
            restored = Some(again);
        }
    }
}

const ONE: &str = r#"{"v":1}"#;
const TWO: &str = r#"{"v":2}"#;
const THREE: &str = r#"{"v":3}"#;
const TICK: &str = r#"{"dt":0.1}"#;
/// Payloads for the sessions: values on both sides of each guard.
const VALUES: &[&str] = &[r#"{"v":0}"#, ONE, THREE, r#"{"v":6}"#, r#"{"v":9}"#];
const NOTHING: &[&str] = &["{}"];

/// `lb` is sampled while `latest(a)` is small, so `b`'s reading and
/// selection carry `a`'s current reading and the caveat `k2` that qualifies
/// it, and so does `d`, which commits on `latest(b)` after that reading
/// departed.
const INHERITED: &str = "claim seen;
evidence glimpse from \"a glimpse\";
evidence g2 from \"g2\";
caveat k2 consequence material;
k2 qualifies glimpse;
readings a from glimpse window 1;
readings b from g2 window 2;
decisions d limit 4;
event la v min 0 max 10;
event lb v min 0 max 10;
event decide;
on la sample a = v supports seen;
on lb when latest(a) < 5 sample b = v supports seen;
on decide commit d because enough using latest(b);
";

/// The same, but `lb`'s guard reads only its payload: nothing carries `a`'s
/// records anywhere but into `a`.
const NOGUARD: &str = "claim seen;
evidence glimpse from \"a glimpse\";
evidence g2 from \"g2\";
caveat k2 consequence material;
k2 qualifies glimpse;
readings a from glimpse window 1;
readings b from g2 window 2;
decisions d limit 4;
event la v min 0 max 10;
event lb v min 0 max 10;
event decide;
on la sample a = v supports seen;
on lb when v < 5 sample b = v supports seen;
on decide commit d because enough using latest(b);
";

const PLAYED: [(&str, &str); 6] = [
    ("la", ONE),
    ("lb", r#"{"v":4}"#),
    ("la", r#"{"v":7}"#),
    ("lb", r#"{"v":6}"#),
    ("la", TWO),
    ("decide", "{}"),
];

#[test]
fn a_commitment_cannot_hold_a_marker_no_rule_carries_into_it() {
    let inherited = plays_and_restores(INHERITED, &PLAYED);
    assert_eq!(
        markers(&inherited),
        holding(&[
            ("commitment d@1", "a"),
            ("commitment d@1 grounds", "a"),
            ("reading stream b", "a"),
            ("reading b@1", "a"),
            ("observation of b@1", "a"),
        ])
    );
    let save = plays_and_restores(NOGUARD, &PLAYED);
    assert_eq!(markers(&save), holding(&[]));
    let expected = no_rule_carries("commitment d@1", "a");
    // The basis INHERITED's commitment holds.
    let mut v1 = save.clone();
    v1["commitment_bases"]["d@1"]["provenance"] =
        inherited["commitment_bases"]["d@1"]["provenance"].clone();
    refused(NOGUARD, &v1, &expected);
    // A marker of `a`'s two departed readings alone, in the basis and its
    // grounds.
    let marker = json!({"history": "a", "read": 1, "from": 1, "through": 2, "departed_at": 5});
    let mut v3 = save.clone();
    v3["commitment_bases"]["d@1"]["provenance"]["departed"] = json!([marker.clone()]);
    v3["commitment_grounds"]["d@1"]["departed"] = json!([marker]);
    refused(NOGUARD, &v3, &expected);
    // The basis with INHERITED's grounds and the caveat its journal entry
    // records, which the grounds' marker accounts for (D049).
    let mut v2 = v1.clone();
    v2["commitment_grounds"]["d@1"] = inherited["commitment_grounds"]["d@1"].clone();
    v2["decision_journal"][0]["caveats"] = json!(["k2"]);
    refused(NOGUARD, &v2, &expected);
}

/// A provenance of every kind the save holds, none of which any rule or
/// declaration carries `a`'s readings into. `a` samples unguarded and
/// nothing reads it; its first two readings have departed.
const ALL: &str = "budget 12;
claim seen;
evidence glimpse from \"a glimpse\";
evidence g2 from \"g2\";
evidence e from \"e\";
evidence e2 from \"e2\";
caveat k consequence low;
caveat shaky consequence low;
readings a from glimpse window 1;
readings b from g2 window 2;
decisions d limit 4;
decisions r limit 4;
state s = 0;
state t = 0;
cue note toast \"Noted\" 1;
event la v min 0 max 10;
event lb v min 0 max 10;
event go;
event decide;
event close;
event reopen;
event later;
event check;
event last;
event tick dt min 0 max 0.1;
clock tick every 0.1;
on la sample a = v supports seen;
on lb when v < 5 sample b = v supports seen;
on go reveal e supports seen;
on go when latest(b) > 100 reveal e2;
on go set s = latest(b) + 1;
on go set t = s because nothing;
on decide commit d because enough using latest(b);
on decide commit r because enough using s;
on close resolve d = 1 because e;
on reopen when committed(r) reopen r because latest(b);
on later when s > 0 qualify e with shaky after 0.2;
on check examine k cost 1;
on last when t > 0 emit note;
";

const ALL_PLAYED: [(&str, &str); 11] = [
    ("la", ONE),
    ("la", TWO),
    ("la", THREE),
    ("lb", r#"{"v":4}"#),
    ("go", "{}"),
    ("decide", "{}"),
    ("close", "{}"),
    ("reopen", "{}"),
    ("later", "{}"),
    ("check", "{}"),
    ("last", "{}"),
];

#[test]
fn no_provenance_can_hold_a_marker_no_rule_carries_into_it() {
    let save = plays_and_restores(ALL, &ALL_PLAYED);
    assert_eq!(markers(&save), holding(&[]));
    let marker = json!([{"history": "a", "read": 1, "from": 1, "through": 2, "departed_at": 3}]);
    for (holder, place) in [
        ("/states/s/lineage", "state s"),
        ("/commitment_bases/d@1/provenance", "commitment d@1"),
        (
            "/reading_streams/b/selection_qualifications",
            "reading stream b",
        ),
        ("/reading_streams/b/occurrences/0/provenance", "reading b@1"),
        (
            "/decision_series/d/selection_qualifications",
            "decision series d",
        ),
        (
            "/decision_series/d/revisions/0/outcome/lineage",
            "outcome of d@1",
        ),
        (
            "/scheduled_qualifications/0/guard",
            "scheduled qualification of e",
        ),
        ("/observation_qualifications/e", "observation of e"),
        ("/examination_qualifications/k", "examination of k"),
        ("/reopening_qualifications/r@1", "reopening of r@1"),
        ("/predicate_qualifications/observed/e2", "observed(e2)"),
        ("/cue_qualifications/0", "cue note"),
    ] {
        let mut forged = save.clone();
        forged.pointer_mut(holder).unwrap()["departed"] = marker.clone();
        refused(ALL, &forged, &no_rule_carries(place, "a"));
    }
    // Grounds lie within their lineage or basis, and a marker in grounds
    // alone is refused as before. With the marker in both, the lineage or
    // basis is what no rule carries it into.
    for (lineage, grounds, place) in [
        ("/states/t/lineage", "/states/t/grounds", "state t"),
        (
            "/commitment_bases/d@1/provenance",
            "/commitment_grounds/d@1",
            "commitment d@1",
        ),
    ] {
        let mut forged = save.clone();
        forged.pointer_mut(grounds).unwrap()["departed"] = marker.clone();
        refused(
            ALL,
            &forged,
            &format!("{place} grounds hold departed a records outside its lineage"),
        );
        forged.pointer_mut(lineage).unwrap()["departed"] = marker.clone();
        refused(ALL, &forged, &no_rule_carries(place, "a"));
    }
}

#[test]
fn no_provenance_can_name_a_record_no_rule_carries_into_it() {
    let save = plays_and_restores(ALL, &ALL_PLAYED);
    // `a@3` would leave a marker of `a` in the state when it departs.
    let mut forged = save.clone();
    forged["states"]["s"]["lineage"]["evidence"] = json!(["a@3", "b@1"]);
    refused(ALL, &forged, &no_rule_carries("state s", "a"));
    // `e` has no window, but a qualification of `e` would reach the state
    // through it, with whatever its guard holds.
    let mut forged = save.clone();
    forged["states"]["s"]["lineage"]["evidence"] = json!(["b@1", "e"]);
    refused(ALL, &forged, &no_rule_carries("state s", "e"));
}

/// No history has a window, so nothing departs and no marker can form:
/// restore reads no name here.
const WINDOWLESS: &str = "claim seen;
evidence e from \"e\";
evidence f from \"f\";
caveat shaky consequence low;
state s = 0;
event see;
event go;
event flag;
on see reveal e supports seen;
on see reveal f supports seen;
on go set s = qualified(1, f);
on flag qualify e with shaky;
";

#[test]
fn a_program_without_a_window_restores_any_name() {
    let save = plays_and_restores(WINDOWLESS, &[("see", "{}"), ("go", "{}")]);
    let mut forged = save.clone();
    forged["states"]["s"]["lineage"]["evidence"] = json!(["e", "f"]);
    let played = plays_on(WINDOWLESS, &forged, &[("flag", "{}"), ("go", "{}")]);
    assert_eq!(played["states"]["s"]["lineage"]["evidence"], json!(["f"]));
}

/// Two commits in one event: the journal keeps the second entry, and the
/// first departs.
const JOURNAL: &str = "claim seen;
journal window 1;
decisions d limit 4;
decisions r limit 4;
event decide;
on decide commit d because enough using 1;
on decide commit r because enough using 2;
";

/// The same, with an evidence named `journal` that a rule carries into `d`.
const JOURNAL_EVIDENCE: &str = "claim seen;
evidence journal from \"a journal\";
journal window 1;
decisions d limit 4;
decisions r limit 4;
event see;
event decide;
on see reveal journal supports seen;
on decide commit d because enough using qualified(1, journal);
on decide commit r because enough using 2;
";

#[test]
fn no_provenance_holds_a_marker_of_the_journal() {
    let mut save = plays_and_restores(JOURNAL, &[("decide", "{}")]);
    assert_eq!(save["journal_departed"], 1);
    // A journal entry's name never enters a provenance.
    save["commitment_bases"]["d@1"]["provenance"]["departed"] =
        json!([{"history": "journal", "read": 1, "from": 1, "through": 1, "departed_at": 1}]);
    refused(
        JOURNAL,
        &save,
        &no_rule_carries("commitment d@1", "journal"),
    );
    // Nor where a rule carries an evidence of that name.
    let mut save = plays_and_restores(JOURNAL_EVIDENCE, &[("see", "{}"), ("decide", "{}")]);
    assert_eq!(save["journal_departed"], 1);
    assert_eq!(
        save["commitment_bases"]["d@1"]["provenance"]["evidence"],
        json!(["journal"])
    );
    save["commitment_bases"]["d@1"]["provenance"]["departed"] =
        json!([{"history": "journal", "read": 1, "from": 1, "through": 1, "departed_at": 2}]);
    refused(
        JOURNAL_EVIDENCE,
        &save,
        &no_rule_carries("commitment d@1", "journal"),
    );
}

/// `go` sets `s` and `decide` commits under a guard on `latest(a)`, and a
/// skipped `reveal` keeps that guard in its predicate record. `copy`
/// commits on whether `d` is committed, which reads `d`'s basis.
const GUARDS: &str = "claim seen;
evidence glimpse from \"a glimpse\";
evidence e2 from \"e2\";
readings a from glimpse window 1;
decisions d limit 4;
decisions d2 limit 4;
state s = 0;
event la v min 0 max 10;
event go;
event decide;
event copy;
on la sample a = v supports seen;
on go when latest(a) > 0 set s = s + 1;
on go when latest(a) > 100 reveal e2;
on decide when latest(a) > 0 commit d because enough using 1;
on copy commit d2 because enough using if(committed(d), 1, 0);
";

#[test]
fn guards_carry_markers() {
    let save = plays_and_restores(
        GUARDS,
        &[
            ("la", ONE),
            ("go", "{}"),
            ("decide", "{}"),
            ("la", TWO),
            ("la", ONE),
        ],
    );
    assert_eq!(
        markers(&save),
        holding(&[
            ("state s", "a"),
            ("commitment d@1", "a"),
            ("observed(e2)", "a"),
        ])
    );
}

#[test]
fn a_marker_a_rule_could_carry_restores_though_no_event_carried_it() {
    // `decide` ran after `a`'s first two readings departed, so its guard
    // read only the third; it could have read the first two.
    let mut save = plays_and_restores(
        GUARDS,
        &[("la", ONE), ("la", TWO), ("la", ONE), ("decide", "{}")],
    );
    assert_eq!(markers(&save), holding(&[]));
    save["commitment_bases"]["d@1"]["provenance"]["departed"] =
        json!([{"history": "a", "read": 1, "from": 1, "through": 2, "departed_at": 3}]);
    // The restored session plays on, and the marker moves into `d2`, which
    // reads `d`: every save it writes restores.
    let played = plays_on(GUARDS, &save, &[("la", TWO), ("copy", "{}"), ("la", ONE)]);
    assert_eq!(
        markers(&played),
        holding(&[("commitment d2@1", "a"), ("commitment d@1", "a")])
    );
}

/// `lb`'s guard and `lc`'s `with` condition read `latest(a)`.
const SAMPLE_GUARDS: &str = "claim seen;
evidence glimpse from \"a glimpse\";
evidence g2 from \"g2\";
evidence g3 from \"g3\";
caveat k2 consequence material;
caveat k3 consequence low;
k2 qualifies glimpse;
readings a from glimpse window 1;
readings b from g2 window 2;
readings c from g3 window 2;
decisions d limit 4;
event la v min 0 max 10;
event lb v min 0 max 10;
event lc v min 0 max 10;
event decide;
on la sample a = v supports seen;
on lb when latest(a) < 5 sample b = v supports seen;
on lc sample c = v with k3 when latest(a) < 5;
on decide commit d because enough using latest(b);
";

#[test]
fn sample_guards_carry_markers() {
    let save = plays_and_restores(
        SAMPLE_GUARDS,
        &[
            ("la", ONE),
            ("lb", r#"{"v":4}"#),
            ("lc", r#"{"v":4}"#),
            ("la", r#"{"v":7}"#),
            ("lb", THREE),
            ("la", TWO),
            ("decide", "{}"),
        ],
    );
    assert_eq!(
        markers(&save),
        holding(&[
            ("commitment d@1", "a"),
            ("commitment d@1 grounds", "a"),
            ("reading stream b", "a"),
            ("reading b@1", "a"),
            ("reading c@1", "a"),
            ("observation of b@1", "a"),
            ("observation of c@1", "a"),
        ])
    );
}

/// `go` sets `s` from `latest(a)` without citing it, so nothing pins the
/// reading, and `decide` commits using `s` once it departed.
const USING: &str = "claim seen;
evidence glimpse from \"a glimpse\";
readings a from glimpse window 1;
decisions d limit 4;
state s = 0;
event la v min 0 max 10;
event go;
event decide;
on la sample a = v supports seen;
on go set s = latest(a) + 1 because nothing;
on decide commit d because enough using s;
";

#[test]
fn using_carries_markers() {
    let save = plays_and_restores(
        USING,
        &[("la", ONE), ("go", "{}"), ("la", TWO), ("decide", "{}")],
    );
    assert_eq!(
        markers(&save),
        holding(&[("state s", "a"), ("commitment d@1", "a")])
    );
}

/// `copy` sets `t` from `s` after the reading `s` was set from departed.
const SET: &str = "claim seen;
evidence glimpse from \"a glimpse\";
readings a from glimpse window 1;
state s = 0;
state t = 0;
event la v min 0 max 10;
event go;
event copy;
on la sample a = v supports seen;
on go set s = latest(a) + 1 because nothing;
on copy set t = s + 1;
";

#[test]
fn set_carries_markers() {
    let save = plays_and_restores(
        SET,
        &[("la", ONE), ("go", "{}"), ("la", TWO), ("copy", "{}")],
    );
    assert_eq!(
        markers(&save),
        holding(&[("state s", "a"), ("state t", "a")])
    );
}

/// `d` resolves under a guard on `latest(runs)`, whose reading departs at
/// the next; `copy2` then commits `d2` using the outcome.
const OUTCOME: &str = "claim c;
evidence e from \"e\";
evidence feed from \"feed\";
caveat k2 consequence material;
k2 qualifies feed;
readings runs from feed window 1;
decisions d limit 2;
decisions d2 limit 3;
event go;
event reading v min 0 max 9;
event close;
event copy2;
on go reveal e supports c;
on go commit d because enough using 1;
on reading sample runs = v;
on close when latest(runs) > 3 resolve d = 1 because e;
on copy2 when resolved(d) commit d2 because enough using outcome(d);
";

#[test]
fn outcomes_carry_markers() {
    let save = plays_and_restores(
        OUTCOME,
        &[
            ("go", "{}"),
            ("reading", r#"{"v":5}"#),
            ("close", "{}"),
            ("reading", TWO),
            ("copy2", "{}"),
        ],
    );
    assert_eq!(
        markers(&save),
        holding(&[
            ("commitment d2@1", "runs"),
            ("commitment d2@1 grounds", "runs"),
            ("outcome of d@1", "runs"),
        ])
    );
}

/// `note` sets `s` from the value it is passed, and `decide_on` commits the
/// series it is passed under a guard on the stream it is passed.
const PROCEDURES: &str = "claim seen;
evidence glimpse from \"a glimpse\";
readings a from glimpse window 1;
decisions d limit 4;
state s = 0;
event la v min 0 max 10;
event go;
event decide;
proc note(x) { set s = x because nothing; };
proc decide_on(r readings, c decisions) { when latest(r) > 0 commit c because enough using 1; };
on la sample a = v supports seen;
on go call note(latest(a));
on decide call decide_on(a, d);
";

#[test]
fn procedure_arguments_carry_markers() {
    let save = plays_and_restores(
        PROCEDURES,
        &[("la", ONE), ("go", "{}"), ("decide", "{}"), ("la", TWO)],
    );
    assert_eq!(
        markers(&save),
        holding(&[("state s", "a"), ("commitment d@1", "a")])
    );
}

/// `d`'s first revision commits under a guard on `latest(a)`, whose reading
/// then departs; the revision after the reopening takes its basis.
const PREDECESSOR: &str = "claim seen;
evidence glimpse from \"a glimpse\";
evidence e from \"e\";
readings a from glimpse window 1;
decisions d limit 4;
event la v min 0 max 10;
event go;
event see;
event re;
event again;
on la sample a = v supports seen;
on go when latest(a) > 0 commit d because enough using 1;
on see reveal e;
on re when committed(d) and not reopened(d) reopen d because e;
on again when reopened(d) commit d because enough using 2;
";

#[test]
fn a_predecessor_basis_carries_markers() {
    let save = plays_and_restores(
        PREDECESSOR,
        &[
            ("la", ONE),
            ("go", "{}"),
            ("la", TWO),
            ("see", "{}"),
            ("re", "{}"),
            ("again", "{}"),
        ],
    );
    assert_eq!(
        markers(&save),
        holding(&[
            ("commitment d@1", "a"),
            ("commitment d@2", "a"),
            ("reopening of d@1", "a"),
        ])
    );
}

/// `re` reopens `d` because of `b`'s reading, and `again` commits the next
/// revision, both unguarded: its basis takes the reopening, as `reopened(d)`
/// gives it, though nothing else of `d` reads `b`.
const REOPENED: &str = "claim seen;
evidence g2 from \"g2\";
readings b from g2 window 1;
decisions d limit 4;
event lb v min 0 max 10;
event go;
event re;
event again;
on lb sample b = v supports seen;
on go commit d because enough using 1;
on re reopen d because latest(b);
on again commit d because enough using 2;
";

#[test]
fn a_reopening_carries_its_cause_into_the_next_revision() {
    let save = plays_and_restores(
        REOPENED,
        &[
            ("lb", ONE),
            ("go", "{}"),
            ("re", "{}"),
            ("again", "{}"),
            ("lb", TWO),
            ("lb", THREE),
        ],
    );
    assert!(carries(&save, "reopening of d@1", "b"), "{save}");
    assert!(carries(&save, "commitment d@2", "b"), "{save}");
}

/// `go` sets `s` from the current occurrence of `e`, which departs at the
/// next renewal.
const RENEWABLE: &str = "claim seen;
evidence e from \"e\";
renewable e window 1;
state s = 0;
event see;
event renew_e;
event go;
on see reveal e supports seen;
on renew_e renew e;
on renew_e reveal e supports seen;
on go when observed(e) set s = qualified(1, e) because nothing;
";

#[test]
fn renewable_evidence_windows_carry_markers() {
    let save = plays_and_restores(
        RENEWABLE,
        &[
            ("see", "{}"),
            ("renew_e", "{}"),
            ("go", "{}"),
            ("renew_e", "{}"),
            ("renew_e", "{}"),
        ],
    );
    assert_eq!(markers(&save), holding(&[("state s", "e")]));
}

/// A reading of `pushes`, sampled under a guard on `latest(a)`, reopens
/// `merge` through the declared trigger.
const TRIGGERS: &str = "claim changed;
evidence glimpse from \"a glimpse\";
evidence push from \"a new commit\";
readings a from glimpse window 1;
readings pushes from push window 1;
decisions merge limit 8 reopened by pushes;
event la v min 0 max 10;
event pushed;
event decide;
on la sample a = v;
on decide commit merge because enough;
on pushed when latest(a) > 0 sample pushes = 1 supports changed;
";

#[test]
fn reopening_triggers_carry_markers() {
    let save = plays_and_restores(
        TRIGGERS,
        &[
            ("la", ONE),
            ("decide", "{}"),
            ("pushed", "{}"),
            ("la", TWO),
            ("pushed", "{}"),
        ],
    );
    assert_eq!(
        markers(&save),
        holding(&[
            ("reading pushes@1", "a"),
            ("observation of pushes@1", "a"),
            ("reopening of merge@1", "a"),
            ("reopened(merge@1)", "a"),
        ])
    );
}

/// `decide` commits under a guard on `latest(a)`, using only a constant.
const GUARDONLY: &str = "claim seen;
evidence glimpse from \"a glimpse\";
caveat k2 consequence material;
k2 qualifies glimpse;
readings a from glimpse window 1;
decisions d limit 4;
event la v min 0 max 10;
event decide;
on la sample a = v supports seen;
on decide when latest(a) < 5 commit d because enough using 1;
";

/// `go` sets `s` from `latest(a)` but grounds it on nothing.
const STATEGROUNDS: &str = "claim seen;
evidence glimpse from \"a glimpse\";
caveat k2 consequence material;
k2 qualifies glimpse;
readings a from glimpse window 1;
state s = 0;
event la v min 0 max 10;
event go;
on la sample a = v supports seen;
on go set s = latest(a) + 1 because nothing;
";

/// `note` sets `s` from its parameter, called with `latest(a)` under a guard
/// on `latest(c)`.
const PARAMETER: &str = "claim seen;
evidence glimpse from \"a glimpse\";
evidence glimpse2 from \"a second glimpse\";
readings a from glimpse window 1;
readings c from glimpse2 window 1;
state s = 0;
event la v min 0 max 10;
event lc v min 0 max 10;
event go;
proc note(x) { set s = x; };
on la sample a = v supports seen;
on lc sample c = v supports seen;
on go when latest(c) > 0 call note(latest(a));
";

#[test]
fn grounds_hold_only_what_their_rule_grounds_on() {
    // A commit's grounds are what it uses, cites and gives as reasons, never
    // its guard.
    let save = plays_and_restores(
        GUARDONLY,
        &[("la", ONE), ("decide", "{}"), ("la", TWO), ("la", THREE)],
    );
    assert_eq!(markers(&save), holding(&[("commitment d@1", "a")]));
    let marker = save["commitment_bases"]["d@1"]["provenance"]["departed"].clone();
    let grounds = no_rule_carries("commitment d@1 grounds", "a");
    let mut forged = save.clone();
    forged["commitment_grounds"]["d@1"] = json!({"departed": marker.clone()});
    refused(GUARDONLY, &forged, &grounds);
    // With the caveat the marker accounts for, as the journal entry records
    // it (D049).
    forged["commitment_grounds"]["d@1"]["caveats"] = json!(["k2"]);
    forged["decision_journal"][0]["caveats"] = json!(["k2"]);
    refused(GUARDONLY, &forged, &grounds);
    // With the journal entry as it was, the journal check refuses first.
    forged["decision_journal"][0]["caveats"] = json!([]);
    refused(
        GUARDONLY,
        &forged,
        "decision journal: commitment witnesses differ from its frozen grounds",
    );

    // A set's grounds are its value's, or with `because` its citations'.
    let save = plays_and_restores(
        STATEGROUNDS,
        &[("la", ONE), ("go", "{}"), ("la", TWO), ("la", THREE)],
    );
    assert_eq!(markers(&save), holding(&[("state s", "a")]));
    let marker = save["states"]["s"]["lineage"]["departed"].clone();
    let grounds = no_rule_carries("state s grounds", "a");
    let mut forged = save.clone();
    forged["states"]["s"]["grounds"] = json!({"caveats": ["k2"], "departed": marker});
    refused(STATEGROUNDS, &forged, &grounds);
    // Grounds a save leaves out are its lineage.
    let mut forged = save.clone();
    forged["states"]["s"]
        .as_object_mut()
        .unwrap()
        .remove("grounds");
    refused(STATEGROUNDS, &forged, &grounds);

    // A parameter grounds a value as its argument does, never on the call's
    // guard.
    let save = plays_and_restores(
        PARAMETER,
        &[
            ("la", ONE),
            ("lc", ONE),
            ("go", "{}"),
            ("la", TWO),
            ("lc", TWO),
        ],
    );
    assert_eq!(markers(&save), holding(&[("state s", "c")]));
    assert_eq!(save["states"]["s"]["grounds"], json!({"evidence": ["a@1"]}));
    let mut forged = save.clone();
    forged["states"]["s"]["grounds"]["departed"] =
        save["states"]["s"]["lineage"]["departed"].clone();
    refused(PARAMETER, &forged, &no_rule_carries("state s grounds", "c"));
}

/// `later` qualifies `e` after a delay under a guard on `latest(a)`, and
/// `later2` qualifies `e2` unguarded; `s` is set from `e2`.
const SCHED: &str = "claim seen;
evidence glimpse from \"a glimpse\";
evidence e from \"e\";
evidence e2 from \"e2\";
caveat k2 consequence material;
caveat shaky consequence low;
k2 qualifies glimpse;
readings a from glimpse window 1;
state s = 0;
event la v min 0 max 10;
event see;
event use;
event later;
event later2;
event tick dt min 0 max 0.1;
clock tick every 0.1;
on la sample a = v supports seen;
on see reveal e supports seen;
on see reveal e2 supports seen;
on use set s = qualified(1, e2) because nothing;
on later when latest(a) < 5 qualify e with shaky after 5;
on later2 qualify e2 with shaky after 5;
";

const SCHED_PLAYED: [(&str, &str); 7] = [
    ("see", "{}"),
    ("use", "{}"),
    ("la", ONE),
    ("later", "{}"),
    ("later2", "{}"),
    ("la", TWO),
    ("la", THREE),
];

/// `e` is revealed under a guard on `latest(a)`, so its records carry `a`;
/// `later` qualifies it unguarded.
const SCHED_RECORDS: &str = "claim seen;
evidence glimpse from \"a glimpse\";
evidence e from \"e\";
caveat shaky consequence low;
readings a from glimpse window 1;
event la v min 0 max 10;
event see;
event later;
event tick dt min 0 max 0.1;
clock tick every 0.1;
on la sample a = v supports seen;
on see when latest(a) > 0 reveal e supports seen;
on later qualify e with shaky after 0.2;
";

#[test]
fn a_scheduled_guard_holds_only_what_its_qualification_evaluates() {
    let save = plays_and_restores(SCHED, &SCHED_PLAYED);
    assert_eq!(
        markers(&save),
        holding(&[("scheduled qualification of e", "a")])
    );
    let scheduled = save["scheduled_qualifications"].as_array().unwrap();
    let at = |evidence: &str| {
        scheduled
            .iter()
            .position(|entry| entry["evidence"] == evidence)
            .unwrap()
    };
    // `e`'s guard moved onto `e2`'s, whose qualification reads nothing: when
    // it applied, `s` would take `a`'s marker and `k2`.
    let mut forged = save.clone();
    forged["scheduled_qualifications"][at("e2")]["guard"] = json!({
        "caveats": ["k2"],
        "departed": scheduled[at("e")]["guard"]["departed"].clone(),
    });
    refused(
        SCHED,
        &forged,
        &no_rule_carries("scheduled qualification of e2", "a"),
    );
    // The genuine guards apply as the clock runs, and every save restores.
    let played = plays_on(SCHED, &save, &[("tick", TICK); 60]);
    assert_eq!(played["scheduled_qualifications"], Value::Null);
    assert_eq!(
        played["states"]["s"]["lineage"],
        json!({"caveats": ["shaky"], "evidence": ["e2"]})
    );

    // A guard its qualification never reads, though `e`'s records carry `a`.
    let save = plays_and_restores(
        SCHED_RECORDS,
        &[("la", THREE), ("see", "{}"), ("later", "{}"), ("la", ONE)],
    );
    assert_eq!(markers(&save), holding(&[("observation of e", "a")]));
    let mut forged = save.clone();
    forged["scheduled_qualifications"][0]["guard"]["departed"] =
        save["observation_qualifications"]["e"]["departed"].clone();
    refused(
        SCHED_RECORDS,
        &forged,
        &no_rule_carries("scheduled qualification of e", "a"),
    );
}

/// `later_on` qualifies the evidence it is passed under a guard on
/// `latest(a)`; `mark` qualifies `f`'s first occurrence after a delay
/// `latest(a)` sets.
const SCHEDULED_FORMS: &str = "claim seen;
evidence glimpse from \"a glimpse\";
evidence e from \"e\";
evidence f from \"f\";
renewable f window 2;
caveat shaky consequence low;
readings a from glimpse window 1;
state s = 0;
state t = 0;
event la v min 0 max 10;
event see;
event renew_f;
event later;
event mark;
event tick dt min 0 max 0.1;
clock tick every 0.1;
proc later_on(x evidence) { when latest(a) < 5 qualify x with shaky after 0.2; };
on la sample a = v supports seen;
on see reveal e supports seen;
on see reveal f supports seen;
on see set s = qualified(1, e);
on see set t = qualified(1, f);
on renew_f renew f;
on renew_f reveal f supports seen;
on later call later_on(e);
on mark qualify occurrence(f, 1) with shaky after latest(a) / 10;
";

#[test]
fn scheduled_guards_carry_markers() {
    let save = plays_and_restores(
        SCHEDULED_FORMS,
        &[
            ("see", "{}"),
            ("la", THREE),
            ("later", "{}"),
            ("mark", "{}"),
            ("la", ONE),
        ],
    );
    assert_eq!(
        markers(&save),
        holding(&[
            ("scheduled qualification of e", "a"),
            ("scheduled qualification of f", "a"),
        ])
    );
    let played = plays_on(
        SCHEDULED_FORMS,
        &save,
        &[
            ("tick", TICK),
            ("tick", TICK),
            ("tick", TICK),
            ("tick", TICK),
            ("renew_f", "{}"),
            ("renew_f", "{}"),
        ],
    );
    assert_eq!(
        markers(&played),
        holding(&[("state s", "a"), ("state t", "a")])
    );
}

/// `b`'s readings take `latest(a)` as their value, under no guard.
const VALUE: &str = "claim seen;
evidence glimpse from \"a glimpse\";
evidence g2 from \"g2\";
caveat k2 consequence material;
k2 qualifies glimpse;
readings a from glimpse window 1;
readings b from g2 window 4;
event la v min 0 max 10;
event lb;
on la sample a = v supports seen;
on lb sample b = latest(a) supports seen;
";

/// `d` commits unguarded and resolves under a guard on `latest(a)`.
const RESOLVEGUARD: &str = "claim seen;
evidence glimpse from \"a glimpse\";
evidence e from \"e\";
caveat k2 consequence material;
k2 qualifies glimpse;
readings a from glimpse window 1;
decisions d limit 4;
event la v min 0 max 10;
event see;
event decide;
event close;
on la sample a = v supports seen;
on see reveal e supports seen;
on decide commit d because enough using 1;
on close when latest(a) < 5 resolve d = 1 because e;
";

const RESOLVEGUARD_PLAYED: [(&str, &str); 6] = [
    ("see", "{}"),
    ("decide", "{}"),
    ("la", ONE),
    ("close", "{}"),
    ("la", TWO),
    ("la", THREE),
];

#[test]
fn a_selection_holds_only_what_skipped_effects_leave() {
    // No sample of `b` has a guard, so nothing is ever left in its selection.
    let save = plays_and_restores(
        VALUE,
        &[("la", ONE), ("lb", "{}"), ("la", TWO), ("la", THREE)],
    );
    assert_eq!(
        markers(&save),
        holding(&[("reading b@1", "a"), ("observation of b@1", "a")])
    );
    let mut forged = save.clone();
    forged["reading_streams"]["b"]["selection_qualifications"] = json!({
        "caveats": ["k2"],
        "departed": save["reading_streams"]["b"]["occurrences"][0]["provenance"]["departed"].clone(),
    });
    refused(VALUE, &forged, &no_rule_carries("reading stream b", "a"));
    // No commit of `d` has a guard; only its resolve reads `a`.
    let save = plays_and_restores(RESOLVEGUARD, &RESOLVEGUARD_PLAYED);
    assert_eq!(markers(&save), holding(&[("outcome of d@1", "a")]));
    let mut forged = save.clone();
    forged["decision_series"]["d"]["selection_qualifications"]["departed"] =
        save["decision_series"]["d"]["revisions"][0]["outcome"]["lineage"]["departed"].clone();
    refused(
        RESOLVEGUARD,
        &forged,
        &no_rule_carries("decision series d", "a"),
    );
}

#[test]
fn a_basis_holds_only_what_its_commits_and_reopenings_carry() {
    // The outcome's marker of `a`, which only the resolve's guard carries.
    let save = plays_and_restores(RESOLVEGUARD, &RESOLVEGUARD_PLAYED);
    let marker =
        save["decision_series"]["d"]["revisions"][0]["outcome"]["lineage"]["departed"].clone();
    let expected = no_rule_carries("commitment d@1", "a");
    let mut forged = save.clone();
    forged["commitment_bases"]["d@1"]["provenance"]["departed"] = marker.clone();
    refused(RESOLVEGUARD, &forged, &expected);
    // With the caveat it accounts for, retained as a commit on that basis
    // would, and in the grounds and the journal entry too.
    forged["commitment_bases"]["d@1"]["provenance"]["caveats"] = json!(["k2"]);
    forged["graph"]["relations"]
        .as_array_mut()
        .unwrap()
        .push(json!(["d@1", "retains", "k2"]));
    refused(RESOLVEGUARD, &forged, &expected);
    forged["commitment_grounds"]["d@1"] = json!({"caveats": ["k2"], "departed": marker});
    forged["decision_journal"][0]["caveats"] = json!(["k2"]);
    refused(RESOLVEGUARD, &forged, &expected);
}

/// `d` and `r` commit with a permission whose scope reads `a`, which their
/// bases carry. No resolve, hold or reopening reads `a`, and no commit is
/// guarded, so nothing carries `a` into an outcome, a reopening or a cause,
/// though a read of either series takes it with the basis.
const PARTS: &str = "claim seen;
evidence glimpse from \"a glimpse\";
evidence g2 from \"g2\";
evidence e from \"e\";
readings a from glimpse window 1;
readings grants from g2 window 1;
decisions d limit 4;
decisions r limit 4;
event la v min 0 max 10;
event see;
event decide;
event close;
event freeze;
event re;
on la sample a = v supports seen;
on see reveal e supports seen;
on see sample grants = 3 supports seen;
on decide commit d because enough using 1 permitted by latest(grants) for latest(a);
on decide commit r because enough using 1 permitted by latest(grants) for latest(a);
on close resolve d = 1 because e;
on freeze hold r because e;
on re reopen d because e;
on re reopen r because e;
";

#[test]
fn outcomes_reopenings_and_causes_hold_only_what_their_rules_carry() {
    let save = plays_and_restores(
        PARTS,
        &[
            ("see", "{}"),
            ("la", THREE),
            ("decide", "{}"),
            ("close", "{}"),
            ("freeze", "{}"),
            ("re", "{}"),
            ("la", TWO),
        ],
    );
    assert_eq!(
        markers(&save),
        holding(&[("commitment d@1", "a"), ("commitment r@1", "a")])
    );
    let marker = save["commitment_bases"]["d@1"]["provenance"]["departed"].clone();
    let mut forged = save.clone();
    forged["decision_series"]["d"]["revisions"][0]["outcome"]["lineage"]["departed"] =
        marker.clone();
    refused(PARTS, &forged, &no_rule_carries("outcome of d@1", "a"));
    let mut forged = save.clone();
    forged["reopening_qualifications"]["d@1"]["departed"] = marker;
    refused(PARTS, &forged, &no_rule_carries("reopening of d@1", "a"));
    // `a@2`, the reading that departed `a@1`, as a cause.
    let mut forged = save.clone();
    forged["decision_series"]["d"]["revisions"][0]["outcome"]["because"] = json!(["e", "a@2"]);
    refused(
        PARTS,
        &forged,
        &no_rule_carries("outcome of d@1 because", "a"),
    );
    // A hold's reason or a deferred cause comes with its journal entry and
    // joins the reopening record (D062), and no rule carries `a` into that
    // record either, so the record is refused first.
    refused(
        PARTS,
        &with_causes(&save, "r", "held", &["e", "a@2"]),
        &no_rule_carries("reopening of r@1", "a"),
    );
    refused(
        PARTS,
        &with_causes(&save, "r", "deferred", &["e", "a@2"]),
        &no_rule_carries("reopening of r@1", "a"),
    );
    // The cause of `d`'s reopening, as the journal, the graph and the
    // reopening record hold it together (D062): the reopening reads only
    // `e`, though the basis holds `a`, so the record is refused first.
    let mut forged = save.clone();
    let entry = forged["decision_journal"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|entry| entry["change"] == "reopened" && entry["commitment"] == "d@1")
        .unwrap();
    entry["because"] = json!(["a@2"]);
    let relation = forged["graph"]["relations"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|relation| **relation == json!(["e", "reopens", "d@1"]))
        .unwrap();
    *relation = json!(["a@2", "reopens", "d@1"]);
    add_to_record(&mut forged, "d@1", "a@2");
    refused(PARTS, &forged, &no_rule_carries("reopening of d@1", "a"));
}

/// A permission's grant reaches the basis but not the grounds, so grounds
/// that name the grant, with the journal entry to match, are refused.
#[test]
fn a_permissions_grant_stays_out_of_the_grounds() {
    let save = plays_and_restores(PARTS, &[("see", "{}"), ("la", THREE), ("decide", "{}")]);
    assert!(carries(&save, "commitment d@1", "grants"), "{save}");
    let mut forged = save.clone();
    forged["commitment_grounds"]["d@1"] = json!({"evidence": ["grants@1"]});
    let entry = forged["decision_journal"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|entry| entry["change"] == "committed" && entry["commitment"] == "d@1")
        .unwrap();
    entry["because"] = json!(["grants@1"]);
    refused(
        PARTS,
        &forged,
        &no_rule_carries("commitment d@1 grounds", "grants"),
    );
}

/// `decide` commits `d` under a guard on `latest(a)` that holds only while
/// `a` is small, so a later `decide` is skipped and leaves its guard in the
/// series' selection; `close` resolves and `re` reopens with it.
const SELECTION: &str = "claim seen;
evidence glimpse from \"a glimpse\";
evidence e from \"e\";
readings a from glimpse window 1;
decisions d limit 4;
event la v min 0 max 10;
event see;
event decide;
event close;
event re;
on la sample a = v supports seen;
on see reveal e supports seen;
on decide when latest(a) < 2 commit d because enough using 1;
on close resolve d = 1 because e;
on re when committed(d) reopen d because e;
";

#[test]
fn a_series_selection_carries_markers() {
    let save = plays_and_restores(
        SELECTION,
        &[
            ("see", "{}"),
            ("la", ONE),
            ("decide", "{}"),
            ("la", r#"{"v":5}"#),
            ("decide", "{}"),
            ("close", "{}"),
            ("re", "{}"),
            ("la", r#"{"v":6}"#),
        ],
    );
    assert_eq!(
        markers(&save),
        holding(&[
            ("commitment d@1", "a"),
            ("decision series d", "a"),
            ("outcome of d@1", "a"),
            ("reopening of d@1", "a"),
        ])
    );
}

/// `close` resolves `d` because of `e`, `freeze` holds `r` because of `e`,
/// `re` reopens `r` because of `e`, deferred while it is held, and `pull`
/// withdraws `f` because of `e`. No rule selects a reading of `a` as a cause.
const CAUSES: &str = "claim seen;
evidence glimpse from \"a glimpse\";
evidence e from \"e\";
evidence f from \"f\";
readings a from glimpse window 1;
decisions d limit 4;
decisions r limit 4;
event la v min 0 max 10;
event see;
event decide;
event close;
event freeze;
event re;
event unfreeze;
event pull;
on la sample a = v supports seen;
on see reveal e supports seen;
on see reveal f supports seen;
on decide commit d because enough using 1;
on decide commit r because enough using 1;
on close resolve d = 1 because e;
on freeze hold r because e;
on re when committed(r) reopen r because e;
on unfreeze release r because e;
on pull withdraw f because e;
";

#[test]
fn a_cause_names_only_what_its_rule_selects() {
    let save = plays_and_restores(
        CAUSES,
        &[
            ("see", "{}"),
            ("la", ONE),
            ("decide", "{}"),
            ("close", "{}"),
            ("freeze", "{}"),
            ("re", "{}"),
            ("la", TWO),
        ],
    );
    let reading = "a@2";
    let mut forged = save.clone();
    forged["decision_series"]["d"]["revisions"][0]["outcome"]["because"] = json!(["e", reading]);
    refused(
        CAUSES,
        &forged,
        &no_rule_carries("outcome of d@1 because", "a"),
    );
    // A hold's reason or a deferred cause comes with its journal entry and
    // joins the reopening record (D062), and no rule carries `a` into that
    // record either, so the record is refused first.
    refused(
        CAUSES,
        &with_causes(&save, "r", "held", &["e", reading]),
        &no_rule_carries("reopening of r@1", "a"),
    );
    refused(
        CAUSES,
        &with_causes(&save, "r", "deferred", &["e", reading]),
        &no_rule_carries("reopening of r@1", "a"),
    );
    // A cause of a reopening the journal records, and a withdrawal's
    // reason, are refused by the checks of the journal and of the event's
    // withdrawals first, as before.
    let save = plays_on(CAUSES, &save, &[("unfreeze", "{}"), ("pull", "{}")]);
    let mut forged = save.clone();
    forged["graph"]["relations"]
        .as_array_mut()
        .unwrap()
        .push(json!([reading, "reopens", "r@1"]));
    refused(
        CAUSES,
        &forged,
        "decision journal: reopening history differs from the graph",
    );
    let mut forged = save.clone();
    forged["withdrawals"][0]["because"] = json!(reading);
    refused(
        CAUSES,
        &forged,
        "the withdrawal of f by pull: no withdraw of that event withdraws f because a@2",
    );
}

/// The same causes, each a reading of `a`.
const READING_CAUSES: &str = "claim seen;
evidence glimpse from \"a glimpse\";
evidence e from \"e\";
readings a from glimpse window 1;
decisions d limit 4;
decisions r limit 4;
state s = 0;
event la v min 0 max 10;
event see;
event decide;
event close;
event freeze;
event re;
event unfreeze;
event use;
on la sample a = v supports seen;
on see reveal e supports seen;
on decide commit d because enough using 1;
on decide commit r because enough using 1;
on close resolve d = 1 because latest(a);
on freeze hold r because latest(a);
on re when committed(r) reopen r because latest(a);
on unfreeze release r because e;
on use set s = outcome(d) + if(reopened(r), 1, 0);
";

#[test]
fn causes_carry_their_readings() {
    let save = plays_and_restores(
        READING_CAUSES,
        &[
            ("see", "{}"),
            ("la", ONE),
            ("decide", "{}"),
            ("close", "{}"),
            ("freeze", "{}"),
            ("la", TWO),
            ("re", "{}"),
            ("la", THREE),
            ("unfreeze", "{}"),
            ("use", "{}"),
            ("la", ONE),
        ],
    );
    let series = &save["decision_series"];
    assert_eq!(
        series["d"]["revisions"][0]["outcome"]["because"],
        json!(["a@1"])
    );
    assert!(save["graph"]["relations"]
        .as_array()
        .unwrap()
        .contains(&json!(["a@2", "reopens", "r@1"])));
    assert!(carries(&save, "state s", "a"));
}

/// Rules that could carry `a` where this session carried nothing: a
/// qualification of `e2` whose guard reads `a` but never holds, a commit of
/// `d` whose guard reads `a`, and a set of `s` whose value reads `a`
/// though the one that ran grounds it on nothing.
const WITHIN: &str = "claim seen;
evidence glimpse from \"a glimpse\";
evidence e from \"e\";
evidence e2 from \"e2\";
caveat k2 consequence material;
caveat shaky consequence low;
k2 qualifies glimpse;
readings a from glimpse window 1;
decisions d limit 4;
decisions d2 limit 4;
state s = 0;
state t = 0;
event la v min 0 max 10;
event see;
event use;
event later;
event later2;
event decide;
event close;
event re;
event again;
event go;
event cite;
event copy;
event tick dt min 0 max 0.1;
clock tick every 0.1;
on la sample a = v supports seen;
on see reveal e supports seen;
on see reveal e2 supports seen;
on use set t = qualified(1, e2) because nothing;
on later when latest(a) < 5 qualify e with shaky after 5;
on later2 qualify e2 with shaky after 5;
on later2 when latest(a) > 100 qualify e2 with shaky after 5;
on decide when latest(a) < 5 commit d because enough using 1;
on close resolve d = 1 because e;
on re when committed(d) reopen d because e;
on again when reopened(d) commit d because enough using 2;
on go set s = latest(a) + 1;
on cite set s = latest(a) + 1 because nothing;
on copy commit d2 because enough using s;
";

#[test]
fn a_save_within_the_bound_plays_on_within_it() {
    let save = plays_and_restores(
        WITHIN,
        &[
            ("see", "{}"),
            ("use", "{}"),
            ("la", ONE),
            ("later", "{}"),
            ("later2", "{}"),
            ("decide", "{}"),
            ("cite", "{}"),
            ("la", TWO),
            ("la", THREE),
        ],
    );
    assert_eq!(
        markers(&save),
        holding(&[
            ("state s", "a"),
            ("commitment d@1", "a"),
            ("scheduled qualification of e", "a"),
        ])
    );
    let marker = save["commitment_bases"]["d@1"]["provenance"]["departed"].clone();
    let scheduled = save["scheduled_qualifications"].as_array().unwrap();
    let on_e2 = scheduled
        .iter()
        .position(|entry| entry["evidence"] == "e2")
        .unwrap();
    // Each part takes the marker some rule could carry into it, though no
    // event of this session did.
    let mut forged = save.clone();
    forged["scheduled_qualifications"][on_e2]["guard"] =
        json!({"caveats": ["k2"], "departed": marker.clone()});
    forged["decision_series"]["d"]["selection_qualifications"] =
        json!({"caveats": ["k2"], "departed": marker.clone()});
    forged["states"]["s"]["grounds"] = json!({"departed": marker});
    // The restored session plays on: each part's content moves into parts
    // whose bound holds it, and every save it writes restores.
    let mut events = vec![
        ("close", "{}"),
        ("re", "{}"),
        ("again", "{}"),
        ("copy", "{}"),
    ];
    events.extend([("tick", TICK); 60]);
    events.push(("la", ONE));
    let played = plays_on(WITHIN, &forged, &events);
    for (place, history) in [
        ("outcome of d@1", "a"),
        ("reopening of d@1", "a"),
        ("commitment d@2", "a"),
        ("commitment d2@1 grounds", "a"),
        ("state t", "a"),
    ] {
        assert!(carries(&played, place, history), "{place}: {played}");
    }
}

/// One series per expression form, each reading `a` only inside that form.
const FORMS_HEAD: &str = "claim seen;
evidence glimpse from \"a glimpse\";
evidence g2 from \"g2\";
evidence e from \"e\";
readings a from glimpse window 1;
readings b from g2 window 2;
decisions d0 limit 4;
decisions d limit 4;
event la v min 0 max 10;
event lb v min 0 max 10;
event go;
event close;
event decide;
on la sample a = v supports seen;
on lb sample b = v supports seen;
on go reveal e supports seen;
";

#[test]
fn every_expression_form_carries_its_reads() {
    // Each form, with what `d0` commits using and resolves to.
    for (form, using, resolved) in [
        ("if(true, latest(a), 2)", "1", "1"),
        ("if(false, 1, latest(a))", "1", "1"),
        ("require(latest(a) > 0, 1)", "1", "1"),
        ("require(true, latest(a))", "1", "1"),
        ("qualified(latest(a), e)", "1", "1"),
        ("floor(latest(a))", "1", "1"),
        // `abs` is the prelude's, so it expands to a call of its body.
        ("abs(latest(a))", "1", "1"),
        ("-latest(a)", "1", "1"),
        ("only_if(latest(a) > 0, 1)", "1", "1"),
        ("only_if(true, latest(a))", "1", "1"),
        ("history_at(b, history_count(a) - 1)", "1", "1"),
        (
            "if(carries_any(reading(b, history_count(a))), 1, 2)",
            "1",
            "1",
        ),
        ("grounds_of(d0)", "latest(a)", "1"),
        ("outcome(d0)", "1", "latest(a)"),
        ("outcome(revision(d0, history_count(a)))", "1", "1"),
        (
            "if(in_basis(d0, e), 1, 0)",
            "latest(a) + qualified(1, e)",
            "1",
        ),
        (
            "if(in_basis(d0, reading(b, history_count(a))), 1, 0)",
            "1",
            "1",
        ),
    ] {
        let source = format!(
            "{FORMS_HEAD}on go commit d0 because enough using {using};
on close resolve d0 = {resolved} because e;
on decide commit d because enough using {form};
"
        );
        eprintln!("form {form}");
        let save = plays_and_restores(
            &source,
            &[
                ("la", THREE),
                ("lb", ONE),
                ("go", "{}"),
                ("close", "{}"),
                ("decide", "{}"),
                ("la", ONE),
            ],
        );
        assert!(carries(&save, "commitment d@1", "a"), "{form}: {save}");
    }
}

/// Two streams and a renewable evidence, for the selectors below.
const SELECTORS_HEAD: &str = "claim seen;
evidence glimpse from \"a glimpse\";
evidence g2 from \"g2\";
evidence e from \"e\";
evidence f from \"f\";
renewable e window 2;
caveat old consequence low;
readings a from glimpse window 1;
readings b from g2 window 2;
decisions d limit 4;
state s = 0;
event la v min 0 max 10;
event lb v min 0 max 10;
event see;
event pull;
event mark;
event flag;
event decide;
event close;
event reconsider;
event renew_e;
on la sample a = v supports seen;
on lb sample b = v supports seen;
on see reveal f supports seen;
on renew_e renew e;
on renew_e reveal e supports seen;
";

#[test]
fn every_selector_and_effect_expression_carries_its_reads() {
    // Each case's rules, its events between `a`'s first reading and its
    // second, and the provenances that hold a record or a marker of each
    // history the case carries.
    for (rules, events, holders) in [
        // A permission's scope.
        (
            "readings grants from g2 window 1;
on see sample grants = 3 supports seen;
on decide commit d because enough using 1 permitted by latest(grants) for latest(a);",
            &[("see", "{}"), ("decide", "{}")][..],
            &[("commitment d@1", "a")][..],
        ),
        // A resolved revision's index.
        (
            "on see reveal e supports seen;
on decide commit d because enough using 1;
on close resolve revision(d, history_count(a)) = 1 because e;",
            &[("see", "{}"), ("decide", "{}"), ("close", "{}")][..],
            &[("outcome of d@1", "a")][..],
        ),
        // A withdrawn occurrence's index, and a guarded withdrawal of one.
        (
            "on see reveal e supports seen;
on see set s = qualified(1, e);
on pull withdraw occurrence(e, history_count(a)) because f;",
            &[("see", "{}"), ("pull", "{}")][..],
            &[("state s", "a")][..],
        ),
        (
            "on see reveal e supports seen;
on see set s = qualified(1, e);
on pull when latest(a) > 0 withdraw occurrence(e, 1) because f;",
            &[("see", "{}"), ("pull", "{}")][..],
            &[("state s", "a")][..],
        ),
        // A guarded withdrawal of every reading of a stream's evidence.
        (
            "on see set s = latest(b);
on pull when latest(a) > 0 withdraw every g2 because f;",
            &[("lb", TWO), ("see", "{}"), ("pull", "{}")][..],
            &[("state s", "a")][..],
        ),
        // A qualified occurrence's index.
        (
            "on see reveal e supports seen;
on see set s = qualified(1, e);
on mark qualify occurrence(e, history_count(a)) with old;",
            &[("see", "{}"), ("mark", "{}")][..],
            &[("state s", "a")][..],
        ),
        // A resolve because an occurrence, with its index.
        (
            "on see reveal e supports seen;
on decide commit d because enough using 1;
on close resolve d = 1 because occurrence(e, history_count(a));",
            &[("see", "{}"), ("decide", "{}"), ("close", "{}")][..],
            &[("outcome of d@1", "a"), ("outcome of d@1", "e")][..],
        ),
        // A reopening because a state's caveated evidence.
        (
            "on see reveal e supports seen;
on see set s = latest(a) + qualified(1, e);
on decide commit d because enough using 1;
on flag qualify e with old;
on reconsider when committed(d) reopen d because caveated(s, old);",
            &[
                ("see", "{}"),
                ("decide", "{}"),
                ("flag", "{}"),
                ("reconsider", "{}"),
            ][..],
            &[("reopening of d@1", "a")][..],
        ),
        // The same, with `a` only in the state's lineage: the selector reads
        // the lineage, though the causes it names come from the grounds.
        (
            "on see reveal e supports seen;
on see when latest(a) > 0 set s = qualified(1, e);
on decide commit d because enough using 1;
on flag qualify e with old;
on reconsider when committed(d) reopen d because caveated(s, old);",
            &[
                ("see", "{}"),
                ("decide", "{}"),
                ("flag", "{}"),
                ("reconsider", "{}"),
            ][..],
            &[("reopening of d@1", "a")][..],
        ),
    ] {
        let source = format!("{SELECTORS_HEAD}{rules}\n");
        eprintln!("rules {rules}");
        let mut played = vec![("la", THREE)];
        played.extend_from_slice(events);
        played.push(("la", ONE));
        let save = plays_and_restores(&source, &played);
        for (place, history) in holders {
            assert!(carries(&save, place, history), "{rules}: {place}: {save}");
        }
    }
}

/// `d` commits using `s`, which `go` sets under a guard on `latest(a)`: the
/// basis takes `s`'s lineage, and the grounds take `s`'s grounds, which
/// hold nothing.
const USES_STATE: &str = "claim seen;
evidence glimpse from \"a glimpse\";
caveat k2 consequence material;
k2 qualifies glimpse;
readings a from glimpse window 1;
decisions d limit 4;
state s = 0;
event la v min 0 max 10;
event go;
event decide;
on la sample a = v supports seen;
on go when latest(a) < 5 set s = 1;
on decide commit d because enough using s;
";

/// Grounds take what a value's reads ground on, as `evaluate_grounds` reads
/// them, and not what their lineages hold: here, not the guard that set the
/// state a commit uses.
#[test]
fn commitment_grounds_take_what_a_used_state_grounds_on() {
    let save = plays_and_restores(
        USES_STATE,
        &[
            ("la", ONE),
            ("go", "{}"),
            ("decide", "{}"),
            ("la", TWO),
            ("la", THREE),
        ],
    );
    assert_eq!(
        markers(&save),
        holding(&[("state s", "a"), ("commitment d@1", "a")])
    );
    let marker = save["commitment_bases"]["d@1"]["provenance"]["departed"].clone();
    let expected = no_rule_carries("commitment d@1 grounds", "a");
    let mut forged = save.clone();
    forged["commitment_grounds"]["d@1"] = json!({"departed": marker});
    refused(USES_STATE, &forged, &expected);
    // With the caveat the marker accounts for, in the journal entry too,
    // which D049 accepts with the grounds' marker.
    forged["commitment_grounds"]["d@1"]["caveats"] = json!(["k2"]);
    forged["decision_journal"][0]["caveats"] = json!(["k2"]);
    refused(USES_STATE, &forged, &expected);
    let mut bases = 0;
    sessions(
        USES_STATE,
        &[("la", VALUES), ("go", NOTHING), ("decide", NOTHING)],
        |save| {
            let held = holders(save, "a");
            assert!(!held.iter().any(|at| at.ends_with(" grounds")), "{save}");
            bases += usize::from(held.iter().any(|at| at.starts_with("commitment ")));
        },
    );
    assert!(bases > 0);
}

/// `t` copies `s`, which `go` sets under a guard on `latest(a)`, and `u`
/// cites it; `check` reads `t`'s grounds through `has_caveat`.
const COPIES_STATE: &str = "claim seen;
evidence glimpse from \"a glimpse\";
caveat k2 consequence material;
k2 qualifies glimpse;
readings a from glimpse window 1;
state s = 0;
state t = 0;
state u = 0;
state w = 0;
event la v min 0 max 10;
event go;
event copy;
event cite;
event check;
on la sample a = v supports seen;
on go when latest(a) < 5 set s = 1;
on copy set t = s;
on cite set u = s + 0 because s;
on check when has_caveat(t, k2) set w = 1;
";

#[test]
fn state_grounds_take_what_a_copied_or_cited_state_grounds_on() {
    let save = plays_and_restores(
        COPIES_STATE,
        &[
            ("la", ONE),
            ("go", "{}"),
            ("copy", "{}"),
            ("cite", "{}"),
            ("la", TWO),
            ("la", THREE),
            ("check", "{}"),
        ],
    );
    // `t`'s grounds are `s`'s, which hold no caveat, so `check` leaves `w`.
    assert_eq!(save["states"]["w"]["value"], json!(0.0));
    for state in ["s", "t", "u"] {
        assert!(carries(&save, &format!("state {state}"), "a"), "{save}");
    }
    let marker = save["states"]["t"]["lineage"]["departed"].clone();
    let expected = no_rule_carries("state t grounds", "a");
    let mut forged = save.clone();
    forged["states"]["t"]["grounds"] = json!({"departed": marker.clone()});
    refused(COPIES_STATE, &forged, &expected);
    // With `k2` too, `check` would set `w`, which no session of the program
    // does.
    forged["states"]["t"]["grounds"]["caveats"] = json!(["k2"]);
    refused(COPIES_STATE, &forged, &expected);
    // Grounds a save leaves out are its lineage.
    let mut forged = save.clone();
    forged["states"]["t"]
        .as_object_mut()
        .unwrap()
        .remove("grounds");
    refused(COPIES_STATE, &forged, &expected);
    // A citation's grounds are the cited state's.
    let mut forged = save.clone();
    forged["states"]["u"]["grounds"] = json!({"caveats": ["k2"], "departed": marker});
    refused(
        COPIES_STATE,
        &forged,
        &no_rule_carries("state u grounds", "a"),
    );
    let mut lineages = 0;
    sessions(
        COPIES_STATE,
        &[
            ("la", VALUES),
            ("go", NOTHING),
            ("copy", NOTHING),
            ("cite", NOTHING),
            ("check", NOTHING),
        ],
        |save| {
            let held = holders(save, "a");
            assert!(!held.iter().any(|at| at.ends_with(" grounds")), "{save}");
            assert_ne!(save["states"]["w"]["value"], json!(1.0), "{save}");
            lineages += usize::from(held.iter().any(|at| at == "state t"));
        },
    );
    assert!(lineages > 0);
}

/// `note` sets `t` from its parameter, which `pass` gives `s`, set under a
/// guard on `latest(a)`.
const PASSES_STATE: &str = "claim seen;
evidence glimpse from \"a glimpse\";
caveat k2 consequence material;
k2 qualifies glimpse;
readings a from glimpse window 1;
state s = 0;
state t = 0;
event la v min 0 max 10;
event go;
event pass;
proc note(x) { set t = x; };
on la sample a = v supports seen;
on go when latest(a) < 5 set s = 1;
on pass call note(s);
";

#[test]
fn parameter_grounds_take_what_a_passed_state_grounds_on() {
    let save = plays_and_restores(
        PASSES_STATE,
        &[
            ("la", ONE),
            ("go", "{}"),
            ("pass", "{}"),
            ("la", TWO),
            ("la", THREE),
        ],
    );
    assert_eq!(
        markers(&save),
        holding(&[("state s", "a"), ("state t", "a")])
    );
    let mut forged = save.clone();
    forged["states"]["t"]["grounds"] = json!({
        "caveats": ["k2"],
        "departed": save["states"]["t"]["lineage"]["departed"].clone(),
    });
    refused(
        PASSES_STATE,
        &forged,
        &no_rule_carries("state t grounds", "a"),
    );
    let mut lineages = 0;
    sessions(
        PASSES_STATE,
        &[("la", VALUES), ("go", NOTHING), ("pass", NOTHING)],
        |save| {
            let held = holders(save, "a");
            assert!(!held.iter().any(|at| at.ends_with(" grounds")), "{save}");
            lineages += usize::from(held.iter().any(|at| at == "state t"));
        },
    );
    assert!(lineages > 0);
}

/// `e` is revealed under a guard on `latest(a)`, so its observation record
/// carries `a`; `s` and `d` read `qualified(1, e)`.
const QUALIFIES: &str = "claim seen;
evidence glimpse from \"a glimpse\";
evidence e from \"e\";
caveat k2 consequence material;
k2 qualifies glimpse;
readings a from glimpse window 1;
decisions d limit 4;
state s = 0;
event la v min 0 max 10;
event see;
event go;
event decide;
on la sample a = v supports seen;
on see when latest(a) < 5 reveal e supports seen;
on go set s = qualified(1, e);
on decide commit d because enough using qualified(1, e);
";

/// `qualified` gives grounds the evidence's name and the caveats on it, not
/// its observation record, which the lineage and the basis take.
#[test]
fn grounds_take_only_the_name_of_qualified_evidence() {
    let save = plays_and_restores(
        QUALIFIES,
        &[
            ("la", ONE),
            ("see", "{}"),
            ("go", "{}"),
            ("decide", "{}"),
            ("la", TWO),
            ("la", THREE),
        ],
    );
    assert_eq!(
        markers(&save),
        holding(&[
            ("state s", "a"),
            ("commitment d@1", "a"),
            ("observation of e", "a"),
        ])
    );
    let marker = save["observation_qualifications"]["e"]["departed"].clone();
    let mut forged = save.clone();
    forged["states"]["s"]["grounds"]["departed"] = marker.clone();
    forged["states"]["s"]["grounds"]["caveats"] = json!(["k2"]);
    refused(QUALIFIES, &forged, &no_rule_carries("state s grounds", "a"));
    let mut forged = save.clone();
    forged["commitment_grounds"]["d@1"]["departed"] = marker;
    forged["commitment_grounds"]["d@1"]["caveats"] = json!(["k2"]);
    forged["decision_journal"][0]["caveats"] = json!(["k2"]);
    refused(
        QUALIFIES,
        &forged,
        &no_rule_carries("commitment d@1 grounds", "a"),
    );
    let mut lineages = 0;
    sessions(
        QUALIFIES,
        &[
            ("la", VALUES),
            ("see", NOTHING),
            ("go", NOTHING),
            ("decide", NOTHING),
        ],
        |save| {
            let held = holders(save, "a");
            assert!(!held.iter().any(|at| at.ends_with(" grounds")), "{save}");
            lineages += usize::from(held.iter().any(|at| at == "state s"));
        },
    );
    assert!(lineages > 0);
}

/// `d` cites `committed(d0)` and `go` sets `s` from it; `d0` commits under
/// a guard on `latest(a)`.
const CITES_COMMITTED: &str = "claim seen;
evidence glimpse from \"a glimpse\";
caveat k2 consequence material;
k2 qualifies glimpse;
readings a from glimpse window 1;
decisions d0 limit 4;
decisions d limit 4;
state s = 0;
event la v min 0 max 10;
event c0;
event decide;
event go;
on la sample a = v supports seen;
on c0 when latest(a) < 5 commit d0 because enough using 1;
on decide commit d because enough citing committed(d0);
on go set s = if(committed(d0), 1, 0);
";

/// `committed` gives grounds the commitment's grounds, not its basis.
#[test]
fn grounds_take_what_a_read_commitment_grounds_on() {
    let save = plays_and_restores(
        CITES_COMMITTED,
        &[
            ("la", ONE),
            ("c0", "{}"),
            ("decide", "{}"),
            ("go", "{}"),
            ("la", TWO),
            ("la", THREE),
        ],
    );
    assert_eq!(
        markers(&save),
        holding(&[
            ("state s", "a"),
            ("commitment d0@1", "a"),
            ("commitment d@1", "a"),
        ])
    );
    let marker = save["commitment_bases"]["d@1"]["provenance"]["departed"].clone();
    let mut forged = save.clone();
    forged["commitment_grounds"]["d@1"] = json!({"caveats": ["k2"], "departed": marker.clone()});
    let entry = forged["decision_journal"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|entry| entry["commitment"] == "d@1")
        .unwrap();
    entry["caveats"] = json!(["k2"]);
    refused(
        CITES_COMMITTED,
        &forged,
        &no_rule_carries("commitment d@1 grounds", "a"),
    );
    let mut forged = save.clone();
    forged["states"]["s"]["grounds"] = json!({"caveats": ["k2"], "departed": marker});
    refused(
        CITES_COMMITTED,
        &forged,
        &no_rule_carries("state s grounds", "a"),
    );
    let mut bases = 0;
    sessions(
        CITES_COMMITTED,
        &[
            ("la", VALUES),
            ("c0", NOTHING),
            ("decide", NOTHING),
            ("go", NOTHING),
        ],
        |save| {
            let held = holders(save, "a");
            assert!(!held.iter().any(|at| at.ends_with(" grounds")), "{save}");
            bases += usize::from(held.iter().any(|at| at.starts_with("commitment d@")));
        },
    );
    assert!(bases > 0);
}

/// `check` examines `k` under a guard on `latest(a)`, and `go` sets `t` from
/// `examined(k)`.
const EXAMINES: &str = "budget 12;
claim seen;
evidence glimpse from \"a glimpse\";
caveat k consequence low;
caveat k2 consequence material;
k2 qualifies glimpse;
k qualifies glimpse;
readings a from glimpse window 1;
state t = 0;
event la v min 0 max 10;
event check;
event go;
on la sample a = v supports seen;
on check when latest(a) < 5 examine k cost 1;
on go set t = if(examined(k), 1, 0);
";

/// `examined` gives grounds the caveat's name, not its examination record.
#[test]
fn grounds_take_only_the_name_of_an_examined_caveat() {
    let save = plays_and_restores(
        EXAMINES,
        &[
            ("la", ONE),
            ("check", "{}"),
            ("go", "{}"),
            ("la", TWO),
            ("la", THREE),
        ],
    );
    assert_eq!(
        markers(&save),
        holding(&[("state t", "a"), ("examination of k", "a")])
    );
    let mut forged = save.clone();
    forged["states"]["t"]["grounds"]["departed"] =
        save["states"]["t"]["lineage"]["departed"].clone();
    refused(EXAMINES, &forged, &no_rule_carries("state t grounds", "a"));
    let mut lineages = 0;
    sessions(
        EXAMINES,
        &[("la", VALUES), ("check", NOTHING), ("go", NOTHING)],
        |save| {
            let held = holders(save, "a");
            assert!(!held.iter().any(|at| at.ends_with(" grounds")), "{save}");
            lineages += usize::from(held.iter().any(|at| at == "state t"));
        },
    );
    assert!(lineages > 0);
}

/// `c`'s sample reads nothing of `c`, and the stance caveat `k3` qualifies
/// only a reading sampled above 5. `decide` reads `latest(c)`, so `d`'s
/// basis takes it.
const STANCE: &str = "claim seen;
evidence glimpse from \"a glimpse\";
caveat k3 consequence low;
readings c from glimpse window 1;
decisions d limit 4;
state s = 0;
state w = 0;
event lc v min 0 max 10;
event decide;
event go;
event check;
on lc sample c = v with k3 when v > 5;
on decide when latest(c) < 100 commit d because enough using 1;
on go set s = latest(c);
on check when has_caveat(s, k3) set w = 1;
";

/// A reading lists its own name, and holds another record of its stream,
/// or a marker of one, only when what samples it reads the stream.
#[test]
fn a_reading_holds_no_marker_of_its_own_stream() {
    let save = plays_and_restores(
        STANCE,
        &[("lc", r#"{"v":7}"#), ("decide", "{}"), ("lc", ONE)],
    );
    assert_eq!(markers(&save), holding(&[("commitment d@1", "c")]));
    let marker = save["commitment_bases"]["d@1"]["provenance"]["departed"].clone();
    // The current reading, sampled at 1, with the departed reading's marker,
    // and with the stance caveat that came with it.
    let expected = no_rule_carries("reading c@2", "c");
    let mut forged = save.clone();
    let reading = &mut forged["reading_streams"]["c"]["occurrences"][0]["provenance"];
    reading["departed"] = marker.clone();
    refused(STANCE, &forged, &expected);
    forged["reading_streams"]["c"]["occurrences"][0]["provenance"]["caveats"] = json!(["k3"]);
    refused(STANCE, &forged, &expected);
    let mut forged = save.clone();
    forged["observation_qualifications"]["c@2"] = json!({"caveats": ["k3"], "departed": marker});
    refused(STANCE, &forged, &no_rule_carries("observation of c@2", "c"));
    // Played on, `s` takes the current reading, which carries no `k3`.
    let played = plays_on(STANCE, &save, &[("go", "{}"), ("check", "{}")]);
    assert_eq!(
        played["states"]["s"]["lineage"],
        json!({"evidence": ["c@2"]})
    );
    assert_eq!(played["states"]["w"]["value"], json!(0.0));
    let mut bases = 0;
    sessions(
        STANCE,
        &[
            ("lc", VALUES),
            ("decide", NOTHING),
            ("go", NOTHING),
            ("check", NOTHING),
        ],
        |save| {
            for (place, history) in markers(save) {
                assert!(
                    !(history == "c"
                        && (place.starts_with("reading ") || place.starts_with("observation "))),
                    "{place}: {save}"
                );
                bases += usize::from(place.starts_with("commitment "));
            }
        },
    );
    assert!(bases > 0);
}

/// `later` qualifies `e` after a delay under a guard on `latest(a)`; `use`
/// sets `t` from `qualified(1, e)`, and `check` reads `t`'s caveats.
const SCHEDULED_JOINS: &str = "claim seen;
evidence glimpse from \"a glimpse\";
evidence e from \"e\";
caveat k2 consequence material;
caveat shaky consequence low;
k2 qualifies glimpse;
readings a from glimpse window 1;
state t = 0;
state w = 0;
event la v min 0 max 10;
event see;
event later;
event use;
event check;
event tick dt min 0 max 0.1;
clock tick every 0.1;
on la sample a = v supports seen;
on see reveal e supports seen;
on later when latest(a) < 5 qualify e with shaky after 0.3;
on use set t = qualified(1, e);
on check when has_caveat(t, k2) set w = 1;
";

/// When it applies, a scheduled guard joins what lists the evidence then,
/// as `apply_qualification` does: never the evidence's own observation
/// record, which a read of the evidence would take at once.
#[test]
fn a_scheduled_guard_joins_what_lists_its_evidence_not_the_evidence() {
    let save = plays_and_restores(
        SCHEDULED_JOINS,
        &[("see", "{}"), ("la", ONE), ("later", "{}"), ("la", TWO)],
    );
    assert_eq!(
        markers(&save),
        holding(&[("scheduled qualification of e", "a")])
    );
    let mut forged = save.clone();
    forged["observation_qualifications"]["e"] =
        save["scheduled_qualifications"][0]["guard"].clone();
    refused(
        SCHEDULED_JOINS,
        &forged,
        &no_rule_carries("observation of e", "a"),
    );
    let played = plays_on(
        SCHEDULED_JOINS,
        &save,
        &[
            ("use", "{}"),
            ("tick", TICK),
            ("tick", TICK),
            ("tick", TICK),
            ("tick", TICK),
        ],
    );
    assert_eq!(markers(&played), holding(&[("state t", "a")]));
    let mut guards = 0;
    sessions(
        SCHEDULED_JOINS,
        &[
            ("la", VALUES),
            ("see", NOTHING),
            ("later", NOTHING),
            ("use", NOTHING),
            ("check", NOTHING),
            ("tick", &[TICK]),
        ],
        |save| {
            let held = holders(save, "a");
            assert!(!held.iter().any(|at| at == "observation of e"), "{save}");
            guards += usize::from(held.iter().any(|at| at.starts_with("scheduled ")));
        },
    );
    assert!(guards > 0);
}

/// `b` is sampled, `p` committed again and `d` reopened again under guards
/// on `latest(c)`, which list `c`'s reading in `b`'s reading and its
/// observation record, and in `p`'s `committed` and `d@1`'s `reopened`
/// records when they are skipped. `q` qualifies that reading of `c` under a
/// guard on `latest(a)`.
const JOINED: &str = "claim seen;
evidence glimpse from \"a glimpse\";
evidence g2 from \"g2\";
evidence g3 from \"g3\";
evidence e from \"e\";
caveat k consequence low;
readings a from glimpse window 1;
readings b from g2 window 4;
readings c from g3 window 4;
decisions d limit 4;
event la v min 0 max 10;
event lb v min 0 max 10;
event lc v min 0 max 10;
event see;
event decide;
event re;
event try;
event q;
on la sample a = v supports seen;
on lc sample c = v supports seen;
on lb when latest(c) > 0 sample b = v supports seen;
on see reveal e supports seen;
on decide commit p because enough using 1;
on decide commit d because enough using 1;
on re reopen d because e;
on try when latest(c) > 100 commit p because enough using 1;
on try when latest(c) > 100 reopen d because e;
on q when latest(a) < 5 qualify latest(c) with k;
";

/// A qualification joins, with its guard, the observation and predicate
/// records that list what it qualifies, but not the readings, bases and
/// reopening records beside them, which keep what was read when they were
/// written.
#[test]
fn a_qualification_joins_live_records_and_not_the_frozen_ones_beside_them() {
    let save = plays_and_restores(
        JOINED,
        &[
            ("see", "{}"),
            ("lc", ONE),
            ("la", ONE),
            ("lb", ONE),
            ("decide", "{}"),
            ("re", "{}"),
            ("try", "{}"),
            ("q", "{}"),
            ("la", TWO),
        ],
    );
    assert_eq!(
        markers(&save),
        holding(&[
            ("observation of b@1", "a"),
            ("committed(p)", "a"),
            ("reopened(d@1)", "a"),
        ])
    );
    let marker = save["observation_qualifications"]["b@1"]["departed"].clone();
    for (pointer, place) in [
        ("/reading_streams/b/occurrences/0/provenance", "reading b@1"),
        ("/commitment_bases/p/provenance", "commitment p"),
        ("/reopening_qualifications/d@1", "reopening of d@1"),
    ] {
        let mut forged = save.clone();
        forged.pointer_mut(pointer).unwrap()["departed"] = marker.clone();
        refused(JOINED, &forged, &no_rule_carries(place, "a"));
    }
    // `q` and `try` twice over, and `c` above 0, so that the sessions reach
    // the joins often.
    let mut records = 0;
    sessions(
        JOINED,
        &[
            ("la", VALUES),
            ("lb", VALUES),
            ("lc", &[ONE, THREE]),
            ("see", NOTHING),
            ("decide", NOTHING),
            ("re", NOTHING),
            ("try", NOTHING),
            ("try", NOTHING),
            ("q", NOTHING),
            ("q", NOTHING),
        ],
        |save| {
            let held = holders(save, "a");
            assert!(
                !held.iter().any(|at| at.starts_with("reading b")
                    || at.starts_with("commitment ")
                    || at.starts_with("reopening ")),
                "{save}"
            );
            records += usize::from(held.iter().any(|at| {
                at.starts_with("observation of b")
                    || at.starts_with("committed(")
                    || at.starts_with("reopened(")
            }));
        },
    );
    assert!(records > 0);
}

/// `d` resolves, `r` is held, and both reopen, each because of `e` under a
/// guard on `latest(a)`: `d`'s outcome and reopening carry `a`, but no
/// `because` selects it.
const GUARDED_CAUSES: &str = "claim seen;
evidence glimpse from \"a glimpse\";
evidence e from \"e\";
readings a from glimpse window 1;
decisions d limit 4;
decisions r limit 4;
event la v min 0 max 10;
event see;
event decide;
event close;
event freeze;
event re;
on la sample a = v supports seen;
on see reveal e supports seen;
on decide commit d because enough using 1;
on decide commit r because enough using 1;
on close when latest(a) < 5 resolve d = 1 because e;
on freeze when latest(a) < 5 hold r because e;
on re when latest(a) < 5 reopen d because e;
on re when latest(a) < 5 reopen r because e;
";

/// A cause is bounded by what the `because` of a resolve, a reopening, a
/// hold or a release selects, not by what the rest of its outcome or
/// reopening carries.
#[test]
fn a_cause_names_only_what_a_because_selects() {
    let save = plays_and_restores(
        GUARDED_CAUSES,
        &[
            ("see", "{}"),
            ("la", ONE),
            ("decide", "{}"),
            ("close", "{}"),
            ("freeze", "{}"),
            ("re", "{}"),
            ("la", TWO),
        ],
    );
    assert!(carries(&save, "outcome of d@1", "a"), "{save}");
    assert!(carries(&save, "reopening of d@1", "a"), "{save}");
    let mut forged = save.clone();
    forged["decision_series"]["d"]["revisions"][0]["outcome"]["because"] = json!(["e", "a@2"]);
    refused(
        GUARDED_CAUSES,
        &forged,
        &no_rule_carries("outcome of d@1 because", "a"),
    );
    refused(
        GUARDED_CAUSES,
        &with_causes(&save, "r", "held", &["e", "a@2"]),
        &no_rule_carries("decision series r hold", "a"),
    );
    refused(
        GUARDED_CAUSES,
        &with_causes(&save, "r", "deferred", &["e", "a@2"]),
        &no_rule_carries("decision series r deferral", "a"),
    );
    let mut forged = save.clone();
    let entry = forged["decision_journal"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|entry| entry["change"] == "reopened" && entry["commitment"] == "d@1")
        .unwrap();
    entry["because"] = json!(["a@2"]);
    let relation = forged["graph"]["relations"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|relation| **relation == json!(["e", "reopens", "d@1"]))
        .unwrap();
    *relation = json!(["a@2", "reopens", "d@1"]);
    add_to_record(&mut forged, "d@1", "a@2");
    refused(
        GUARDED_CAUSES,
        &forged,
        &no_rule_carries("relation a@2 reopens d@1", "a"),
    );
}

/// `warn` emits `note` under a guard on `latest(a)`; `ping` emits `other`
/// unguarded.
const CUES: &str = "claim seen;
evidence glimpse from \"a glimpse\";
readings a from glimpse window 1;
cue note toast \"Noted\" 1;
cue other toast \"Other\" 1;
event la v min 0 max 10;
event warn;
event ping;
on la sample a = v supports seen;
on warn when latest(a) < 5 emit note;
on ping emit other;
";

/// Each cue holds the guards of its own emits.
#[test]
fn a_cue_holds_only_what_its_emits_read() {
    let save = plays_and_restores(CUES, &[("la", ONE), ("warn", "{}")]);
    assert!(carries(&save, "cue note", "a"), "{save}");
    let save = plays_and_restores(CUES, &[("la", ONE), ("la", TWO), ("ping", "{}")]);
    assert_eq!(save["cues"], json!(["other"]));
    let expected = no_rule_carries("cue other", "a");
    let mut forged = save.clone();
    forged["cue_qualifications"][0] = json!({"evidence": ["a@2"]});
    refused(CUES, &forged, &expected);
    forged["cue_qualifications"][0] = json!({"departed": [
        {"history": "a", "read": 1, "from": 1, "through": 1, "departed_at": 2},
    ]});
    refused(CUES, &forged, &expected);
    let mut notes = 0;
    sessions(
        CUES,
        &[("la", VALUES), ("warn", NOTHING), ("ping", NOTHING)],
        |save| {
            let held = holders(save, "a");
            assert!(!held.iter().any(|at| at == "cue other"), "{save}");
            notes += usize::from(held.iter().any(|at| at == "cue note"));
        },
    );
    assert!(notes > 0);
}

/// `c` is an action a rule commits, not a series. `decide` commits it using
/// `s`, which `prep` sets under a guard on `latest(a)`, and `re` reopens it
/// under one, so its basis, its reopening and a read of `c` carry `a`; `try`
/// would commit it again under a guard on `latest(b)` that never holds,
/// which its `committed` record keeps. Nothing carries `a` into that record.
const COMMITTED: &str = "claim seen;
evidence glimpse from \"a glimpse\";
evidence g2 from \"g2\";
evidence e from \"e\";
readings a from glimpse window 1;
readings b from g2 window 4;
state s = 0;
event la v min 0 max 10;
event lb v min 0 max 10;
event see;
event prep;
event decide;
event re;
event try;
on la sample a = v supports seen;
on lb sample b = v supports seen;
on see reveal e supports seen;
on prep when latest(a) < 5 set s = 1;
on decide commit c because enough using s;
on re when latest(a) < 5 reopen c because e;
on try when latest(b) > 100 commit c because enough using 1;
";

const COMMITTED_PLAYED: [(&str, &str); 9] = [
    ("see", "{}"),
    ("la", ONE),
    ("lb", ONE),
    ("prep", "{}"),
    ("decide", "{}"),
    ("re", "{}"),
    ("try", "{}"),
    ("la", TWO),
    ("la", THREE),
];

/// A `committed` record holds the guards of skipped commits, and what a
/// later qualification joins, but nothing else of its commitment; and a
/// skipped commit of a commitment leaves its guard there, not in a
/// selection the reopening would inherit.
#[test]
fn a_committed_record_holds_only_what_skipped_commits_leave() {
    let save = plays_and_restores(COMMITTED, &COMMITTED_PLAYED);
    assert_eq!(
        save["predicate_qualifications"]["committed"]["c"]["evidence"],
        json!(["b@1"])
    );
    let marker = save["reopening_qualifications"]["c"]["departed"].clone();
    assert_eq!(marker[0]["history"], "a", "{save}");
    let mut forged = save.clone();
    forged["predicate_qualifications"]["committed"]["c"]["departed"] = marker;
    refused(COMMITTED, &forged, &no_rule_carries("committed(c)", "a"));
    let mut forged = save.clone();
    // The reopened entry cites `e`, so the record keeps it (D062).
    forged["reopening_qualifications"]["c"]["evidence"] = json!(["b@1", "e"]);
    refused(COMMITTED, &forged, &no_rule_carries("reopening of c", "b"));
}

/// `d`'s bases carry `a` through the permission's scope; `poke` would reopen
/// `d` under a guard on `latest(b)` that never holds, which the `reopened`
/// record of `d@1` keeps.
const REOPENED_BASIS: &str = "claim seen;
evidence glimpse from \"a glimpse\";
evidence g2 from \"g2\";
evidence g3 from \"g3\";
evidence e from \"e\";
readings a from glimpse window 1;
readings grants from g2 window 1;
readings b from g3 window 4;
decisions d limit 4;
event la v min 0 max 10;
event lb v min 0 max 10;
event see;
event decide;
event poke;
on la sample a = v supports seen;
on lb sample b = v supports seen;
on see reveal e supports seen;
on see sample grants = 3 supports seen;
on decide commit d because enough using 1 permitted by latest(grants) for latest(a);
on poke when latest(b) > 100 reopen d because e;
";

/// A `reopened` record holds the guards of skipped reopenings, and what a
/// later qualification joins, but not the bases the next revision takes it
/// into.
#[test]
fn a_reopened_record_holds_only_what_skipped_reopenings_leave() {
    let save = plays_and_restores(
        REOPENED_BASIS,
        &[
            ("see", "{}"),
            ("la", THREE),
            ("lb", ONE),
            ("decide", "{}"),
            ("poke", "{}"),
            ("la", TWO),
            ("la", ONE),
        ],
    );
    assert_eq!(
        save["predicate_qualifications"]["reopened"]["d@1"]["evidence"],
        json!(["b@1"])
    );
    let mut forged = save.clone();
    forged["predicate_qualifications"]["reopened"]["d@1"]["departed"] =
        save["commitment_bases"]["d@1"]["provenance"]["departed"].clone();
    refused(
        REOPENED_BASIS,
        &forged,
        &no_rule_carries("reopened(d@1)", "a"),
    );
}

/// `decide` commits `d` under a guard on `latest(a)` that holds only while
/// `a` is small, so a later `decide` is skipped and leaves its guard in the
/// series' selection; `re` reopens `d` unguarded.
const SELECTION_UNGUARDED: &str = "claim seen;
evidence glimpse from \"a glimpse\";
evidence e from \"e\";
readings a from glimpse window 1;
decisions d limit 4;
event la v min 0 max 10;
event see;
event decide;
event re;
on la sample a = v supports seen;
on see reveal e supports seen;
on decide when latest(a) < 2 commit d because enough using 1;
on re reopen d because e;
";

/// A reopening inherits the series' selection, though its own rule reads
/// nothing of `a`.
#[test]
fn a_reopening_inherits_the_series_selection() {
    let save = plays_and_restores(
        SELECTION_UNGUARDED,
        &[
            ("see", "{}"),
            ("la", ONE),
            ("decide", "{}"),
            ("la", r#"{"v":5}"#),
            ("decide", "{}"),
            ("re", "{}"),
            ("la", r#"{"v":6}"#),
        ],
    );
    assert!(carries(&save, "reopening of d@1", "a"), "{save}");
}

/// `use` sets `s` from `b`'s current reading, and `pull` withdraws every
/// reading of `b` under a guard on `latest(a)`.
const WITHDRAW_EVERY: &str = "claim seen;
evidence glimpse from \"a glimpse\";
evidence g2 from \"g2\";
evidence e from \"e\";
readings a from glimpse window 1;
readings b from g2 window 4;
state s = 0;
event la v min 0 max 10;
event lb v min 0 max 10;
event see;
event use;
event pull;
on la sample a = v supports seen;
on lb sample b = v supports seen;
on see reveal e supports seen;
on use set s = latest(b);
on pull when latest(a) < 5 withdraw every b because e;
";

/// `withdraw every` joins its guard to what lists each reading it withdraws.
#[test]
fn a_withdrawal_of_every_reading_carries_its_guard() {
    let save = plays_and_restores(
        WITHDRAW_EVERY,
        &[
            ("see", "{}"),
            ("la", ONE),
            ("lb", ONE),
            ("use", "{}"),
            ("pull", "{}"),
            ("la", TWO),
            ("la", THREE),
        ],
    );
    assert!(carries(&save, "state s", "a"), "{save}");
}

/// `s` is set only from a payload; `clear` unsets it under a guard on
/// `latest(a)`, and `use` sets `t` under one.
const UNSET: &str = "claim seen;
evidence glimpse from \"a glimpse\";
readings a from glimpse window 1;
state s unset;
state t = 0;
event la v min 0 max 10;
event put v min 0 max 10;
event clear;
event use;
on la sample a = v supports seen;
on put set s = v;
on clear when latest(a) < 5 unset s;
on use when latest(a) < 5 set t = 1;
";

/// An unset writes its guard nowhere, so `a`'s marker in `s` is refused.
#[test]
fn an_unset_carries_nothing_into_its_state() {
    let save = plays_and_restores(
        UNSET,
        &[
            ("la", ONE),
            ("put", TWO),
            ("use", "{}"),
            ("clear", "{}"),
            ("put", THREE),
            ("la", TWO),
            ("la", THREE),
        ],
    );
    let marker = save["states"]["t"]["lineage"]["departed"].clone();
    assert_eq!(marker[0]["history"], "a", "{save}");
    let mut forged = save.clone();
    // Grounds given apart, so that only the lineage holds the marker.
    forged["states"]["s"]["grounds"] = json!({});
    forged["states"]["s"]["lineage"] = json!({"departed": marker});
    refused(UNSET, &forged, &no_rule_carries("state s", "a"));
}

/// `uq` takes `k` back from `b`'s current reading under a guard on
/// `latest(a)`, and `use` sets `t` under one.
const TAKEN_BACK: &str = "claim seen;
evidence glimpse from \"a glimpse\";
evidence g2 from \"g2\";
caveat k consequence low;
readings a from glimpse window 1;
readings b from g2 window 4;
state t = 0;
event la v min 0 max 10;
event lb v min 0 max 10;
event q;
event uq;
event use;
on la sample a = v supports seen;
on lb sample b = v supports seen;
on q qualify latest(b) with k;
on uq when latest(a) < 5 unqualify latest(b) with k;
on use when latest(a) < 5 set t = 1;
";

/// A take-back writes its guard nowhere, so `a`'s marker in `b`'s reading,
/// or in its observation record, is refused.
#[test]
fn a_take_back_carries_nothing_into_its_records() {
    let save = plays_and_restores(
        TAKEN_BACK,
        &[
            ("la", ONE),
            ("lb", ONE),
            ("use", "{}"),
            ("q", "{}"),
            ("uq", "{}"),
            ("la", TWO),
            ("la", THREE),
        ],
    );
    let marker = save["states"]["t"]["lineage"]["departed"].clone();
    assert_eq!(marker[0]["history"], "a", "{save}");
    let mut forged = save.clone();
    forged["reading_streams"]["b"]["occurrences"][0]["provenance"]["departed"] = marker.clone();
    refused(TAKEN_BACK, &forged, &no_rule_carries("reading b@1", "a"));
    let mut forged = save.clone();
    forged["observation_qualifications"]["b@1"]["departed"] = marker;
    refused(
        TAKEN_BACK,
        &forged,
        &no_rule_carries("observation of b@1", "a"),
    );
}

/// `now` qualifies `f` at once under a guard on `latest(a)`; `later`
/// schedules a qualification of `f` under a guard on `latest(c)`.
const SCHEDULED: &str = "claim seen;
evidence glimpse from \"a glimpse\";
evidence g2 from \"g2\";
evidence f from \"f\";
caveat k consequence low;
caveat k2 consequence low;
readings a from glimpse window 1;
readings c from g2 window 4;
state t = 0;
event la v min 0 max 10;
event lc v min 0 max 10;
event see;
event now;
event later;
event use;
event tick dt min 0 max 0.1;
clock tick every 0.1;
on la sample a = v supports seen;
on lc sample c = v supports seen;
on see reveal f supports seen;
on now when latest(a) < 5 qualify f with k;
on later when latest(c) > 0 qualify f with k2 after 100;
on use when latest(a) < 5 set t = 1;
";

/// What a qualification at once evaluates is not scheduled, so `a`'s marker
/// in `f`'s scheduled guard is refused.
#[test]
fn a_qualification_at_once_leaves_no_scheduled_guard() {
    let save = plays_and_restores(
        SCHEDULED,
        &[
            ("see", "{}"),
            ("la", ONE),
            ("lc", ONE),
            ("use", "{}"),
            ("now", "{}"),
            ("later", "{}"),
            ("la", TWO),
            ("la", THREE),
        ],
    );
    let marker = save["states"]["t"]["lineage"]["departed"].clone();
    assert_eq!(marker[0]["history"], "a", "{save}");
    let mut forged = save.clone();
    forged["scheduled_qualifications"][0]["guard"]["departed"] = marker;
    refused(
        SCHEDULED,
        &forged,
        &no_rule_carries("scheduled qualification of f", "a"),
    );
}

/// `decide` retains `k` because of `e`.
const RETAINED: &str = "claim seen;
evidence glimpse from \"a glimpse\";
evidence e from \"e\";
caveat k consequence low;
readings a from glimpse window 1;
decisions d limit 4;
event la v min 0 max 10;
event see;
event decide;
on la sample a = v supports seen;
on see reveal e supports seen;
on decide commit d because enough retaining k because e;
";

/// A retained caveat's reason joins the basis.
#[test]
fn a_retained_caveats_reason_joins_the_basis() {
    let save = plays_and_restores(
        RETAINED,
        &[("see", "{}"), ("la", ONE), ("decide", "{}"), ("la", TWO)],
    );
    assert!(carries(&save, "commitment d@1", "e"), "{save}");
}

/// `s`'s declaration reads `e` only in its `only_if` condition, and `use`
/// copies `s` into `t`, since a declared state is in the save only once a
/// rule sets it.
const DECLARED_ONLY_IF: &str = "claim seen;
evidence glimpse from \"a glimpse\";
evidence e from \"e\";
e supports seen;
readings a from glimpse window 1;
state s = only_if(qualified(1, e) > 0, 1);
state t = 0;
event la v min 0 max 10;
event use;
on la sample a = v supports seen;
on use set t = s;
";

/// A declaration's lineage takes what its condition reads, which its
/// grounds leave out.
#[test]
fn a_declarations_lineage_takes_its_condition() {
    let save = plays_and_restores(DECLARED_ONLY_IF, &[("la", ONE), ("use", "{}"), ("la", TWO)]);
    assert!(carries(&save, "state t", "e"), "{save}");
    assert!(!carries(&save, "state t grounds", "e"), "{save}");
}

/// `put` sets `s` from `latest(a)`, and `use` reads `is_set(s)`.
const IS_SET: &str = "claim seen;
evidence glimpse from \"a glimpse\";
readings a from glimpse window 1;
state s unset;
state t = 0;
event la v min 0 max 10;
event put;
event use;
on la sample a = v supports seen;
on put set s = latest(a);
on use set t = if(is_set(s), 1, 0);
";

/// `is_set` carries what its state holds.
#[test]
fn is_set_carries_what_its_state_holds() {
    let save = plays_and_restores(
        IS_SET,
        &[("la", ONE), ("put", "{}"), ("use", "{}"), ("la", TWO)],
    );
    assert!(carries(&save, "state t", "a"), "{save}");
}

/// `use2` folds `b`'s readings from `latest(a)`.
const FOLDED: &str = "claim seen;
evidence glimpse from \"a glimpse\";
evidence g2 from \"g2\";
readings a from glimpse window 2;
readings b from g2 window 2;
state t = 0;
event la v min 0 max 10;
event lb v min 0 max 10;
event use2;
fn sum(total, reading) = total + reading;
on la sample a = v supports seen;
on lb sample b = v supports seen;
on use2 set t = fold_history(b, latest(a), sum);
";

/// A fold carries its initial value.
#[test]
fn a_fold_carries_its_initial_value() {
    let save = plays_and_restores(
        FOLDED,
        &[
            ("la", ONE),
            ("lb", ONE),
            ("use2", "{}"),
            ("la", TWO),
            ("la", THREE),
        ],
    );
    assert!(carries(&save, "state t", "a"), "{save}");
}

/// `q` qualifies `c`'s current reading under a guard on `latest(a)`. Before
/// it, `see` reveals `e` and `check` examines `k2` under guards on
/// `latest(c)`, `try` would commit `p` under one that never holds, which
/// `p`'s `committed` record keeps, `close` resolves `o` because of `c`'s
/// reading, and `use` sets `s` from it. After it, `lb` samples `b` under a
/// guard on `committed(p)`, and `wait` schedules a qualification of `f` and
/// emits `ping` under guards on `latest(c)`.
const QUALIFIED_LATER: &str = "budget 12;
claim seen;
evidence glimpse from \"a glimpse\";
evidence g2 from \"g2\";
evidence g3 from \"g3\";
evidence e from \"e\";
evidence f from \"f\";
caveat k consequence low;
caveat k2 consequence low;
caveat k3 consequence low;
readings a from glimpse window 1;
readings b from g2 window 4;
readings c from g3 window 4;
decisions o limit 4;
state s = 0;
cue ping toast \"Ping\" 1;
event la v min 0 max 10;
event lb v min 0 max 10;
event lc v min 0 max 10;
event see;
event check;
event decide;
event try;
event close;
event use;
event q;
event wait;
event tick dt min 0 max 0.1;
clock tick every 0.1;
on la sample a = v supports seen;
on lc sample c = v supports seen;
on see when latest(c) > 0 reveal e supports seen;
on see reveal f supports seen;
on check when latest(c) > 0 examine k2 cost 1;
on decide commit p because enough using 1;
on decide commit o because enough using 1;
on try when latest(c) > 100 commit p because enough using 1;
on close when latest(c) > 0 resolve o = 1 because latest(c);
on use set s = latest(c);
on q when latest(a) < 5 qualify latest(c) with k;
on lb when committed(p) sample b = v supports seen;
on wait when latest(c) > 0 qualify f with k3 after 5;
on wait when latest(c) > 0 emit ping;
";

/// A qualification joins its guard to the live records that list what it
/// qualifies when it applies: `e`'s observation record, `k2`'s examination
/// record, `p`'s `committed` record and `s`'s lineage, and `b`'s reading
/// takes it from `committed(p)`. An outcome, a scheduled guard, a cue and a
/// state's grounds take none, and a cause names only what its `because`
/// selects, so `a` in any of them is refused.
#[test]
fn a_later_qualification_joins_live_records_only() {
    let save = plays_and_restores(
        QUALIFIED_LATER,
        &[
            ("lc", ONE),
            ("la", ONE),
            ("see", "{}"),
            ("check", "{}"),
            ("decide", "{}"),
            ("try", "{}"),
            ("close", "{}"),
            ("use", "{}"),
            ("q", "{}"),
            ("lb", ONE),
            ("la", TWO),
            ("wait", "{}"),
        ],
    );
    for place in [
        "observation of e",
        "examination of k2",
        "committed(p)",
        "state s",
        "reading b@1",
    ] {
        assert!(carries(&save, place, "a"), "{place}: {save}");
    }
    let marker = save["states"]["s"]["lineage"]["departed"].clone();
    for (pointer, place) in [
        (
            "/decision_series/o/revisions/0/outcome/lineage",
            "outcome of o@1",
        ),
        (
            "/scheduled_qualifications/0/guard",
            "scheduled qualification of f",
        ),
        ("/cue_qualifications/0", "cue ping"),
        ("/states/s/grounds", "state s grounds"),
    ] {
        let mut forged = save.clone();
        forged.pointer_mut(pointer).unwrap()["departed"] = marker.clone();
        refused(QUALIFIED_LATER, &forged, &no_rule_carries(place, "a"));
    }
    let mut forged = save.clone();
    forged["decision_series"]["o"]["revisions"][0]["outcome"]["because"] = json!(["c@1", "a@2"]);
    refused(
        QUALIFIED_LATER,
        &forged,
        &no_rule_carries("outcome of o@1 because", "a"),
    );
}

/// `re` reopens `d` unguarded, and `try` would reopen it again under a guard
/// on `latest(c)` that never holds, which `d@1`'s `reopened` record keeps.
/// `q` then qualifies `c`'s reading under a guard on `latest(a)`, which
/// joins that record, and `again` commits the next revision.
const REOPENED_LATER: &str = "claim seen;
evidence glimpse from \"a glimpse\";
evidence g3 from \"g3\";
evidence e from \"e\";
caveat k consequence low;
readings a from glimpse window 1;
readings c from g3 window 4;
decisions d limit 4;
event la v min 0 max 10;
event lc v min 0 max 10;
event see;
event decide;
event try;
event q;
event re;
event again;
on la sample a = v supports seen;
on lc sample c = v supports seen;
on see reveal e supports seen;
on decide commit d because enough using 1;
on try when latest(c) > 100 reopen d because e;
on q when latest(a) < 5 qualify latest(c) with k;
on re reopen d because e;
on again commit d because enough using 2;
";

/// A revision's basis takes its predecessor's `reopened` record, with what
/// a qualification joined to it.
#[test]
fn a_revision_takes_what_joins_its_predecessors_reopened_record() {
    let save = plays_and_restores(
        REOPENED_LATER,
        &[
            ("lc", ONE),
            ("la", ONE),
            ("see", "{}"),
            ("decide", "{}"),
            ("re", "{}"),
            ("try", "{}"),
            ("q", "{}"),
            ("again", "{}"),
            ("la", TWO),
        ],
    );
    assert!(carries(&save, "commitment d@2", "a"), "{save}");
}

/// `c` is an action a rule commits, not a series: `re` reopens it because
/// of `b`'s reading, which `lb` samples under a guard on `latest(a)`, and
/// `use` sets `t` from `reopened(c)`.
const PLAIN_REOPENED: &str = "claim seen;
evidence glimpse from \"a glimpse\";
evidence g2 from \"g2\";
readings a from glimpse window 1;
readings b from g2 window 1;
state t = 0;
event la v min 0 max 10;
event lb v min 0 max 10;
event go;
event re;
event use;
on la sample a = v supports seen;
on lb when latest(a) < 5 sample b = v supports seen;
on go commit c because enough using 1;
on re reopen c because latest(b);
on use set t = if(reopened(c), 1, 0);
";

/// A read of a commitment takes its reopening, with the reading it was
/// reopened because of and what that reading's sample read.
#[test]
fn a_read_of_a_plain_commitment_takes_its_reopening() {
    let save = plays_and_restores(
        PLAIN_REOPENED,
        &[
            ("la", ONE),
            ("lb", ONE),
            ("go", "{}"),
            ("re", "{}"),
            ("use", "{}"),
            ("lb", TWO),
            ("la", TWO),
        ],
    );
    assert!(carries(&save, "state t", "b"), "{save}");
    assert!(carries(&save, "state t", "a"), "{save}");
}

/// `see` reveals `r` under a guard on `latest(a)`, and `x` and `z`
/// unguarded. `pull` withdraws `x` because of `r`, `skip` would withdraw `z`
/// because of `r` under a guard on `latest(a)` that never holds, and `use`
/// sets `s` from `withdrawn(x)`.
const WITHDRAWALS: &str = "claim seen;
evidence glimpse from \"a glimpse\";
evidence r from \"r\";
evidence x from \"x\";
evidence z from \"z\";
readings a from glimpse window 1;
state s = 0;
event la v min 0 max 10;
event see;
event pull;
event skip;
event use;
on la sample a = v supports seen;
on see when latest(a) < 5 reveal r supports seen;
on see reveal x supports seen;
on see reveal z supports seen;
on pull withdraw x because r;
on skip when latest(a) > 100 withdraw z because r;
on use set s = if(withdrawn(x), 1, 0);
";

/// `withdrawn(x)` reads the withdrawal's reasons: `s`'s lineage takes what
/// `r`'s observation holds, the guard `r` was revealed under, and its
/// grounds take `r`'s name alone. The skipped withdrawal leaves its guard in
/// `withdrawn(z)`.
#[test]
fn a_withdrawal_predicate_reads_its_reasons_and_grounds_on_their_names() {
    let save = plays_and_restores(
        WITHDRAWALS,
        &[
            ("la", ONE),
            ("see", "{}"),
            ("pull", "{}"),
            ("skip", "{}"),
            ("use", "{}"),
            ("la", TWO),
        ],
    );
    assert!(holds(&save["states"]["s"]["lineage"], "a"), "{save}");
    assert!(holds(&grounds_of(&save, "s"), "r"), "{save}");
    assert!(!holds(&grounds_of(&save, "s"), "a"), "{save}");
    assert!(carries(&save, "withdrawn(z)", "a"), "{save}");
    refused(
        WITHDRAWALS,
        &grounds_given(&save, "s", "a"),
        &no_rule_carries("state s grounds", "a"),
    );
}

/// `w` is declared from `qualified(1, e)`. `pull` withdraws `e` under a
/// guard on `latest(a)`, which joins `w`'s lineage, and `use` copies `w`
/// into `t`.
const DECLARED: &str = "claim seen;
evidence glimpse from \"a glimpse\";
evidence e from \"e\";
evidence y from \"y\";
e supports seen;
readings a from glimpse window 1;
state w = qualified(1, e);
state t = 0;
event la v min 0 max 10;
event see;
event pull;
event use;
on la sample a = v supports seen;
on see reveal y supports seen;
on pull when latest(a) < 5 withdraw e because y;
on use set t = w;
";

/// A declared state grounds on the evidence its initializer names, and its
/// lineage takes what a withdrawal of that evidence joins.
#[test]
fn a_declared_state_grounds_on_its_evidence_and_takes_what_joins_it() {
    let save = plays_and_restores(
        DECLARED,
        &[
            ("la", ONE),
            ("see", "{}"),
            ("pull", "{}"),
            ("use", "{}"),
            ("la", TWO),
        ],
    );
    assert!(holds(&save["states"]["t"]["lineage"], "a"), "{save}");
    assert!(holds(&grounds_of(&save, "t"), "e"), "{save}");
}

/// Streams, evidence, a caveat and a series for the predicates below: each
/// case adds rules, and `use` sets `s` from one predicate.
const GROUNDS_HEAD: &str = "claim seen;
evidence glimpse from \"a glimpse\";
evidence g2 from \"g2\";
evidence e from \"e\";
evidence f from \"f\";
evidence r from \"r\";
evidence x from \"x\";
caveat k consequence low;
readings a from glimpse window 1;
readings b from g2 window 4;
decisions d limit 4;
state t unset;
state s = 0;
event la v min 0 max 10;
event lb v min 0 max 10;
event see;
event prep;
event decide;
event change;
event use;
on la sample a = v supports seen;
on see reveal f supports seen;
on see reveal r supports seen;
";

/// A predicate in a set's value gives the state's grounds what the records
/// it reads ground on: a value or a citation, a commit's grounds, a cause,
/// a withdrawal's reasons, an evidence's name. The guard of the effect that
/// wrote the record stays out of them, though the lineage takes it, so `a`
/// in the grounds is refused there; what a predicate reads can still hold
/// another rule's guard, as `b`'s reading holds its sample's.
#[test]
fn each_predicate_gives_grounds_what_its_records_ground_on() {
    // Each case's rules, its predicate, and the history the grounds hold,
    // or none when only the lineage holds `a`.
    for (rules, predicate, held) in [
        ("on prep set t = latest(a);", "has_caveat(t, k)", Some("a")),
        (
            "on decide commit d because enough using latest(a);",
            "committed(d)",
            Some("a"),
        ),
        (
            "on decide commit d because enough using latest(a);
on change reopen d because f;",
            "reopened(d)",
            Some("a"),
        ),
        (
            "on decide commit d because enough using 1;
on change reopen d because latest(a);",
            "reopened(d)",
            Some("a"),
        ),
        (
            "on decide commit d because enough using 1;
on change hold d because latest(a);",
            "on_hold(d)",
            Some("a"),
        ),
        ("on change withdraw f because r;", "withdrawn(f)", Some("r")),
        ("", "observed(f)", Some("f")),
        (
            "on decide commit d because enough using 1;
on change resolve d = 1 because latest(a);",
            "resolved(d)",
            Some("a"),
        ),
        (
            "on lb when latest(a) > 0 sample b = v supports seen;",
            "has_sample(b)",
            Some("a"),
        ),
        ("on prep when latest(a) > 0 set t = 1;", "is_set(t)", None),
        (
            "on prep when latest(a) > 0 set t = 1;",
            "has_caveat(t, k)",
            None,
        ),
        (
            "on decide when latest(a) > 0 commit d because enough using 1;
on change reopen d because f;",
            "reopened(d)",
            None,
        ),
        (
            "on decide commit d because enough using 1;
on change when latest(a) > 0 reopen d because f;",
            "reopened(d)",
            None,
        ),
        (
            "on decide when latest(a) > 0 commit d because enough using 1;
on change hold d because f;",
            "on_hold(d)",
            None,
        ),
        (
            "on prep when latest(a) > 0 reveal e supports seen;
on prep when latest(a) > 0 reveal x supports seen;
on change withdraw e because x;",
            "withdrawn(e)",
            None,
        ),
        (
            "on prep when latest(a) > 0 reveal e supports seen;",
            "observed(e)",
            None,
        ),
        (
            "on prep when latest(a) > 0 reveal e supports seen;",
            "carries_any(e)",
            None,
        ),
        (
            "on lb when latest(a) > 0 sample b = v supports seen;",
            "carries_any(latest(b))",
            None,
        ),
        (
            "on decide when latest(a) > 0 commit d because enough using qualified(1, f);",
            "in_basis(d, f)",
            None,
        ),
    ] {
        let source = format!("{GROUNDS_HEAD}{rules}\non use set s = if({predicate}, 1, 0);\n");
        check_grounds(&source, predicate, held);
    }
    check_grounds(
        &format!("{GROUNDS_HEAD}on use set s = only_if(latest(a) > 0, 1);\n"),
        "only_if",
        None,
    );
}

/// Plays a case of the test above and checks what `s`'s grounds hold.
fn check_grounds(source: &str, case: &str, held: Option<&str>) {
    let save = plays_and_restores(
        source,
        &[
            ("la", THREE),
            ("lb", ONE),
            ("see", "{}"),
            ("prep", "{}"),
            ("decide", "{}"),
            ("change", "{}"),
            ("use", "{}"),
            ("la", ONE),
        ],
    );
    match held {
        Some(history) => assert!(holds(&grounds_of(&save, "s"), history), "{case}: {save}"),
        None => {
            assert!(!holds(&grounds_of(&save, "s"), "a"), "{case}: {save}");
            assert!(carries(&save, "state s", "a"), "{case}: {save}");
            refused(
                source,
                &grounds_given(&save, "s", "a"),
                &no_rule_carries("state s grounds", "a"),
            );
        }
    }
}

/// `see` reveals `e` and sets `s` from it under a guard on `latest(a)`, and
/// `flag` qualifies `e` with `old`. `close` resolves `d` because of `e`, and
/// `d2` because of the evidence in `s`'s grounds that `old` qualifies.
const CAUSE_NAMES: &str = "claim seen;
evidence glimpse from \"a glimpse\";
evidence e from \"e\";
caveat old consequence low;
readings a from glimpse window 1;
decisions d limit 4;
decisions d2 limit 4;
state s = 0;
event la v min 0 max 10;
event see;
event flag;
event decide;
event close;
on la sample a = v supports seen;
on see when latest(a) > 0 reveal e supports seen;
on see when latest(a) > 0 set s = qualified(1, e);
on flag qualify e with old;
on decide commit d because enough using 1;
on decide commit d2 because enough using 1;
on close resolve d = 1 because e;
on close resolve d2 = 1 because caveated(s, old);
";

/// A cause is a name its `because` selects, never what that name's record
/// read: `e`'s observation and `s`'s lineage hold `a`, but neither outcome's
/// `because` can name a record of `a`.
#[test]
fn a_cause_is_the_name_its_because_selects() {
    let save = plays_and_restores(
        CAUSE_NAMES,
        &[
            ("la", ONE),
            ("see", "{}"),
            ("flag", "{}"),
            ("decide", "{}"),
            ("close", "{}"),
            ("la", TWO),
        ],
    );
    for series in ["d", "d2"] {
        let because = &save["decision_series"][series]["revisions"][0]["outcome"]["because"];
        assert_eq!(because, &json!(["e"]), "{save}");
        let mut forged = save.clone();
        forged["decision_series"][series]["revisions"][0]["outcome"]["because"] =
            json!(["e", "a@2"]);
        refused(
            CAUSE_NAMES,
            &forged,
            &no_rule_carries(&format!("outcome of {series}@1 because"), "a"),
        );
    }
}

/// `outer` passes its argument on to `inner`, which sets `s` from it, and
/// `decide_on` commits `d` using its argument: `go` and `decide` call them
/// with `latest(a)`.
const NESTED: &str = "claim seen;
evidence glimpse from \"a glimpse\";
readings a from glimpse window 1;
decisions d limit 4;
state s = 0;
event la v min 0 max 10;
event go;
event decide;
proc inner(y) { set s = y; };
proc outer(x) { call inner(x); };
proc decide_on(x) { commit d because enough using x; };
on la sample a = v supports seen;
on go call outer(latest(a));
on decide call decide_on(latest(a));
";

/// What an argument grounds on reaches the grounds of what its procedure,
/// or one it calls, sets or commits with it.
#[test]
fn a_procedure_passes_its_arguments_grounds_on_to_the_procedures_it_calls() {
    let save = plays_and_restores(
        NESTED,
        &[("la", ONE), ("go", "{}"), ("decide", "{}"), ("la", TWO)],
    );
    assert!(holds(&grounds_of(&save, "s"), "a"), "{save}");
    assert!(carries(&save, "commitment d@1 grounds", "a"), "{save}");
}

/// `prep` sets `t` under a guard on `latest(a)`, and `go` sets `s` citing
/// `t`.
const CITES: &str = "claim seen;
evidence glimpse from \"a glimpse\";
readings a from glimpse window 1;
state s = 0;
state t = 0;
event la v min 0 max 10;
event prep;
event go;
on la sample a = v supports seen;
on prep when latest(a) < 5 set t = 1;
on go set s = 1 because t;
";

/// `go` sets `u` from `latest(a)`, citing it.
const CITES_READING: &str = "claim seen;
evidence glimpse from \"a glimpse\";
readings a from glimpse window 1;
state u = 0;
event la v min 0 max 10;
event go;
on la sample a = v supports seen;
on go set u = latest(a) because latest(a);
";

/// A citation gives a state what the cited value grounds on and nothing
/// more: `t`'s grounds hold nothing, so `a`'s marker in `s`'s lineage is
/// refused, though `t`'s lineage holds one. With `because`, the grounds are
/// the citations', so `u`'s hold `a`.
#[test]
fn a_citation_gives_grounds_and_nothing_more() {
    let save = plays_and_restores(
        CITES,
        &[
            ("la", ONE),
            ("prep", "{}"),
            ("go", "{}"),
            ("la", TWO),
            ("la", THREE),
        ],
    );
    let marker = save["states"]["t"]["lineage"]["departed"].clone();
    assert_eq!(marker[0]["history"], "a", "{save}");
    let mut forged = save.clone();
    forged["states"]["s"]["lineage"]["departed"] = marker;
    refused(CITES, &forged, &no_rule_carries("state s", "a"));
    let save = plays_and_restores(CITES_READING, &[("la", ONE), ("go", "{}"), ("la", TWO)]);
    assert!(holds(&grounds_of(&save, "u"), "a"), "{save}");
}

/// `lb` samples `b` under a guard on `latest(a)`. `poke` would reopen `d`
/// under a guard on `latest(c)` that never holds, and `re` reopens it
/// because of `b`'s reading.
const REOPEN_READING: &str = "claim seen;
evidence glimpse from \"a glimpse\";
evidence g2 from \"g2\";
evidence g3 from \"g3\";
evidence e from \"e\";
readings a from glimpse window 1;
readings b from g2 window 4;
readings c from g3 window 4;
decisions d limit 4;
event la v min 0 max 10;
event lb v min 0 max 10;
event lc v min 0 max 10;
event see;
event decide;
event poke;
event re;
on la sample a = v supports seen;
on lb when latest(a) < 5 sample b = v supports seen;
on lc sample c = v supports seen;
on see reveal e supports seen;
on decide commit d because enough using 1;
on poke when latest(c) > 100 reopen d because e;
on re reopen d because latest(b);
";

/// A reopening reads its cause, so it carries what `b`'s reading read. A
/// `reopened` record holds the guards of skipped reopenings, never a cause,
/// so a record of `b` there is refused.
#[test]
fn a_reopening_reads_its_cause_and_a_skipped_one_leaves_its_guard() {
    let save = plays_and_restores(
        REOPEN_READING,
        &[
            ("see", "{}"),
            ("la", ONE),
            ("lb", ONE),
            ("lc", ONE),
            ("decide", "{}"),
            ("poke", "{}"),
            ("re", "{}"),
            ("la", TWO),
        ],
    );
    assert!(carries(&save, "reopening of d@1", "a"), "{save}");
    let mut forged = save.clone();
    forged["predicate_qualifications"]["reopened"]["d@1"]["evidence"] = json!(["b@1", "c@1"]);
    refused(
        REOPEN_READING,
        &forged,
        &no_rule_carries("reopened(d@1)", "b"),
    );
}

/// `merge` commits using `latest(a)` and is reopened by each reading of
/// `pushes`, which `pushed` samples under a guard on `latest(c)`; `use` sets
/// `s` from `reopened(merge)`.
const TRIGGERED: &str = "claim changed;
evidence glimpse from \"a glimpse\";
evidence glimpse2 from \"a second glimpse\";
evidence push from \"a new commit\";
readings a from glimpse window 1;
readings c from glimpse2 window 1;
readings pushes from push window 1;
decisions merge limit 8 reopened by pushes;
state s = 0;
event la v min 0 max 10;
event lc v min 0 max 10;
event pushed;
event decide;
event use;
on la sample a = v;
on lc sample c = v;
on decide commit merge because enough using latest(a);
on pushed when latest(c) > 0 sample pushes = 1 supports changed;
on use set s = if(reopened(merge), 1, 0);
";

/// A declared trigger's reopening reads what its condition reads, the
/// commitment it reopens and so `a`, and its cause names the reading of
/// `pushes`: `s`'s grounds take that name, never `c`, which the reading
/// holds.
#[test]
fn a_reopening_trigger_reads_its_condition_and_names_its_stream() {
    let save = plays_and_restores(
        TRIGGERED,
        &[
            ("la", ONE),
            ("lc", ONE),
            ("decide", "{}"),
            ("pushed", "{}"),
            ("use", "{}"),
            ("la", TWO),
            ("lc", TWO),
        ],
    );
    assert!(carries(&save, "reopening of merge@1", "a"), "{save}");
    assert!(carries(&save, "state s", "c"), "{save}");
    refused(
        TRIGGERED,
        &grounds_given(&save, "s", "c"),
        &no_rule_carries("state s grounds", "c"),
    );
}

/// `hold` is declared committed, and reopened because of `sensor`, which
/// `see` reveals under a guard on `latest(a)`; `use` sets `s` from
/// `reopened(hold)`.
const DECLARED_REOPENING: &str = "claim safe;
evidence glimpse from \"a glimpse\";
evidence sensor from \"sensor\";
readings a from glimpse window 1;
commit hold because enough;
reopen hold because sensor;
state s = 0;
event la v min 0 max 10;
event see;
event use;
on la sample a = v supports safe;
on see when latest(a) < 5 reveal sensor supports safe;
on use set s = if(reopened(hold), 1, 0);
";

/// A declared reopening reads its evidence's records, so a read of the
/// commitment takes what `sensor`'s observation holds.
#[test]
fn a_declared_reopening_reads_its_evidence() {
    let save = plays_and_restores(
        DECLARED_REOPENING,
        &[("la", ONE), ("see", "{}"), ("use", "{}"), ("la", TWO)],
    );
    assert!(carries(&save, "state s", "a"), "{save}");
}

/// `pull` withdraws every occurrence of `e` under a guard on `latest(a)`,
/// and `q` qualifies `b`'s reading.
const WITHDRAW_EVERY_EVIDENCE: &str = "claim seen;
evidence glimpse from \"a glimpse\";
evidence g2 from \"g2\";
evidence e from \"e\";
evidence f from \"f\";
caveat k consequence low;
readings a from glimpse window 1;
readings b from g2 window 4;
event la v min 0 max 10;
event lb v min 0 max 10;
event see;
event q;
event pull;
on la sample a = v supports seen;
on lb sample b = v supports seen;
on see reveal e supports seen;
on see reveal f supports seen;
on q qualify latest(b) with k;
on pull when latest(a) > 0 withdraw every e because f;
";

/// `withdraw every` of an evidence joins its guard to that evidence's
/// records only, so `a`'s marker in `b@1`'s observation record is refused.
#[test]
fn a_withdrawal_of_every_occurrence_joins_only_its_evidence() {
    let save = plays_and_restores(
        WITHDRAW_EVERY_EVIDENCE,
        &[
            ("see", "{}"),
            ("la", ONE),
            ("lb", ONE),
            ("q", "{}"),
            ("pull", "{}"),
            ("la", TWO),
        ],
    );
    let mut forged = save.clone();
    forged["observation_qualifications"]["b@1"]["departed"] =
        json!([{"history": "a", "read": 1, "from": 1, "through": 1, "departed_at": 2}]);
    refused(
        WITHDRAW_EVERY_EVIDENCE,
        &forged,
        &no_rule_carries("observation of b@1", "a"),
    );
}

/// `later` schedules a qualification of `b`'s current reading under a guard
/// on `latest(c)`.
const SCHEDULED_READING: &str = "claim seen;
evidence glimpse from \"a glimpse\";
evidence g2 from \"g2\";
evidence g3 from \"g3\";
caveat k consequence low;
readings a from glimpse window 1;
readings b from g2 window 4;
readings c from g3 window 4;
event la v min 0 max 10;
event lb v min 0 max 10;
event lc v min 0 max 10;
event later;
event tick dt min 0 max 0.1;
clock tick every 0.1;
on la sample a = v supports seen;
on lb sample b = v supports seen;
on lc sample c = v supports seen;
on later when latest(c) > 0 qualify latest(b) with k after 100;
";

/// The guards scheduled on a reading are its stream's.
#[test]
fn a_scheduled_qualification_of_a_reading_belongs_to_its_stream() {
    let save = plays_and_restores(
        SCHEDULED_READING,
        &[("la", ONE), ("lb", ONE), ("lc", ONE), ("later", "{}")],
    );
    assert!(
        carries(&save, "scheduled qualification of b@1", "c"),
        "{save}"
    );
}

/// `c` is sampled from nothing of `c`.
const OWN_STREAM: &str = "claim seen;
evidence glimpse from \"a glimpse\";
readings c from glimpse window 2;
event lc v min 0 max 10;
on lc sample c = v supports seen;
";

/// A reading names itself, and another reading of its stream only when
/// what samples it reads the stream, so `c@2` naming `c@1` is refused.
#[test]
fn a_reading_names_no_earlier_reading_of_its_stream() {
    let save = plays_and_restores(OWN_STREAM, &[("lc", ONE), ("lc", TWO)]);
    let mut forged = save.clone();
    forged["reading_streams"]["c"]["occurrences"][1]["provenance"]["evidence"] =
        json!(["c@1", "c@2"]);
    refused(OWN_STREAM, &forged, &no_rule_carries("reading c@2", "c"));
}

/// `ALL`, with a binding that divides by zero only for a value of `s` no
/// event gives it: `s` is 0, or `latest(b) + 1` with `b` sampled from 0 up.
const BOUND: &str = "budget 12;
claim seen;
evidence glimpse from \"a glimpse\";
evidence g2 from \"g2\";
evidence e from \"e\";
evidence e2 from \"e2\";
caveat k consequence low;
caveat shaky consequence low;
readings a from glimpse window 1;
readings b from g2 window 2;
decisions d limit 4;
decisions r limit 4;
state s = 0;
state t = 0;
cue note toast \"Noted\" 1;
event la v min 0 max 10;
event lb v min 0 max 10;
event go;
event decide;
event close;
event reopen;
event later;
event check;
event last;
event tick dt min 0 max 0.1;
clock tick every 0.1;
on la sample a = v supports seen;
on lb when v < 5 sample b = v supports seen;
on go reveal e supports seen;
on go when latest(b) > 100 reveal e2;
on go set s = latest(b) + 1;
on go set t = s because nothing;
on decide commit d because enough using latest(b);
on decide commit r because enough using s;
on close resolve d = 1 because e;
on reopen when committed(r) reopen r because latest(b);
on later when s > 0 qualify e with shaky after 0.2;
on check examine k cost 1;
on last when t > 0 emit note;
bind hud.v = 10 / (s + 1);
";

/// The check runs after the bindings are evaluated, so a save whose state a
/// binding cannot evaluate keeps the refusal it had before, also when it
/// holds a marker out of the bound.
#[test]
fn a_save_a_binding_refuses_keeps_its_message() {
    let save = plays_and_restores(
        BOUND,
        &[
            ("la", ONE),
            ("la", TWO),
            ("la", THREE),
            ("lb", r#"{"v":4}"#),
            ("go", "{}"),
            ("decide", "{}"),
            ("close", "{}"),
            ("reopen", "{}"),
            ("later", "{}"),
            ("check", "{}"),
            ("last", "{}"),
        ],
    );
    let marker = json!([{"history": "a", "read": 1, "from": 1, "through": 2, "departed_at": 3}]);
    let mut out_of_bound = save.clone();
    out_of_bound["states"]["s"]["lineage"]["departed"] = marker;
    refused(BOUND, &out_of_bound, &no_rule_carries("state s", "a"));
    let binding = "binding hud.v: division by zero in expression";
    let mut unevaluable = save.clone();
    unevaluable["states"]["s"]["value"] = json!(-1.0);
    refused(BOUND, &unevaluable, binding);
    let mut both = out_of_bound;
    both["states"]["s"]["value"] = json!(-1.0);
    refused(BOUND, &both, binding);
}

/// `s` is declared from `qualified(1, e)`: it grounds on `e`, which `k`
/// qualifies, by its declaration. `rv` reveals `e` again under a guard on
/// `latest(a)`, which `e`'s observation record keeps, and `cd` commits `d`.
/// Each test adds the rule of `sel`, which selects `caveated(s, k)`.
const SELECTS_DECLARED: &str = "claim seen;
claim other;
evidence e from \"e\";
evidence g from \"g\";
evidence ga from \"ga\";
caveat k consequence low;
readings a from ga window 1;
decisions d limit 4;
e supports seen;
g supports seen;
k qualifies e;
state s = qualified(1, e);
state t = 0;
event sa v min 0 max 10;
event rv;
event cd;
event pre;
event sel;
event rd;
on sa sample a = v;
on rv when latest(a) > 0 reveal e opposes other;
on cd commit d because enough using 1;
";

/// A `because caveated(STATE, CAVEAT)` reads each evidence it selects from
/// the state's grounds through its records, which can hold what the state's
/// lineage does not: `s` names `e` by its declaration, and the outcome takes
/// the guard `e` was revealed under. The state still holds nothing of `a`.
#[test]
fn a_caveated_resolve_reads_declared_grounds_through_their_records() {
    let source = format!("{SELECTS_DECLARED}on sel resolve d = 1 because caveated(s, k);\n");
    let save = plays_and_restores(
        &source,
        &[
            ("sa", ONE),
            ("rv", "{}"),
            ("cd", "{}"),
            ("sel", "{}"),
            ("sa", TWO),
            ("sa", THREE),
        ],
    );
    let outcome = &save["decision_series"]["d"]["revisions"][0]["outcome"];
    assert_eq!(outcome["because"], json!(["e"]), "{save}");
    assert!(carries(&save, "observation of e", "a"), "{save}");
    assert!(carries(&save, "outcome of d@1", "a"), "{save}");
    assert!(!carries(&save, "state s", "a"), "{save}");
    let mut forged = save.clone();
    forged["states"]["s"] = json!({
        "value": 1.0,
        "lineage": {
            "evidence": ["e"],
            "caveats": ["k"],
            "departed": outcome["lineage"]["departed"],
        },
        "grounds": {"evidence": ["e"], "caveats": ["k"]},
    });
    refused(&source, &forged, &no_rule_carries("state s", "a"));
}

/// `reopened` reads each evidence that reopened the commitment through its
/// records as they stand when it is read: the reopening is recorded before
/// `rv` writes `a`'s reading into `e`'s record, and `t` takes it all the
/// same.
#[test]
fn reopened_reads_what_reopened_a_commitment_through_its_records() {
    let source = format!(
        "{SELECTS_DECLARED}on sel reopen d because caveated(s, k);
on rd set t = if(reopened(d), 1, 0);
"
    );
    let save = plays_and_restores(
        &source,
        &[
            ("sa", ONE),
            ("cd", "{}"),
            ("sel", "{}"),
            ("rv", "{}"),
            ("rd", "{}"),
            ("sa", TWO),
            ("sa", THREE),
        ],
    );
    assert!(!carries(&save, "reopening of d@1", "a"), "{save}");
    assert!(carries(&save, "state t", "a"), "{save}");
}

/// A hold because of the same selection: the reopening record takes what
/// `e`'s record holds, and `on_hold` carries it into `t`.
#[test]
fn a_caveated_hold_reads_declared_grounds_through_their_records() {
    let source = format!(
        "{SELECTS_DECLARED}on sel hold d because caveated(s, k);
on rd set t = if(on_hold(d), 1, 0);
"
    );
    let save = plays_and_restores(
        &source,
        &[
            ("sa", ONE),
            ("rv", "{}"),
            ("cd", "{}"),
            ("sel", "{}"),
            ("rd", "{}"),
            ("sa", TWO),
            ("sa", THREE),
        ],
    );
    assert_eq!(
        save["decision_series"]["d"]["on_hold"],
        json!(["e"]),
        "{save}"
    );
    assert!(carries(&save, "reopening of d@1", "a"), "{save}");
    assert!(carries(&save, "state t", "a"), "{save}");
}

/// A release because of the same selection, of a hold because of `g`: the
/// series keeps nothing the release selects as a cause, and its reopening
/// record takes what `e`'s record holds.
#[test]
fn a_caveated_release_reads_declared_grounds_through_their_records() {
    let source = format!(
        "{SELECTS_DECLARED}on pre hold d because g;
on sel release d because caveated(s, k);
on rd set t = if(reopened(d), 1, 0) + if(committed(d), 1, 0);
"
    );
    let save = plays_and_restores(
        &source,
        &[
            ("sa", ONE),
            ("cd", "{}"),
            ("pre", "{}"),
            ("rv", "{}"),
            ("sel", "{}"),
            ("rd", "{}"),
            ("sa", TWO),
            ("sa", THREE),
        ],
    );
    assert!(carries(&save, "reopening of d@1", "a"), "{save}");
    assert!(carries(&save, "state t", "a"), "{save}");
}

/// The grounds a rule copies from a declared state are read the same way.
#[test]
fn a_caveated_selector_reads_copied_grounds_through_their_records() {
    let source = format!(
        "{SELECTS_DECLARED}on pre set t = s;
on sel resolve d = 1 because caveated(t, k);
"
    );
    let save = plays_and_restores(
        &source,
        &[
            ("sa", ONE),
            ("pre", "{}"),
            ("rv", "{}"),
            ("cd", "{}"),
            ("sel", "{}"),
            ("sa", TWO),
            ("sa", THREE),
        ],
    );
    assert!(carries(&save, "outcome of d@1", "a"), "{save}");
    assert!(!carries(&save, "state t", "a"), "{save}");
}

/// The same selection where `e` is revealed under a guard on plain evidence
/// `g`: `e`'s observation record holds the name `g`.
const SELECTS_PLAIN: &str = "claim seen;
claim other;
evidence e from \"e\";
evidence g from \"g\";
caveat k consequence low;
decisions d limit 4;
e supports seen;
g supports seen;
k qualifies e;
state s = qualified(1, e);
event rv;
event cd;
event sel;
on rv when observed(g) reveal e opposes other;
on cd commit d because enough using 1;
on sel resolve d = 1 because caveated(s, k);
";

/// A name without a window is bounded as a marker's history is: the outcome
/// takes `g` from the record of what it selects, in a program whose window
/// is on a stream nothing samples, and in the same program without the
/// window, where restore reads no name and debug builds still check every
/// event.
#[test]
fn a_caveated_selector_reads_a_plain_name_through_the_record() {
    let windowed =
        format!("evidence ga from \"ga\";\nreadings a from ga window 1;\n{SELECTS_PLAIN}");
    for source in [windowed.as_str(), SELECTS_PLAIN] {
        let save = plays_and_restores(source, &[("rv", "{}"), ("cd", "{}"), ("sel", "{}")]);
        assert!(carries(&save, "outcome of d@1", "g"), "{save}");
        assert!(!carries(&save, "state s", "g"), "{save}");
    }
}

/// `st` would let `s` ground on `e` by a citation alone, and is refused
/// unless `t`'s lineage lists `e`, which nothing here brings about.
const SELECTS_CITED: &str = "claim seen;
claim other;
evidence e from \"e\";
evidence ga from \"ga\";
caveat k consequence low;
readings a from ga window 1;
decisions d limit 4;
e supports seen;
k qualifies e;
state t = 0;
state s = 0;
event sa v min 0 max 10;
event rv;
event cd;
event st;
event sel;
on sa sample a = v;
on rv when latest(a) > 0 reveal e opposes other;
on cd commit d because enough using 1;
on st set s = t because qualified(1, e);
on sel resolve d = 1 because caveated(s, k);
";

/// A session restored from a save within the bound stays within it, also
/// where no session of the program writes that save. The citation lets
/// `s`'s grounds name `e`, so restore accepts a save edited to say so; the
/// session it gives resolves `d` because of `e`, read through its record,
/// and each save it writes restores.
#[test]
fn a_save_only_a_citation_allows_plays_on_within_the_bound() {
    let save = plays_and_restores(SELECTS_CITED, &[("sa", ONE), ("rv", "{}"), ("cd", "{}")]);
    assert!(save["states"]["s"].is_null(), "{save}");
    let mut forged = save.clone();
    forged["states"]["s"] = json!({"value": 0.0, "lineage": {"evidence": ["e"], "caveats": ["k"]}});
    let save = plays_on(
        SELECTS_CITED,
        &forged,
        &[("sel", "{}"), ("sa", TWO), ("sa", THREE)],
    );
    assert!(carries(&save, "outcome of d@1", "a"), "{save}");
}

/// `save` with `marker` and the caveat `k2`, which qualifies the marker's
/// history, in `commitment`'s basis and grounds, the `retains` relation that
/// comes with the caveat, and `k2` in the commitment's journal entry, which
/// the grounds' marker accounts for (D049): the finding's second shape.
fn with_marker_and_caveat(save: &Value, commitment: &str, marker: &Value) -> Value {
    fn give(provenance: &mut Value, marker: &Value) {
        let mut departed = provenance["departed"]
            .as_array()
            .cloned()
            .unwrap_or_default();
        departed.push(marker.clone());
        provenance["departed"] = json!(departed);
        let mut caveats = provenance["caveats"]
            .as_array()
            .cloned()
            .unwrap_or_default();
        caveats.push(json!("k2"));
        provenance["caveats"] = json!(caveats);
    }
    let mut forged = save.clone();
    give(
        &mut forged["commitment_bases"][commitment]["provenance"],
        marker,
    );
    give(&mut forged["commitment_grounds"][commitment], marker);
    forged["graph"]["relations"]
        .as_array_mut()
        .unwrap()
        .push(json!([commitment, "retains", "k2"]));
    let entry = forged["decision_journal"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|entry| entry["change"] == "committed" && entry["commitment"] == commitment)
        .unwrap();
    let mut caveats = entry["caveats"].as_array().cloned().unwrap_or_default();
    caveats.push(json!("k2"));
    entry["caveats"] = json!(caveats);
    forged
}

/// `save` with `marker` added to state `name`'s lineage, and the state's
/// grounds as they were.
fn lineage_given(save: &Value, name: &str, marker: &Value) -> Value {
    let mut forged = save.clone();
    let grounds = grounds_of(save, name);
    let state = &mut forged["states"][name];
    state["grounds"] = if grounds.is_null() {
        json!({})
    } else {
        grounds
    };
    let mut departed = state["lineage"]["departed"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    departed.push(marker.clone());
    state["lineage"]["departed"] = json!(departed);
    forged
}

/// `pull` withdraws `c`'s current reading under a guard on `latest(a)`,
/// `decide` commits `d` using `latest(c)` and `go` sets `s` from it.
const WITHDRAWN_STREAM: &str = "claim seen;
evidence glimpse from \"a glimpse\";
evidence g3 from \"g3\";
evidence e from \"e\";
caveat k2 consequence material;
k2 qualifies glimpse;
readings a from glimpse window 1;
readings c from g3 window 4;
decisions d limit 4;
state s = 0;
event la v min 0 max 10;
event lc v min 0 max 10;
event see;
event pull;
event decide;
event go;
on la sample a = v supports seen;
on lc sample c = v supports seen;
on see reveal e supports seen;
on pull when latest(a) < 5 withdraw latest(c) because e;
on decide commit d because enough using latest(c);
on go set s = latest(c);
";

/// A read of a stream takes its reading and nothing of the guard of a
/// withdrawal of that reading. When the withdrawal applies, its guard joins
/// what lists the reading then, as `s`'s lineage does; no commit or set
/// reads it. So that guard's marker is refused in the basis, alone or in
/// the finding's second shape, in the reading's observation record and in
/// the state's grounds.
#[test]
fn a_read_of_a_stream_takes_nothing_of_a_withdrawals_guard() {
    let save = plays_and_restores(
        WITHDRAWN_STREAM,
        &[
            ("la", ONE),
            ("lc", ONE),
            ("see", "{}"),
            ("decide", "{}"),
            ("go", "{}"),
            ("pull", "{}"),
            ("la", TWO),
            ("la", THREE),
        ],
    );
    assert_eq!(markers(&save), holding(&[("state s", "a")]));
    let marker = &save["states"]["s"]["lineage"]["departed"][0];
    refused(
        WITHDRAWN_STREAM,
        &with_marker_and_caveat(&save, "d@1", marker),
        &no_rule_carries("commitment d@1", "a"),
    );
    let mut forged = save.clone();
    forged["commitment_bases"]["d@1"]["provenance"]["departed"] = json!([marker]);
    refused(
        WITHDRAWN_STREAM,
        &forged,
        &no_rule_carries("commitment d@1", "a"),
    );
    let mut forged = save.clone();
    forged["observation_qualifications"]["c@1"]["departed"] = json!([marker]);
    refused(
        WITHDRAWN_STREAM,
        &forged,
        &no_rule_carries("observation of c@1", "a"),
    );
    refused(
        WITHDRAWN_STREAM,
        &grounds_given(&save, "s", "a"),
        &no_rule_carries("state s grounds", "a"),
    );
}

/// The same with plain evidence: `pull` withdraws `f` under a guard on
/// `latest(a)`, `decide` commits `d` using `qualified(1, f)` and `go` sets
/// `s` from it.
const WITHDRAWN_EVIDENCE: &str = "claim seen;
evidence glimpse from \"a glimpse\";
evidence e from \"e\";
evidence f from \"f\";
caveat k2 consequence material;
k2 qualifies glimpse;
readings a from glimpse window 1;
decisions d limit 4;
state s = 0;
state t = 0;
event la v min 0 max 10;
event see;
event pull;
event note;
event decide;
event go;
on la sample a = v supports seen;
on see reveal e supports seen;
on see reveal f supports seen;
on pull when latest(a) < 5 withdraw f because e;
on note when latest(a) < 5 set t = 1;
on decide commit d because enough using qualified(1, f);
on go set s = qualified(1, f);
";

/// `qualified(1, f)` reads `f` through its observation and `observed`
/// records, which take nothing of the guard of a withdrawal of `f`: an
/// applied one joins what lists `f`, and a skipped one leaves its guard in
/// `f`'s `withdrawn` record, which only a withdrawal predicate reads.
#[test]
fn a_read_of_evidence_takes_nothing_of_a_withdrawals_guard() {
    let save = plays_and_restores(
        WITHDRAWN_EVIDENCE,
        &[
            ("la", ONE),
            ("see", "{}"),
            ("decide", "{}"),
            ("go", "{}"),
            ("pull", "{}"),
            ("la", TWO),
            ("la", THREE),
        ],
    );
    assert_eq!(markers(&save), holding(&[("state s", "a")]));
    let marker = &save["states"]["s"]["lineage"]["departed"][0];
    let mut forged = save.clone();
    forged["commitment_bases"]["d@1"]["provenance"]["departed"] = json!([marker]);
    refused(
        WITHDRAWN_EVIDENCE,
        &forged,
        &no_rule_carries("commitment d@1", "a"),
    );
    let mut forged = save.clone();
    forged["observation_qualifications"]["f"]["departed"] = json!([marker]);
    refused(
        WITHDRAWN_EVIDENCE,
        &forged,
        &no_rule_carries("observation of f", "a"),
    );
    // The same in a session where no withdrawal ran: the marker `note`
    // leaves in `t`, with the caveat of `a`'s readings, in `f`'s
    // observation record.
    let save = plays_and_restores(
        WITHDRAWN_EVIDENCE,
        &[
            ("la", ONE),
            ("see", "{}"),
            ("note", "{}"),
            ("la", TWO),
            ("la", THREE),
        ],
    );
    let marker = &save["states"]["t"]["lineage"]["departed"][0];
    assert_eq!(marker["history"], "a", "{save}");
    let mut forged = save.clone();
    forged["observation_qualifications"]["f"] = json!({"caveats": ["k2"], "departed": [marker]});
    refused(
        WITHDRAWN_EVIDENCE,
        &forged,
        &no_rule_carries("observation of f", "a"),
    );
}

/// Streams, evidence and two series for the reads below. Each case adds
/// how `lb` samples `b`, `c0` commits `d0`, `re` reopens it and `close`
/// resolves it, so that one part of `d0` holds `a`, and one read of `d0`:
/// `decide` commits `d` using it, `go` sets `s` from it, `use` sets `t` from
/// it under a guard on `latest(a)`, and `peek` sets `u` under a guard on the
/// read, which gives `u` the read's lineage and no grounds.
const READS_ONE_PART_HEAD: &str = "claim seen;
evidence glimpse from \"a glimpse\";
evidence g2 from \"g2\";
evidence g3 from \"g3\";
evidence e from \"e\";
caveat k2 consequence material;
k2 qualifies glimpse;
readings a from glimpse window 1;
readings b from g3 window 4;
readings grants from g2 window 4;
decisions d0 limit 4;
decisions d limit 4;
state s = 0;
state t = 0;
state u = 0;
event la v min 0 max 10;
event lb v min 0 max 10;
event see;
event c0;
event re;
event close;
event decide;
event go;
event use;
event peek;
fn sum(total, reading) = total + reading;
on la sample a = v supports seen;
on see reveal e supports seen;
on see sample grants = 3 supports seen;
";

/// One way `d0` comes to hold `a`: the rules of `lb`, `c0`, `re` and
/// `close`, the events that play them, and where the save then holds `a`'s
/// marker.
type OnePart = (
    &'static str,
    &'static [(&'static str, &'static str)],
    &'static str,
);

/// `d0`'s basis alone holds `a`, through its permission's scope.
const IN_BASIS: OnePart = (
    "on c0 commit d0 because enough using 1 permitted by latest(grants) for latest(a);
on close resolve d0 = 1 because e;",
    &[("c0", "{}"), ("close", "{}")],
    "/commitment_bases/d0@1/provenance",
);

/// `d0`'s grounds hold `a`, and so does its basis: its commit uses `b`'s
/// reading, sampled under a guard on a reading of `a` that departs before
/// the commit.
const IN_GROUNDS: OnePart = (
    "on lb when latest(a) < 5 sample b = v supports seen;
on c0 commit d0 because enough using latest(b);
on close resolve d0 = 1 because e;",
    &[("lb", ONE), ("la", TWO), ("c0", "{}"), ("close", "{}")],
    "/commitment_grounds/d0@1",
);

/// `d0`'s outcome alone holds `a`, through the resolve's guard.
const IN_OUTCOME: OnePart = (
    "on c0 commit d0 because enough using 1;
on close when latest(a) < 5 resolve d0 = 1 because e;",
    &[("c0", "{}"), ("close", "{}")],
    "/decision_series/d0/revisions/0/outcome/lineage",
);

/// `d0@1`'s reopening record holds `a`, through the reopening's guard, and
/// so does the basis of `d0@2`, which `c0` commits after it.
const IN_REOPENING: OnePart = (
    "on c0 commit d0 because enough using 1;
on re when latest(a) < 5 reopen d0 because e;
on close resolve d0 = 1 because e;",
    &[("c0", "{}"), ("re", "{}"), ("c0", "{}"), ("close", "{}")],
    "/reopening_qualifications/d0@1",
);

/// `d0@1`'s `reopened` record alone holds `a`, through the guard of a
/// reopening that is skipped.
const IN_REOPENED: OnePart = (
    "on c0 commit d0 because enough using 1;
on re when latest(a) > 5 reopen d0 because e;
on close resolve d0 = 1 because e;",
    &[("c0", "{}"), ("re", "{}"), ("close", "{}")],
    "/predicate_qualifications/reopened/d0@1",
);

/// The program of one case and one read, and its save after `see`, a
/// reading of `a`, the case's events, `decide`, `go`, `use`, `peek` and two
/// more readings of `a`.
fn reads_one_part((rules, played, _): OnePart, read: &str) -> (String, Value) {
    let source = format!(
        "{READS_ONE_PART_HEAD}{rules}
on decide commit d because enough using {read};
on go set s = {read};
on use when latest(a) > 0 set t = {read};
on peek when {read} > 0 set u = 1;
"
    );
    let mut events = vec![("see", "{}"), ("la", THREE)];
    events.extend(played);
    events.extend([
        ("decide", "{}"),
        ("go", "{}"),
        ("use", "{}"),
        ("peek", "{}"),
        ("la", TWO),
        ("la", ONE),
    ]);
    let save = plays_and_restores(&source, &events);
    (source, save)
}

/// A read of a series takes the parts the runtime reads for it and no
/// other. `grounds_of` takes the grounds, and `outcome` and `resolved` the
/// outcome, never the basis, the reopening record or the `reopened` record;
/// a value read, `committed`, `in_basis`, `reopened` and `on_hold` take the
/// basis and never the outcome; and a withdrawal predicate takes none of
/// them. So a marker only another part of `d0` holds is refused in the
/// basis of the commit that reads `d0`, in the finding's second shape, in
/// the lineage of the state set from the same read and of the state set
/// under a guard on it, and in the grounds of the state set from it under a
/// guard that brings the marker into its lineage.
#[test]
fn a_read_of_a_series_takes_only_the_parts_it_reads() {
    for (part, read) in [
        (IN_BASIS, "grounds_of(d0)"),
        (IN_BASIS, "outcome(d0)"),
        (IN_BASIS, "outcome(revision(d0, 1))"),
        (IN_BASIS, "if(resolved(d0), 1, 2)"),
        (IN_BASIS, "if(resolved(revision(d0, 1)), 1, 2)"),
        (IN_BASIS, "if(rests_on_withdrawn(d0), 1, 2)"),
        (IN_BASIS, "if(permission_withdrawn(d0), 1, 2)"),
        (IN_GROUNDS, "outcome(d0)"),
        (IN_GROUNDS, "outcome(revision(d0, 1))"),
        (IN_GROUNDS, "if(resolved(d0), 1, 2)"),
        (IN_GROUNDS, "if(resolved(revision(d0, 1)), 1, 2)"),
        (IN_GROUNDS, "if(rests_on_withdrawn(d0), 1, 2)"),
        (IN_GROUNDS, "if(permission_withdrawn(d0), 1, 2)"),
        (IN_OUTCOME, "latest(d0)"),
        (IN_OUTCOME, "grounds_of(d0)"),
        (IN_OUTCOME, "history_at(d0, 0)"),
        (IN_OUTCOME, "history_count(d0)"),
        (IN_OUTCOME, "fold_history(d0, 0, sum)"),
        (IN_OUTCOME, "if(committed(d0), 1, 2)"),
        (IN_OUTCOME, "if(in_basis(d0, e), 1, 2)"),
        (IN_OUTCOME, "if(reopened(d0), 1, 2)"),
        (IN_OUTCOME, "if(on_hold(d0), 1, 2)"),
        (IN_OUTCOME, "if(rests_on_withdrawn(revision(d0, 1)), 1, 2)"),
        (IN_REOPENING, "grounds_of(d0)"),
        (IN_REOPENING, "outcome(d0)"),
        (IN_REOPENING, "outcome(revision(d0, 2))"),
        (IN_REOPENING, "if(resolved(d0), 1, 2)"),
        (IN_REOPENING, "if(resolved(revision(d0, 2)), 1, 2)"),
        (IN_REOPENING, "if(rests_on_withdrawn(d0), 1, 2)"),
        (IN_REOPENED, "grounds_of(d0)"),
        (IN_REOPENED, "outcome(d0)"),
        (IN_REOPENED, "if(resolved(d0), 1, 2)"),
        (IN_REOPENED, "if(permission_withdrawn(d0), 1, 2)"),
    ] {
        let (source, save) = reads_one_part(part, read);
        let marker = &save.pointer(part.2).unwrap()["departed"][0];
        assert_eq!(marker["history"], "a", "{read}: {save}");
        assert!(!carries(&save, "commitment d@1", "a"), "{read}: {save}");
        assert!(!carries(&save, "state s", "a"), "{read}: {save}");
        assert!(!carries(&save, "state u", "a"), "{read}: {save}");
        assert!(carries(&save, "state t", "a"), "{read}: {save}");
        assert!(!holds(&grounds_of(&save, "t"), "a"), "{read}: {save}");
        for (forged, place) in [
            (
                with_marker_and_caveat(&save, "d@1", marker),
                "commitment d@1",
            ),
            (lineage_given(&save, "s", marker), "state s"),
            (lineage_given(&save, "u", marker), "state u"),
            (grounds_given(&save, "t", "a"), "state t grounds"),
        ] {
            match ReactiveSession::restore_json(&source, &forged.to_string()) {
                Err(error) => assert_eq!(
                    error,
                    format!("cannot restore save: {}", no_rule_carries(place, "a")),
                    "{read}"
                ),
                Ok(_) => panic!("{read}: accepted in {place}: {forged}"),
            }
        }
    }
}

/// And it carries each part the runtime reads for it: a value read,
/// `committed`, `in_basis`, `reopened` and `on_hold` what the basis alone
/// holds, and with it what the reopening before it left there; `grounds_of`
/// what the grounds hold; `outcome` and `resolved` what the outcome alone
/// holds; and `reopened` and `on_hold` what a skipped reopening left in the
/// `reopened` record. A guard on the read carries the same into what its
/// rule sets, which takes the read's lineage and none of its grounds.
#[test]
fn a_read_of_a_series_carries_the_parts_it_reads() {
    for (part, read) in [
        (IN_BASIS, "latest(d0)"),
        (IN_BASIS, "history_at(d0, 0)"),
        (IN_BASIS, "history_count(d0)"),
        (IN_BASIS, "fold_history(d0, 0, sum)"),
        (IN_BASIS, "if(committed(d0), 1, 2)"),
        (IN_BASIS, "if(in_basis(d0, e), 1, 2)"),
        (IN_BASIS, "if(reopened(d0), 1, 2)"),
        (IN_BASIS, "if(on_hold(d0), 1, 2)"),
        (IN_GROUNDS, "grounds_of(d0)"),
        (IN_OUTCOME, "outcome(d0)"),
        (IN_OUTCOME, "outcome(revision(d0, 1))"),
        (IN_OUTCOME, "if(resolved(d0), 1, 2)"),
        (IN_OUTCOME, "if(resolved(revision(d0, 1)), 1, 2)"),
        (IN_REOPENING, "latest(d0)"),
        (IN_REOPENING, "if(committed(d0), 1, 2)"),
        (IN_REOPENED, "if(reopened(d0), 1, 2)"),
        (IN_REOPENED, "if(on_hold(d0), 1, 2)"),
    ] {
        let (_, save) = reads_one_part(part, read);
        assert!(carries(&save, "commitment d@1", "a"), "{read}: {save}");
        assert!(carries(&save, "state s", "a"), "{read}: {save}");
        assert!(carries(&save, "state u", "a"), "{read}: {save}");
        assert!(!holds(&grounds_of(&save, "u"), "a"), "{read}: {save}");
    }
}

/// `look` examines `k` under a guard on `latest(a)`, `decide` commits `d`
/// using `qualified(1, e, k)` and `go` sets `s` from it.
const QUALIFIED_EXAMINED: &str = "budget 12;
claim seen;
evidence glimpse from \"a glimpse\";
evidence e from \"e\";
caveat k consequence low;
caveat k2 consequence material;
k2 qualifies glimpse;
k qualifies e;
readings a from glimpse window 1;
decisions d limit 4;
state s = 0;
event la v min 0 max 10;
event see;
event look;
event decide;
event go;
on la sample a = v supports seen;
on see reveal e supports seen;
on look when latest(a) < 5 examine k cost 1;
on decide commit d because enough using qualified(1, e, k);
on go set s = qualified(1, e, k);
";

/// `qualified(V, E, K)` attaches `K` by name: nothing of its examination
/// record, which holds the guard it was examined under, reaches the basis
/// or the lineage.
#[test]
fn qualified_with_a_caveat_takes_nothing_of_its_examination_record() {
    let save = plays_and_restores(
        QUALIFIED_EXAMINED,
        &[
            ("la", ONE),
            ("see", "{}"),
            ("look", "{}"),
            ("decide", "{}"),
            ("go", "{}"),
            ("la", TWO),
            ("la", THREE),
        ],
    );
    assert_eq!(markers(&save), holding(&[("examination of k", "a")]));
    let marker = &save["examination_qualifications"]["k"]["departed"][0];
    let mut forged = save.clone();
    forged["commitment_bases"]["d@1"]["provenance"]["departed"] = json!([marker]);
    refused(
        QUALIFIED_EXAMINED,
        &forged,
        &no_rule_carries("commitment d@1", "a"),
    );
    refused(
        QUALIFIED_EXAMINED,
        &lineage_given(&save, "s", marker),
        &no_rule_carries("state s", "a"),
    );
}

/// `freeze` holds `r` because of `e`, `re` reopens it because of `b`'s
/// reading, which the hold defers, and `unfreeze` releases it because of
/// `f`.
const CAUSE_KINDS: &str = "claim seen;
evidence glimpse from \"a glimpse\";
evidence g2 from \"g2\";
evidence e from \"e\";
evidence f from \"f\";
readings a from glimpse window 1;
readings b from g2 window 1;
decisions r limit 6;
state s = 0;
event la v min 0 max 10;
event lb v min 0 max 10;
event see;
event decide;
event freeze;
event re;
event unfreeze;
event use;
on la sample a = v supports seen;
on lb sample b = v supports seen;
on see reveal e supports seen;
on see reveal f supports seen;
on decide commit r because enough using 1;
on freeze hold r because e;
on re when committed(r) reopen r because latest(b);
on unfreeze release r because f;
on use set s = if(reopened(r), 1, 0);
";

/// Each kind of cause has its own bound: a hold names only what a hold
/// selects, a deferral and a reopening only what a reopen selects, and the
/// series keeps nothing a release selects. The release then reopens `r`
/// because of the deferred reading alone.
#[test]
fn a_hold_a_deferral_and_a_release_each_keep_only_their_own_causes() {
    let save = plays_and_restores(
        CAUSE_KINDS,
        &[
            ("see", "{}"),
            ("la", ONE),
            ("lb", ONE),
            ("decide", "{}"),
            ("freeze", "{}"),
            ("re", "{}"),
        ],
    );
    let series = &save["decision_series"]["r"];
    assert_eq!(series["on_hold"], json!(["e"]), "{save}");
    assert_eq!(series["deferred"], json!(["b@1"]), "{save}");
    for (change, causes, place, history) in [
        ("deferred", &["f"][..], "deferral", "f"),
        ("deferred", &["e", "b@1"][..], "deferral", "e"),
        ("held", &["f"][..], "hold", "f"),
        ("held", &["e", "b@1"][..], "hold", "b"),
    ] {
        refused(
            CAUSE_KINDS,
            &with_causes(&save, "r", change, causes),
            &no_rule_carries(&format!("decision series r {place}"), history),
        );
    }
    let save = plays_on(CAUSE_KINDS, &save, &[("unfreeze", "{}"), ("use", "{}")]);
    let reopens = save["graph"]["relations"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|relation| relation[1] == "reopens")
        .collect::<Vec<_>>();
    assert_eq!(reopens, [&json!(["b@1", "reopens", "r@1"])], "{save}");
}

/// `c` is an action a rule commits, not a series. `poke` would reopen it
/// under a guard on `latest(b)` that never holds, which `c`'s `reopened`
/// record keeps; `mark` then qualifies `b`'s reading under a guard on
/// `latest(a)`, which joins that record, and `again` commits `d` using
/// `reopened(c)`.
const PLAIN_SKIPPED_REOPEN: &str = "claim seen;
evidence glimpse from \"a glimpse\";
evidence g2 from \"g2\";
evidence e from \"e\";
caveat old consequence low;
readings a from glimpse window 1;
readings b from g2 window 4;
decisions d limit 4;
event la v min 0 max 10;
event lb v min 0 max 10;
event see;
event go;
event poke;
event mark;
event again;
on la sample a = v supports seen;
on lb sample b = v supports seen;
on see reveal e supports seen;
on go commit c because enough using 1;
on poke when latest(b) > 100 reopen c because e;
on mark when latest(a) < 5 qualify latest(b) with old;
on again commit d because enough using if(reopened(c), 1, 0);
";

const PLAIN_SKIPPED_REOPEN_PLAYED: [(&str, &str); 8] = [
    ("see", "{}"),
    ("la", ONE),
    ("lb", ONE),
    ("go", "{}"),
    ("poke", "{}"),
    ("mark", "{}"),
    ("again", "{}"),
    ("la", TWO),
];

/// A read of a plain commitment takes its `reopened` record with what a
/// later qualification joined to it, into a part nothing joins later: the
/// basis of `d` holds `a`, which only that record carries there.
#[test]
fn a_read_of_a_plain_commitment_takes_what_joins_its_reopened_record() {
    let save = plays_and_restores(PLAIN_SKIPPED_REOPEN, &PLAIN_SKIPPED_REOPEN_PLAYED);
    assert!(carries(&save, "reopened(c)", "a"), "{save}");
    assert!(carries(&save, "commitment d@1", "a"), "{save}");
}

/// A plain commitment is made once, so its basis never takes what its
/// `reopened` record holds.
#[test]
fn a_plain_commitments_basis_takes_nothing_of_its_reopened_record() {
    let save = plays_and_restores(PLAIN_SKIPPED_REOPEN, &PLAIN_SKIPPED_REOPEN_PLAYED);
    let marker = save["predicate_qualifications"]["reopened"]["c"]["departed"].clone();
    assert_eq!(marker[0]["history"], "a", "{save}");
    let mut forged = save.clone();
    forged["commitment_bases"]["c"]["provenance"]["departed"] = marker;
    refused(
        PLAIN_SKIPPED_REOPEN,
        &forged,
        &no_rule_carries("commitment c", "a"),
    );
}

/// `close` resolves `d` because of `e`, and `shut` would resolve `r` because
/// of `e`; `re` reopens both because of `f`, and `freeze` holds `r` because
/// of `f`.
const TWO_CAUSES: &str = "claim seen;
evidence glimpse from \"a glimpse\";
evidence e from \"e\";
evidence f from \"f\";
readings a from glimpse window 1;
decisions d limit 4;
decisions r limit 4;
event la v min 0 max 10;
event see;
event decide;
event close;
event shut;
event freeze;
event re;
on la sample a = v supports seen;
on see reveal e supports seen;
on see reveal f supports seen;
on decide commit d because enough using 1;
on decide commit r because enough using 1;
on close resolve d = 1 because e;
on shut resolve r = 1 because e;
on freeze hold r because f;
on re reopen d because f;
on re reopen r because f;
";

const TWO_CAUSES_PLAYED: [(&str, &str); 7] = [
    ("see", "{}"),
    ("la", THREE),
    ("decide", "{}"),
    ("close", "{}"),
    ("freeze", "{}"),
    ("re", "{}"),
    ("la", TWO),
];

/// What a resolve's `because` selects reaches the outcome, not the series'
/// reopening records.
#[test]
fn a_resolves_cause_stays_out_of_the_reopening_records() {
    let save = plays_and_restores(TWO_CAUSES, &TWO_CAUSES_PLAYED);
    assert_eq!(
        save["decision_series"]["d"]["revisions"][0]["outcome"]["because"],
        json!(["e"]),
        "{save}"
    );
    assert!(carries(&save, "reopening of d@1", "f"), "{save}");
    assert!(!carries(&save, "reopening of d@1", "e"), "{save}");
    let mut forged = save.clone();
    let mut evidence = forged["reopening_qualifications"]["d@1"]["evidence"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    evidence.insert(0, json!("e"));
    forged["reopening_qualifications"]["d@1"]["evidence"] = json!(evidence);
    refused(
        TWO_CAUSES,
        &forged,
        &no_rule_carries("reopening of d@1", "e"),
    );
}

/// What a reopening's `because` selects reaches the reopening record, not
/// the series' outcomes.
#[test]
fn a_reopenings_cause_stays_out_of_the_outcomes() {
    let save = plays_and_restores(TWO_CAUSES, &TWO_CAUSES_PLAYED);
    assert!(carries(&save, "outcome of d@1", "e"), "{save}");
    assert!(!carries(&save, "outcome of d@1", "f"), "{save}");
    let mut forged = save.clone();
    let mut evidence = forged["decision_series"]["d"]["revisions"][0]["outcome"]["lineage"]
        ["evidence"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    evidence.push(json!("f"));
    forged["decision_series"]["d"]["revisions"][0]["outcome"]["lineage"]["evidence"] =
        json!(evidence);
    refused(TWO_CAUSES, &forged, &no_rule_carries("outcome of d@1", "f"));
}

/// A hold names only what a hold selects and a deferral only what a reopen
/// selects, never what a resolve of the series selects.
#[test]
fn a_hold_names_nothing_a_resolve_selects() {
    let save = plays_and_restores(TWO_CAUSES, &TWO_CAUSES_PLAYED);
    assert_eq!(
        save["decision_series"]["r"]["on_hold"],
        json!(["f"]),
        "{save}"
    );
    // A hold's reason or a deferred cause comes with its journal entry and
    // joins the reopening record (D062), and no rule carries `e` into that
    // record either, so the record is refused first.
    refused(
        TWO_CAUSES,
        &with_causes(&save, "r", "held", &["e", "f"]),
        &no_rule_carries("reopening of r@1", "e"),
    );
    refused(
        TWO_CAUSES,
        &with_causes(&save, "r", "deferred", &["e", "f"]),
        &no_rule_carries("reopening of r@1", "e"),
    );
}

/// An outcome names only what a resolve selects, never what a reopening of
/// the series selects.
#[test]
fn an_outcome_names_nothing_a_reopening_selects() {
    let save = plays_and_restores(TWO_CAUSES, &TWO_CAUSES_PLAYED);
    let mut forged = save.clone();
    forged["decision_series"]["d"]["revisions"][0]["outcome"]["because"] = json!(["e", "f"]);
    refused(
        TWO_CAUSES,
        &forged,
        &no_rule_carries("outcome of d@1 because", "f"),
    );
}

/// `use` sets `s` from `b`'s current reading; `pull` withdraws that reading
/// under a guard on `latest(a)`, and `skip` would withdraw it under a guard
/// on `latest(c)` that never holds.
const WITHDRAW_LATEST: &str = "claim seen;
evidence glimpse from \"a glimpse\";
evidence g2 from \"g2\";
evidence g3 from \"g3\";
evidence y from \"y\";
readings a from glimpse window 1;
readings b from g2 window 4;
readings c from g3 window 1;
state s = 0;
event la v min 0 max 10;
event lb v min 0 max 10;
event lc v min 0 max 10;
event see;
event use;
event skip;
event pull;
on la sample a = v supports seen;
on lb sample b = v supports seen;
on lc sample c = v supports seen;
on see reveal y supports seen;
on use set s = latest(b);
on skip when latest(c) > 100 withdraw latest(b) because y;
on pull when latest(a) < 5 withdraw latest(b) because y;
";

/// A withdrawal of `latest(b)` joins its guard to what lists the reading,
/// and a skipped one leaves its guard in the reading's `withdrawn` record.
#[test]
fn a_withdrawal_of_the_current_reading_carries_its_guard() {
    let save = plays_and_restores(
        WITHDRAW_LATEST,
        &[
            ("see", "{}"),
            ("la", ONE),
            ("lb", ONE),
            ("lc", ONE),
            ("use", "{}"),
            ("skip", "{}"),
            ("pull", "{}"),
            ("la", TWO),
            ("lc", TWO),
        ],
    );
    assert!(carries(&save, "state s", "a"), "{save}");
    assert!(carries(&save, "withdrawn(b@1)", "c"), "{save}");
}

/// `decide` commits `d` permitted by `grants`' reading for `latest(a)`.
const SCOPED: &str = "claim seen;
evidence glimpse from \"a glimpse\";
evidence g2 from \"g2\";
readings a from glimpse window 1;
readings grants from g2 window 1;
decisions d limit 4;
event la v min 0 max 10;
event see;
event decide;
on la sample a = v supports seen;
on see sample grants = 3 supports seen;
on decide commit d because enough using 1 permitted by latest(grants) for latest(a);
";

/// A permission's scope reaches the basis but not the grounds, so grounds
/// that name the scope's reading, with the journal entry to match, are
/// refused.
#[test]
fn a_permissions_scope_stays_out_of_the_grounds() {
    let save = plays_and_restores(SCOPED, &[("see", "{}"), ("la", THREE), ("decide", "{}")]);
    assert!(carries(&save, "commitment d@1", "a"), "{save}");
    let mut forged = save.clone();
    forged["commitment_grounds"]["d@1"] = json!({"evidence": ["a@1"]});
    let entry = forged["decision_journal"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|entry| entry["change"] == "committed" && entry["commitment"] == "d@1")
        .unwrap();
    entry["because"] = json!(["a@1"]);
    refused(
        SCOPED,
        &forged,
        &no_rule_carries("commitment d@1 grounds", "a"),
    );
}

/// `lb` samples `b` unguarded, and `skip` would withdraw its current reading
/// under a guard on `latest(a)` that never holds: `b`'s records take `a`,
/// and nothing carries `b` into them.
const OWN_RECORDS: &str = "claim seen;
evidence glimpse from \"a glimpse\";
evidence g2 from \"g2\";
evidence y from \"y\";
caveat old consequence low;
readings a from glimpse window 1;
readings b from g2 window 4;
event la v min 0 max 10;
event lb v min 0 max 10;
event see;
event skip;
on la sample a = v supports seen;
on lb sample b = v supports seen;
on see reveal y supports seen;
on skip when latest(a) > 100 withdraw latest(b) because y;
";

/// A reading lists its own name, but its observation record and its
/// predicate records do not: there the name is bounded like any other.
#[test]
fn a_readings_records_do_not_hold_its_name() {
    let save = plays_and_restores(
        OWN_RECORDS,
        &[
            ("see", "{}"),
            ("la", ONE),
            ("lb", ONE),
            ("skip", "{}"),
            ("la", TWO),
        ],
    );
    assert!(carries(&save, "withdrawn(b@1)", "a"), "{save}");
    assert!(
        save["reading_streams"]["b"]["occurrences"][0]["provenance"]["evidence"]
            .as_array()
            .unwrap()
            .contains(&json!("b@1")),
        "{save}"
    );
    let mut forged = save.clone();
    forged["predicate_qualifications"]["withdrawn"]["b@1"]["evidence"] = json!(["b@1"]);
    refused(
        OWN_RECORDS,
        &forged,
        &no_rule_carries("withdrawn(b@1)", "b"),
    );
    let mut forged = save.clone();
    forged["observation_qualifications"]["b@1"] = json!({"evidence": ["b@1"]});
    refused(
        OWN_RECORDS,
        &forged,
        &no_rule_carries("observation of b@1", "b"),
    );
}

const WITHDRAWAL_PREDICATES_HEAD: &str = "claim seen;
evidence glimpse from \"a glimpse\";
evidence g2 from \"g2\";
evidence f from \"f\";
evidence r from \"r\";
readings a from glimpse window 1;
readings b from g2 window 4;
decisions d limit 4;
state s = 0;
event la v min 0 max 10;
event lb v min 0 max 10;
event see;
event decide;
event change;
event use;
on la sample a = v supports seen;
on see reveal f supports seen;
on see reveal r supports seen;
";

const UNGUARDED_B: &str = "on lb sample b = v supports seen;\n";

/// Plays a session of the head above with `rules`, where `use` sets `s`
/// from `condition`, which must hold, and checks that `s`'s grounds hold a
/// record or a marker of `history`.
fn grounds_hold(rules: &str, condition: &str, history: &str) {
    let source =
        format!("{WITHDRAWAL_PREDICATES_HEAD}{rules}\non use set s = if({condition}, 1, 0);\n");
    let save = plays_and_restores(
        &source,
        &[
            ("la", THREE),
            ("lb", ONE),
            ("see", "{}"),
            ("decide", "{}"),
            ("change", "{}"),
            ("use", "{}"),
            ("la", ONE),
        ],
    );
    assert_eq!(
        save["states"]["s"]["value"],
        json!(1.0),
        "{condition}: {save}"
    );
    assert!(
        holds(&grounds_of(&save, "s"), history),
        "{condition}: {save}"
    );
}

#[test]
fn withdrawn_latest_gives_grounds_the_withdrawals_reason() {
    grounds_hold(
        &format!("{UNGUARDED_B}on change withdraw latest(b) because r;"),
        "withdrawn(latest(b))",
        "r",
    );
}

#[test]
fn permission_withdrawn_gives_grounds_the_withdrawals_reason() {
    grounds_hold(
        "on decide commit d because enough using 1 permitted by f;
on change withdraw f because r;",
        "permission_withdrawn(d)",
        "r",
    );
}

#[test]
fn rests_on_withdrawn_gives_grounds_the_withdrawals_reason() {
    grounds_hold(
        "on decide commit d because enough using qualified(1, f);
on change withdraw f because r;",
        "rests_on_withdrawn(d)",
        "r",
    );
}

#[test]
fn rests_on_withdrawn_of_a_revision_gives_grounds_the_withdrawals_reason() {
    grounds_hold(
        "on decide commit d because enough using qualified(1, f);
on change withdraw f because r;",
        "rests_on_withdrawn(revision(d, 1))",
        "r",
    );
}

/// A count of a stream's readings and an indexed read of one give grounds
/// what the readings hold, as they give the lineage: here the guard `b` is
/// sampled under (mutants g_historycount_named, g_historyat_named).
#[test]
fn history_reads_give_grounds_what_the_readings_hold() {
    let guarded = "on lb when latest(a) > 0 sample b = v supports seen;";
    grounds_hold(guarded, "history_count(b) > 0", "a");
    grounds_hold(guarded, "history_at(b, 0) > 0", "a");
}

/// Plays a session of the head above with `rules`, where `use` sets `s`
/// to 1 or 2 by `condition` under `guard`, which carries `history` into
/// `s`'s lineage, and checks that `s`'s grounds cannot hold it.
fn grounds_leave_out(rules: &str, guard: &str, condition: &str, history: &str) {
    let source = format!(
        "{WITHDRAWAL_PREDICATES_HEAD}{rules}\non use when {guard} set s = if({condition}, 1, 2);\n"
    );
    let save = plays_and_restores(
        &source,
        &[
            ("la", THREE),
            ("lb", ONE),
            ("see", "{}"),
            ("decide", "{}"),
            ("change", "{}"),
            ("use", "{}"),
        ],
    );
    assert!(carries(&save, "state s", history), "{condition}: {save}");
    assert!(
        !holds(&grounds_of(&save, "s"), history),
        "{condition}: {save}"
    );
    refused(
        &source,
        &grounds_given(&save, "s", history),
        &no_rule_carries("state s grounds", history),
    );
}

/// `committed` gives grounds the commitment's grounds, never the cause of
/// its reopening.
#[test]
fn committed_gives_grounds_no_reopening_cause() {
    grounds_leave_out(
        "on decide commit d because enough using 1;
on change reopen d because latest(a);",
        "latest(a) > 0",
        "committed(d)",
        "a",
    );
}

/// `in_basis` gives grounds the revision's grounds: neither the cause of
/// its reopening nor the evidence it looks for.
#[test]
fn in_basis_gives_grounds_only_the_revisions_grounds() {
    grounds_leave_out(
        "on decide commit d because enough using 1;
on change reopen d because latest(a);",
        "latest(a) > 0",
        "in_basis(d, f)",
        "a",
    );
    grounds_leave_out(
        "on decide commit d because enough using 1;",
        "observed(f)",
        "in_basis(d, f)",
        "f",
    );
}

/// `check` examines `k` under a guard on `latest(a)`, and `go` sets `t` from
/// `qualified(1, e, k)` under one.
const QUALIFIED_WITH: &str = "budget 12;
claim seen;
evidence glimpse from \"a glimpse\";
evidence e from \"e\";
caveat k consequence low;
readings a from glimpse window 1;
state t = 0;
event la v min 0 max 10;
event see;
event check;
event go;
on la sample a = v supports seen;
on see reveal e supports seen;
on check when latest(a) < 5 examine k cost 1;
on go when latest(a) < 5 set t = qualified(1, e, k);
";

/// `qualified(V, E, K)` gives grounds the evidence's name and the caveat,
/// not the caveat's examination record.
#[test]
fn qualified_with_a_caveat_gives_grounds_no_examination_record() {
    let save = plays_and_restores(
        QUALIFIED_WITH,
        &[("la", ONE), ("see", "{}"), ("check", "{}"), ("go", "{}")],
    );
    assert!(carries(&save, "examination of k", "a"), "{save}");
    assert!(carries(&save, "state t", "a"), "{save}");
    assert!(!holds(&grounds_of(&save, "t"), "a"), "{save}");
    refused(
        QUALIFIED_WITH,
        &grounds_given(&save, "t", "a"),
        &no_rule_carries("state t grounds", "a"),
    );
}

/// `decide` commits `d` under a guard on `latest(a)`.
const GUARDED_SERIES: &str = "claim seen;
evidence glimpse from \"a glimpse\";
readings a from glimpse window 1;
decisions d limit 4;
event la v min 0 max 10;
event decide;
on la sample a = v supports seen;
on decide when latest(a) < 5 commit d because enough using 1;
";

/// A `committed` record of a series' revision holds nothing of the guard of
/// its commits, which a skipped one leaves in the series' selection.
#[test]
fn a_series_committed_record_holds_no_commit_guard() {
    let save = plays_and_restores(
        GUARDED_SERIES,
        &[
            ("la", ONE),
            ("decide", "{}"),
            ("la", r#"{"v":7}"#),
            ("decide", "{}"),
        ],
    );
    assert!(carries(&save, "decision series d", "a"), "{save}");
    let mut forged = save.clone();
    forged["predicate_qualifications"]["committed"]["d@1"] = json!({"evidence": ["a@2"]});
    refused(
        GUARDED_SERIES,
        &forged,
        &no_rule_carries("committed(d@1)", "a"),
    );
}

/// `check` examines `k1` and `k2` for 3 each and `cheap` both for 1, so the
/// least either costs is 1; `go` sets `s`, and nothing carries `a` into it.
const SPENDING: &str = "budget 20;
claim seen;
evidence glimpse from \"a glimpse\";
caveat k1 consequence low;
caveat k2 consequence low;
readings a from glimpse window 1;
state s = 0;
event la v min 0 max 10;
event go;
event check;
event cheap;
on la sample a = v supports seen;
on go set s = 1;
on check examine k1 cost 3;
on check examine k2 cost 3;
on cheap examine k1 cost 1;
on cheap examine k2 cost 1;
";

/// A save D053's check refuses is refused with its message, though a state
/// of it also names a record no rule carries into it: the D054 check runs
/// after every other.
#[test]
fn a_save_the_spending_check_refuses_keeps_its_message() {
    let save = plays_and_restores(SPENDING, &[("la", ONE), ("go", "{}"), ("check", "{}")]);
    assert_eq!(save["resources"]["spent"], 6, "{save}");
    let mut named = save.clone();
    named["states"]["s"]["lineage"]["evidence"] = json!(["a@1"]);
    refused(SPENDING, &named, &no_rule_carries("state s", "a"));
    let mut spent = save.clone();
    spent["resources"]["spent"] = 3.into();
    spent["resources"]["remaining"] = 17.into();
    let message = "examine effects spend 6 attention together, more than the 3 the budget spent";
    refused(SPENDING, &spent, message);
    let mut both = named.clone();
    both["resources"]["spent"] = 3.into();
    both["resources"]["remaining"] = 17.into();
    refused(SPENDING, &both, message);
}

/// One series per expression form, each reading `a` only inside that form.
const GUARD_FORMS_HEAD: &str = "claim seen;
evidence glimpse from \"a glimpse\";
evidence g2 from \"g2\";
evidence e from \"e\";
readings a from glimpse window 1;
readings b from g2 window 2;
decisions d0 limit 4;
decisions d limit 4;
event la v min 0 max 10;
event lb v min 0 max 10;
event go;
event close;
event decide;
fn sum(total, reading) = total + reading;
on la sample a = v supports seen;
on lb sample b = v supports seen;
on go reveal e supports seen;
";

/// A guard gives the basis its lineage and the grounds nothing, so each
/// form here carries `a` into `d`'s basis through the lineage walk alone.
#[test]
fn every_expression_form_carries_its_reads_in_a_guard() {
    // Each form, with what `d0` commits using and resolves to.
    for (form, using, resolved) in [
        ("if(true, latest(a), 2)", "1", "1"),
        ("if(false, 1, latest(a))", "1", "1"),
        ("require(latest(a) > 0, 1)", "1", "1"),
        ("require(true, latest(a))", "1", "1"),
        ("qualified(latest(a), e)", "1", "1"),
        ("floor(latest(a))", "1", "1"),
        // `abs` is the prelude's, so it expands to a call of its body.
        ("abs(latest(a))", "1", "1"),
        ("-latest(a)", "1", "1"),
        ("only_if(true, latest(a))", "1", "1"),
        ("1 + latest(a)", "1", "1"),
        ("history_at(a, 0)", "1", "1"),
        ("history_at(b, history_count(a) - 1)", "1", "1"),
        ("if(carries_any(reading(a, 1)), 1, 2)", "1", "1"),
        (
            "if(carries_any(reading(b, history_count(a))), 1, 2)",
            "1",
            "1",
        ),
        ("grounds_of(d0)", "latest(a)", "1"),
        ("outcome(d0)", "1", "latest(a)"),
        ("outcome(revision(d0, history_count(a)))", "1", "1"),
        (
            "if(in_basis(d0, reading(b, history_count(a))), 1, 0)",
            "1",
            "1",
        ),
        ("fold_history(a, 0, sum)", "1", "1"),
        ("fold_history(b, latest(a), sum)", "1", "1"),
    ] {
        let source = format!(
            "{GUARD_FORMS_HEAD}on go commit d0 because enough using {using};
on close resolve d0 = {resolved} because e;
on decide when {form} > -1000 commit d because enough using 1;
"
        );
        let save = plays_and_restores(
            &source,
            &[
                ("la", THREE),
                ("lb", ONE),
                ("go", "{}"),
                ("close", "{}"),
                ("decide", "{}"),
                ("la", ONE),
            ],
        );
        assert!(carries(&save, "commitment d@1", "a"), "{form}: {save}");
        assert!(
            !holds(&save["commitment_grounds"]["d@1"], "a"),
            "{form}: {save}"
        );
    }
}

/// `r` is revealed under a guard on `latest(a)`, `pull` withdraws `f`
/// because of it, and `decide` commits `d` when the first revision of `d0`,
/// which uses `f`, rests on withdrawn evidence.
const GUARD_AT_REVISION: &str = "claim seen;
evidence glimpse from \"a glimpse\";
evidence f from \"f\";
evidence r from \"r\";
readings a from glimpse window 1;
decisions d0 limit 4;
decisions d limit 4;
event la v min 0 max 10;
event see;
event go;
event pull;
event decide;
on la sample a = v supports seen;
on see reveal f supports seen;
on see when latest(a) > 0 reveal r supports seen;
on go commit d0 because enough using qualified(1, f);
on pull withdraw f because r;
on decide when rests_on_withdrawn(revision(d0, 1)) commit d because enough using 1;
";

/// A withdrawal predicate asked of a revision reads the withdrawals'
/// reasons through their records, as one asked of a name does.
#[test]
fn a_withdrawal_predicate_of_a_revision_reads_the_reasons() {
    let save = plays_and_restores(
        GUARD_AT_REVISION,
        &[
            ("la", THREE),
            ("see", "{}"),
            ("go", "{}"),
            ("pull", "{}"),
            ("decide", "{}"),
            ("la", ONE),
        ],
    );
    assert!(carries(&save, "commitment d@1", "a"), "{save}");
}

/// `use` sets `s` from `in_basis(d, reading(b, 1))`; `d` commits using 1, so
/// nothing of `b` is in its basis and no event carries `b` into `s`.
const IN_BASIS_READING: &str = "claim seen;
evidence glimpse from \"a glimpse\";
evidence g2 from \"g2\";
readings a from glimpse window 1;
readings b from g2 window 1;
decisions d limit 4;
state s = 0;
event la v min 0 max 10;
event lb v min 0 max 10;
event decide;
event use;
on la sample a = v supports seen;
on lb sample b = v supports seen;
on decide commit d because enough using 1;
on use set s = if(in_basis(d, reading(b, 1)), 1, 2);
";

/// `in_basis` answers from the revision's records: the evidence it looks
/// for reaches its reader only through the series.
#[test]
fn in_basis_carries_nothing_of_the_evidence_it_looks_for() {
    let save = plays_and_restores(
        IN_BASIS_READING,
        &[("lb", ONE), ("decide", "{}"), ("use", "{}")],
    );
    let mut forged = save.clone();
    forged["states"]["s"]["lineage"] = json!({"evidence": ["b@1"]});
    forged["states"]["s"]["grounds"] = json!({});
    refused(IN_BASIS_READING, &forged, &no_rule_carries("state s", "b"));
}

/// `check` examines `k` under a guard on `latest(a)`, and `go` sets `t` from
/// `qualified(1, e, k)`, unguarded: no event carries `a` into `t`.
const QUALIFIED_UNGUARDED: &str = "budget 12;
claim seen;
evidence glimpse from \"a glimpse\";
evidence e from \"e\";
caveat k consequence low;
readings a from glimpse window 1;
state t = 0;
event la v min 0 max 10;
event see;
event check;
event go;
on la sample a = v supports seen;
on see reveal e supports seen;
on check when latest(a) < 5 examine k cost 1;
on go set t = qualified(1, e, k);
";

/// `qualified(V, E, K)` attaches `K` by name: no live name its examination
/// record holds reaches the value's lineage either.
#[test]
fn qualified_with_a_caveat_carries_nothing_of_its_examination() {
    let save = plays_and_restores(
        QUALIFIED_UNGUARDED,
        &[("la", ONE), ("see", "{}"), ("check", "{}"), ("go", "{}")],
    );
    let mut forged = save.clone();
    forged["states"]["t"]["grounds"] = save["states"]["t"]["lineage"].clone();
    forged["states"]["t"]["lineage"]["evidence"] = json!(["a@1", "e"]);
    refused(
        QUALIFIED_UNGUARDED,
        &forged,
        &no_rule_carries("state t", "a"),
    );
}

/// `committed` and `in_basis` asked of a plain commitment read its basis,
/// its reopening record and its `committed` record, and nothing of its
/// `reopened` record: what a later qualification joined to that record alone
/// is refused in the basis of a commit that uses `committed(c)` or
/// `in_basis(c, e)`.
#[test]
fn committed_takes_nothing_of_a_plain_commitments_reopened_record() {
    let source = format!(
        "{PLAIN_SKIPPED_REOPEN}decisions d2 limit 4;
decisions d3 limit 4;
event note;
on note commit d2 because enough using if(committed(c), 1, 0);
on note commit d3 because enough using if(in_basis(c, e), 1, 0);
"
    );
    let save = plays_and_restores(
        &source,
        &[
            ("see", "{}"),
            ("la", ONE),
            ("lb", ONE),
            ("go", "{}"),
            ("poke", "{}"),
            ("mark", "{}"),
            ("note", "{}"),
            ("la", TWO),
        ],
    );
    let marker = save["predicate_qualifications"]["reopened"]["c"]["departed"].clone();
    assert_eq!(marker[0]["history"], "a", "{save}");
    for commitment in ["d2@1", "d3@1"] {
        let place = format!("commitment {commitment}");
        assert!(!carries(&save, &place, "a"), "{save}");
        let mut forged = save.clone();
        forged["commitment_bases"][commitment]["provenance"]["departed"] = marker.clone();
        refused(&source, &forged, &no_rule_carries(&place, "a"));
    }
}

/// `go` commits the action `c`. `again` would commit it once more under a
/// guard on `grounds_of(d0)` that never holds, which `c`'s `committed`
/// record keeps: the guard names `e`, which `d0`'s grounds name, and reads
/// none of `e`'s records. `mark` then qualifies `e` under a guard on
/// `latest(a)`, which joins the `committed` record and no basis, and
/// `decide` commits `d` using `in_basis(c, e)`.
const PLAIN_COMMITTED_AGAIN: &str = "claim seen;
evidence glimpse from \"a glimpse\";
evidence e from \"e\";
caveat old consequence low;
readings a from glimpse window 1;
decisions d0 limit 4;
decisions d limit 4;
event la v min 0 max 10;
event see;
event c0;
event go;
event again;
event mark;
event decide;
on la sample a = v supports seen;
on see reveal e supports seen;
on c0 commit d0 because enough using qualified(1, e);
on go commit c because enough using 1;
on again when grounds_of(d0) > 100 commit c because enough using 1;
on mark when latest(a) < 5 qualify e with old;
on decide commit d because enough using if(in_basis(c, e), 1, 0);
";

/// `in_basis` asked of a plain commitment reads its `committed` record, as
/// `committed` does, with what a later qualification joined to it and to no
/// basis: the basis of the commit that asks holds it, and the basis of the
/// commitment asked about does not.
#[test]
fn in_basis_takes_what_joins_a_plain_commitments_committed_record() {
    let save = plays_and_restores(
        PLAIN_COMMITTED_AGAIN,
        &[
            ("see", "{}"),
            ("la", ONE),
            ("c0", "{}"),
            ("go", "{}"),
            ("again", "{}"),
            ("mark", "{}"),
            ("decide", "{}"),
            ("la", TWO),
        ],
    );
    assert!(carries(&save, "committed(c)", "a"), "{save}");
    assert!(!carries(&save, "commitment c", "a"), "{save}");
    assert!(!carries(&save, "commitment d0@1", "a"), "{save}");
    assert!(carries(&save, "commitment d@1", "a"), "{save}");
}

/// `c` is an action no rule has committed yet: `early` would commit it
/// under a guard on `latest(b)` that never holds, which `c`'s `committed`
/// record keeps; `mark` then qualifies `b`'s reading under a guard on
/// `latest(a)`, which joins that record. `one` commits `d` using
/// `committed(c)` and `two` commits `d2` using `reopened(c)`.
const PLAIN_SKIPPED_COMMIT: &str = "claim seen;
evidence glimpse from \"a glimpse\";
evidence g2 from \"g2\";
caveat old consequence low;
readings a from glimpse window 1;
readings b from g2 window 4;
decisions d limit 4;
decisions d2 limit 4;
event la v min 0 max 10;
event lb v min 0 max 10;
event early;
event mark;
event one;
event two;
on la sample a = v supports seen;
on lb sample b = v supports seen;
on early when latest(b) > 100 commit c because enough using 1;
on mark when latest(a) < 5 qualify latest(b) with old;
on one commit d because enough using if(committed(c), 1, 0);
on two commit d2 because enough using if(reopened(c), 1, 0);
";

/// `committed` reads the `committed` record of an action, with what a later
/// qualification joined to it, and so does `reopened`: whether an action can
/// be reopened depends on the decision that left it uncommitted.
#[test]
fn a_read_of_an_uncommitted_action_takes_what_joins_its_committed_record() {
    let save = plays_and_restores(
        PLAIN_SKIPPED_COMMIT,
        &[
            ("la", ONE),
            ("lb", ONE),
            ("early", "{}"),
            ("mark", "{}"),
            ("one", "{}"),
            ("two", "{}"),
            ("la", TWO),
        ],
    );
    assert!(carries(&save, "committed(c)", "a"), "{save}");
    assert!(carries(&save, "commitment d@1", "a"), "{save}");
    assert!(carries(&save, "commitment d2@1", "a"), "{save}");
}

/// `c` is an action `go` commits using `latest(a)`, and `re` reopens it
/// because of `b`'s reading. `one` sets `s` from `committed(c)` and `two`
/// sets `t` from `reopened(c)`.
const PLAIN_PARTS: &str = "claim seen;
evidence glimpse from \"a glimpse\";
evidence g2 from \"g2\";
readings a from glimpse window 1;
readings b from g2 window 1;
state s = 0;
state t = 0;
event la v min 0 max 10;
event lb v min 0 max 10;
event go;
event re;
event one;
event two;
on la sample a = v supports seen;
on lb sample b = v supports seen;
on go commit c because enough using latest(a);
on re reopen c because latest(b);
on one set s = if(committed(c), 1, 0);
on two set t = if(reopened(c), 1, 0);
";

/// `committed` and `reopened` asked of a commitment each read its basis and
/// its reopening record.
#[test]
fn committed_and_reopened_read_the_basis_and_the_reopening_record() {
    let save = plays_and_restores(
        PLAIN_PARTS,
        &[
            ("la", ONE),
            ("lb", ONE),
            ("go", "{}"),
            ("re", "{}"),
            ("one", "{}"),
            ("two", "{}"),
            ("la", TWO),
            ("lb", TWO),
        ],
    );
    for state in ["state s", "state t"] {
        assert!(carries(&save, state, "a"), "{state}: {save}");
        assert!(carries(&save, state, "b"), "{state}: {save}");
    }
}

/// `skip` would withdraw `x` under a guard on `latest(b)` that never holds,
/// which `x`'s `withdrawn` record keeps; `mark` then qualifies `b`'s reading
/// under a guard on `latest(a)`, which joins that record, and `use` commits
/// `d` using `withdrawn(x)`.
const WITHDRAWN_JOINED: &str = "claim seen;
evidence glimpse from \"a glimpse\";
evidence g2 from \"g2\";
evidence r from \"r\";
evidence x from \"x\";
caveat old consequence low;
readings a from glimpse window 1;
readings b from g2 window 4;
decisions d limit 4;
event la v min 0 max 10;
event lb v min 0 max 10;
event see;
event skip;
event mark;
event use;
on la sample a = v supports seen;
on lb sample b = v supports seen;
on see reveal r supports seen;
on see reveal x supports seen;
on skip when latest(b) > 100 withdraw x because r;
on mark when latest(a) < 5 qualify latest(b) with old;
on use commit d because enough using if(withdrawn(x), 1, 0);
";

/// A `withdrawn` record takes what a later qualification joins to it, and a
/// withdrawal predicate carries that into a basis, which nothing joins
/// later.
#[test]
fn a_withdrawal_predicate_takes_what_joins_the_withdrawn_record() {
    let save = plays_and_restores(
        WITHDRAWN_JOINED,
        &[
            ("see", "{}"),
            ("la", ONE),
            ("lb", ONE),
            ("skip", "{}"),
            ("mark", "{}"),
            ("use", "{}"),
            ("la", TWO),
        ],
    );
    assert!(carries(&save, "withdrawn(x)", "a"), "{save}");
    assert!(carries(&save, "commitment d@1", "a"), "{save}");
}

/// `x` is revealed, and `d0` committed, under a guard on `latest(b)`. `skip`
/// would withdraw `x` under a guard on `latest(a)` that never holds, which
/// `x`'s `withdrawn` record keeps. `use` sets `s` from `withdrawn(x)` and
/// `t` from `rests_on_withdrawn(d0)`.
const WITHDRAWAL_READS: &str = "claim seen;
evidence glimpse from \"a glimpse\";
evidence g2 from \"g2\";
evidence f from \"f\";
evidence r from \"r\";
evidence x from \"x\";
readings a from glimpse window 1;
readings b from g2 window 1;
decisions d0 limit 4;
state s = 0;
state t = 0;
event la v min 0 max 10;
event lb v min 0 max 10;
event see;
event c0;
event skip;
event use;
on la sample a = v supports seen;
on lb sample b = v supports seen;
on see reveal f supports seen;
on see reveal r supports seen;
on see when latest(b) > 0 reveal x supports seen;
on c0 when latest(b) > 0 commit d0 because enough using qualified(1, f);
on skip when latest(a) > 100 withdraw x because r;
on use set s = if(withdrawn(x), 1, 2);
on use set t = if(rests_on_withdrawn(d0), 1, 2);
";

/// A withdrawal predicate reads the `withdrawn` record of what it asks
/// about, where a skipped withdrawal leaves its guard, and the reasons of
/// the withdrawals it finds, and nothing else of the evidence or the
/// decision it asks about: not `x`'s observation record, and not `d0`'s
/// basis.
#[test]
fn a_withdrawal_predicate_reads_the_withdrawn_record_and_no_other_part() {
    let save = plays_and_restores(
        WITHDRAWAL_READS,
        &[
            ("la", ONE),
            ("lb", ONE),
            ("see", "{}"),
            ("c0", "{}"),
            ("skip", "{}"),
            ("use", "{}"),
            ("la", TWO),
            ("lb", TWO),
        ],
    );
    assert_eq!(
        markers(&save),
        holding(&[
            ("state s", "a"),
            ("commitment d0@1", "b"),
            ("observation of x", "b"),
            ("withdrawn(x)", "a"),
        ])
    );
    let of_a = &save["predicate_qualifications"]["withdrawn"]["x"]["departed"][0];
    let of_b = &save["observation_qualifications"]["x"]["departed"][0];
    refused(
        WITHDRAWAL_READS,
        &lineage_given(&save, "s", of_b),
        &no_rule_carries("state s", "b"),
    );
    refused(
        WITHDRAWAL_READS,
        &lineage_given(&save, "t", of_b),
        &no_rule_carries("state t", "b"),
    );
    refused(
        WITHDRAWAL_READS,
        &lineage_given(&save, "t", of_a),
        &no_rule_carries("state t", "a"),
    );
}

/// `r` is revealed under a guard on `latest(a)` and `pull` withdraws `x`
/// because of it, so a withdrawal predicate's lineage can hold `a`. `use`
/// sets `s` from `carries(f, withdrawn_once)`, which is no withdrawal
/// predicate, whatever its caveat is called.
const CARRIES_NAMED: &str = "claim seen;
evidence glimpse from \"a glimpse\";
evidence f from \"f\";
evidence r from \"r\";
evidence x from \"x\";
caveat withdrawn_once consequence low;
readings a from glimpse window 1;
state s = 0;
event la v min 0 max 10;
event see;
event pull;
event use;
on la sample a = v supports seen;
on see reveal f supports seen;
on see reveal x supports seen;
on see when latest(a) > 0 reveal r supports seen;
on pull withdraw x because r;
on use set s = if(carries(f, withdrawn_once), 1, 2);
";

/// `carries` reads the evidence through its records and no withdrawal's
/// reason.
#[test]
fn carries_reads_no_withdrawals_reason() {
    let save = plays_and_restores(
        CARRIES_NAMED,
        &[
            ("la", ONE),
            ("see", "{}"),
            ("pull", "{}"),
            ("use", "{}"),
            ("la", TWO),
            ("la", THREE),
        ],
    );
    let marker = &save["observation_qualifications"]["r"]["departed"][0];
    assert_eq!(marker["history"], "a", "{save}");
    assert!(!carries(&save, "state s", "a"), "{save}");
    refused(
        CARRIES_NAMED,
        &lineage_given(&save, "s", marker),
        &no_rule_carries("state s", "a"),
    );
}

/// `freeze` holds `r` because of `a`'s reading and `re` reopens it because
/// of `b`'s, which the hold defers. `use` sets `s` from `on_hold(r)` and `t`
/// from `reopened(r)`, each under a guard that reads both streams.
const HELD_AND_REOPENED: &str = "claim seen;
evidence glimpse from \"a glimpse\";
evidence g2 from \"g2\";
readings a from glimpse window 1;
readings b from g2 window 1;
decisions r limit 6;
state s = 0;
state t = 0;
event la v min 0 max 10;
event lb v min 0 max 10;
event decide;
event freeze;
event re;
event use;
on la sample a = v supports seen;
on lb sample b = v supports seen;
on decide commit r because enough using 1;
on freeze hold r because latest(a);
on re reopen r because latest(b);
on use when latest(a) > 0 and latest(b) > 0 set s = if(on_hold(r), 1, 2);
on use when latest(a) > 0 and latest(b) > 0 set t = if(reopened(r), 1, 2);
";

/// `on_hold` gives grounds the reasons of the hold, never what reopens the
/// series, and `reopened` what reopens it, never the reasons of its hold.
#[test]
fn on_hold_and_reopened_give_grounds_their_own_causes() {
    let save = plays_and_restores(
        HELD_AND_REOPENED,
        &[
            ("la", ONE),
            ("lb", ONE),
            ("decide", "{}"),
            ("freeze", "{}"),
            ("re", "{}"),
            ("use", "{}"),
        ],
    );
    assert_eq!(save["states"]["s"]["value"], json!(1.0), "{save}");
    assert!(holds(&grounds_of(&save, "s"), "a"), "{save}");
    assert!(!holds(&grounds_of(&save, "s"), "b"), "{save}");
    assert!(carries(&save, "state s", "b"), "{save}");
    refused(
        HELD_AND_REOPENED,
        &grounds_given(&save, "s", "b"),
        &no_rule_carries("state s grounds", "b"),
    );
    assert!(carries(&save, "state t", "a"), "{save}");
    assert!(!holds(&grounds_of(&save, "t"), "a"), "{save}");
    refused(
        HELD_AND_REOPENED,
        &grounds_given(&save, "t", "a"),
        &no_rule_carries("state t grounds", "a"),
    );
}

/// `c0` commits `d0` under a guard on `latest(a)`; a skipped one leaves its
/// guard in the series' selection. `go` sets `s` from `grounds_of(d0)`.
const GROUNDS_OF_SELECTION: &str = "claim seen;
evidence glimpse from \"a glimpse\";
readings a from glimpse window 1;
decisions d0 limit 4;
state s = 0;
event la v min 0 max 10;
event c0;
event go;
on la sample a = v supports seen;
on c0 when latest(a) < 5 commit d0 because enough using 1;
on go set s = grounds_of(d0);
";

/// `grounds_of` inherits the series' selection with the revision's grounds.
#[test]
fn grounds_of_inherits_the_series_selection() {
    let save = plays_and_restores(
        GROUNDS_OF_SELECTION,
        &[
            ("la", ONE),
            ("c0", "{}"),
            ("la", r#"{"v":7}"#),
            ("c0", "{}"),
            ("go", "{}"),
            ("la", TWO),
            ("la", THREE),
        ],
    );
    assert!(carries(&save, "decision series d0", "a"), "{save}");
    assert!(!carries(&save, "commitment d0@1 grounds", "a"), "{save}");
    assert!(carries(&save, "state s", "a"), "{save}");
}

/// `save` with evidence `name` added to state `state`'s lineage, and the
/// state's grounds as they were.
fn lineage_naming(save: &Value, state: &str, name: &str) -> Value {
    let mut forged = save.clone();
    let grounds = grounds_of(save, state);
    let cell = &mut forged["states"][state];
    cell["grounds"] = if grounds.is_null() {
        json!({})
    } else {
        grounds
    };
    let mut evidence = cell["lineage"]["evidence"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    evidence.push(json!(name));
    cell["lineage"]["evidence"] = json!(evidence);
    forged
}

/// `freeze` holds `d0` because of `x`, which no commit, resolve or reopening
/// of it selects. `go` sets `s` from `grounds_of(d0)` and `t` from
/// `outcome(d0)`.
const HELD_READS: &str = "claim seen;
evidence glimpse from \"a glimpse\";
evidence e from \"e\";
evidence x from \"x\";
readings a from glimpse window 1;
decisions d0 limit 4;
state s = 0;
state t = 0;
event see;
event c0;
event close;
event freeze;
event go;
on see reveal e supports seen;
on see reveal x supports seen;
on c0 commit d0 because enough using 1;
on close resolve d0 = 1 because e;
on freeze hold d0 because x;
on go set s = grounds_of(d0);
on go set t = outcome(d0);
";

/// The reason of a hold goes into the reopening record, which neither
/// `grounds_of` nor `outcome` reads: its name is refused in what they set.
#[test]
fn grounds_of_and_outcome_take_nothing_of_a_holds_reason() {
    let save = plays_and_restores(
        HELD_READS,
        &[
            ("see", "{}"),
            ("c0", "{}"),
            ("close", "{}"),
            ("freeze", "{}"),
            ("go", "{}"),
        ],
    );
    assert_eq!(
        save["decision_series"]["d0"]["on_hold"],
        json!(["x"]),
        "{save}"
    );
    assert!(carries(&save, "reopening of d0@1", "x"), "{save}");
    assert!(carries(&save, "state t", "e"), "{save}");
    for state in ["s", "t"] {
        assert!(!carries(&save, &format!("state {state}"), "x"), "{save}");
        refused(
            HELD_READS,
            &lineage_naming(&save, state, "x"),
            &no_rule_carries(&format!("state {state}"), "x"),
        );
    }
}

/// A guard that reads `reopened` of a commitment a declaration reopened
/// names the declaration's evidence, which no rule selects, in the lineage
/// of what it guards, and not in its grounds.
#[test]
fn a_guard_on_a_declared_reopening_names_its_evidence() {
    let source = DECLARED_REOPENING.replace(
        "on use set s = if(reopened(hold), 1, 0);",
        "on use when reopened(hold) set s = 1;",
    );
    assert_ne!(source, DECLARED_REOPENING);
    let save = plays_and_restores(
        &source,
        &[("la", ONE), ("see", "{}"), ("use", "{}"), ("la", TWO)],
    );
    assert!(carries(&save, "state s", "sensor"), "{save}");
    assert!(carries(&save, "state s", "a"), "{save}");
    assert!(!holds(&grounds_of(&save, "s"), "sensor"), "{save}");
    refused(
        &source,
        &grounds_given(&save, "s", "sensor"),
        &no_rule_carries("state s grounds", "sensor"),
    );
}

/// `c` is sampled under a guard on `latest(b)`, and `r` revealed under one
/// on `latest(a)`. `pull` withdraws `c`'s current reading because of `r`,
/// and `use` sets `s` from `withdrawn(latest(c))`.
const WITHDRAWN_LATEST_READS: &str = "claim seen;
evidence glimpse from \"a glimpse\";
evidence g2 from \"g2\";
evidence g3 from \"g3\";
evidence r from \"r\";
readings a from glimpse window 1;
readings b from g2 window 1;
readings c from g3 window 4;
state s = 0;
event la v min 0 max 10;
event lb v min 0 max 10;
event lc v min 0 max 10;
event see;
event pull;
event use;
on la sample a = v supports seen;
on lb sample b = v supports seen;
on lc when latest(b) > 0 sample c = v supports seen;
on see when latest(a) > 0 reveal r supports seen;
on pull withdraw latest(c) because r;
on use set s = if(withdrawn(latest(c)), 1, 2);
";

/// `withdrawn(latest(S))` carries the reason of the withdrawal it finds,
/// read through its records, and nothing of the reading it asks about.
#[test]
fn withdrawn_of_the_current_reading_reads_the_reason_and_not_the_reading() {
    let save = plays_and_restores(
        WITHDRAWN_LATEST_READS,
        &[
            ("la", ONE),
            ("lb", ONE),
            ("lc", ONE),
            ("see", "{}"),
            ("pull", "{}"),
            ("use", "{}"),
            ("la", TWO),
            ("lb", TWO),
        ],
    );
    assert_eq!(save["states"]["s"]["value"], json!(1.0), "{save}");
    assert!(carries(&save, "state s", "r"), "{save}");
    assert!(carries(&save, "state s", "a"), "{save}");
    assert!(!carries(&save, "state s", "b"), "{save}");
    let marker = &save["reading_streams"]["c"]["occurrences"][0]["provenance"]["departed"][0];
    assert_eq!(marker["history"], "b", "{save}");
    refused(
        WITHDRAWN_LATEST_READS,
        &lineage_given(&save, "s", marker),
        &no_rule_carries("state s", "b"),
    );
}
