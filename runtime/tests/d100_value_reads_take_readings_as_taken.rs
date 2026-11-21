//! D100: restore bounded a value read of a stream (`latest`, `history_at`,
//! `history_count`, a fold) and `has_sample` by everything the stream's
//! records can hold, its observation records among them. The runtime reads
//! for it only the reading as it was taken, the stream's selection and the
//! caveats live on what the reading lists. A qualification joins the
//! observation records that list the record it qualifies, but never a
//! reading, which keeps what its sample read when it was taken. So when a
//! sample's guard reads another stream by value, what such a qualification's
//! guard read joins that stream's observation records and none of the
//! readings taken, and no session holds it there. Restore accepted a forged
//! marker, or a forged kept take-back caveat, of such a history in those
//! readings, in their stream's selection, and in grounds that read them,
//! though no release accepts it in grounds a citation reads (D057, D099).
//! The bound now gives a value read of a stream the reading's name and what
//! the readings and the selection can hold, and restore refuses these. A
//! state lineage that lists a record still takes what joins it later, and a
//! read through the records (`observed`, `carries`, `qualified`) still
//! takes the observation records. See spec/caveat-save-0.1.md ("Restore
//! validation", "Changes").
use caveat_runtime::reactive::ReactiveSession;
use serde_json::{json, Value};

fn answer(session: &mut ReactiveSession, event: &str, payload: &str) -> Value {
    let outcome = session
        .dispatch_outcome_json(event, payload)
        .unwrap_or_else(|fatal| panic!("{event} was fatal: {}", fatal.message));
    serde_json::to_value(outcome).unwrap()
}

/// Plays `events`, each accepted. After each, the save restores and saves the
/// same bytes again, and every session restored so far answers the events
/// still to come as the live one does. Returns the last save.
fn plays_and_restores(source: &str, events: &[(&str, &str)]) -> Value {
    let mut live = ReactiveSession::from_source(source).unwrap();
    let mut restored: Vec<ReactiveSession> = Vec::new();
    for (index, (event, payload)) in events.iter().enumerate() {
        let outcome = answer(&mut live, event, payload);
        assert_eq!(
            outcome["outcome"], "accepted",
            "{index}: {event}: {outcome}"
        );
        for run in &mut restored {
            assert_eq!(answer(run, event, payload), outcome, "{index}: {event}");
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

/// The marker `x@1` leaves where a provenance names it: it departs at
/// sequence 7, the second `sx`.
fn x_marker() -> Value {
    json!({"history": "x", "read": 1, "from": 1, "through": 1, "departed_at": 7})
}

fn has_x_marker(provenance: &Value) -> bool {
    provenance["departed"]
        .as_array()
        .is_some_and(|markers| markers.iter().any(|marker| marker["history"] == "x"))
}

/// A state's grounds in a save, written out from its lineage where the save
/// leaves them out as equal to it.
fn state_grounds<'v>(save: &'v mut Value, state: &str) -> &'v mut Value {
    let cell = &mut save["states"][state];
    if cell.get("grounds").is_none() {
        cell["grounds"] = cell["lineage"].clone();
    }
    &mut cell["grounds"]
}

/// The provenance of stream `b`'s first reading in a save.
fn reading(save: &mut Value) -> &mut Value {
    let reading = &mut save["reading_streams"]["b"]["occurrences"][0];
    assert_eq!(reading["id"], "b@1");
    &mut reading["provenance"]
}

fn add_marker(provenance: &mut Value, marker: Value) {
    let mut departed = provenance["departed"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    departed.push(marker);
    departed.sort_by(|x, y| x["history"].as_str().cmp(&y["history"].as_str()));
    provenance["departed"] = json!(departed);
}

fn add_kept(provenance: &mut Value, caveat: &str) {
    for field in ["caveats", "kept"] {
        let mut names: Vec<String> = provenance[field]
            .as_array()
            .into_iter()
            .flatten()
            .map(|name| name.as_str().unwrap().to_string())
            .collect();
        assert!(!names.iter().any(|name| name == caveat), "{provenance}");
        names.push(caveat.to_string());
        names.sort();
        provenance[field] = json!(names);
    }
}

/// `h`'s sample guard reads `y`'s reading, so `h@1`'s observation record
/// lists `y@1`. `qy` qualifies `y@1` under a guard that reads `x`'s reading,
/// so `x@1` joins every record that lists `y@1`: `h@1`'s observation record
/// among them, never `h@1` itself. `b`'s sample guard reads `h` by `READ`;
/// `t` cites `b`'s reading, and `u` reads `t`. `shaky` is a take-back caveat
/// that qualifies only `x`'s readings.
fn nested(read: &str) -> String {
    format!(
        "evidence gb from \"gb\";
evidence gh from \"gh\";
evidence gy from \"gy\";
evidence gx from \"gx\";
readings b from gb window 1;
readings h from gh window 1;
readings y from gy window 1;
readings x from gx window 1;
caveat flag consequence low;
caveat shaky consequence low;
state t = 0;
state u = 0;
fn add(a, v) = a + v;
event sx n min 0 max 10;
on sx sample x = n;
event sy n min 0 max 10;
on sy sample y = n;
event sh n min 0 max 10;
on sh when latest(y) >= 0 sample h = n;
event qx;
on qx qualify latest(x) with shaky;
event ux;
on ux unqualify latest(x) with shaky;
event qy;
on qy when latest(x) >= 0 qualify latest(y) with flag;
event sb n min 0 max 10;
on sb when {read} sample b = n;
event st;
on st set t = latest(b) because latest(b);
event cu;
on cu set u = t;
"
    )
}

/// Each value read of `h`: what `latest` reads, or the selection alone.
const VALUE_READS: [&str; 5] = [
    "latest(h) >= 0",
    "has_sample(h)",
    "history_at(h, 0) >= 0",
    "history_count(h) >= 1",
    "fold_history(h, 0, add) >= 0",
];

const PLAY: [(&str, &str); 9] = [
    ("sx", r#"{"n":1}"#),
    ("sy", r#"{"n":1}"#),
    ("sh", r#"{"n":1}"#),
    ("qx", "{}"),
    ("qy", "{}"),
    ("sb", r#"{"n":1}"#),
    ("sx", r#"{"n":2}"#),
    ("st", "{}"),
    ("cu", "{}"),
];

/// By the time `b`'s guard reads `h@1` by value, `h@1`'s observation record
/// holds `x@1`, and the read takes none of it: `b@1`, `t` and `u` never hold
/// `x` or `shaky`. Restore refuses a marker of `x` in `b@1`, in `b`'s
/// selection, or in the grounds of `t` and `u` beyond their lineage, and
/// `shaky` kept there.
#[test]
fn a_value_read_in_a_sample_takes_no_later_join_of_the_records() {
    for read in VALUE_READS {
        let source = nested(read);
        let save = plays_and_restores(&source, &PLAY);
        assert!(
            has_x_marker(&save["observation_qualifications"]["h@1"]),
            "{read}: {}",
            save["observation_qualifications"]
        );
        let mut held = save.clone();
        assert!(!has_x_marker(reading(&mut held)), "{read}");
        for state in ["t", "u"] {
            assert!(!has_x_marker(state_grounds(&mut held, state)), "{read}");
            let mut forged = save.clone();
            add_marker(state_grounds(&mut forged, state), x_marker());
            refused(
                &source,
                &forged,
                &format!("state {state} grounds hold departed x records outside its lineage"),
            );
            let mut forged = save.clone();
            add_kept(state_grounds(&mut forged, state), "shaky");
            refused(
                &source,
                &forged,
                &format!("state {state} grounds include caveat shaky outside its lineage"),
            );
        }
        let mut forged = save.clone();
        add_marker(reading(&mut forged), x_marker());
        refused(
            &source,
            &forged,
            "reading b@1: no rule or declaration of this program carries records of x into it",
        );
        let mut forged = save.clone();
        add_marker(
            &mut forged["reading_streams"]["b"]["selection_qualifications"],
            x_marker(),
        );
        refused(
            &source,
            &forged,
            "reading stream b: no rule or declaration of this program carries records of x into it",
        );
    }
}

/// A read through `h`'s records takes `h@1`'s observation record as it
/// stands, `x@1` joined: `b@1` holds it, and the marker it leaves, and so do
/// `t` and `u`, lineage and grounds, which restore accepts.
#[test]
fn a_read_through_the_records_takes_what_joins_them() {
    let source = nested("observed(reading(h, 1))");
    let save = plays_and_restores(&source, &PLAY);
    let mut held = save.clone();
    assert!(
        has_x_marker(reading(&mut held)),
        "{}",
        save["reading_streams"]
    );
    for state in ["t", "u"] {
        assert!(has_x_marker(&save["states"][state]["lineage"]), "{state}");
        assert!(has_x_marker(state_grounds(&mut held, state)), "{state}");
    }
}

/// A state's lineage lists what its value read, so a later qualification of
/// a record it lists joins it. Here `qy` runs after `t` and `u` are set,
/// so `x@1` joins their lineage through `y@1`, and the marker it leaves
/// stays there, which restore accepts. Their grounds keep what was cited
/// when it was cited, and a qualification adds only its caveats to them, so
/// they never hold `x`: restore refuses the marker there, within the
/// lineage's though it is.
#[test]
fn a_lineage_still_takes_what_joins_the_records_it_lists() {
    for read in VALUE_READS {
        let source = nested(read);
        let save = plays_and_restores(
            &source,
            &[
                ("sx", r#"{"n":1}"#),
                ("sy", r#"{"n":1}"#),
                ("sh", r#"{"n":1}"#),
                ("sb", r#"{"n":1}"#),
                ("st", "{}"),
                ("cu", "{}"),
                ("qy", "{}"),
                ("sx", r#"{"n":2}"#),
            ],
        );
        for state in ["t", "u"] {
            let cell = &save["states"][state];
            assert!(has_x_marker(&cell["lineage"]), "{read}: {state}: {cell}");
            assert!(!has_x_marker(&cell["grounds"]), "{read}: {state}: {cell}");
            let mut forged = save.clone();
            add_marker(state_grounds(&mut forged, state), x_marker());
            refused(
                &source,
                &forged,
                &format!(
                    "state {state} grounds: no rule or declaration of this program carries records of x into it"
                ),
            );
        }
    }
}
