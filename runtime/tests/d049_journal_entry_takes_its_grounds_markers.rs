//! D049: restore refused a save the runtime wrote when a commitment's
//! grounds carried a caveat through a record that had since departed:
//! "decision journal: k2 cannot qualify any of its evidence". A departed
//! record's qualifications stay with what carried it, and the grounds keep a
//! departure marker for its history, which lets restore account for such a
//! caveat. The commit's journal entry records the grounds' evidence and
//! caveats but not their markers, and restore checked the entry's caveats
//! without them. It now checks a commit's entry with the markers of the
//! grounds it records. A commit `using outcome(D)`, accepted since D041, was
//! one way to reach this. See spec/caveat-save-0.1.md ("Changes").
use caveat_runtime::reactive::ReactiveSession;
use serde_json::Value;

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
    let mut live = ReactiveSession::from_source(source).unwrap();
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

/// The histories of the departure markers `commitment`'s grounds hold, and
/// the caveats its journal entry records.
fn markers_and_caveats(save: &Value, commitment: &str) -> (Vec<String>, Value) {
    let histories = save["commitment_grounds"][commitment]["departed"]
        .as_array()
        .unwrap_or_else(|| panic!("{commitment} has no markers: {save}"))
        .iter()
        .map(|marker| marker["history"].as_str().unwrap().to_string())
        .collect();
    let entry = save["decision_journal"]
        .as_array()
        .unwrap()
        .iter()
        .rfind(|entry| entry["commitment"] == commitment && entry["change"] == "committed")
        .unwrap_or_else(|| panic!("{commitment} has no journal entry"));
    (histories, entry["caveats"].clone())
}

/// `lb` is sampled while `latest(a)` is small, so `b`'s selection inherits
/// `a`'s current reading and the caveat `k2` that qualifies it. `decide`
/// commits on `latest(b)` after that reading departed.
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

#[test]
fn a_commit_on_an_inherited_reading_that_departed_restores() {
    let save = plays_and_restores(
        INHERITED,
        &[
            ("la", r#"{"v":1}"#),
            ("lb", r#"{"v":4}"#),
            ("la", r#"{"v":7}"#),
            ("lb", r#"{"v":3}"#),
            ("la", r#"{"v":2}"#),
            ("decide", "{}"),
        ],
    );
    let (histories, caveats) = markers_and_caveats(&save, "d@1");
    assert_eq!(histories, ["a"]);
    assert_eq!(caveats, serde_json::json!(["k2"]));
}

/// `d` resolves on runs@1, which `k2` qualifies and which departs at the
/// next reading; `copy2` then commits `d2` using the outcome.
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
fn a_commit_on_an_outcome_whose_record_departed_restores() {
    let save = plays_and_restores(
        OUTCOME,
        &[
            ("go", "{}"),
            ("reading", r#"{"v":5}"#),
            ("close", "{}"),
            ("reading", r#"{"v":2}"#),
            ("copy2", "{}"),
        ],
    );
    let (histories, caveats) = markers_and_caveats(&save, "d2@1");
    assert_eq!(histories, ["runs"]);
    assert_eq!(caveats, serde_json::json!(["k2"]));
}

/// `withdrawit` withdraws runs@1, `reopen` reopens `d` because of a later
/// reading, and `recommit` commits it again using the revision before,
/// whose basis read the withdrawn reading, after it departed.
const WITHDRAWN: &str = "claim c;
evidence feed from \"feed\";
evidence corr from \"corr\";
readings runs from feed window 2;
journal window 1;
decisions d limit 5;
state a = 0;
event go;
event reading v min 0 max 9;
event withdrawit;
event reopen;
event recommit;
on go commit d because enough using a + 1;
on reading sample runs = v supports c;
on reopen when committed(d) and not reopened(d) reopen d because latest(runs);
on recommit when reopened(d) commit d because enough using latest(d);
on withdrawit reveal corr;
on withdrawit withdraw latest(runs) because corr;
";

#[test]
fn a_commit_carrying_withdrawn_from_a_departed_record_restores() {
    let save = plays_and_restores(
        WITHDRAWN,
        &[
            ("reading", r#"{"v":0}"#),
            ("withdrawit", "{}"),
            ("go", "{}"),
            ("reopen", "{}"),
            ("go", "{}"),
            ("reading", r#"{"v":0}"#),
            ("reading", r#"{"v":0}"#),
            ("reopen", "{}"),
            ("recommit", "{}"),
        ],
    );
    let current = save["decision_series"]["d"]["current"]
        .as_str()
        .unwrap()
        .to_string();
    let (histories, caveats) = markers_and_caveats(&save, &current);
    assert_eq!(histories, ["runs"]);
    assert_eq!(caveats, serde_json::json!(["withdrawn"]));
}

#[test]
fn a_caveat_no_marker_accounts_for_is_still_refused() {
    let save = plays_and_restores(
        INHERITED,
        &[
            ("la", r#"{"v":1}"#),
            ("lb", r#"{"v":4}"#),
            ("la", r#"{"v":7}"#),
            ("lb", r#"{"v":3}"#),
            ("la", r#"{"v":2}"#),
            ("decide", "{}"),
        ],
    );
    let mut forged = save.clone();
    forged["commitment_grounds"]["d@1"]
        .as_object_mut()
        .unwrap()
        .remove("departed");
    match ReactiveSession::restore_json(INHERITED, &forged.to_string()) {
        Err(error) => assert_eq!(
            error,
            "cannot restore save: decision journal: k2 cannot qualify any of its evidence"
        ),
        Ok(_) => panic!("accepted a save whose grounds lost their marker"),
    }
}
