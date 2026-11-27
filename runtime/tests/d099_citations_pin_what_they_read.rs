//! D099: D057's bound let restore accept a forged grounds marker, and
//! D098's a forged kept take-back caveat, of the records a `because`
//! citation reads itself, and of the histories it only names. The bound
//! started with all a citation gives grounds. A record the grounds read or
//! name is pinned while they hold it, so it does not depart and no session
//! holds a marker of it beyond the lineage; restore before D057 refused
//! these saves, as every release does. Restore now starts the bound with
//! what the provenances a citation reads can hold: a reading brings what
//! it holds and its stream's selection; a name alone brings nothing, nor do
//! the reasons and causes a withdrawal predicate, `on_hold` or `reopened`
//! names. A reading taken while its template is qualified with a take-back
//! caveat keeps that caveat, so a citation of the reading's stream brings
//! it kept, as a citation of anything whose provenance kept it does, and
//! what reads those grounds takes it along; a name, a reason or a cause
//! alone brings nothing kept. Restore accepts what a reading is taken
//! keeping from its template only where the lineage carries it live and
//! the grounds name such a reading, and never in a commitment's grounds
//! beyond its basis. See
//! spec/caveat-lineage-compaction-0.1.md ("Meaning", "Caveats") and
//! spec/caveat-save-0.1.md ("Changes").
use caveat_runtime::reactive::ReactiveSession;
use serde_json::{json, Value};

fn answer(session: &mut ReactiveSession, event: &str, payload: &str) -> Value {
    let outcome = session
        .dispatch_outcome_json(event, payload)
        .unwrap_or_else(|fatal| panic!("{event} was fatal: {}", fatal.message));
    serde_json::to_value(outcome).unwrap()
}

/// Plays `events`, each with the outcome it must have (`accepted`, or the
/// code it is rejected with). After each, the save restores and saves the
/// same bytes again, and every session restored so far answers the events
/// still to come as the live one does. Returns the last save.
fn plays_and_restores(source: &str, events: &[(&str, &str, &str)]) -> Value {
    let mut live = ReactiveSession::from_source(source).unwrap();
    let mut restored: Vec<ReactiveSession> = Vec::new();
    for (index, (event, payload, expected)) in events.iter().enumerate() {
        let outcome = answer(&mut live, event, payload);
        if *expected == "accepted" {
            assert_eq!(
                outcome["outcome"], "accepted",
                "{index}: {event}: {outcome}"
            );
        } else {
            assert_eq!(
                outcome["outcome"], "rejected",
                "{index}: {event}: {outcome}"
            );
            assert_eq!(outcome["code"], *expected, "{index}: {event}: {outcome}");
        }
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

fn marker(history: &str, read: u64, from: u64, through: u64, departed_at: u64) -> Value {
    json!({
        "history": history,
        "read": read,
        "from": from,
        "through": through,
        "departed_at": departed_at,
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

fn add_marker(provenance: &mut Value, marker: Value) {
    let mut departed = provenance["departed"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    departed.push(marker);
    departed.sort_by(|x, y| x["history"].as_str().cmp(&y["history"].as_str()));
    provenance["departed"] = json!(departed);
}

/// The grounds of a state, or of a commitment (`r@1`), in a save.
fn grounds<'v>(save: &'v mut Value, place: &str) -> &'v mut Value {
    if place.contains('@') {
        &mut save["commitment_grounds"][place]
    } else {
        state_grounds(save, place)
    }
}

/// `save` with `caveat` added to `place`'s grounds, kept: to their caveats
/// unless they carry it live already. A commitment's journal entry lists the
/// caveats of its grounds too.
fn with_kept(save: &Value, place: &str, caveat: &str) -> Value {
    let mut forged = save.clone();
    let held = grounds(&mut forged, place);
    for field in ["caveats", "kept"] {
        let mut names: Vec<String> = held[field]
            .as_array()
            .into_iter()
            .flatten()
            .map(|name| name.as_str().unwrap().to_string())
            .collect();
        if names.iter().any(|name| name == caveat) {
            assert_eq!(field, "caveats", "{place} keeps {caveat} already: {held}");
            continue;
        }
        names.push(caveat.to_string());
        names.sort();
        held[field] = json!(names);
    }
    let caveats = held["caveats"].clone();
    if place.contains('@') {
        for entry in forged["decision_journal"].as_array_mut().unwrap() {
            if entry["commitment"] == place && entry["change"] == "committed" {
                entry["caveats"] = caveats.clone();
            }
        }
    }
    forged
}

/// Restore refuses, at each of `places`, grounds with `marker` added, and
/// grounds with `caveat` added and kept.
fn forgeries_refused(source: &str, save: &Value, places: &[&str], marker: &Value, caveat: &str) {
    let history = marker["history"].as_str().unwrap();
    for place in places {
        let what = if place.contains('@') {
            format!("commitment {place}")
        } else {
            format!("state {place}")
        };
        let mut forged = save.clone();
        add_marker(grounds(&mut forged, place), marker.clone());
        refused(
            source,
            &forged,
            &format!("{what} grounds hold departed {history} records outside its lineage"),
        );
        refused(
            source,
            &with_kept(save, place, caveat),
            &format!("{what} grounds include caveat {caveat} outside its lineage"),
        );
    }
}

/// `t` cites what its value reads, `latest(b)`; `u` reads `t`, `p` gives it
/// to `z`, and `r` cites it. `t`'s grounds hold nothing or the reading they
/// name and pin, so no grounds here holds a `b` marker beyond its lineage
/// or basis. Nor do they keep `shaky`: `qual` puts it on a reading after
/// the reading is taken, so the reading carries it live.
const SAME: &str = "evidence gb from \"gb\";
readings b from gb window 1;
caveat shaky consequence low;
decisions r limit 4;
state t = 0;
state u = 0;
state z = 0;
proc p(y) { set z = y; };
event sb v min 0 max 10;
on sb sample b = v;
event qual v min 0 max 10;
on qual qualify latest(b) with shaky;
event uq v min 0 max 10;
on uq unqualify latest(b) with shaky;
event st v min 0 max 10;
on st set t = latest(b) because latest(b);
event cu v min 0 max 10;
on cu set u = t;
event cr v min 0 max 10;
on cr commit r because enough citing t;
event cz v min 0 max 10;
on cz call p(t);
";

#[test]
fn a_citation_of_what_the_value_reads_carries_no_marker_of_it() {
    let save = plays_and_restores(
        SAME,
        &[
            ("sb", r#"{"v":5}"#, "accepted"),
            ("sb", r#"{"v":6}"#, "accepted"),
            ("sb", r#"{"v":7}"#, "accepted"),
            ("qual", r#"{"v":1}"#, "accepted"),
            ("st", r#"{"v":1}"#, "accepted"),
            ("cu", r#"{"v":1}"#, "accepted"),
            ("cr", r#"{"v":1}"#, "accepted"),
            ("cz", r#"{"v":1}"#, "accepted"),
            ("uq", r#"{"v":1}"#, "accepted"),
            ("sb", r#"{"v":8}"#, "accepted"),
            ("sb", r#"{"v":9}"#, "accepted"),
        ],
    );
    // `b@3`, which `t`'s grounds name, stays; `b@1` and `b@2` departed at
    // sequences 2 and 3.
    for state in ["t", "u", "z"] {
        let mut forged = save.clone();
        add_marker(state_grounds(&mut forged, state), marker("b", 2, 1, 2, 3));
        refused(
            SAME,
            &forged,
            &format!("state {state} grounds hold departed b records outside its lineage"),
        );
        let mut forged = save.clone();
        let grounds = state_grounds(&mut forged, state);
        assert_eq!(grounds.get("caveats"), None, "{state}");
        grounds["caveats"] = json!(["shaky"]);
        grounds["kept"] = json!(["shaky"]);
        refused(
            SAME,
            &forged,
            &format!("state {state} grounds include caveat shaky outside its lineage"),
        );
    }
    let mut forged = save.clone();
    add_marker(
        &mut forged["commitment_grounds"]["r@1"],
        marker("b", 2, 1, 2, 3),
    );
    refused(
        SAME,
        &forged,
        "commitment r@1 grounds hold departed b records outside its lineage",
    );
}

/// The usual `because`: it cites part of what the value reads. The value
/// reads `temp`'s latest reading, so the live check finds it in the
/// lineage, and the grounds pin it, so neither `alarm`'s grounds nor those
/// of `act`, which cites `alarm`, hold a `temp` marker beyond their lineage
/// or basis.
const NARROW: &str = "evidence gt from \"gt\";
evidence gh from \"gh\";
readings temp from gt window 1;
readings hum from gh window 1;
decisions act limit 4;
state alarm = 0;
event st v min 0 max 50;
on st sample temp = v;
event sh v min 0 max 100;
on sh sample hum = v;
event check;
on check set alarm = if(latest(temp) > 30 and latest(hum) > 50, 1, 0) because latest(temp);
event decide;
on decide commit act because enough citing alarm;
";

#[test]
fn a_citation_of_part_of_what_the_value_reads_carries_no_marker_of_it() {
    let save = plays_and_restores(
        NARROW,
        &[
            ("st", r#"{"v":20}"#, "accepted"),
            ("sh", r#"{"v":60}"#, "accepted"),
            ("st", r#"{"v":35}"#, "accepted"),
            ("sh", r#"{"v":70}"#, "accepted"),
            ("check", "{}", "accepted"),
            ("decide", "{}", "accepted"),
            ("st", r#"{"v":36}"#, "accepted"),
            ("sh", r#"{"v":71}"#, "accepted"),
            ("st", r#"{"v":37}"#, "accepted"),
        ],
    );
    // `temp@1` departed at sequence 3.
    let mut forged = save.clone();
    add_marker(
        state_grounds(&mut forged, "alarm"),
        marker("temp", 1, 1, 1, 3),
    );
    refused(
        NARROW,
        &forged,
        "state alarm grounds hold departed temp records outside its lineage",
    );
    let mut forged = save.clone();
    add_marker(
        &mut forged["commitment_grounds"]["act@1"],
        marker("temp", 1, 1, 1, 3),
    );
    refused(
        NARROW,
        &forged,
        "commitment act@1 grounds hold departed temp records outside its lineage",
    );
}

/// `st` cites a reading its value never reads, so every `st` is rejected
/// and `t` is never written.
const NEVER: &str = "evidence ga from \"ga\";
evidence gb from \"gb\";
readings a from ga window 1;
readings b from gb window 1;
state t = 0;
event sa v min 0 max 10;
on sa sample a = v;
event sb v min 0 max 10;
on sb sample b = v;
event st v min 0 max 10;
on st set t = latest(a) because latest(b);
";

#[test]
fn a_citation_never_accepted_carries_no_marker() {
    let save = plays_and_restores(
        NEVER,
        &[
            ("st", r#"{"v":1}"#, "expression"),
            ("sb", r#"{"v":5}"#, "accepted"),
            ("sa", r#"{"v":5}"#, "accepted"),
            ("st", r#"{"v":1}"#, "ungrounded_citation"),
            ("sb", r#"{"v":6}"#, "accepted"),
            ("sb", r#"{"v":7}"#, "accepted"),
            ("st", r#"{"v":1}"#, "ungrounded_citation"),
        ],
    );
    assert_eq!(save["states"], json!({}));
    let sequence = save["sequence"].as_u64().unwrap();
    let mut forged = save.clone();
    forged["states"]["t"] = json!({
        "value": 0.0,
        "lineage": {},
        "grounds": {"departed": [marker("b", 2, 1, 2, sequence)]},
    });
    refused(
        NEVER,
        &forged,
        "state t grounds hold departed b records outside its lineage",
    );
}

/// What a cited stream's selection holds can still bring a marker beyond
/// the lineage: `latest(b)` inherits the guard of a skipped sample of `b`,
/// which read `c@1`, and `c@1` departs. `t`'s lineage is `d`'s basis, made
/// before the skip, so the marker is beyond it, and the save restores
/// (D057).
const SELECTED: &str = "evidence gb from \"gb\";
evidence gc from \"gc\";
readings b from gb window 4;
readings c from gc window 1;
decisions d limit 4;
state t = 0;
event sb v min 0 max 10;
on sb sample b = v;
event sk v min 0 max 10;
on sk when latest(c) > 100 sample b = v;
event sc v min 0 max 10;
on sc sample c = v;
event cd v min 0 max 10;
on cd when latest(b) > 0 commit d because enough using latest(b);
event st v min 0 max 10;
on st set t = latest(d) because latest(b);
";

#[test]
fn what_a_cited_reading_holds_still_restores() {
    let save = plays_and_restores(
        SELECTED,
        &[
            ("sb", r#"{"v":5}"#, "accepted"),
            ("cd", r#"{"v":1}"#, "accepted"),
            ("sc", r#"{"v":3}"#, "accepted"),
            ("sk", r#"{"v":1}"#, "accepted"),
            ("sc", r#"{"v":4}"#, "accepted"),
            ("st", r#"{"v":1}"#, "accepted"),
            ("sb", r#"{"v":6}"#, "accepted"),
            ("st", r#"{"v":1}"#, "ungrounded_citation"),
        ],
    );
    let t = &save["states"]["t"];
    assert_eq!(t["grounds"]["departed"][0]["history"], "c", "{t}");
    assert_eq!(t["lineage"].get("departed"), None, "{t}");
}

/// The same for a kept take-back caveat: the skipped sample's guard read
/// `a@1` while it was `shaky`, so the stored selection keeps `shaky`, and
/// `latest(b)` gives it to `t`'s grounds kept. `t`'s lineage carries `shaky`
/// live through `a@2` until `uq` takes it back; the grounds keep it after,
/// with the departed `a@1`'s marker, and the save restores (D098).
const SELECTED_KEPT: &str = "evidence ga from \"ga\";
evidence gb from \"gb\";
readings a from ga window 1;
readings b from gb window 4;
caveat shaky consequence low;
decisions d limit 4;
state t = 0;
event sa v min 0 max 10;
on sa sample a = v;
event sb v min 0 max 10;
on sb sample b = v;
event sk v min 0 max 10;
on sk when latest(a) > 100 sample b = v;
event qual v min 0 max 10;
on qual qualify latest(a) with shaky;
event uq v min 0 max 10;
on uq unqualify latest(a) with shaky;
event cd v min 0 max 10;
on cd when latest(b) > 0 commit d because enough using latest(b);
event st v min 0 max 10;
on st set t = if(latest(a) > 0, latest(d), 0) because latest(b);
";

#[test]
fn a_kept_caveat_a_cited_reading_holds_still_restores() {
    let save = plays_and_restores(
        SELECTED_KEPT,
        &[
            ("sb", r#"{"v":5}"#, "accepted"),
            ("cd", r#"{"v":1}"#, "accepted"),
            ("sa", r#"{"v":3}"#, "accepted"),
            ("qual", r#"{"v":1}"#, "accepted"),
            ("sk", r#"{"v":1}"#, "accepted"),
            ("sa", r#"{"v":4}"#, "accepted"),
            ("qual", r#"{"v":1}"#, "accepted"),
            ("st", r#"{"v":1}"#, "accepted"),
            ("uq", r#"{"v":1}"#, "accepted"),
            ("sa", r#"{"v":5}"#, "accepted"),
        ],
    );
    let t = &save["states"]["t"];
    assert_eq!(t["grounds"]["kept"], json!(["shaky"]), "{t}");
    assert_eq!(t["grounds"]["departed"][0]["history"], "a", "{t}");
    assert_eq!(t["lineage"].get("caveats"), None, "{t}");
}

/// A reading is taken keeping what qualifies its template, and its
/// provenance, frozen then, keeps a take-back caveat among it: `q` puts
/// `shaky` on `gc`, `c`'s template, and `uq` takes `shaky` back from `ge`,
/// so `shaky` is one. `t` cites a read of `c`, so its grounds keep `shaky`,
/// while its lineage, which reads `c`'s reading through `carries`, carries
/// it live. `u` reads `t`, `p` gives it to `z`, and `r` cites it. `v` reads
/// `c`'s reading the same way but cites `latest(b)`, and so does `stl`,
/// which writes `t`; `w` reads `committed(r)` and `c`'s reading the same
/// way. `firm` can qualify only `ge`.
const BORN: &str = "evidence ge from \"ge\";
evidence gc from \"gc\";
evidence gb from \"gb\";
readings c from gc window 1;
readings b from gb window 1;
caveat shaky consequence low;
caveat firm consequence low;
decisions r limit 4;
state t = 0;
state u = 0;
state z = 0;
state v = 0;
state w = 0;
proc p(y) { set z = y; };
fn add(x, w) = x + w;
event see;
on see reveal gc;
on see reveal ge;
event q;
on q qualify gc with shaky;
event uq;
on uq unqualify ge with shaky;
event qf;
on qf qualify ge with firm;
event uf;
on uf unqualify ge with firm;
event sc n min 0 max 10;
on sc sample c = n;
event sb n min 0 max 10;
on sb sample b = n;
event st;
on st set t = if(carries(latest(c), shaky), 1, 0) because CITATION;
event sv;
on sv set v = if(carries(latest(c), shaky), 1, 0) + latest(b) because latest(b);
event stl;
on stl set t = if(carries(latest(c), shaky), 1, 0) + latest(b) because latest(b);
event cu;
on cu set u = t;
event cz;
on cz call p(t);
event cr;
on cr commit r because enough citing t;
event cw;
on cw set w = if(committed(r), 1, 0) + if(carries(latest(c), shaky), 1, 0);
";

/// The citations of `c`'s readings that `BORN` tries in turn.
const BORN_CITATIONS: [&str; 5] = [
    "latest(c)",
    "history_at(c, 0)",
    "history_count(c)",
    "has_sample(c)",
    "fold_history(c, 0, add)",
];

#[test]
fn a_cited_reading_born_keeping_a_take_back_caveat_restores() {
    for citation in BORN_CITATIONS {
        let source = BORN.replace("CITATION", citation);
        let save = plays_and_restores(
            &source,
            &[
                ("see", "{}", "accepted"),
                ("q", "{}", "accepted"),
                ("sc", r#"{"n":1}"#, "accepted"),
                ("sb", r#"{"n":2}"#, "accepted"),
                ("st", "{}", "accepted"),
                ("sv", "{}", "accepted"),
                ("cu", "{}", "accepted"),
                ("cz", "{}", "accepted"),
                ("cr", "{}", "accepted"),
                ("sc", r#"{"n":2}"#, "accepted"),
                ("sc", r#"{"n":3}"#, "accepted"),
                ("sb", r#"{"n":3}"#, "accepted"),
            ],
        );
        for state in ["t", "u", "z"] {
            let cell = &save["states"][state];
            assert_eq!(
                cell["grounds"]["kept"],
                json!(["shaky"]),
                "{citation}: {cell}"
            );
            assert_eq!(cell["lineage"].get("kept"), None, "{citation}: {cell}");
        }
        assert_eq!(
            save["commitment_grounds"]["r@1"]["kept"],
            json!(["shaky"]),
            "{citation}"
        );
        // `c@2` departed at sequence 11, while `t`'s grounds held `c@1`.
        // `firm` is a take-back caveat no reading is taken keeping.
        forgeries_refused(
            &source,
            &save,
            &["t", "u", "z", "r@1"],
            &marker("c", 1, 2, 2, 11),
            "firm",
        );
        // `v`'s lineage carries `shaky` live; its citation reads no reading
        // taken keeping it.
        refused(
            &source,
            &with_kept(&save, "v", "shaky"),
            "state v grounds keep a caveat its lineage does not keep",
        );
    }
}

/// What a reading is taken keeping lets grounds keep it only where their
/// lineage carries it live and they hold the reading, or a marker of it,
/// and never lets a commitment's grounds keep it beyond their basis: the
/// basis carries the reading, and a commit freezes it, so it keeps what the
/// grounds keep. Taken before `q`, `c@1` carries no `shaky`, and nothing
/// lets grounds keep it where no lineage or basis carries it. `stl` writes
/// `t` citing `latest(b)` while its value reads a reading taken keeping
/// `shaky`, so its lineage carries `shaky` live and its grounds hold `b@1`
/// alone. `r@1` is made before `q`, and `w` reads it and, through
/// `carries`, a reading taken after `q`. Restore before this accepted each
/// of these grounds keeping `shaky`, which no session writes (D099).
#[test]
fn a_kept_caveat_needs_the_reading_taken_keeping_it() {
    let outside = |place: &str| {
        let what = if place.contains('@') {
            format!("commitment {place}")
        } else {
            format!("state {place}")
        };
        format!("{what} grounds include caveat shaky outside its lineage")
    };
    let beyond =
        |state: &str| format!("state {state} grounds keep a caveat its lineage does not keep");
    for citation in BORN_CITATIONS {
        let source = BORN.replace("CITATION", citation);
        let save = plays_and_restores(
            &source,
            &[
                ("see", "{}", "accepted"),
                ("sc", r#"{"n":1}"#, "accepted"),
                ("sb", r#"{"n":2}"#, "accepted"),
                ("st", "{}", "accepted"),
                ("cu", "{}", "accepted"),
                ("cz", "{}", "accepted"),
                ("cr", "{}", "accepted"),
                ("q", "{}", "accepted"),
            ],
        );
        for place in ["t", "u", "z", "r@1"] {
            refused(&source, &with_kept(&save, place, "shaky"), &outside(place));
        }
        let save = plays_and_restores(
            &source,
            &[
                ("see", "{}", "accepted"),
                ("q", "{}", "accepted"),
                ("sc", r#"{"n":1}"#, "accepted"),
                ("sb", r#"{"n":2}"#, "accepted"),
                ("stl", "{}", "accepted"),
                ("cu", "{}", "accepted"),
                ("cz", "{}", "accepted"),
            ],
        );
        for state in ["t", "u", "z"] {
            assert_eq!(
                save["states"][state]["lineage"]["caveats"],
                json!(["shaky"]),
                "{citation}: {state}"
            );
            refused(&source, &with_kept(&save, state, "shaky"), &beyond(state));
        }
        let save = plays_and_restores(
            &source,
            &[
                ("see", "{}", "accepted"),
                ("sc", r#"{"n":1}"#, "accepted"),
                ("st", "{}", "accepted"),
                ("cr", "{}", "accepted"),
                ("q", "{}", "accepted"),
                ("sc", r#"{"n":2}"#, "accepted"),
                ("cw", "{}", "accepted"),
            ],
        );
        assert_eq!(
            save["states"]["w"]["lineage"]["caveats"],
            json!(["shaky"]),
            "{citation}"
        );
        refused(&source, &with_kept(&save, "w", "shaky"), &beyond("w"));
        refused(&source, &with_kept(&save, "r@1", "shaky"), &outside("r@1"));
    }
}

/// A stream no rule samples has no reading to be taken keeping anything:
/// `x` cites `has_sample(c)` while its value reads `gc`, `c`'s template,
/// through `carries`, so its lineage carries `shaky` live and its grounds
/// hold nothing (D099).
const UNSAMPLED: &str = "evidence ge from \"ge\";
evidence gc from \"gc\";
evidence gb from \"gb\";
readings c from gc window 1;
readings b from gb window 1;
caveat shaky consequence low;
state x = 0;
event see;
on see reveal gc;
on see reveal ge;
event q;
on q qualify gc with shaky;
event uq;
on uq unqualify ge with shaky;
event sb n min 0 max 10;
on sb sample b = n;
event sx;
on sx set x = latest(b) + if(carries(gc, shaky), 1, 0) because has_sample(c);
";

#[test]
fn a_stream_no_rule_samples_brings_no_kept_caveat() {
    let save = plays_and_restores(
        UNSAMPLED,
        &[
            ("see", "{}", "accepted"),
            ("q", "{}", "accepted"),
            ("sb", r#"{"n":3}"#, "accepted"),
            ("sx", "{}", "accepted"),
        ],
    );
    assert_eq!(save["states"]["x"]["lineage"]["caveats"], json!(["shaky"]));
    refused(
        UNSAMPLED,
        &with_kept(&save, "x", "shaky"),
        "state x grounds keep a caveat its lineage does not keep",
    );
}

/// The same through a renewable template, qualified before it is renewed,
/// whose later readings still copy that qualification, or after, when the
/// qualification lands on the current occurrence, whose qualifications a
/// reading takes (D024); and through a procedure whose stream parameter
/// the citation reads.
const BORN_RENEWABLE: &str = "evidence ge from \"ge\";
evidence gc from \"gc\";
renewable gc limit 4;
readings c from gc window 2;
caveat shaky consequence low;
state t = 0;
event see;
on see reveal gc;
event q;
on q qualify gc with shaky;
event uq;
on uq unqualify ge with shaky;
event rn;
on rn renew gc;
event sc n min 0 max 10;
on sc sample c = n;
event st;
on st set t = if(carries(latest(c), shaky), 1, 0) because latest(c);
";

const BORN_PROCEDURE: &str = "evidence ge from \"ge\";
evidence gc from \"gc\";
readings c from gc window 2;
caveat shaky consequence low;
state t = 0;
proc pc(s readings) { set t = if(carries(latest(s), shaky), 1, 0) because latest(s); };
event see;
on see reveal gc;
event q;
on q qualify gc with shaky;
event uq;
on uq unqualify ge with shaky;
event sc n min 0 max 10;
on sc sample c = n;
event st;
on st call pc(c);
";

#[test]
fn a_reading_born_through_a_renewable_template_or_a_procedure_restores() {
    let save = plays_and_restores(
        BORN_RENEWABLE,
        &[
            ("see", "{}", "accepted"),
            ("q", "{}", "accepted"),
            ("sc", r#"{"n":1}"#, "accepted"),
            ("st", "{}", "accepted"),
            ("rn", "{}", "accepted"),
            ("sc", r#"{"n":2}"#, "accepted"),
        ],
    );
    assert_eq!(save["states"]["t"]["grounds"]["kept"], json!(["shaky"]));
    let save = plays_and_restores(
        BORN_RENEWABLE,
        &[
            ("see", "{}", "accepted"),
            ("rn", "{}", "accepted"),
            ("see", "{}", "accepted"),
            ("q", "{}", "accepted"),
            ("sc", r#"{"n":1}"#, "accepted"),
            ("st", "{}", "accepted"),
            ("sc", r#"{"n":2}"#, "accepted"),
            ("sc", r#"{"n":3}"#, "accepted"),
        ],
    );
    assert!(
        save.to_string().contains(r#"["shaky","qualifies","gc@2"]"#),
        "{save}"
    );
    assert_eq!(save["states"]["t"]["grounds"]["kept"], json!(["shaky"]));
    let save = plays_and_restores(
        BORN_PROCEDURE,
        &[
            ("see", "{}", "accepted"),
            ("q", "{}", "accepted"),
            ("sc", r#"{"n":1}"#, "accepted"),
            ("st", "{}", "accepted"),
            ("sc", r#"{"n":2}"#, "accepted"),
        ],
    );
    assert_eq!(save["states"]["t"]["grounds"]["kept"], json!(["shaky"]));
}

/// Two streams whose readings are taken keeping a take-back caveat, `zc`
/// keeping `shaky` from `gc` and `ad` keeping `firm` from `gd`, after
/// `fill` other evidence, so that what each is taken keeping sits on either
/// side of the bound's first 64 histories for some `fill`. `t` cites `zc`'s
/// reading, `u` cites `ad`'s, and `w` reads both.
fn two_born_streams(fill: usize) -> String {
    let mut source = String::new();
    for index in 0..fill {
        source.push_str(&format!("evidence e{index:02} from \"e{index:02}\";\n"));
    }
    source.push_str(
        "evidence ge from \"ge\";
evidence gc from \"gc\";
evidence gd from \"gd\";
readings zc from gc window 1;
readings ad from gd window 1;
caveat shaky consequence low;
caveat firm consequence low;
state t = 0;
state u = 0;
state w = 0;
event see;
on see reveal gc;
on see reveal gd;
on see reveal ge;
event q;
on q qualify gc with shaky;
event qf;
on qf qualify gd with firm;
event uq;
on uq unqualify ge with shaky;
event uf;
on uf unqualify ge with firm;
event sc n min 0 max 10;
on sc sample zc = n;
event sd n min 0 max 10;
on sd sample ad = n;
event st;
on st set t = if(carries(latest(zc), shaky), 1, 0) because latest(zc);
event su;
on su set u = if(carries(latest(ad), firm), 1, 0) because latest(ad);
event sw;
on sw set w = t + u;
",
    );
    source
}

/// A citation of one stream's reading brings only what that stream's
/// readings are taken keeping, wherever the bound holds it (D099).
#[test]
fn two_streams_keep_apart_what_their_readings_are_taken_keeping() {
    for fill in 54..=62 {
        let source = two_born_streams(fill);
        let save = plays_and_restores(
            &source,
            &[
                ("see", "{}", "accepted"),
                ("q", "{}", "accepted"),
                ("qf", "{}", "accepted"),
                ("sc", r#"{"n":1}"#, "accepted"),
                ("sd", r#"{"n":1}"#, "accepted"),
                ("st", "{}", "accepted"),
                ("su", "{}", "accepted"),
                ("sw", "{}", "accepted"),
                ("sc", r#"{"n":2}"#, "accepted"),
                ("sd", r#"{"n":2}"#, "accepted"),
            ],
        );
        assert_eq!(
            save["states"]["t"]["grounds"]["kept"],
            json!(["shaky"]),
            "{fill}: {}",
            save["states"]["t"]
        );
        assert_eq!(
            save["states"]["u"]["grounds"]["kept"],
            json!(["firm"]),
            "{fill}: {}",
            save["states"]["u"]
        );
        for (state, caveat) in [("t", "firm"), ("u", "shaky")] {
            refused(
                &source,
                &with_kept(&save, state, caveat),
                &format!("state {state} grounds include caveat {caveat} outside its lineage"),
            );
        }
    }
}

/// `b`'s sample is guarded by `latest(h)`, so `b`'s observation records
/// list `h`'s readings, and `qh` qualifies `h`'s reading under a guard that
/// reads `c`. What `c`'s records hold reaches `b`'s observation records,
/// never `b`'s readings or selection, which are all `latest(b)` gives
/// grounds; `shaky` can qualify only `c`'s readings.
const OBSERVATION: &str = "evidence gb from \"gb\";
evidence gh from \"gh\";
evidence gc from \"gc\";
readings b from gb window 1;
readings h from gh window 1;
readings c from gc window 1;
caveat flag consequence low;
caveat shaky consequence low;
decisions r limit 4;
state t = 0;
state u = 0;
state z = 0;
proc p(y) { set z = y; };
event sh n min 0 max 10;
on sh sample h = n;
event sc n min 0 max 10;
on sc sample c = n;
event sb n min 0 max 10;
on sb when latest(h) >= 0 sample b = n;
event qh;
on qh when latest(c) >= 0 qualify latest(h) with flag;
event qc;
on qc qualify latest(c) with shaky;
event uc;
on uc unqualify latest(c) with shaky;
event st;
on st set t = latest(b) because latest(b);
event cu;
on cu set u = t;
event cz;
on cz call p(t);
event cr;
on cr commit r because enough citing t;
";

#[test]
fn a_cited_reading_brings_its_readings_not_its_observation_records() {
    let save = plays_and_restores(
        OBSERVATION,
        &[
            ("sh", r#"{"n":1}"#, "accepted"),
            ("sc", r#"{"n":1}"#, "accepted"),
            ("sc", r#"{"n":2}"#, "accepted"),
            ("sc", r#"{"n":3}"#, "accepted"),
            ("sb", r#"{"n":5}"#, "accepted"),
            ("st", "{}", "accepted"),
            ("cu", "{}", "accepted"),
            ("cz", "{}", "accepted"),
            ("cr", "{}", "accepted"),
            ("sb", r#"{"n":6}"#, "accepted"),
            ("sc", r#"{"n":4}"#, "accepted"),
        ],
    );
    // `c@1` departed at sequence 3.
    forgeries_refused(
        OBSERVATION,
        &save,
        &["t", "u", "z", "r@1"],
        &marker("c", 1, 1, 1, 3),
        "shaky",
    );
}

/// A withdrawal predicate, `reopened` and `on_hold` give grounds the names
/// of the withdrawals' reasons, or of the reopening's or hold's causes,
/// with what qualifies them now; the withdrawal, reopening or hold pins
/// them, and so do the grounds. `w` renews into a window of one, and
/// `shaky` can qualify only `w`'s occurrences.
const NAMES: &str = "evidence ga from \"ga\";
evidence w from \"w\";
renewable w window 1;
readings a from ga limit 8;
caveat shaky consequence low;
decisions d limit 4;
decisions r limit 4;
state t = 0;
state u = 0;
state z = 0;
proc p(y) { set z = y; };
event sa n min 0 max 10;
on sa sample a = n;
event rw;
on rw renew w;
on rw reveal w;
event qw;
on qw qualify w with shaky;
event uw;
on uw unqualify w with shaky;
event cd;
on cd commit d because enough using latest(a);
event wd;
on wd withdraw latest(a) because w;
event ro;
on ro reopen d because w;
event ho;
on ho hold d because w;
event st;
on st set t = if(VALUE, 1, 0) because VALUE;
event cu;
on cu set u = t;
event cz;
on cz call p(t);
event cr;
on cr commit r because enough citing t;
";

#[test]
fn reasons_and_causes_a_citation_names_bring_nothing() {
    for (value, names) in [
        ("withdrawn(latest(a))", "wd"),
        ("rests_on_withdrawn(d)", "wd"),
        ("reopened(d)", "ro"),
        ("on_hold(d)", "ho"),
    ] {
        let source = NAMES.replace("VALUE", value);
        let save = plays_and_restores(
            &source,
            &[
                ("sa", r#"{"n":3}"#, "accepted"),
                ("cd", "{}", "accepted"),
                ("rw", "{}", "accepted"),
                ("rw", "{}", "accepted"),
                ("rw", "{}", "accepted"),
                (names, "{}", "accepted"),
                ("st", "{}", "accepted"),
                ("cu", "{}", "accepted"),
                ("cz", "{}", "accepted"),
                ("cr", "{}", "accepted"),
                ("rw", "{}", "accepted"),
                ("rw", "{}", "accepted"),
            ],
        );
        assert_eq!(save["states"]["t"]["value"], json!(1.0), "{value}");
        // `w@2` and `w@3` departed at sequences 4 and 5; the reason `w@4`
        // stays.
        forgeries_refused(
            &source,
            &save,
            &["t", "u", "z", "r@1"],
            &marker("w", 2, 2, 3, 5),
            "shaky",
        );
    }
}
