// Decision series under random events: what Reactive 0.5, the decision
// journal, reopening triggers and permission promise, checked against the
// snapshot after every accepted event, with saves and restores in place.
//
// - A series' revisions are numbered from 1, each after its predecessor, at
//   most `limit` of them; `current` is the last; every earlier one is open,
//   since a new revision needs the current one reopened first.
// - `committed(D)` and `reopened(D)` say what `current` and its `open` say.
// - The journal is append-only and transactional: one `committed` entry per
//   revision, `reopened` entries only after it, no entry twice for one
//   reopening edge, sequences never decreasing. A revision is open exactly
//   when it has a reopening entry.
// - A revision's grounds and basis are frozen at its commit.
// - `rests_on_withdrawn(D)` is true exactly when evidence in the current
//   revision's grounds is withdrawn; `permission_withdrawn(P)` when the grant
//   recorded on P's current revision is.
// - A reading into a triggering stream reopens the revision in force, naming
//   the reading, and adds nothing to a revision already open.
// - `history_count(D)` is the revision count and `latest(D)` the current
//   revision's recorded value.
use caveat_runtime::reactive::ReactiveSession;
use serde_json::Value;
use std::collections::BTreeMap;

const SOURCE: &str = r#"
caveat flaky consequence material;
claim ok;
evidence e from "tool";
renewable e limit 4;
evidence f from "operator";
evidence g from "grant";
evidence h from "sensor";
readings s from f limit 6;
readings t from h limit 4;
decisions d limit 5 reopened by s opposing ok;
decisions p limit 3;
state a = 0;
state b = 0;
event good n min 0 max 50;
event see_e;
event see_f;
event see_g;
event new_e;
event read_for;
event read_against;
event read_e;
event decide;
event decide_cite;
event approve;
event reconsider;
event reconsider_latest;
event reconsider_twice;
event doubt;
event undoubt;
event retract_s;
event retract_e;
event retract_g;
event hold;
on good set a = a + n;
on see_e reveal e supports ok;
on see_f reveal f;
on see_g reveal g;
on new_e renew e;
on read_for sample s = a supports ok;
on read_against sample s = a opposes ok;
on read_e when observed(e) sample t = b + 1 supports ok;
on decide when observed(e) commit d because enough using qualified(a, e);
on decide_cite when has_sample(s) commit d because budget citing latest(s), a;
on approve when has_sample(t) commit p because deadline using latest(t) permitted by g;
on reconsider when committed(d) reopen d because e;
on reconsider_latest when committed(d) and has_sample(s) reopen d because latest(s);
on reconsider_twice when committed(d) reopen d because e;
on reconsider_twice when committed(d) reopen d because e;
on doubt when observed(e) qualify e with flaky;
on undoubt unqualify e with flaky;
on retract_s when has_sample(s) withdraw latest(s) because f;
on retract_e when observed(e) withdraw e because f;
on retract_g when observed(g) withdraw g because f;
on hold when observed(e) set b = qualified(a, e);
bind view.committed_d = if(committed(d), 1, 0);
bind view.reopened_d = if(reopened(d), 1, 0);
bind view.stale_d = if(rests_on_withdrawn(d), 1, 0) when committed(d);
bind view.count_d = history_count(d);
bind view.latest_d = latest(d) when committed(d);
bind view.committed_p = if(committed(p), 1, 0);
bind view.revoked_p = if(permission_withdrawn(p), 1, 0) when committed(p);
bind view.count_s = history_count(s);
"#;

const EVENTS: [(&str, &str); 23] = [
    ("good", r#"{"n": 2}"#),
    ("good", r#"{"n": 7}"#),
    ("see_e", "{}"),
    ("see_f", "{}"),
    ("see_g", "{}"),
    ("new_e", "{}"),
    ("read_for", "{}"),
    ("read_against", "{}"),
    ("read_e", "{}"),
    ("decide", "{}"),
    ("decide_cite", "{}"),
    ("approve", "{}"),
    ("reconsider", "{}"),
    ("reconsider_latest", "{}"),
    ("reconsider_twice", "{}"),
    ("doubt", "{}"),
    ("undoubt", "{}"),
    ("retract_s", "{}"),
    ("retract_e", "{}"),
    ("retract_g", "{}"),
    ("hold", "{}"),
    ("read_against", "{}"),
    ("decide", "{}"),
];

fn snapshot(game: &ReactiveSession) -> Value {
    serde_json::to_value(game.snapshot()).unwrap()
}

fn bound(shown: &Value, name: &str) -> Option<f64> {
    shown["bindings"]["view"][name].as_f64()
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

/// Every check above, against one snapshot. `frozen` remembers each
/// revision's grounds and basis from the first snapshot that showed it.
fn check(shown: &Value, frozen: &mut BTreeMap<String, (Value, Value)>, at: &str) {
    let withdrawn = shown["withdrawals"]
        .as_array()
        .map(|list| {
            list.iter()
                .map(|item| item["evidence"].as_str().unwrap().to_string())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let commitments = shown["commitments"].as_array().unwrap();
    let journal = shown["decision_journal"].as_array().unwrap();
    // Journal order.
    let mut last_sequence = 0;
    for entry in journal {
        let sequence = entry["sequence"].as_u64().unwrap();
        assert!(
            sequence >= last_sequence,
            "{at}: journal sequence goes back: {journal:?}"
        );
        last_sequence = sequence;
    }
    for (series, record) in shown["decision_series"].as_object().unwrap() {
        let revisions = record["revisions"].as_array().unwrap();
        let limit = record["limit"].as_u64().unwrap() as usize;
        assert!(
            revisions.len() <= limit,
            "{at}: {series} has {} revisions over its limit {limit}",
            revisions.len()
        );
        let current = record["current"].as_str();
        let mut previous: Option<&str> = None;
        for (index, revision) in revisions.iter().enumerate() {
            let id = revision["id"].as_str().unwrap();
            assert_eq!(
                revision["ordinal"].as_u64().unwrap() as usize,
                index + 1,
                "{at}: {id} ordinal"
            );
            assert_eq!(id, format!("{series}@{}", index + 1), "{at}: revision id");
            assert_eq!(
                revision["previous"].as_str(),
                previous,
                "{at}: {id} previous"
            );
            previous = Some(id);
            let commitment = commitments
                .iter()
                .find(|item| item["action"] == id)
                .unwrap_or_else(|| panic!("{at}: {id} has no commitment record"));
            let open = commitment["open"].as_bool().unwrap();
            let committed_entries = journal
                .iter()
                .filter(|entry| entry["commitment"] == id && entry["change"] == "committed")
                .count();
            assert_eq!(
                committed_entries, 1,
                "{at}: {id} committed entries: {journal:?}"
            );
            let committed_at = journal
                .iter()
                .position(|entry| entry["commitment"] == id && entry["change"] == "committed")
                .unwrap();
            let reopenings = journal
                .iter()
                .enumerate()
                .filter(|(_, entry)| entry["commitment"] == id && entry["change"] == "reopened")
                .collect::<Vec<_>>();
            for (position, entry) in &reopenings {
                assert!(
                    *position > committed_at,
                    "{at}: {id} reopened before committed: {journal:?}"
                );
                assert!(
                    !strings(&entry["because"]).is_empty(),
                    "{at}: {id} reopened because nothing: {entry}"
                );
            }
            let witnesses = reopenings
                .iter()
                .map(|(_, entry)| strings(&entry["because"]))
                .collect::<Vec<_>>();
            for (index, witness) in witnesses.iter().enumerate() {
                assert!(
                    !witnesses[..index].contains(witness),
                    "{at}: {id} reopened twice by {witness:?}: {journal:?}"
                );
            }
            assert_eq!(
                open,
                !reopenings.is_empty(),
                "{at}: {id} open {open} with reopenings {reopenings:?}"
            );
            if Some(id) != current {
                assert!(open, "{at}: superseded revision {id} is not open");
            }
            // Frozen grounds and basis.
            let grounds = shown["commitment_grounds"][id].clone();
            let basis = shown["commitment_bases"][id].clone();
            match frozen.get(id) {
                Some((first_grounds, first_basis)) => {
                    assert_eq!(
                        &grounds, first_grounds,
                        "{at}: {id} grounds changed after its commit"
                    );
                    assert_eq!(
                        &basis, first_basis,
                        "{at}: {id} basis changed after its commit"
                    );
                }
                None => {
                    frozen.insert(id.to_string(), (grounds, basis));
                }
            }
        }
        if let Some(current) = current {
            assert_eq!(
                current,
                revisions.last().unwrap()["id"].as_str().unwrap(),
                "{at}: {series} current is not the last revision"
            );
        } else {
            assert!(
                revisions.is_empty(),
                "{at}: {series} has revisions but no current"
            );
        }
    }
    // The predicates, through the bindings.
    let series = &shown["decision_series"];
    let current_d = series["d"]["current"].as_str();
    assert_eq!(
        bound(shown, "committed_d"),
        Some(if current_d.is_some() { 1.0 } else { 0.0 }),
        "{at}: committed(d)"
    );
    let open_d = current_d.map(|id| {
        commitments
            .iter()
            .find(|item| item["action"] == id)
            .unwrap()["open"]
            .as_bool()
            .unwrap()
    });
    assert_eq!(
        bound(shown, "reopened_d"),
        Some(if open_d == Some(true) { 1.0 } else { 0.0 }),
        "{at}: reopened(d)"
    );
    let count_d = series["d"]["revisions"].as_array().unwrap().len() as f64;
    assert_eq!(
        bound(shown, "count_d"),
        Some(count_d),
        "{at}: history_count(d)"
    );
    match current_d {
        Some(id) => {
            let grounds = strings(&shown["commitment_grounds"][id]["evidence"]);
            let stale = grounds.iter().any(|evidence| withdrawn.contains(evidence));
            assert_eq!(bound(shown, "stale_d"), Some(if stale { 1.0 } else { 0.0 }), "{at}: rests_on_withdrawn(d) with grounds {grounds:?} and withdrawals {withdrawn:?}");
            assert_eq!(
                bound(shown, "latest_d"),
                shown["commitment_bases"][id]["value"].as_f64(),
                "{at}: latest(d)"
            );
        }
        None => {
            assert_eq!(
                bound(shown, "stale_d"),
                None,
                "{at}: stale_d shown without a decision"
            );
            assert_eq!(
                bound(shown, "latest_d"),
                None,
                "{at}: latest_d shown without a decision"
            );
        }
    }
    let current_p = series["p"]["current"].as_str();
    assert_eq!(
        bound(shown, "committed_p"),
        Some(if current_p.is_some() { 1.0 } else { 0.0 }),
        "{at}: committed(p)"
    );
    match current_p {
        Some(id) => {
            let grant = shown["commitment_permissions"][id]["grant"]
                .as_str()
                .map(str::to_string);
            let revoked = grant
                .as_ref()
                .is_some_and(|grant| withdrawn.contains(grant));
            assert_eq!(
                bound(shown, "revoked_p"),
                Some(if revoked { 1.0 } else { 0.0 }),
                "{at}: permission_withdrawn(p) with grant {grant:?} and withdrawals {withdrawn:?}"
            );
        }
        None => assert_eq!(
            bound(shown, "revoked_p"),
            None,
            "{at}: revoked_p shown without a decision"
        ),
    }
    let count_s = shown["reading_streams"]["s"]["occurrences"]
        .as_array()
        .map_or(0, Vec::len) as f64;
    assert_eq!(
        bound(shown, "count_s"),
        Some(count_s),
        "{at}: history_count(s)"
    );
}

#[test]
fn the_decision_program_loads() {
    ReactiveSession::from_source(SOURCE).unwrap();
}

#[test]
fn decision_series_keep_their_promises_under_random_events() {
    let mut seed: u64 = 0x5851_f42d_4c95_7f2d;
    let mut accepted = 0;
    let mut triggered = 0;
    let mut already_open = 0;
    for run in 0..300 {
        let mut game = ReactiveSession::from_source(SOURCE).unwrap();
        let mut frozen = BTreeMap::new();
        let mut trail = Vec::new();
        let mut before = snapshot(&game);
        check(&before, &mut frozen, &format!("run {run} start"));
        for step in 0..60 {
            seed = seed
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            let pick = (seed >> 33) as usize % (EVENTS.len() + 2);
            if pick >= EVENTS.len() {
                trail.push(("save", ""));
                let saved = game.save_json().unwrap();
                game = ReactiveSession::restore_json(SOURCE, &saved).unwrap_or_else(|error| {
                    panic!("run {run} step {step} restore: {error}: {trail:?}")
                });
                assert_eq!(
                    snapshot(&game),
                    before,
                    "run {run} step {step} snapshot after restore: {trail:?}"
                );
                continue;
            }
            let (event, payload) = EVENTS[pick];
            trail.push((event, payload));
            let outcome =
                serde_json::to_value(game.dispatch_outcome_json(event, payload).unwrap_or_else(
                    |fatal| panic!("run {run} step {step} fatal {}: {trail:?}", fatal.message),
                ))
                .unwrap();
            if outcome["outcome"] != "accepted" {
                continue;
            }
            accepted += 1;
            let shown = snapshot(&game);
            let at = format!("run {run} step {step} after {event}: {trail:?}");
            check(&shown, &mut frozen, &at);
            // Append-only: the earlier journal is a prefix of the new one.
            let old = before["decision_journal"].as_array().unwrap();
            let new = shown["decision_journal"].as_array().unwrap();
            assert!(
                new.len() >= old.len() && new[..old.len()] == old[..],
                "{at}: journal rewritten\n{old:?}\n{new:?}"
            );
            let added = &new[old.len()..];
            // The trigger: a reading opposing `ok` reopens d's revision in
            // force, naming the reading, and adds nothing to one already open.
            if event == "read_against" {
                let current = before["decision_series"]["d"]["current"].as_str();
                let in_force = current.is_some_and(|id| {
                    before["commitments"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .any(|item| item["action"] == id && item["open"] == false)
                });
                let reading = format!(
                    "s@{}",
                    shown["reading_streams"]["s"]["occurrences"]
                        .as_array()
                        .unwrap()
                        .len()
                );
                let reopenings = added
                    .iter()
                    .filter(|entry| entry["change"] == "reopened")
                    .collect::<Vec<_>>();
                if in_force {
                    triggered += 1;
                    assert_eq!(
                        reopenings.len(),
                        1,
                        "{at}: a reading opposing ok while in force reopens once: {added:?}"
                    );
                    assert_eq!(
                        reopenings[0]["commitment"].as_str(),
                        current,
                        "{at}: the trigger reopened another revision"
                    );
                    assert_eq!(
                        strings(&reopenings[0]["because"]),
                        [reading],
                        "{at}: the trigger names the reading"
                    );
                    assert_eq!(reopenings[0]["event"], "read_against", "{at}");
                    assert_eq!(
                        bound(&shown, "reopened_d"),
                        Some(1.0),
                        "{at}: reopened(d) after the trigger"
                    );
                } else {
                    if current.is_some() {
                        already_open += 1;
                    }
                    assert!(
                        reopenings.is_empty(),
                        "{at}: a reading reopened a revision not in force: {added:?}"
                    );
                }
            } else if event == "read_for" {
                assert!(
                    added.iter().all(|entry| entry["change"] != "reopened"),
                    "{at}: a supporting reading reopened: {added:?}"
                );
            }
            before = shown;
        }
    }
    eprintln!("{accepted} accepted events; the trigger reopened {triggered} times and found a revision already open {already_open} times");
    assert!(
        triggered > 100 && already_open > 100,
        "the alphabet barely reached the trigger: {triggered} / {already_open}"
    );
}
