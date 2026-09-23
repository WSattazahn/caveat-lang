//! Live state grounds can be inspected and select exact reopening witnesses.
//! See spec/caveat-state-caveats-0.1.md.
use caveat_runtime::reactive::{BindingValue, ReactiveSession};
use std::collections::{BTreeMap, BTreeSet};

fn names(values: &[&str]) -> BTreeSet<String> {
    values.iter().map(|value| (*value).to_string()).collect()
}

fn send(session: &mut ReactiveSession, event: &str) {
    session.apply(event, &BTreeMap::new()).unwrap();
}

const RENEWING: &str = r#"
claim safe;
caveat stale consequence material;
evidence sight from "a sighting";
renewable sight limit 3;
state latest_sight = 0;
state basis = 0;
decisions route limit 3;
event scout;
event again;
event choose;
event advance dt min 0 max 30;
clock advance every 0.1;
on scout reveal sight supports safe;
on scout set latest_sight = qualified(1, sight);
on scout qualify sight with stale after 30;
on again renew sight;
on again reveal sight supports safe;
on again set latest_sight = qualified(1, sight);
on again qualify sight with stale after 30;
on choose set basis = latest_sight;
on choose commit route because enough using basis;
on advance when committed(route) and not reopened(route) and has_caveat(basis, stale)
    reopen route because caveated(basis, stale);
bind hud.old = has_caveat(basis, stale) because basis;
bind hud.current = carries(sight, stale);
"#;

#[test]
fn an_older_renewable_occurrence_reopens_the_plan_without_changing_its_frozen_basis() {
    let mut session = ReactiveSession::from_source(RENEWING).unwrap();
    send(&mut session, "scout");
    send(&mut session, "choose");
    session.dispatch_json("advance", r#"{"dt":29}"#).unwrap();
    send(&mut session, "again");
    let frozen = session.snapshot().commitment_grounds["route@1"].clone();
    let snapshot = session.dispatch_json("advance", r#"{"dt":1}"#).unwrap();
    assert_eq!(snapshot.bindings["hud"]["old"], BindingValue::Bool(true));
    assert_eq!(
        snapshot.bindings["hud"]["current"],
        BindingValue::Bool(false)
    );
    assert_eq!(snapshot.value_grounds["basis"].evidence, names(&["sight"]));
    assert_eq!(snapshot.value_grounds["basis"].caveats, names(&["stale"]));
    assert_eq!(snapshot.commitment_grounds["route@1"], frozen);
    assert!(snapshot.decision_journal[0].caveats.is_empty());
    assert_eq!(snapshot.decision_journal[1].because, ["sight"]);
    assert_eq!(snapshot.decision_journal[1].caveats, ["stale"]);
    assert_eq!(snapshot.commitments[0].reopened_by, ["sight"]);
}

#[test]
fn guard_lineage_cannot_make_clean_grounds_caveated_or_become_a_reopening_witness() {
    let source = r#"
claim safe;
caveat stale consequence material;
evidence warning from "a warning used as control";
evidence clear from "the content";
stale qualifies warning;
warning supports safe;
clear supports safe;
state basis = 0;
state probe = 0;
event choose;
event reconsider;
on choose when qualified(1, warning) > 0 set basis = qualified(1, clear);
on choose commit route because enough using basis;
on choose set probe = if(has_caveat(basis, stale), 1, 0);
on reconsider reopen route because caveated(basis, stale);
bind hud.old = has_caveat(basis, stale) because basis;
"#;
    let mut session = ReactiveSession::from_source(source).unwrap();
    send(&mut session, "choose");
    let snapshot = session.snapshot();
    assert_eq!(snapshot.bindings["hud"]["old"], BindingValue::Bool(false));
    assert_eq!(snapshot.values["probe"], 0.0);
    assert_eq!(
        snapshot.qualified_values["probe"].provenance.evidence,
        names(&["warning", "clear"])
    );
    assert_eq!(
        snapshot.qualified_values["probe"].provenance.caveats,
        names(&["stale"])
    );
    assert_eq!(snapshot.value_grounds["probe"].evidence, names(&["clear"]));
    assert!(snapshot.value_grounds["probe"].caveats.is_empty());
    assert_eq!(
        snapshot.binding_explanations["hud"]["old"].evidence,
        names(&["clear"])
    );
    assert!(session
        .apply("reconsider", &BTreeMap::new())
        .unwrap_err()
        .contains("selects no observed evidence"));
    assert_eq!(session.snapshot(), snapshot);
}

#[test]
fn replacing_only_state_grounds_invalidates_the_binding_even_if_the_number_is_unchanged() {
    let mut session = ReactiveSession::from_source(
        r#"
claim safe;
caveat stale consequence material;
evidence old from "old";
evidence new from "new";
stale qualifies old;
old supports safe;
new supports safe;
state basis = qualified(1, old);
state initially_stale = if(has_caveat(basis, stale), 1, 0);
event replace;
on replace set basis = qualified(1, new);
bind hud.old = has_caveat(basis, stale) because basis;
"#,
    )
    .unwrap();
    let before = session.snapshot();
    assert_eq!(before.bindings["hud"]["old"], BindingValue::Bool(true));
    assert_eq!(before.values["initially_stale"], 1.0);
    send(&mut session, "replace");
    let after = session.snapshot();
    assert_eq!(after.relations, before.relations);
    assert_eq!(after.values["basis"], before.values["basis"]);
    assert_eq!(after.bindings["hud"]["old"], BindingValue::Bool(false));
    assert_eq!(
        after.binding_explanations["hud"]["old"].evidence,
        names(&["new"])
    );
}

const MULTIPLE: &str = r#"
claim safe;
caveat stale consequence material;
caveat secondhand consequence low;
evidence alpha from "observed second";
evidence zulu from "observed first";
evidence fresh from "observed third";
secondhand qualifies zulu;
state basis = 0;
state changes = 0;
event see;
event age;
event reconsider;
event age_last;
event broken;
on see reveal zulu supports safe;
on see reveal alpha supports safe;
on see reveal fresh supports safe;
on see set basis = qualified(1, alpha) + qualified(1, fresh) + qualified(1, zulu);
on see commit route because enough using basis;
on age qualify alpha with stale;
on age qualify zulu with stale;
on age reopen route because caveated(basis, stale);
on reconsider set changes = changes + 1;
on reconsider reopen route because caveated(basis, stale);
on age_last qualify fresh with stale;
on age_last reopen route because caveated(basis, stale);
on broken qualify alpha with stale;
on broken qualify zulu with stale;
on broken reopen route because caveated(basis, stale);
on broken set changes = 1 / 0;
"#;

#[test]
fn multiple_causes_share_one_ordered_entry_and_existing_witnesses_are_not_repeated() {
    let mut session = ReactiveSession::from_source(MULTIPLE).unwrap();
    send(&mut session, "see");
    send(&mut session, "age");
    let after = session.snapshot();
    assert_eq!(after.decision_journal.len(), 2);
    assert_eq!(after.decision_journal[1].because, ["zulu", "alpha"]);
    assert_eq!(after.decision_journal[1].caveats, ["secondhand", "stale"]);
    assert_eq!(after.commitments[0].reopened_by, ["zulu", "alpha"]);
    assert_eq!(
        after
            .effects
            .iter()
            .filter(|effect| matches!(
                effect,
                caveat_runtime::reactive::EffectReport::Reopen { .. }
            ))
            .count(),
        2
    );
    send(&mut session, "reconsider");
    assert_eq!(session.snapshot().decision_journal, after.decision_journal);
    assert!(session.snapshot().effects.is_empty());
    send(&mut session, "age_last");
    let after = session.snapshot();
    assert_eq!(after.decision_journal.len(), 3);
    assert_eq!(after.decision_journal[2].because, ["fresh"]);
    assert_eq!(after.commitments[0].reopened_by, ["zulu", "alpha", "fresh"]);
}

#[test]
fn an_empty_selection_and_a_later_failure_roll_back_the_entire_event() {
    let mut session = ReactiveSession::from_source(MULTIPLE).unwrap();
    send(&mut session, "see");
    let before = session.snapshot();
    let save = session.save_json().unwrap();
    assert!(session
        .apply("reconsider", &BTreeMap::new())
        .unwrap_err()
        .contains("selects no observed evidence"));
    assert_eq!(session.snapshot(), before);
    assert_eq!(session.save_json().unwrap(), save);
    assert!(session.apply("broken", &BTreeMap::new()).is_err());
    assert_eq!(session.snapshot(), before);
    assert_eq!(session.save_json().unwrap(), save);
}

#[test]
fn an_authored_extra_caveat_is_queryable_but_does_not_invent_a_witness() {
    let mut session = ReactiveSession::from_source(
        r#"
claim safe;
caveat stale consequence material;
evidence chart from "chart";
chart supports safe;
state basis = qualified(1, chart, stale);
event choose;
event reconsider;
on choose commit route because enough using basis;
on reconsider reopen route because caveated(basis, stale);
bind hud.old = has_caveat(basis, stale);
"#,
    )
    .unwrap();
    send(&mut session, "choose");
    let before = session.snapshot();
    assert_eq!(before.bindings["hud"]["old"], BindingValue::Bool(true));
    assert!(session
        .apply("reconsider", &BTreeMap::new())
        .unwrap_err()
        .contains("selects no observed evidence"));
    assert_eq!(session.snapshot(), before);
}

#[test]
fn invalid_state_caveat_and_selector_names_fail_when_loaded() {
    const BASE: &str = r#"
claim safe;
caveat stale consequence material;
evidence chart from "chart";
state basis = 0;
event run;
on run commit route because enough;
"#;
    for expression in [
        "has_caveat(missing, stale)",
        "has_caveat(chart, stale)",
        "has_caveat(basis, safe)",
        "has_caveat(basis, missing)",
        "has_caveat(basis + 1, stale)",
        "has_caveat(basis)",
        "has_caveat(basis, stale, stale)",
    ] {
        assert!(
            ReactiveSession::from_source(&format!("{BASE} bind hud.old = {expression};")).is_err(),
            "{expression}"
        );
    }
    for selector in [
        "caveated(missing, stale)",
        "caveated(chart, stale)",
        "caveated(basis, safe)",
        "caveated(basis, missing)",
        "caveated(basis + 1, stale)",
        "caveated(basis)",
        "caveated(basis, stale, stale)",
    ] {
        assert!(
            ReactiveSession::from_source(&format!(
                "{BASE} on run reopen route because {selector};"
            ))
            .is_err(),
            "{selector}"
        );
    }
    assert!(ReactiveSession::from_source(&format!("{BASE} fn has_caveat(x) = x;")).is_err());
}

#[test]
fn state_caveats_work_through_modules_defines_and_nested_typed_procedures() {
    use caveat_runtime::link::{bundle, BundlePart};
    let source = bundle(&[
        BundlePart {
            name: "deciding".into(),
            source: r#"
module deciding;
state basis = 0;
decisions trust limit 3;
proc choose(e evidence, c claim) {
    reveal e supports c;
    set basis = qualified(1, e);
    commit trust because enough using basis;
};
proc reconsider(c caveat) {
    when has_caveat(basis, c) reopen trust because caveated(basis, c);
};
proc indirect(c caveat) { call reconsider(c); };
"#
            .into(),
        },
        BundlePart {
            name: "main".into(),
            source: r#"
use deciding;
claim safe;
caveat stale consequence material;
evidence chart from "chart";
event choose;
event age;
define old_basis = has_caveat(deciding::basis, stale);
on choose call deciding::choose(chart, safe);
on age qualify chart with stale;
on age call deciding::indirect(stale);
bind hud.old = old_basis because deciding::basis;
"#
            .into(),
        },
    ]);
    let mut session = ReactiveSession::from_source(&source).unwrap();
    send(&mut session, "choose");
    send(&mut session, "age");
    let after = session.snapshot();
    assert_eq!(after.bindings["hud"]["old"], BindingValue::Bool(true));
    assert_eq!(after.decision_journal.len(), 2);
    assert_eq!(after.decision_journal[1].because, ["chart"]);
    assert_eq!(after.decision_journal[1].caveats, ["stale"]);
}

#[test]
fn browser_views_and_restored_sessions_keep_the_same_grounds_and_older_witnesses() {
    use caveat_runtime::web::WebReactiveSession;
    let mut original = WebReactiveSession::new(RENEWING).unwrap();
    original.dispatch_view("scout", "{}").unwrap();
    original.dispatch_view("choose", "{}").unwrap();
    original.dispatch_view("advance", r#"{"dt":29}"#).unwrap();
    original.dispatch_view("again", "{}").unwrap();
    let mut restored = WebReactiveSession::restore(RENEWING, &original.save().unwrap()).unwrap();
    assert_eq!(restored.view(), original.view());
    for payload in [r#"{"dt":1}"#, r#"{"dt":29}"#] {
        assert_eq!(
            original.dispatch_view("advance", payload),
            restored.dispatch_view("advance", payload)
        );
        assert_eq!(original.save().unwrap(), restored.save().unwrap());
    }
    let shown: serde_json::Value = serde_json::from_str(&restored.view()).unwrap();
    assert_eq!(shown["bindings"]["hud"]["old"], true);
    assert_eq!(
        shown["decision_journal"][1]["because"],
        serde_json::json!(["sight"])
    );
    assert_eq!(shown["decision_journal"].as_array().unwrap().len(), 2);
}
