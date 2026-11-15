//! D057's readers, end to end. A `because` citation can give a state's
//! grounds a departed marker its lineage lacks, and every grounds that reads
//! those grounds takes the marker along, beyond its own lineage or basis: a
//! read of the state (its value, `is_set`, `has_caveat`), a procedure
//! parameter given it, through nested and positional calls, a specialized
//! procedure and a step that commits or cites, and a commitment's grounds
//! read through `committed`, `reopened` or `in_basis`, by a state, a commit
//! or a parameter (spec/caveat-lineage-compaction-0.1.md, "Meaning"). Each
//! such save restores. The second review of D057 found its tests played only
//! a few of these paths. After every event the save restores, saves the same
//! bytes again, and every session restored so far answers each later event
//! as the live one does.
use caveat_runtime::reactive::ReactiveSession;
use serde_json::Value;

fn answer(session: &mut ReactiveSession, event: &str, payload: &str) -> Value {
    let outcome = session
        .dispatch_outcome_json(event, payload)
        .unwrap_or_else(|fatal| panic!("{event} was fatal: {}", fatal.message));
    serde_json::to_value(outcome).unwrap()
}

/// A departed marker: its history, `from`, `through` and `departed_at`.
type Marker = (String, u64, u64, u64);

fn markers(provenance: Option<&Value>) -> Vec<Marker> {
    provenance
        .and_then(|provenance| provenance.get("departed"))
        .and_then(Value::as_array)
        .map(|list| {
            list.iter()
                .map(|marker| {
                    (
                        marker["history"].as_str().unwrap().to_string(),
                        marker["from"].as_u64().unwrap(),
                        marker["through"].as_u64().unwrap(),
                        marker["departed_at"].as_u64().unwrap(),
                    )
                })
                .collect()
        })
        .unwrap_or_default()
}

fn within(inner: &Marker, outer: &Marker) -> bool {
    inner.0 == outer.0 && outer.1 <= inner.1 && inner.2 <= outer.2 && inner.3 <= outer.3
}

/// Each place in `save` whose grounds hold a marker beyond its lineage or
/// basis, as "state NAME:HISTORY" or "commitment NAME:HISTORY".
fn beyond(save: &Value) -> Vec<String> {
    let mut found = Vec::new();
    let mut check = |what: String, grounds: Option<&Value>, lineage: Option<&Value>| {
        let outer = markers(lineage);
        for marker in markers(grounds) {
            if !outer.iter().any(|outer| within(&marker, outer)) {
                found.push(format!("{what}:{}", marker.0));
            }
        }
    };
    if let Some(states) = save["states"].as_object() {
        for (name, cell) in states {
            check(
                format!("state {name}"),
                cell.get("grounds"),
                cell.get("lineage"),
            );
        }
    }
    if let Some(grounds) = save["commitment_grounds"].as_object() {
        for (name, provenance) in grounds {
            check(
                format!("commitment {name}"),
                Some(provenance),
                save["commitment_bases"][name].get("provenance"),
            );
        }
    }
    found
}

/// Plays `events`, accepted or not; after each, the save restores and saves
/// the same bytes again, and every session restored so far answers the
/// events still to come as the live one does. Returns each place seen
/// holding a marker beyond its lineage or basis.
fn places_beyond(source: &str, events: &[(&str, &str)]) -> Vec<String> {
    let mut live = ReactiveSession::from_source(source).unwrap();
    let mut restored: Vec<ReactiveSession> = Vec::new();
    let mut seen: Vec<String> = Vec::new();
    for (index, (event, payload)) in events.iter().enumerate() {
        // A repeated commit is refused while its commitment is in force;
        // that save must restore too.
        let outcome = answer(&mut live, event, payload);
        for run in &mut restored {
            assert_eq!(answer(run, event, payload), outcome, "{index}: {event}");
        }
        let save = live.save_json().unwrap();
        let again = ReactiveSession::restore_json(source, &save)
            .unwrap_or_else(|error| panic!("{index}: {event}: {error}"));
        assert_eq!(again.save_json().unwrap(), save, "{index}: {event}");
        restored.push(again);
        for place in beyond(&serde_json::from_str(&save).unwrap()) {
            if !seen.contains(&place) {
                seen.push(place);
            }
        }
    }
    seen
}

/// `d` commits on `b@1`, which departs, and `t` cites `d`: `t`'s grounds
/// hold `b`'s marker, which its lineage, read from `b@2`, lacks.
const BASE: &str = "evidence gb from \"gb\";
evidence e from \"e\";
readings b from gb window 1;
decisions d limit 4;
decisions r limit 4;
event sb v min 0 max 10;
event cd v min 0 max 10;
event st v min 0 max 10;
event see;
on sb sample b = v;
on cd when latest(b) > 0 commit d because enough using 1;
on st set t = latest(b) because latest(d);
on see reveal e;
";

/// The start every program shares, then `more` twice, around later
/// readings.
fn events(more: &[(&'static str, &'static str)]) -> Vec<(&'static str, &'static str)> {
    let mut all = vec![
        ("see", "{}"),
        ("sb", r#"{"v":5}"#),
        ("cd", r#"{"v":1}"#),
        ("sb", r#"{"v":6}"#),
        ("st", r#"{"v":1}"#),
    ];
    all.extend_from_slice(more);
    all.extend_from_slice(&[("sb", r#"{"v":7}"#), ("sb", r#"{"v":8}"#)]);
    all.extend_from_slice(more);
    all.push(("sb", r#"{"v":9}"#));
    all
}

fn assert_seen(seen: &[String], places: &[&str]) {
    for place in places {
        assert!(seen.iter().any(|seen| seen == place), "{place}: {seen:?}");
    }
}

#[test]
fn reopened_reads_a_commitment_citing_the_state() {
    let source = format!(
        "{BASE}state t = 0;
state w = 0;
event cr; event rr; event cw;
on cr commit r because enough citing t;
on rr when committed(r) reopen r because e;
on cw set w = if(reopened(r), 1, 0);
"
    );
    let seen = places_beyond(
        &source,
        &events(&[("cr", "{}"), ("rr", "{}"), ("cw", "{}")]),
    );
    assert_seen(&seen, &["state t:b", "commitment r@1:b", "state w:b"]);
}

#[test]
fn in_basis_reads_a_commitment_citing_the_state() {
    let source = format!(
        "{BASE}state t = 0;
state w = 0;
event cr; event cw;
on cr commit r because enough citing t;
on cw set w = if(in_basis(r, e), 1, 2);
"
    );
    let seen = places_beyond(&source, &events(&[("cr", "{}"), ("cw", "{}")]));
    assert_seen(&seen, &["state w:b"]);
}

#[test]
fn is_set_and_has_caveat_read_the_state() {
    let source = format!(
        "{BASE}caveat k consequence low;
state t unset;
state w = 0;
state w2 = 0;
event cw;
on cw set w = if(is_set(t), 1, 0);
on cw set w2 = if(has_caveat(t, k), 1, 2);
"
    );
    let seen = places_beyond(&source, &events(&[("cw", "{}")]));
    assert_seen(&seen, &["state w:b", "state w2:b"]);
}

#[test]
fn nested_procedures_pass_it_through_the_same_parameter_name() {
    let source = format!(
        "{BASE}state t = 0;
state z = 0;
proc inner(x) {{ set z = x; }};
proc outer(x) {{ call inner(x); }};
event go;
on go call outer(t);
"
    );
    let seen = places_beyond(&source, &events(&[("go", "{}")]));
    assert_seen(&seen, &["state z:b"]);
}

#[test]
fn positional_parameters_keep_it_to_their_own_argument() {
    let source = format!(
        "{BASE}state t = 0;
state y = 0;
state z = 0;
proc two(a, x) {{ set z = x; set y = a; }};
event g2; event g3;
on g2 call two(1, t);
on g3 call two(t, 1);
"
    );
    let seen = places_beyond(&source, &events(&[("g2", "{}"), ("g3", "{}")]));
    assert_seen(&seen, &["state z:b", "state y:b"]);
}

#[test]
fn a_step_commits_using_a_parameter_through_two_calls() {
    let source = format!(
        "{BASE}state t = 0;
proc pc(x) {{ commit r because enough using x; }};
proc outer(x) {{ call pc(x); }};
event go;
on go call outer(t);
"
    );
    let seen = places_beyond(&source, &events(&[("go", "{}")]));
    assert_seen(&seen, &["commitment r@1:b"]);
}

#[test]
fn a_commit_uses_committed_of_a_commitment_citing_the_state() {
    let source = format!(
        "{BASE}state t = 0;
decisions q limit 4;
event cr; event cq;
on cr commit r because enough citing t;
on cq commit q because enough using if(committed(r), 1, 0);
"
    );
    let seen = places_beyond(&source, &events(&[("cr", "{}"), ("cq", "{}")]));
    assert_seen(&seen, &["commitment q@1:b"]);
}

#[test]
fn a_parameter_is_given_committed_of_a_commitment_citing_the_state() {
    let source = format!(
        "{BASE}state t = 0;
state z = 0;
proc inner(x) {{ set z = x; }};
event cr; event go;
on cr commit r because enough citing t;
on go call inner(if(committed(r), 1, 0));
"
    );
    let seen = places_beyond(&source, &events(&[("cr", "{}"), ("go", "{}")]));
    assert_seen(&seen, &["state z:b"]);
}

#[test]
fn a_plain_commitment_cites_the_state() {
    let source = format!(
        "{BASE}state t = 0;
event cp;
on cp commit plan because enough citing t;
"
    );
    let seen = places_beyond(&source, &events(&[("cp", "{}")]));
    assert_seen(&seen, &["commitment plan:b"]);
}

#[test]
fn a_step_cites_a_parameter() {
    let source = format!(
        "{BASE}state t = 0;
state z = 0;
proc pcite(x) {{ set z = latest(b) because x; }};
event go;
on go call pcite(t);
"
    );
    let seen = places_beyond(&source, &events(&[("go", "{}")]));
    assert_seen(&seen, &["state z:b"]);
}

#[test]
fn a_specialized_procedure_reads_it_through_a_parameter() {
    let source = format!(
        "{BASE}state t = 0;
state z = 0;
proc sp(x, s readings) {{ set z = x + latest(s); }};
event go;
on go call sp(t, b);
"
    );
    let seen = places_beyond(&source, &events(&[("go", "{}")]));
    assert_seen(&seen, &["state z:b"]);
}

#[test]
fn committed_of_a_commitment_citing_the_state_reaches_a_state() {
    let source = format!(
        "{BASE}state t = 0;
state w = 0;
event cr; event cw;
on cr commit r because enough citing t;
on cw set w = if(committed(r), 1, 0);
"
    );
    let seen = places_beyond(&source, &events(&[("cr", "{}"), ("cw", "{}")]));
    assert_seen(&seen, &["state w:b"]);
}
