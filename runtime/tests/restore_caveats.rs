//! Restore refuses a saved provenance carrying a caveat no mechanism of the
//! source can attach to its evidence (findings F330, F333, F335-F337). See
//! spec/caveat-save-0.1.md and docs/RESTORE_TRUST_BOUNDARY.md.
use caveat_runtime::reactive::ReactiveSession;
use serde_json::{json, Value};

/// Every record kind the check covers: an observation, an examination and a
/// reopening record, a skipped effect's predicate guard, a state with
/// authored grounds, and a commitment's basis and grounds. `drift` qualifies
/// gauge by declaration; `late` could qualify sensor through a rule whose
/// guard never holds; `stale` is read by `examined(...)`; nothing attaches
/// `forged`.
const PROGRAM: &str = r#"
budget 5;
claim safe;
evidence sensor from "sensor";
evidence gauge from "gauge";
evidence other from "other";
caveat stale consequence low;
caveat drift consequence low;
caveat late consequence low;
caveat forged consequence low;
drift qualifies gauge;
decisions route limit 4;
state level = 0 min 0 max 100;
event go;
event look;
event redo;
event skip;
event never;
on go when not observed(sensor) reveal sensor supports safe;
on go when not observed(gauge) reveal gauge supports safe;
on go when not committed(route) commit route because enough using qualified(1, sensor);
on go set level = qualified(2, gauge) + qualified(0, sensor) because qualified(2, gauge);
on look when observed(gauge) and not examined(stale) examine stale cost 1;
on redo when committed(route) and observed(gauge) reopen route because gauge;
on skip when not observed(gauge) reveal other supports safe;
on never when level > 1000 qualify sensor with late;
"#;

const EVENTS: [&str; 4] = ["go", "look", "redo", "skip"];

fn send(game: &mut ReactiveSession, event: &str) {
    let outcome = serde_json::to_value(
        game.dispatch_outcome_json(event, "{}")
            .unwrap_or_else(|fatal| panic!("{event}: fatal {}", fatal.message)),
    )
    .unwrap();
    assert_eq!(outcome["outcome"], "accepted", "{event}: {outcome}");
}

fn played() -> ReactiveSession {
    let mut game = ReactiveSession::from_source(PROGRAM).unwrap();
    for event in EVENTS {
        send(&mut game, event);
    }
    game
}

fn saved() -> Value {
    serde_json::to_value(played().save().unwrap()).unwrap()
}

fn refused(save: &Value, expected: &str) {
    match ReactiveSession::restore_json(PROGRAM, &save.to_string()) {
        Err(error) => assert_eq!(error, format!("cannot restore save: {expected}")),
        Ok(_) => panic!("accepted: {expected}"),
    }
}

fn restores(save: &Value) {
    if let Err(error) = ReactiveSession::restore_json(PROGRAM, &save.to_string()) {
        panic!("{error}");
    }
}

/// `caveat` added to the provenance at `pointer`, kept in name order.
fn with_caveat(save: &Value, pointer: &str, caveat: &str) -> Value {
    let mut save = save.clone();
    let provenance = save
        .pointer_mut(pointer)
        .unwrap_or_else(|| panic!("{pointer}"));
    let caveats = provenance
        .as_object_mut()
        .unwrap()
        .entry("caveats")
        .or_insert_with(|| json!([]))
        .as_array_mut()
        .unwrap();
    caveats.push(caveat.into());
    caveats.sort_by(|left, right| left.as_str().cmp(&right.as_str()));
    save
}

// The genuine save holds every record the check covers, and restores.
#[test]
fn a_genuine_save_with_caveats_in_every_record_restores() {
    let save = saved();
    for pointer in [
        "/observation_qualifications/sensor",
        "/examination_qualifications/stale",
        "/reopening_qualifications/route@1",
        "/predicate_qualifications/observed/other",
        "/states/level/lineage",
        "/states/level/grounds",
        "/commitment_bases/route@1/provenance",
        "/commitment_grounds/route@1",
    ] {
        assert!(save.pointer(pointer).is_some(), "{pointer}");
    }
    assert_eq!(
        save["states"]["level"]["lineage"],
        json!({"caveats": ["drift"], "evidence": ["gauge", "sensor"]})
    );
    assert_eq!(
        save["states"]["level"]["grounds"],
        json!({"caveats": ["drift"], "evidence": ["gauge"]})
    );
    restores(&save);
}

// F330, F333, F335-F337: a caveat no mechanism of the source attaches to
// anything, written into each kind of record, restored live and reached the
// commit `using` read path and the `examined()`, `reopened()` and
// `committed()` guards. Each is refused.
#[test]
fn f330_a_caveat_nothing_attaches_is_refused_in_every_record() {
    let save = saved();
    for (pointer, what) in [
        (
            "/observation_qualifications/sensor",
            "observation of sensor",
        ),
        ("/examination_qualifications/stale", "examination of stale"),
        ("/reopening_qualifications/route@1", "reopening of route@1"),
        (
            "/predicate_qualifications/observed/other",
            "observed(other)",
        ),
        ("/states/level/lineage", "state level"),
        ("/commitment_bases/route@1/provenance", "commitment route@1"),
    ] {
        refused(
            &with_caveat(&save, pointer, "forged"),
            &format!("{what}: forged cannot qualify any of its evidence"),
        );
    }
}

// Grounds lie within a lineage or basis, whose caveats are checked: a
// caveat nothing attaches is refused there, or as outside it.
#[test]
fn f330_a_caveat_nothing_attaches_is_refused_in_grounds() {
    let save = saved();
    let state = with_caveat(&save, "/states/level/grounds", "forged");
    refused(
        &with_caveat(&state, "/states/level/lineage", "forged"),
        "state level: forged cannot qualify any of its evidence",
    );
    refused(
        &state,
        "state level grounds include caveat forged outside its lineage",
    );
    let commitment = with_caveat(&save, "/commitment_grounds/route@1", "forged");
    refused(
        &with_caveat(
            &commitment,
            "/commitment_bases/route@1/provenance",
            "forged",
        ),
        "commitment route@1: forged cannot qualify any of its evidence",
    );
}

// A caveat the source attaches, but only to other evidence: drift qualifies
// gauge, and route@1's basis rests on sensor alone.
#[test]
fn f333_a_caveat_of_other_evidence_is_refused() {
    refused(
        &with_caveat(&saved(), "/commitment_bases/route@1/provenance", "drift"),
        "commitment route@1: drift cannot qualify any of its evidence",
    );
}

// The boundary. Guards are not evaluated, so a caveat that some mechanism
// could have attached restores without proof that it did: `late` through a
// rule whose guard never holds, and `stale`, which `examined(stale)` carries
// with no evidence. Grounds may carry a caveat of any evidence in their
// lineage: level's grounds rest on gauge, its lineage on sensor too.
// Restore does not authenticate history (spec/caveat-save-0.1.md).
#[test]
fn a_caveat_the_source_could_have_attached_restores_without_proof_it_did() {
    let save = saved();
    restores(&with_caveat(
        &save,
        "/commitment_bases/route@1/provenance",
        "late",
    ));
    restores(&with_caveat(
        &save,
        "/predicate_qualifications/observed/other",
        "stale",
    ));
    let lineage = with_caveat(&save, "/states/level/lineage", "late");
    restores(&with_caveat(&lineage, "/states/level/grounds", "late"));
}
