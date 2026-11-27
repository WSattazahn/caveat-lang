//! After an event, the session publishes to its archive the provenance nodes
//! that its qualification records came to hold, record by record in the
//! order of their names. These sessions hold such a node in an examination
//! record and nowhere else, so only that pass can publish it.
use caveat_runtime::reactive::ReactiveSession;
use serde_json::Value;

/// Two streams with a window of one reading. `both` samples both, so their
/// earlier readings depart in one event. `k_one` is examined on a guard that
/// read `s`, `k_two` on one that read `u`: after the departure each
/// examination record holds the marker of one departed reading, with a root
/// of its own. Nothing else holds either root: no state, commitment,
/// predicate or reading was computed from the readings.
const SOURCE: &str = r#"claim c;
caveat k_one consequence low;
caveat k_two consequence low;
evidence ts from "ts";
evidence tu from "tu";
evidence e from "e";
readings s from ts window 1;
readings u from tu window 1;
budget 10;
commit a because enough;
event both n min 0 max 9;
event see;
event one;
event two;
on both sample s = n;
on both sample u = n;
on see reveal e;
on one when latest(s) >= 0 examine k_one cost 1;
on two when latest(u) >= 0 examine k_two cost 1;
"#;

fn send(session: &mut ReactiveSession, event: &str, payload: &str) {
    let outcome = session
        .dispatch_outcome_json(event, payload)
        .unwrap_or_else(|fatal| panic!("{event} was fatal: {}", fatal.message));
    let outcome = serde_json::to_value(outcome).unwrap();
    assert_eq!(outcome["outcome"], "accepted", "{event}: {outcome}");
}

/// The provenance nodes the archive holds, in the order it published them.
fn published(session: &mut ReactiveSession) -> Vec<String> {
    session
        .drain_archive()
        .iter()
        .map(|item| serde_json::to_value(item).unwrap())
        .filter(|item| item["kind"] == "provenance")
        .map(|item| item["id"].as_str().unwrap().to_string())
        .collect()
}

/// The roots the examination record `name` holds in `save`.
fn roots(save: &Value, name: &str) -> Vec<String> {
    let departed = &save["examination_qualifications"][name]["departed"];
    let markers: Vec<&Value> = match departed {
        Value::Array(items) => items.iter().collect(),
        Value::Object(items) => items.values().collect(),
        _ => Vec::new(),
    };
    markers
        .into_iter()
        .filter_map(|marker| marker["archive_ref"].as_str())
        .map(str::to_string)
        .collect()
}

/// A session in which both caveats were examined and the readings their
/// guards read have departed, with the nodes that departure published and
/// the save after it.
fn departed() -> (ReactiveSession, Vec<String>, Value) {
    let mut session = ReactiveSession::from_source(SOURCE).unwrap();
    send(&mut session, "see", "{}");
    send(&mut session, "both", r#"{"n":1}"#);
    send(&mut session, "one", "{}");
    send(&mut session, "two", "{}");
    assert!(
        published(&mut session).is_empty(),
        "nothing has departed yet"
    );
    // s@1 and u@1 leave their windows and depart in this one event.
    send(&mut session, "both", r#"{"n":2}"#);
    let nodes = published(&mut session);
    let save: Value = serde_json::from_str(&session.save_json().unwrap()).unwrap();
    (session, nodes, save)
}

#[test]
fn a_join_only_an_examination_record_holds_is_published() {
    // Both caveats are examined again, on the second readings, which then
    // depart too: each record held the marker of its history's first reading
    // and now holds one whose root joins two departed readings. A departed
    // record's own node is published with its archive entry whoever holds
    // it, but only a holder publishes a join.
    let (mut session, mut nodes, _) = departed();
    send(&mut session, "one", "{}");
    send(&mut session, "two", "{}");
    nodes.extend(published(&mut session));
    send(&mut session, "both", r#"{"n":3}"#);
    nodes.extend(published(&mut session));
    let save: Value = serde_json::from_str(&session.save_json().unwrap()).unwrap();
    for name in ["k_one", "k_two"] {
        let held = roots(&save, name);
        assert_eq!(held.len(), 1, "{}", save["examination_qualifications"]);
        assert!(
            nodes.contains(&held[0]),
            "{name} names root {}, which the archive never published: {nodes:?}",
            held[0]
        );
    }
}

#[test]
fn records_are_settled_in_the_order_of_their_names() {
    let (_, nodes, save) = departed();
    let (one, two) = (roots(&save, "k_one"), roots(&save, "k_two"));
    assert_eq!(
        (one.len(), two.len()),
        (1, 1),
        "{}",
        save["examination_qualifications"]
    );
    assert_ne!(one[0], two[0], "the two records hold different roots");
    let at = |root: &String| nodes.iter().position(|id| id == root);
    let (first, second) = (at(&one[0]), at(&two[0]));
    assert!(
        first.is_some() && second.is_some(),
        "{first:?} {second:?} {nodes:?}"
    );
    // k_one sorts before k_two, so its root is published first.
    assert!(
        first < second,
        "k_one's root at {first:?}, k_two's at {second:?}: {nodes:?}"
    );
}

#[test]
fn a_restored_session_publishes_what_the_live_one_does() {
    // The same events played on a session restored just before the
    // departure, whose qualification records restore built.
    let mut session = ReactiveSession::from_source(SOURCE).unwrap();
    for (event, payload) in [
        ("see", "{}"),
        ("both", r#"{"n":1}"#),
        ("one", "{}"),
        ("two", "{}"),
    ] {
        send(&mut session, event, payload);
    }
    let mut restored =
        ReactiveSession::restore_json(SOURCE, &session.save_json().unwrap()).unwrap();
    send(&mut session, "both", r#"{"n":2}"#);
    send(&mut restored, "both", r#"{"n":2}"#);
    assert_eq!(published(&mut restored), published(&mut session));
    assert_eq!(restored.save_json().unwrap(), session.save_json().unwrap());
}
