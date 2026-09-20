use caveat_runtime::reactive::{BindingValue, ReactiveSession, ReactiveSnapshot};
use serde_json::{json, Value};

const SOURCE: &str = include_str!("../../examples/slime_glow_ability.cav");

fn dispatch(session: &mut ReactiveSession, event: &str) -> ReactiveSnapshot {
    session.dispatch_json(event, "{}").unwrap()
}

fn ability(snapshot: &ReactiveSnapshot) -> Value {
    serde_json::to_value(&snapshot.bindings["ability"]).unwrap()
}

fn learned() -> (ReactiveSession, ReactiveSnapshot) {
    let mut session = ReactiveSession::from_source(SOURCE).unwrap();
    dispatch(&mut session, "glow_renderer_ready");
    dispatch(&mut session, "cave_entered");
    let snapshot = dispatch(&mut session, "absorb_mushroom");
    (session, snapshot)
}

#[test]
fn locked_toggles_and_cave_entry_do_not_award_or_consume_the_mushroom() {
    let mut session = ReactiveSession::from_source(SOURCE).unwrap();
    let initial = session.snapshot();
    for event in ["toggle_glow", "toggle_glow", "toggle_glow"] {
        let snapshot = dispatch(&mut session, event);
        assert_eq!(ability(&snapshot), ability(&initial));
        assert!(snapshot.commitment_bases.is_empty());
        assert_eq!(
            snapshot.bindings["item"]["consumed"],
            BindingValue::Bool(false)
        );
        assert_eq!(
            snapshot.bindings["journal"]["visible"],
            BindingValue::Bool(false)
        );
    }
    let entered = dispatch(&mut session, "cave_entered");
    assert_eq!(ability(&entered), ability(&initial));
    assert_eq!(
        entered.bindings["objective"]["text"],
        BindingValue::Text("Absorb the cave mushroom.".into())
    );
}

#[test]
fn clearing_context_changes_only_unfinished_guidance_and_resets_with_the_round() {
    let mut session = ReactiveSession::from_source(SOURCE).unwrap();
    let initial = session.snapshot();
    let clearing = dispatch(&mut session, "clearing_started");
    assert_eq!(ability(&clearing), ability(&initial));
    assert!(clearing.commitment_bases.is_empty());
    assert_eq!(
        clearing.bindings["objective"]["text"],
        BindingValue::Text(
            "Squeeze through the gap. Approach the mushroom and press E to absorb it.".into()
        )
    );
    let duplicate = dispatch(&mut session, "clearing_started");
    assert_eq!(duplicate.relations, clearing.relations);
    let cave = dispatch(&mut session, "cave_entered");
    assert_eq!(cave.bindings["objective"], clearing.bindings["objective"]);
    let acquired = dispatch(&mut session, "absorb_mushroom");
    let (_, old_opening_acquired) = learned();
    assert_eq!(
        acquired.commitment_bases,
        old_opening_acquired.commitment_bases
    );
    assert_eq!(
        acquired.bindings["objective"]["text"],
        BindingValue::Text("Mushroom absorbed. Clearing complete.".into())
    );
    let after_learning = dispatch(&mut session, "clearing_started");
    assert_eq!(after_learning.bindings, acquired.bindings);
    let reset = ReactiveSession::from_source(SOURCE).unwrap().snapshot();
    assert_eq!(reset, initial);
    assert_eq!(
        reset.bindings["objective"]["text"],
        BindingValue::Text("Find shelter in the cave.".into())
    );
}

#[test]
fn unavailable_renderer_records_discovery_without_promising_light_or_toggling() {
    let mut session = ReactiveSession::from_source(SOURCE).unwrap();
    let initial = session.snapshot();
    let discovered = dispatch(&mut session, "absorb_mushroom");
    assert_eq!(
        discovered.bindings["ability"]["active"],
        BindingValue::Bool(false)
    );
    assert_eq!(
        discovered.bindings["ability"]["toggleAvailable"],
        BindingValue::Bool(false)
    );
    assert_eq!(
        discovered.bindings["ability"]["name"],
        BindingValue::Text("Mushroom discovery".into())
    );
    assert_eq!(
        discovered.bindings["objective"]["text"],
        BindingValue::Text("Mushroom absorbed.".into())
    );
    assert_eq!(
        discovered.bindings["journal"]["text"],
        BindingValue::Text(
            "You absorbed one mushroom. Its effects are not yet established.".into()
        )
    );
    let (_, ready_acquisition) = learned();
    assert_eq!(
        discovered.commitment_bases,
        ready_acquisition.commitment_bases
    );
    let initial_bytes = serde_json::to_vec(&discovered).unwrap().len();
    for _ in 0..1_001 {
        let ignored = dispatch(&mut session, "toggle_glow");
        assert_eq!(ignored.bindings, discovered.bindings);
        assert_eq!(ignored.relations, discovered.relations);
        assert_eq!(ignored.commitment_bases, discovered.commitment_bases);
        assert!(serde_json::to_vec(&ignored).unwrap().len() < initial_bytes + 128);
    }
    let enabled = dispatch(&mut session, "glow_renderer_ready");
    assert_eq!(
        enabled.bindings["ability"]["toggleAvailable"],
        BindingValue::Bool(true)
    );
    assert_eq!(
        enabled.bindings["ability"]["active"],
        BindingValue::Bool(false)
    );
    assert_eq!(
        enabled.bindings["ability"]["text"],
        BindingValue::Text("Glow off".into())
    );
    assert_eq!(enabled.commitment_bases, discovered.commitment_bases);
    let duplicate = dispatch(&mut session, "glow_renderer_ready");
    assert_eq!(duplicate.bindings, enabled.bindings);
    assert_eq!(duplicate.relations, enabled.relations);
    assert_eq!(
        dispatch(&mut session, "toggle_glow").bindings["ability"]["active"],
        BindingValue::Bool(true)
    );
    let reset = ReactiveSession::from_source(SOURCE).unwrap().snapshot();
    assert_eq!(reset, initial);
}

#[test]
fn verified_absorption_is_sufficient_before_the_story_threshold() {
    let mut session = ReactiveSession::from_source(SOURCE).unwrap();
    dispatch(&mut session, "glow_renderer_ready");
    let acquired = dispatch(&mut session, "absorb_mushroom");
    assert_eq!(
        acquired.bindings["ability"]["learned"],
        BindingValue::Bool(true)
    );
    assert_eq!(
        acquired.bindings["ability"]["active"],
        BindingValue::Bool(true)
    );
    assert_eq!(
        acquired.bindings["item"]["available"],
        BindingValue::Bool(false)
    );
    assert_eq!(
        acquired.bindings["objective"]["complete"],
        BindingValue::Bool(true)
    );
    let entered = dispatch(&mut session, "cave_entered");
    assert_eq!(ability(&entered), ability(&acquired));
    assert_eq!(entered.commitment_bases, acquired.commitment_bases);
}

#[test]
fn absorption_earns_one_qualified_receipt_and_duplicates_do_not_reenable() {
    let (mut session, acquired) = learned();
    assert_eq!(
        acquired.bindings["ability"]["learned"],
        BindingValue::Bool(true)
    );
    assert_eq!(
        acquired.bindings["ability"]["active"],
        BindingValue::Bool(true)
    );
    assert_eq!(
        acquired.bindings["item"]["consumed"],
        BindingValue::Bool(true)
    );
    assert_eq!(
        acquired.bindings["objective"]["complete"],
        BindingValue::Bool(true)
    );
    let basis = &acquired.commitment_bases["keep_glow"];
    let basis_json = serde_json::to_value(basis).unwrap();
    assert_eq!(basis_json["value"].as_f64(), Some(1.0));
    assert_eq!(
        basis_json["provenance"],
        json!({
            "evidence": ["first_mushroom"],
            "caveats": ["single_absorption"]
        })
    );
    let disabled = dispatch(&mut session, "toggle_glow");
    assert_eq!(
        disabled.bindings["ability"]["active"],
        BindingValue::Bool(false)
    );
    for event in ["absorb_mushroom", "cave_entered", "absorb_mushroom"] {
        let duplicate = dispatch(&mut session, event);
        assert_eq!(ability(&duplicate), ability(&disabled));
        assert_eq!(duplicate.commitment_bases, acquired.commitment_bases);
        assert_eq!(duplicate.relations, acquired.relations);
    }
}

#[test]
fn ten_thousand_toggles_keep_graph_and_receipt_bounded() {
    let (mut session, acquired) = learned();
    let initial_bytes = serde_json::to_vec(&acquired).unwrap().len();
    for index in 0..10_001 {
        let snapshot = dispatch(&mut session, "toggle_glow");
        assert_eq!(
            snapshot.bindings["ability"]["active"],
            BindingValue::Bool(index % 2 == 1)
        );
        assert_eq!(snapshot.symbols, acquired.symbols);
        assert_eq!(snapshot.relations, acquired.relations);
        assert_eq!(snapshot.commitment_bases, acquired.commitment_bases);
        assert!(snapshot.reading_streams.is_empty());
        assert!(snapshot.decision_series.is_empty());
        assert_eq!(
            snapshot.qualified_values["active"].provenance,
            acquired.qualified_values["active"].provenance
        );
        // Sequence digits and label lengths change; no per-toggle archive grows.
        assert!(serde_json::to_vec(&snapshot).unwrap().len() < initial_bytes + 128);
    }
}

#[test]
fn source_only_policy_variation_changes_automatic_activation_not_the_receipt() {
    let changed = SOURCE.replace(
        "set active = learned * renderer_ready;",
        "set active = learned * renderer_ready * 0;",
    );
    assert_ne!(changed, SOURCE);
    let mut session = ReactiveSession::from_source(&changed).unwrap();
    dispatch(&mut session, "glow_renderer_ready");
    dispatch(&mut session, "cave_entered");
    let revised = dispatch(&mut session, "absorb_mushroom");
    let (_, original) = learned();
    assert_eq!(
        revised.bindings["ability"]["active"],
        BindingValue::Bool(false)
    );
    assert_eq!(
        original.bindings["ability"]["active"],
        BindingValue::Bool(true)
    );
    assert_eq!(revised.commitment_bases, original.commitment_bases);
    assert_eq!(
        revised.qualified_values["active"].provenance,
        original.qualified_values["active"].provenance
    );
    assert_eq!(
        dispatch(&mut session, "toggle_glow").bindings["ability"]["active"],
        BindingValue::Bool(true)
    );
}

#[test]
fn malformed_input_and_late_failure_publish_no_learning_or_consumption() {
    let mut session = ReactiveSession::from_source(SOURCE).unwrap();
    dispatch(&mut session, "cave_entered");
    for (event, payload) in [
        ("absorb_mushroom", "{\"id\":1}"),
        ("toggle_glow", "{\"active\":true}"),
        ("glow_renderer_ready", "{\"ready\":true}"),
        ("reset", "{}"),
        ("absorb_mushroom", "null"),
    ] {
        let before = session.snapshot();
        assert!(session.dispatch_json(event, payload).is_err());
        assert_eq!(session.snapshot(), before);
    }
    let failing_source = SOURCE.replace(
        "set active = learned * renderer_ready;",
        "set active = 1 / 0;",
    );
    let mut failing = ReactiveSession::from_source(&failing_source).unwrap();
    dispatch(&mut failing, "cave_entered");
    let before = failing.snapshot();
    assert!(failing.dispatch_json("absorb_mushroom", "{}").is_err());
    assert_eq!(failing.snapshot(), before);
}

#[test]
fn a_new_round_has_new_state_without_rewriting_the_previous_receipt() {
    let (mut old, acquired) = learned();
    dispatch(&mut old, "toggle_glow");
    let new_round = ReactiveSession::from_source(SOURCE).unwrap().snapshot();
    assert_eq!(new_round.sequence, 0);
    assert_eq!(
        new_round.bindings["ability"]["learned"],
        BindingValue::Bool(false)
    );
    assert_eq!(
        new_round.bindings["ability"]["active"],
        BindingValue::Bool(false)
    );
    assert!(new_round.commitment_bases.is_empty());
    assert_eq!(
        new_round.bindings["ability"]["toggleAvailable"],
        BindingValue::Bool(false)
    );
    assert_eq!(old.snapshot().commitment_bases, acquired.commitment_bases);
}
