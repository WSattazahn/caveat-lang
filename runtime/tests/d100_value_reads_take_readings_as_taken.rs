//! D100: restore bounded a value read of a stream (`latest`, `history_at`,
//! `history_count`, a fold) and `has_sample` by everything the stream's records
//! can hold, its observation records among them. The runtime reads for it only
//! the reading as it was taken, the stream's selection and the caveats live on
//! what the reading lists. A qualification or a withdrawal joins the
//! observation records that list the record it qualifies, but never a reading,
//! which keeps what its sample read when it was taken. So what such a join's
//! guard read reaches a stream's observation records and none of its readings,
//! and nothing a value read of the stream writes into a reading, a selection,
//! a basis, an outcome, a reopening record, a scheduled qualification's guard
//! or a cue holds it, wherever the effect evaluates the read: its guard, its
//! values, what it cites or selects, or a call's arguments. Restore accepted a
//! forged marker of such a history in each such part: the readings a sample
//! takes when its guard or its value reads the stream, their stream's
//! selection when its guard does, a commitment's basis, a series' selection,
//! an outcome, a reopening record, a scheduled qualification's guard and a
//! cue, with a forged kept take-back caveat beside the marker, which only the
//! marker let qualify the part; and in grounds that read or cite such a part,
//! and grounds that read those. In grounds that cite such a part, and grounds
//! that read those, it accepted a forged kept take-back caveat alone as well.
//! The same held for the `latest(S)` a `because` or a `permitted by` selects,
//! which the runtime reads by value too: in the outcome, the basis and the
//! reopening record of a hold or a release it writes, and in grounds that
//! cite them. The bound now gives each of these reads the reading's name and
//! what the readings and the selection can hold, and restore refuses these.
//! A state lineage, and an observation, examination or predicate record, that
//! lists a record still takes what joins it later, and a read through the
//! records (`observed`, `carries`, `qualified`) still takes the observation
//! records. So do `reopened` and `on_hold` of a commitment that a
//! `reopen ... because latest(S)` reopened, and with them its next revision's
//! basis: they read the reading that reopened it through its records as they
//! stand then. The commitment's reopening record is bounded with what
//! `reopened` reads, and `committed` and `in_basis` carry that bound. See
//! spec/caveat-save-0.1.md ("Restore validation", "Changes").
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

fn restores(source: &str, save: &Value, case: &str) {
    if let Err(error) = ReactiveSession::restore_json(source, &save.to_string()) {
        panic!("{case}: {error}");
    }
}

/// The marker `x@1` left in `h@1`'s observation record when it departed.
fn departed_x(save: &Value) -> Value {
    let record = &save["observation_qualifications"]["h@1"];
    record["departed"]
        .as_array()
        .and_then(|markers| markers.iter().find(|marker| marker["history"] == "x"))
        .cloned()
        .unwrap_or_else(|| panic!("no marker of x in {record}"))
}

fn has_x_marker(provenance: &Value) -> bool {
    provenance["departed"]
        .as_array()
        .is_some_and(|markers| markers.iter().any(|marker| marker["history"] == "x"))
}

fn has_caveat(provenance: &Value, caveat: &str) -> bool {
    ["caveats", "kept"].iter().any(|field| {
        provenance[*field]
            .as_array()
            .is_some_and(|names| names.iter().any(|name| name == caveat))
    })
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
/// lists `y@1`. `qy` qualifies or withdraws `y@1` (`JOIN`) under a guard that
/// reads `x`'s reading, so `x@1` joins every record that lists `y@1`: `h@1`'s
/// observation record among them, never `h@1` itself. `b`'s sample
/// (`SAMPLE`) reads `h` in its guard or its value; `t` cites `b`'s reading,
/// and `u` reads `t`. `shaky` is a take-back caveat that qualifies only
/// `x`'s readings. `e`, which a declaration observes, is a withdrawal's
/// reason.
fn nested(sample: &str, join: &str) -> String {
    format!(
        "claim c;
evidence e from \"e\";
e supports c;
evidence gb from \"gb\";
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
on qy when latest(x) >= 0 {join};
event sb n min 0 max 10;
on sb {sample};
event st;
on st set t = latest(b) because latest(b);
event cu;
on cu set u = t;
"
    )
}

/// The joins `qy` makes.
const QUALIFY: &str = "qualify latest(y) with flag";
const WITHDRAW: &str = "withdraw latest(y) because e";

/// Each value read of `h`: what `latest` reads, or the selection alone.
const VALUE_READS: [&str; 5] = [
    "latest(h) >= 0",
    "has_sample(h)",
    "history_at(h, 0) >= 0",
    "history_count(h) >= 1",
    "fold_history(h, 0, add) >= 0",
];

/// The same reads as `b`'s sample value.
const VALUES: [&str; 5] = [
    "latest(h)",
    "if(has_sample(h), 1, 0)",
    "history_at(h, 0)",
    "history_count(h)",
    "fold_history(h, 0, add)",
];

/// `b`'s sample under a guard that reads `h` by `read`.
fn guarded(read: &str) -> String {
    format!("when {read} sample b = n")
}

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

/// By the time `b`'s sample reads `h@1` by value, in its guard or its value,
/// `h@1`'s observation record holds `x@1`, joined by a qualification or by a
/// withdrawal, and the read takes none of it: `b@1`, `t` and `u` never hold
/// `x` or `shaky`. Restore refuses a marker of `x` in `b@1`, in the grounds
/// of `t` and `u` beyond their lineage and, where the guard reads `h`, in
/// `b`'s selection, which only a skipped sample's guard writes, and `shaky`
/// kept in those grounds. Nothing `b@1` or the selection lists can carry
/// `shaky`, so a kept `shaky` alone was refused there already; beside a
/// marker of `x`, which can, it restored with the marker.
#[test]
fn a_value_read_in_a_sample_takes_no_later_join_of_the_records() {
    let samples = VALUE_READS.iter().map(|read| (guarded(read), true)).chain(
        VALUES
            .iter()
            .map(|value| (format!("sample b = {value}"), false)),
    );
    for (sample, guard) in samples {
        for join in [QUALIFY, WITHDRAW] {
            check_sample(&sample, guard, join);
        }
    }
}

/// One case of the test above: `b`'s sample, whether its guard reads `h`,
/// and the join `qy` makes.
fn check_sample(sample: &str, guard: bool, join: &str) {
    let source = nested(sample, join);
    let save = plays_and_restores(&source, &PLAY);
    assert!(
        has_x_marker(&save["observation_qualifications"]["h@1"]),
        "{join}: {sample}: {}",
        save["observation_qualifications"]
    );
    let marker = departed_x(&save);
    let mut held = save.clone();
    assert!(!has_x_marker(reading(&mut held)), "{join}: {sample}");
    assert!(!has_caveat(reading(&mut held), "shaky"), "{join}: {sample}");
    for state in ["t", "u"] {
        assert!(
            !has_x_marker(state_grounds(&mut held, state)),
            "{join}: {sample}"
        );
        assert!(
            !has_caveat(state_grounds(&mut held, state), "shaky"),
            "{join}: {sample}"
        );
        let mut forged = save.clone();
        add_marker(state_grounds(&mut forged, state), marker.clone());
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
    let places: &[(bool, &str)] = if guard {
        &[(false, "reading b@1"), (true, "reading stream b")]
    } else {
        &[(false, "reading b@1")]
    };
    for &(selection, place) in places {
        let mut forged = save.clone();
        add_marker(in_place(&mut forged, selection), marker.clone());
        refused(
            &source,
            &forged,
            &format!(
                "{place}: no rule or declaration of this program carries records of x into it"
            ),
        );
        let mut forged = save.clone();
        add_kept(in_place(&mut forged, selection), "shaky");
        refused(
            &source,
            &forged,
            &format!("{place}: shaky cannot qualify any of its evidence"),
        );
        add_marker(in_place(&mut forged, selection), marker.clone());
        refused(
            &source,
            &forged,
            &format!(
                "{place}: no rule or declaration of this program carries records of x into it"
            ),
        );
    }
}

/// `b@1`'s provenance in a save, or `b`'s selection.
fn in_place(save: &mut Value, selection: bool) -> &mut Value {
    if selection {
        &mut save["reading_streams"]["b"]["selection_qualifications"]
    } else {
        reading(save)
    }
}

/// A read through `h`'s records (`observed`, or `carries`, whose answer the
/// guard ignores) takes `h@1`'s observation record as it stands, `x@1`
/// joined: `b@1` holds it, and the marker it leaves, and so do `t` and `u`,
/// lineage and grounds, which restore accepts.
#[test]
fn a_read_through_the_records_takes_what_joins_them() {
    for read in [
        "observed(reading(h, 1))",
        "if(carries(latest(h), flag), 1, 1) >= 1",
    ] {
        let source = nested(&guarded(read), QUALIFY);
        let save = plays_and_restores(&source, &PLAY);
        let mut held = save.clone();
        assert!(
            has_x_marker(reading(&mut held)),
            "{read}: {}",
            save["reading_streams"]
        );
        for state in ["t", "u"] {
            assert!(
                has_x_marker(&save["states"][state]["lineage"]),
                "{read}: {state}"
            );
            assert!(
                has_x_marker(state_grounds(&mut held, state)),
                "{read}: {state}"
            );
        }
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
        let source = nested(&guarded(read), QUALIFY);
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
        let marker = departed_x(&save);
        for state in ["t", "u"] {
            let cell = &save["states"][state];
            assert!(has_x_marker(&cell["lineage"]), "{read}: {state}: {cell}");
            let mut held = save.clone();
            assert!(
                !has_x_marker(state_grounds(&mut held, state)),
                "{read}: {state}: {cell}"
            );
            let mut forged = save.clone();
            add_marker(state_grounds(&mut forged, state), marker.clone());
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

/// What `because latest(h)` and `permitted by latest(h)` select is `h`'s
/// reading as it was taken, with `h`'s selection, as a value read takes it:
/// `select_because` and `check_permission` call `latest`. So the outcome a
/// resolve writes, a commitment's basis, and the reopening record a hold or
/// a release joins never hold what joins `h@1`'s observation record later,
/// and nor do the grounds of `t` and `w`, which cite that outcome and that
/// basis, of `t2` and `w2`, which read those, or of `s`, which cites
/// `has_sample(h)`. A reopening's record is bounded with what `reopened`
/// reads through the records of what reopened it
/// (docs/RESTORE_TRUST_BOUNDARY.md), so a marker there still restores.
fn selected() -> &'static str {
    "evidence gh from \"gh\";
evidence gy from \"gy\";
evidence gx from \"gx\";
evidence e from \"e\";
readings h from gh window 1;
readings y from gy window 1;
readings x from gx window 1;
caveat flag consequence low;
caveat shaky consequence low;
decisions d limit 4;
decisions p limit 4;
decisions g limit 4;
decisions r limit 4;
decisions k limit 4;
state t = 0;
state w = 0;
state s = 0;
state t2 = 0;
state w2 = 0;
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
event go;
on go reveal e;
event cd;
on cd commit d because enough using 1;
event rd;
on rd resolve d = 1 because latest(h);
event cp;
on cp commit p because enough using 1 permitted by latest(h);
event cg;
on cg commit g because enough using 1;
event hg;
on hg hold g because latest(h);
event cr;
on cr commit r because enough using 1;
event hr;
on hr hold r because e;
event rr;
on rr release r because latest(h);
event ck;
on ck commit k because enough using 1;
event rk;
on rk when committed(k) and not reopened(k) reopen k because latest(h);
event st;
on st set t = outcome(d) because outcome(d);
event sw;
on sw set w = latest(p) because latest(p);
event ss;
on ss set s = if(has_sample(h), 1, 0) because has_sample(h);
event rt;
on rt set t2 = t;
on rt set w2 = w;
"
}

#[test]
fn a_selected_latest_takes_the_reading_as_taken() {
    let source = selected();
    // `qy` joins `x@1` to `h@1`'s observation record before the reads, or
    // after them, when it joins the lineages of `t`, `w` and `s` too.
    for early in [true, false] {
        let mut play = vec![
            ("sx", r#"{"n":1}"#),
            ("sy", r#"{"n":1}"#),
            ("sh", r#"{"n":1}"#),
            ("go", "{}"),
            ("qx", "{}"),
        ];
        let reads = [
            "cd", "rd", "cp", "cg", "hg", "cr", "hr", "rr", "ck", "rk", "st", "sw", "ss", "rt",
        ];
        if early {
            play.push(("qy", "{}"));
        }
        play.extend(reads.iter().map(|event| (*event, "{}")));
        if !early {
            play.push(("qy", "{}"));
        }
        play.extend([("ux", "{}"), ("sx", r#"{"n":2}"#), ("sy", r#"{"n":2}"#)]);
        let save = plays_and_restores(source, &play);
        assert!(
            has_x_marker(&save["observation_qualifications"]["h@1"]),
            "{early}: {}",
            save["observation_qualifications"]
        );
        let marker = departed_x(&save);
        for (pointer, place) in [
            (
                "/decision_series/d/revisions/0/outcome/lineage",
                "outcome of d@1",
            ),
            ("/commitment_bases/p@1/provenance", "commitment p@1"),
            ("/reopening_qualifications/g@1", "reopening of g@1"),
            ("/reopening_qualifications/r@1", "reopening of r@1"),
        ] {
            let held = save.pointer(pointer).unwrap();
            assert!(!has_x_marker(held), "{early}: {place}: {held}");
            assert!(!has_caveat(held, "shaky"), "{early}: {place}: {held}");
            let marked = format!(
                "{place}: no rule or declaration of this program carries records of x into it"
            );
            let mut forged = save.clone();
            add_marker(forged.pointer_mut(pointer).unwrap(), marker.clone());
            refused(source, &forged, &marked);
            let mut forged = save.clone();
            add_kept(forged.pointer_mut(pointer).unwrap(), "shaky");
            refused(
                source,
                &forged,
                &format!("{place}: shaky cannot qualify any of its evidence"),
            );
            add_marker(forged.pointer_mut(pointer).unwrap(), marker.clone());
            refused(source, &forged, &marked);
        }
        let mut forged = save.clone();
        add_marker(
            forged.pointer_mut("/reopening_qualifications/k@1").unwrap(),
            marker.clone(),
        );
        restores(source, &forged, &format!("{early}: k@1"));
        // For `s`, D099 refused all but the late marker already: that one
        // only reading `has_sample` by value refuses.
        for state in ["t", "w", "s", "t2", "w2"] {
            assert_eq!(
                has_x_marker(&save["states"][state]["lineage"]),
                !early,
                "{early}: {state}"
            );
            let mut held = save.clone();
            assert!(
                !has_x_marker(state_grounds(&mut held, state)),
                "{early}: {state}"
            );
            let mut forged = save.clone();
            add_marker(state_grounds(&mut forged, state), marker.clone());
            refused(
                source,
                &forged,
                &if early {
                    format!("state {state} grounds hold departed x records outside its lineage")
                } else {
                    format!(
                        "state {state} grounds: no rule or declaration of this program carries records of x into it"
                    )
                },
            );
            let mut forged = save.clone();
            add_kept(state_grounds(&mut forged, state), "shaky");
            refused(
                source,
                &forged,
                &format!("state {state} grounds include caveat shaky outside its lineage"),
            );
        }
    }
}

/// A `reopen ... because latest(h)` keeps the wider bound. `reopened` and
/// `on_hold` of `k`, and with them `k`'s next revision's basis, read what
/// reopened `k` through its records as they stand then, though the effect
/// reads `h` by value as well. `committed` and `in_basis` read `k`'s
/// reopening as its record holds it, and `k@1`'s reopening record, written
/// before `qy` joins `x@1` to `h@1`'s observation record, never lists `x@1`.
/// When `qy` runs before `cq`, `qr@1` and `qo@1`, which `cq` writes from
/// `reopened` and `on_hold`, list `x@1` and keep `shaky` once `ux` takes it
/// back, and so do the grounds of `vr` and `vo`, which cite them, while
/// `q@1` and `qi@1`, written from `committed` and `in_basis`, and the
/// grounds of `v` and `vi` hold neither. `k@2`'s basis, committed after the
/// join, lists `x@1`, and so do the grounds of `w`, which cite it, and of
/// `u`, which reads `w`. When `qy` runs after `cq`, none of the four bases
/// holds `x`, and `x@1` departs. The reopening record is bounded with what
/// `reopened` reads, and `committed` and `in_basis` carry that bound, so
/// restore accepts a marker of `x` in each of the four bases and in the
/// grounds that cite them, and `shaky` kept in those grounds. The save
/// restores after every event.
#[test]
fn reads_of_a_reopened_commitment_go_through_what_reopened_it() {
    let source = "evidence gh from \"gh\";
evidence gy from \"gy\";
evidence gx from \"gx\";
evidence e from \"e\";
readings h from gh window 1;
readings y from gy window 1;
readings x from gx window 1;
caveat flag consequence low;
caveat shaky consequence low;
decisions k limit 4;
decisions q limit 4;
decisions qi limit 4;
decisions qr limit 4;
decisions qo limit 4;
state v = 0;
state vi = 0;
state vr = 0;
state vo = 0;
state w = 0;
state u = 0;
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
event go;
on go reveal e;
event ck;
on ck commit k because enough using 1;
event rk;
on rk when committed(k) and not reopened(k) reopen k because latest(h);
event cq;
on cq when latest(h) >= 0 commit q because enough using if(committed(k), 1, 0);
on cq when latest(h) >= 0 commit qi because enough using if(in_basis(k, e), 1, 0);
on cq when latest(h) >= 0 commit qr because enough using if(reopened(k), 1, 0);
on cq when latest(h) >= 0 commit qo because enough using if(on_hold(k), 1, 0);
event sv;
on sv set v = latest(q) because latest(q);
on sv set vi = latest(qi) because latest(qi);
on sv set vr = latest(qr) because latest(qr);
on sv set vo = latest(qo) because latest(qo);
event sw;
on sw set w = latest(k) because latest(k);
event su;
on su set u = w;
";
    let lists_x = |provenance: &Value| {
        provenance["evidence"]
            .as_array()
            .is_some_and(|names| names.iter().any(|name| name == "x@1"))
    };
    let keeps_shaky = |provenance: &Value| {
        provenance["kept"]
            .as_array()
            .is_some_and(|names| names.iter().any(|name| name == "shaky"))
    };
    // Each series `cq` commits, the state that cites it, and whether `cq`
    // reads `k` through what reopened it.
    let parts = [
        ("q", "v", false),
        ("qi", "vi", false),
        ("qr", "vr", true),
        ("qo", "vo", true),
    ];
    for join_first in [true, false] {
        let mut play = vec![
            ("sx", r#"{"n":1}"#),
            ("sy", r#"{"n":1}"#),
            ("sh", r#"{"n":1}"#),
            ("go", "{}"),
            ("qx", "{}"),
            ("ck", "{}"),
            ("rk", "{}"),
        ];
        if join_first {
            play.extend([
                ("qy", "{}"),
                ("cq", "{}"),
                ("sv", "{}"),
                ("ck", "{}"),
                ("sw", "{}"),
                ("su", "{}"),
                ("ux", "{}"),
            ]);
        } else {
            // Nothing names `x@1`, so the second `sx` departs it.
            play.extend([
                ("cq", "{}"),
                ("qy", "{}"),
                ("sv", "{}"),
                ("ux", "{}"),
                ("sx", r#"{"n":2}"#),
            ]);
        }
        let save = plays_and_restores(source, &play);
        let reopening = &save["reopening_qualifications"]["k@1"];
        assert!(!lists_x(reopening), "{join_first}: {reopening}");
        let mut held = save.clone();
        for (series, state, through) in parts {
            let holds = join_first && through;
            let basis = &save["commitment_bases"][format!("{series}@1")]["provenance"];
            assert_eq!(lists_x(basis), holds, "{join_first}: {series}@1: {basis}");
            assert_eq!(
                keeps_shaky(basis),
                holds,
                "{join_first}: {series}@1: {basis}"
            );
            let grounds = state_grounds(&mut held, state);
            assert_eq!(lists_x(grounds), holds, "{join_first}: {state}: {grounds}");
            assert_eq!(
                keeps_shaky(grounds),
                holds,
                "{join_first}: {state}: {grounds}"
            );
        }
        if join_first {
            let basis = &save["commitment_bases"]["k@2"]["provenance"];
            assert!(lists_x(basis), "{basis}");
            for state in ["w", "u"] {
                let grounds = state_grounds(&mut held, state);
                assert!(lists_x(grounds), "{state}: {grounds}");
            }
            continue;
        }
        let marker = departed_x(&save);
        for (series, state, _) in parts {
            let mut forged = save.clone();
            add_marker(
                forged
                    .pointer_mut(&format!("/commitment_bases/{series}@1/provenance"))
                    .unwrap(),
                marker.clone(),
            );
            restores(source, &forged, &format!("{series}@1"));
            let mut forged = save.clone();
            add_marker(state_grounds(&mut forged, state), marker.clone());
            restores(source, &forged, state);
            let mut forged = save.clone();
            add_kept(state_grounds(&mut forged, state), "shaky");
            restores(source, &forged, state);
        }
    }
}

/// A value read of `h` anywhere else takes the reading as taken too: in a
/// commit's guard, its `using`, what it cites, a permission's `for` and a
/// call's argument (the basis), a resolve's value (the outcome), a hold's
/// and a reopen's guard (the reopening record), a skipped commit's guard
/// (the series' selection), a scheduled qualification's guard and its
/// delay, and an emit's guard (the cue). None of them holds `x`, whether
/// `qy` joins it to `h@1`'s observation record before the reads or after,
/// and restore refuses a marker of `x` in each, alone or with `shaky` kept
/// beside it. A kept `shaky` alone was refused there already. The grounds
/// of `vq`, `vq2` and `vd`, which cite a basis or an outcome, and of `bq`,
/// `bq2` and `bd`, which read those, never hold `x` or `shaky` either, and
/// restore refuses a marker of `x` there, whether or not their lineage holds
/// one, and `shaky` kept there.
fn parts() -> &'static str {
    "evidence gh from \"gh\";
evidence gy from \"gy\";
evidence gx from \"gx\";
evidence gg from \"gg\";
evidence e from \"e\";
readings h from gh window 1;
readings y from gy window 1;
readings x from gx window 1;
readings gr from gg window 1;
caveat flag consequence low;
caveat shaky consequence low;
caveat later consequence low;
caveat sooner consequence low;
cue note toast \"Noted\" 1;
decisions q limit 4;
decisions q2 limit 4;
decisions qc limit 4;
decisions qp limit 4;
decisions qs limit 4;
decisions d limit 4;
decisions g limit 4;
decisions sk limit 4;
decisions k limit 4;
state vq = 0;
state vq2 = 0;
state vd = 0;
state bq = 0;
state bq2 = 0;
state bd = 0;
proc step(v) { commit qs because enough using 1; };
event tick dt min 0 max 0.1;
clock tick every 0.1;
event sx n min 0 max 10;
on sx sample x = n;
on sx when has_sample(h) emit note;
event sy n min 0 max 10;
on sy sample y = n;
event sh n min 0 max 10;
on sh when latest(y) >= 0 sample h = n;
event sg n min 0 max 10;
on sg sample gr = n;
event qx;
on qx qualify latest(x) with shaky;
event ux;
on ux unqualify latest(x) with shaky;
event qy;
on qy when latest(x) >= 0 qualify latest(y) with flag;
event go;
on go reveal e;
event cq;
on cq when latest(h) >= 0 commit q because enough using 1;
event cq2;
on cq2 commit q2 because enough using latest(h);
event cqc;
on cqc commit qc because enough citing latest(h);
event cqp;
on cqp commit qp because enough using 1 permitted by latest(gr) for latest(h);
event cqs;
on cqs call step(latest(h));
event cd;
on cd commit d because enough using 1;
event rd;
on rd resolve d = latest(h) because e;
event cg;
on cg commit g because enough using 1;
event hg;
on hg when latest(h) >= 0 hold g because e;
event csk;
on csk when latest(h) < 0 commit sk because enough using 1;
event ck;
on ck commit k because enough using 1;
event rk;
on rk when latest(h) >= 0 reopen k because e;
event sq;
on sq when latest(h) >= 0 qualify e with later after 1000;
event sq2;
on sq2 qualify e with sooner after latest(h) + 1000;
event cv;
on cv set vq = latest(q) because latest(q);
on cv set vq2 = latest(q2) because latest(q2);
on cv set vd = outcome(d) because outcome(d);
event bv;
on bv set bq = vq;
on bv set bq2 = vq2;
on bv set bd = vd;
"
}

#[test]
fn a_value_read_anywhere_takes_the_reading_as_taken() {
    let source = parts();
    for early in [true, false] {
        let mut play = vec![
            ("sx", r#"{"n":1}"#),
            ("sy", r#"{"n":1}"#),
            ("sh", r#"{"n":1}"#),
            ("sg", r#"{"n":1}"#),
            ("go", "{}"),
            ("qx", "{}"),
        ];
        let reads = [
            "cq", "cq2", "cqc", "cqp", "cqs", "cd", "rd", "cg", "hg", "csk", "ck", "rk", "sq",
            "sq2", "cv", "bv",
        ];
        if early {
            play.push(("qy", "{}"));
        }
        play.extend(reads.iter().map(|event| (*event, "{}")));
        if !early {
            play.push(("qy", "{}"));
        }
        // The second `sx` departs `x@1` and emits the cue.
        play.extend([("ux", "{}"), ("sx", r#"{"n":2}"#)]);
        let save = plays_and_restores(source, &play);
        assert!(
            has_x_marker(&save["observation_qualifications"]["h@1"]),
            "{early}: {}",
            save["observation_qualifications"]
        );
        assert_eq!(save["cues"], json!(["note"]), "{early}");
        let marker = departed_x(&save);
        for (pointer, place, kept_place) in [
            (
                "/commitment_bases/q@1/provenance",
                "commitment q@1",
                "commitment q@1",
            ),
            (
                "/commitment_bases/q2@1/provenance",
                "commitment q2@1",
                "commitment q2@1",
            ),
            (
                "/commitment_bases/qc@1/provenance",
                "commitment qc@1",
                "commitment qc@1",
            ),
            (
                "/commitment_bases/qp@1/provenance",
                "commitment qp@1",
                "commitment qp@1",
            ),
            (
                "/commitment_bases/qs@1/provenance",
                "commitment qs@1",
                "commitment qs@1",
            ),
            (
                "/decision_series/d/revisions/0/outcome/lineage",
                "outcome of d@1",
                "outcome of d@1",
            ),
            (
                "/reopening_qualifications/g@1",
                "reopening of g@1",
                "reopening of g@1",
            ),
            (
                "/reopening_qualifications/k@1",
                "reopening of k@1",
                "reopening of k@1",
            ),
            (
                "/decision_series/sk/selection_qualifications",
                "decision series sk",
                "decision series sk",
            ),
            (
                "/scheduled_qualifications/0/guard",
                "scheduled qualification of e",
                "scheduled qualification",
            ),
            (
                "/scheduled_qualifications/1/guard",
                "scheduled qualification of e",
                "scheduled qualification",
            ),
            ("/cue_qualifications/0", "cue note", "cue note"),
        ] {
            let held = save.pointer(pointer).unwrap();
            assert!(!has_x_marker(held), "{early}: {place}: {held}");
            assert!(!has_caveat(held, "shaky"), "{early}: {place}: {held}");
            let marked = format!(
                "{place}: no rule or declaration of this program carries records of x into it"
            );
            let mut forged = save.clone();
            add_marker(forged.pointer_mut(pointer).unwrap(), marker.clone());
            refused(source, &forged, &marked);
            let mut forged = save.clone();
            add_kept(forged.pointer_mut(pointer).unwrap(), "shaky");
            refused(
                source,
                &forged,
                &format!("{kept_place}: shaky cannot qualify any of its evidence"),
            );
            add_marker(forged.pointer_mut(pointer).unwrap(), marker.clone());
            refused(source, &forged, &marked);
        }
        for state in ["vq", "vq2", "vd", "bq", "bq2", "bd"] {
            assert_eq!(
                has_x_marker(&save["states"][state]["lineage"]),
                !early,
                "{early}: {state}"
            );
            let mut held = save.clone();
            let grounds = state_grounds(&mut held, state);
            assert!(!has_x_marker(grounds), "{early}: {state}: {grounds}");
            assert!(!has_caveat(grounds, "shaky"), "{early}: {state}: {grounds}");
            let mut forged = save.clone();
            add_marker(state_grounds(&mut forged, state), marker.clone());
            refused(
                source,
                &forged,
                &if early {
                    format!("state {state} grounds hold departed x records outside its lineage")
                } else {
                    format!(
                        "state {state} grounds: no rule or declaration of this program carries records of x into it"
                    )
                },
            );
            let mut forged = save.clone();
            add_kept(state_grounds(&mut forged, state), "shaky");
            refused(
                source,
                &forged,
                &format!("state {state} grounds include caveat shaky outside its lineage"),
            );
        }
    }
}
