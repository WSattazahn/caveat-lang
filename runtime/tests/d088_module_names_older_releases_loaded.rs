//! D088: a module of a bundle that names something after a word the language
//! took later, and loaded in the releases before it, loads again, read as the
//! newest release that could load it read it. Each case loaded in releases up
//! to 1.0.3, 1.1.0, 1.2.0 or 1.3.0, and main refused it: a module's
//! `only_if(…)` beside its own `state only_if`, which called the bundle
//! program's `fn m__only_if` before 1.4.0 ("expression requires boolean, found
//! number"); a module that declares `irreversible` or `forecloses` (refused
//! from 1.2.0: "module lib declares irreversible, which the grammar already
//! uses"); `reopened by changed ,r` beside a module's stream `changed` (refused
//! from 1.1.0: "decisions m__d reopened by: changed must name a declared
//! reading stream"); a module's `fn resolve (y)` or `event resolve revision`
//! (refused from 1.3.0: "unknown source function m__resolve"); and a module's
//! `once` after a retained caveat's `because` (refused from 1.4.0: "retaining …
//! because names evidence, not once").
use caveat_runtime::link;
use caveat_runtime::reactive::{ModuleBindings, ReactiveSession, RestoreOptions};
use serde_json::{json, Value};

const ROOT: &str =
    "use m;\nclaim c;\nevidence e from \"e\";\nstate s = 0;\nevent go;\non go set s = s + 1;\n";

fn bundle(module: &str, root: &str) -> String {
    format!(
        "#caveat-bundle 1\n#module m {}\n{module}#module main {}\n{root}",
        module.len(),
        root.len()
    )
}

fn load(source: &str) -> ReactiveSession {
    ReactiveSession::from_source(source).unwrap_or_else(|error| panic!("{error}\n{source}"))
}

fn values(session: &ReactiveSession) -> Value {
    serde_json::to_value(session.snapshot()).unwrap()["values"].clone()
}

fn send(session: &mut ReactiveSession, event: &str) {
    session
        .dispatch_json(event, "{}")
        .unwrap_or_else(|error| panic!("{event}: {error}"));
}

/// Before 1.4.0 the module's `only_if` was its own name, so `only_if(1, 2)`
/// called the program's `fn m__only_if`. The form, with a number for its
/// condition, does not load, so 1.4 could not load the bundle, and it is
/// read as 1.3.0 read it.
#[test]
fn a_module_only_if_beside_its_state_calls_the_program_function() {
    let root = format!("{ROOT}fn m__only_if(c, v) = if(c > 0, v, 0);\n");
    let rule = bundle(
        "module m;\nevent tick;\nstate t = 0;\nstate only_if = 0;\non tick set t = only_if(1, 2);\n",
        &root,
    );
    let mut session = load(&rule);
    send(&mut session, "m__tick");
    assert_eq!(values(&session)["m__t"], json!(2.0));
    let saved = session.save_json().unwrap();
    assert!(!saved.contains("module_bindings"), "{saved}");
    let restored = ReactiveSession::restore_json(&rule, &saved).unwrap();
    assert_eq!(restored.snapshot(), session.snapshot());

    let initial = bundle(
        "module m;\nstate only_if = 0;\nstate t = only_if(1, 2);\n",
        &root,
    );
    assert_eq!(values(&load(&initial))["m__t"], json!(2.0));
    load(&bundle(
        "module m;\nstate only_if = 0;\ndefine d = only_if(1, 2);\n",
        &root,
    ));
    let body = bundle(
        "module m;\nstate only_if = 0;\nfn f(x) = only_if(x, x);\nstate w = f(1);\n",
        &format!("{ROOT}fn m__only_if(c, v) = v;\n"),
    );
    assert_eq!(values(&load(&body))["m__w"], json!(1.0));

    // Without the program's function the call has nothing to reach, so the
    // form's own error stands.
    let error = ReactiveSession::from_source(&bundle(
        "module m;\nevent tick;\nstate t = 0;\nstate only_if = 0;\non tick set t = only_if(1, 2);\n",
        ROOT,
    ))
    .unwrap_err();
    assert!(
        error.contains("expression requires boolean, found number"),
        "{error}"
    );
}

/// A form that loads keeps its meaning, as in 1.4, though the program
/// declares the function the call would reach: here the call would give 6.
#[test]
fn a_module_only_if_form_that_loads_is_kept() {
    let source = bundle(
        "module m;\nstate only_if = 0;\nstate k = 0;\nstate u = 0;\nevent tick;\non tick set u = only_if(k >= 0, 5);\n",
        &format!("{ROOT}fn m__only_if(c, v) = v + 1;\n"),
    );
    let mut session = load(&source);
    send(&mut session, "m__tick");
    assert_eq!(values(&session)["m__u"], json!(5.0));
}

/// 1.4 read the module's `hold` two words before `because` as the
/// program's define, so `only_if( hold , because )` was the form with a
/// truth value for its condition. Read with module scope, `hold` is the
/// module's number, so the form does not load: 1.4 loaded the bundle only
/// through that capture, and it is read as 1.1.0 read it, the call. A save
/// that does not say which reading made it needs a choice, since releases
/// before 1.2.0 and from 1.2.0 both loaded it and read it differently; each
/// choice continues as its releases did.
#[test]
fn a_module_only_if_beside_a_captured_hold_reads_as_before_1_2() {
    let source = bundle(
        "module m;\nstate only_if = 0;\nstate hold = 0;\nstate u = 0;\nevent tick;\non tick set u = only_if( hold , because );\n",
        &format!("{ROOT}state because = 1;\ndefine hold = s > 0;\nfn m__only_if(c, v) = v;\n"),
    );
    let mut scoped = load(&source);
    let unmarked = scoped.save_json().unwrap();
    send(&mut scoped, "m__tick");
    assert_eq!(values(&scoped)["m__u"], json!(1.0));
    let saved: Value = serde_json::from_str(&scoped.save_json().unwrap()).unwrap();
    assert_eq!(saved["module_bindings"], json!("scoped"));

    let mut unmarked: Value = serde_json::from_str(&unmarked).unwrap();
    unmarked.as_object_mut().unwrap().remove("module_bindings");
    let unmarked = unmarked.to_string();
    let error = ReactiveSession::restore_json(&source, &unmarked).unwrap_err();
    assert!(error.contains("ambiguous"), "{error}");
    let legacy = RestoreOptions {
        module_bindings: Some(ModuleBindings::LegacyHoldCapture),
    };
    let mut held = ReactiveSession::restore_json_with_options(&source, &unmarked, &legacy).unwrap();
    send(&mut held, "m__tick");
    assert_eq!(values(&held)["m__u"], json!(0.0));
    send(&mut held, "go");
    send(&mut held, "m__tick");
    assert_eq!(values(&held)["m__u"], json!(1.0));
}

/// From 1.2.0 a module could not declare `irreversible` or `forecloses`,
/// and this runtime's linker refuses one still. Before 1.2.0 a module could,
/// and its names were its own wherever they stood, so such a bundle is read
/// as 1.1.0 read it, where 1.1.0 could load it.
#[test]
fn a_module_that_declares_irreversible_or_forecloses_reads_as_1_1() {
    for word in ["irreversible", "forecloses"] {
        let module = format!(
            "module lib;\nevidence me from \"m\";\nreadings {word} from me limit 2;\ndecisions d limit 3 reopened by {word};\nstate {word}_seen = 0;\nevent tick;\non tick set {word}_seen = 1;\n"
        );
        let root = "claim c;\nevidence e from \"e\";\nstate s = 0;\nevent go;\non go set s = s + 1;\nuse lib;\n";
        let source = format!(
            "#caveat-bundle 1\n#module lib {}\n{module}#module main {}\n{root}",
            module.len(),
            root.len()
        );
        let error = link::link(&source).unwrap_err();
        assert!(
            error.contains(&format!(
                "module lib declares {word}, which the grammar already uses"
            )),
            "{error}"
        );
        let mut session = load(&source);
        send(&mut session, "lib__tick");
        assert_eq!(values(&session)[format!("lib__{word}_seen")], json!(1.0));
    }
    let source = bundle(
        "module m;\nstate irreversible = 2;\nstate t = irreversible + 1;\n",
        ROOT,
    );
    assert_eq!(values(&load(&source))["m__t"], json!(3.0));

    // 1.1.0 called the program's `fn m__only_if` from the module's
    // `only_if(…)`; as linked now the rule would call its `fn only_if`.
    let source = bundle(
        "module m;\nstate irreversible = 0;\nstate only_if = 0;\nevent tick;\nstate u = 0;\non tick set u = only_if(2, 2);\n",
        &format!("{ROOT}fn m__only_if(c, v) = v + 100;\nfn only_if(c, v) = v + 10;\n"),
    );
    let mut session = load(&source);
    send(&mut session, "m__tick");
    assert_eq!(values(&session)["m__u"], json!(102.0));

    // A series that is irreversible came in 1.2.0, which refused the
    // module, and 1.1.0 read the word as the module's: no release loaded it.
    for module in [
        "module m;\ndecisions forecloses limit 2;\ndecisions d limit 1 irreversible;\n",
        "module m;\nstate irreversible = 1;\ndecisions d limit 1 irreversible;\n",
    ] {
        let error = ReactiveSession::from_source(&bundle(module, ROOT)).unwrap_err();
        assert!(error.contains("which the grammar already uses"), "{error}");
    }
}

/// A module that reads its own stream `changed` in `reopened by changed ,r`,
/// with nothing that came in 1.1.0.
const CHANGED: &str = "module m;\nreadings changed from e limit 2;\nreadings r from e limit 2;\ndecisions d limit 3 reopened by changed ,r;\nevent mc v min 0 max 1;\non mc sample changed = v supports c;\nevent commitd;\non commitd when not committed(d) or reopened(d) commit d because enough;\nstate seen = 0;\nevent check;\non check set seen = if(reopened(d), 1, 0);\n";

/// Whether `m__d`, committed, is reopened after `event` samples a stream.
fn reopened_by(source: &str, event: &str) -> f64 {
    let mut session = load(source);
    send(&mut session, "m__commitd");
    session
        .dispatch_json(event, "{\"v\":1}")
        .unwrap_or_else(|error| panic!("{event}: {error}"));
    send(&mut session, "m__check");
    values(&session)["m__seen"].as_f64().unwrap()
}

/// 1.1.0 to 1.4.1 ended a trigger only at a word that ends in `,` or `;`, so
/// `changed ,r` and `changed ;` left a module's `changed` bare, naming the
/// program's stream `changed`. Where the program declares one, they loaded,
/// and still read so: the program's stream reopens the series, the module's
/// does not. That holds for a module whose `changed` is a state, too.
#[test]
fn a_bare_changed_names_the_program_stream_as_in_1_1_to_1_4() {
    let root = format!(
        "{ROOT}readings changed from e limit 2;\nevent rc v min 0 max 1;\non rc sample changed = v;\n"
    );
    let source = bundle(CHANGED, &root);
    let linked = link::link(&source).unwrap();
    assert!(
        linked.contains("decisions m__d limit 3 reopened by changed ,m__r;"),
        "{linked}"
    );
    assert_eq!(reopened_by(&source, "rc"), 1.0);
    assert_eq!(reopened_by(&source, "m__mc"), 0.0);
    let saved = load(&source).save_json().unwrap();
    assert!(!saved.contains("module_bindings"), "{saved}");

    for trigger in [
        "changed ,r",
        "changed ;",
        "changed , r",
        "changed supporting c ;",
    ] {
        let module = format!(
            "module m;\nstate changed = 0;\nreadings r from e limit 2;\ndecisions d limit 3 reopened by {trigger};\nevent commitd;\non commitd when not committed(d) or reopened(d) commit d because enough;\nstate seen = 0;\nevent check;\non check set seen = if(reopened(d), 1, 0);\n"
        );
        let source = bundle(&module, &root);
        let linked = link::link(&source).unwrap();
        let trigger = trigger.replace(",r", ",m__r").replace(", r", ", m__r");
        assert!(
            linked.contains(&format!("reopened by {trigger};")),
            "{linked}"
        );
        // A sample reopens through `supporting c` only when it supports c.
        let expected = if trigger.contains("supporting") {
            0.0
        } else {
            1.0
        };
        assert_eq!(reopened_by(&source, "rc"), expected, "{trigger}");
    }
}

/// Before 1.1.0, `changed` was no clause word, so in `changed ,r` a module's
/// own stream `changed` was its own. From 1.1.0 such a module did not load
/// beside a program without a stream `changed`; it loads again, read as the
/// releases before 1.1.0 read it, and its saves continue.
#[test]
fn a_module_stream_changed_before_a_comma_is_its_own_as_before_1_1() {
    let source = bundle(CHANGED, ROOT);
    assert_eq!(reopened_by(&source, "m__mc"), 1.0);
    let mut session = load(&source);
    send(&mut session, "m__commitd");
    let saved = session.save_json().unwrap();
    assert!(!saved.contains("module_bindings"), "{saved}");
    let mut restored = ReactiveSession::restore_json(&source, &saved).unwrap();
    for session in [&mut session, &mut restored] {
        session.dispatch_json("m__mc", "{\"v\":1}").unwrap();
        send(session, "m__check");
    }
    assert_eq!(restored.save_json().unwrap(), session.save_json().unwrap());
    assert_eq!(values(&restored)["m__seen"], json!(1.0));

    // Those releases had no `changed` triggers, and from 1.1.0 `changed x
    // ,r` did not keep `changed` bare, so no release loaded this.
    let module = CHANGED.replace(
        "reopened by changed ,r;",
        "reopened by changed x ,r;\nreadings x from e limit 2;\nevent mx v min 0 max 1;\non mx sample x = v supports c;",
    );
    assert!(ReactiveSession::from_source(&bundle(&module, ROOT)).is_err());
    // Nor one that uses a form 1.1.0 added, here a neutral sample.
    let module = CHANGED.replace("sample changed = v supports c;", "sample changed = v;");
    assert!(ReactiveSession::from_source(&bundle(&module, ROOT)).is_err());
}

/// 1.1.0 read a module's `hold` two words before `because` as its own, and
/// `changed ,r` as the program's stream `changed`. Where only releases
/// before 1.2.0 could load the bundle, an unmarked save restores without a
/// choice and continues as 1.1.0 did; where releases from 1.2.0, which read
/// the program's `hold`, could load it too, the save needs one.
#[test]
fn an_older_save_beside_a_hold_and_a_bare_changed_continues_as_1_1_did() {
    let module = CHANGED.replace(
        "state seen = 0;",
        "state hold = 0;\nstate t = 0;\nevent tap;\non tap set t = max(1, hold ) because nothing;\nstate seen = 0;",
    );
    let root = format!(
        "{ROOT}readings changed from e limit 2;\nevent rc v min 0 max 1;\non rc sample changed = v;\n"
    );
    for (root, legacy_loads) in [
        (root.clone(), false),
        (format!("{root}state hold = 5;\n"), true),
    ] {
        let source = bundle(&module, &root);
        assert_eq!(reopened_by(&source, "rc"), 1.0);
        assert_eq!(reopened_by(&source, "m__mc"), 0.0);
        let mut session = load(&source);
        send(&mut session, "m__commitd");
        let mut unmarked: Value = serde_json::from_str(&session.save_json().unwrap()).unwrap();
        assert_eq!(
            unmarked.as_object_mut().unwrap().remove("module_bindings"),
            Some(json!("scoped"))
        );
        let unmarked = unmarked.to_string();
        match ReactiveSession::restore_json(&source, &unmarked) {
            Ok(mut restored) => {
                assert!(!legacy_loads);
                for session in [&mut session, &mut restored] {
                    send(session, "m__tap");
                    session.dispatch_json("rc", "{\"v\":1}").unwrap();
                    send(session, "m__check");
                }
                assert_eq!(values(&restored)["m__t"], json!(1.0));
                assert_eq!(values(&restored)["m__seen"], json!(1.0));
            }
            Err(error) => assert!(legacy_loads && error.contains("ambiguous"), "{error}"),
        }
    }
}

/// From 1.1.0 to 1.3.x a module's `only_if(…)` beside its own `state
/// only_if` called the program's `fn m__only_if`, and `changed ,r` named the
/// program's stream `changed`. The form does not load, so 1.4 could not load
/// the bundle, and both are read as 1.3.0 read them.
#[test]
fn a_module_only_if_call_keeps_the_bare_changed_of_its_releases() {
    let module = CHANGED
        .replace(
            "readings changed from e limit 2;",
            "state changed = 0;\nstate only_if = 0;\nstate u = 0;\nevent tick;\non tick set u = only_if(1, 2);",
        )
        .replace("event mc v min 0 max 1;\non mc sample changed = v supports c;\n", "");
    let root = format!(
        "{ROOT}fn m__only_if(c, v) = v;\nreadings changed from e limit 2;\nevent rc v min 0 max 1;\non rc sample changed = v;\n"
    );
    let source = bundle(&module, &root);
    assert_eq!(reopened_by(&source, "rc"), 1.0);
    let mut session = load(&source);
    send(&mut session, "m__tick");
    assert_eq!(values(&session)["m__u"], json!(2.0));
}

/// The releases that could load a bundle as it is linked decide how it is
/// read. A function's body that calls a function named like a later
/// built-in, which releases from that built-in on refused (D084), ends
/// them: `on_hold` from 1.2.0 and `outcome` from 1.3.0, so 1.4 could not
/// load this bundle either. The module's `only_if(2, 2)` calls
/// `fn m__only_if`, as in the last release that loaded it, though the call
/// of the program's `fn only_if` loads too.
#[test]
fn a_function_body_later_releases_refused_keeps_the_reading_of_the_releases_before() {
    let module =
        "module m;\nstate only_if = 0;\nevent tick;\nstate u = 0;\non tick set u = only_if(2, 2);\n";
    for built_in in ["on_hold", "outcome"] {
        let source = bundle(
            module,
            &format!(
                "{ROOT}fn m__only_if(c, v) = v + 100;\nfn only_if(c, v) = v + 10;\nfn a_call(x) = {built_in}(x);\nfn {built_in}(x) = x + 1;\nstate w = 0;\non go set w = a_call(s);\n"
            ),
        );
        let mut session = load(&source);
        send(&mut session, "m__tick");
        send(&mut session, "go");
        assert_eq!(values(&session)["m__u"], json!(102.0), "{built_in}");
        assert_eq!(values(&session)["w"], json!(2.0), "{built_in}");
    }
    // 1.2.0 to 1.4.1 read the module's `hold` two words before `because` as
    // the program's, which declares none, and refused it: 1.1.0 loaded it.
    let source = bundle(
        &format!("{module}state hold = 0;\nstate mm = 0;\nevent tap;\non tap set mm = max(1, hold ) because nothing;\n"),
        &format!("{ROOT}fn m__only_if(c, v) = v + 100;\nfn only_if(c, v) = v + 10;\n"),
    );
    let mut session = load(&source);
    send(&mut session, "m__tick");
    assert_eq!(values(&session)["m__u"], json!(102.0));
}

/// 1.1.0 refused a function's body that calls `in_basis`, so only the
/// releases before it could load this bundle, and they read the module's
/// `changed ,r` as its own stream, and its `only_if(1, 2)` as a call of
/// `fn m__only_if`, though the program declares a stream `changed`.
#[test]
fn a_bundle_only_releases_before_1_1_could_load_reads_as_they_did() {
    let module = format!(
        "{CHANGED}state only_if = 0;\nstate t = 0;\nevent tick;\non tick set t = only_if(1, 2);\n"
    );
    let root = format!(
        "{ROOT}readings changed from e limit 2;\nevent rc v min 0 max 1;\non rc sample changed = v supports c;\nfn m__only_if(c, v) = v + 100;\nfn a_call(x) = in_basis(x, x);\nfn in_basis(x, y) = x + y;\nstate w = 0;\non go set w = a_call(s);\n"
    );
    let source = bundle(&module, &root);
    assert_eq!(reopened_by(&source, "m__mc"), 1.0);
    assert_eq!(reopened_by(&source, "rc"), 0.0);
    let mut session = load(&source);
    send(&mut session, "m__tick");
    send(&mut session, "go");
    assert_eq!(values(&session)["m__t"], json!(102.0));
    assert_eq!(values(&session)["w"], json!(2.0));
    let saved = session.save_json().unwrap();
    assert!(!saved.contains("module_bindings"), "{saved}");
    let restored = ReactiveSession::restore_json(&source, &saved).unwrap();
    assert_eq!(restored.snapshot(), session.snapshot());
}

/// An older reading is taken only where a release that read a bundle so
/// could load it: not with a form that came later, here the `only_if` form
/// with a truth value, or a function body a release refused.
#[test]
fn a_bundle_no_release_loaded_is_not_read_the_way_older_releases_linked() {
    let refused = [
        bundle(
            "module m;\nreadings changed from e limit 2;\nreadings r from e limit 2;\ndecisions d limit 3 reopened by changed ,r;\nevent mc v min 0 max 1;\non mc sample changed = v supports c;\nstate u = 0;\nevent tick;\non tick set u = only_if(1 > 0, 2);\n",
            ROOT,
        ),
        bundle(
            "module m;\nstate only_if = 0;\nevent tick;\nstate u = 0;\non tick set u = only_if(2, 2);\n",
            &format!(
                "{ROOT}fn m__only_if(c, v) = v + 100;\nfn only_if(c, v) = c > v;\nfn g(x) = only_if(x > 0, x);\n"
            ),
        ),
    ];
    for source in refused {
        assert!(ReactiveSession::from_source(&source).is_err(), "{source}");
    }
}

/// 1.3.0 began keeping a module's `resolve` bare before a name and `=`, or
/// before `revision`, for outcomes; before it the word was the module's own
/// wherever it stood. A module's `fn resolve (y)`, with a space before its
/// parameters, or `event resolve revision`, loaded up to 1.2.0 and is read
/// so again: the event is the module's `m__resolve`, also where only 1.1.0
/// and the releases before it could load the bundle. `link::link` reads it
/// as 1.4 did.
#[test]
fn a_module_resolve_before_a_name_and_equals_is_its_own_as_before_1_3() {
    let source = bundle(
        "module m;\nstate t = 0;\nevent tick;\nfn resolve (y) = y + 1;\non tick set t = resolve(t);\n",
        ROOT,
    );
    assert!(link::link(&source)
        .unwrap()
        .contains("fn resolve (y) = y + 1;"));
    let mut session = load(&source);
    send(&mut session, "m__tick");
    send(&mut session, "m__tick");
    assert_eq!(values(&session)["m__t"], json!(2.0));

    for stream in [
        "",
        "evidence me from \"m\";\nreadings irreversible from me limit 2;\ndecisions d limit 3 reopened by irreversible;\n",
    ] {
        let source = bundle(
            &format!("module m;\n{stream}state t = 0;\nevent resolve revision min 0 max 9;\non resolve set t = t + revision;\n"),
            ROOT,
        );
        let mut session = load(&source);
        session
            .dispatch_json("m__resolve", "{\"revision\":4}")
            .unwrap();
        assert_eq!(values(&session)["m__t"], json!(4.0));
        assert!(session
            .dispatch_json("resolve", "{\"revision\":4}")
            .is_err());
    }
}

/// The module's retained caveat `gap` because its own evidence `once`.
const ONCE: &str = "module m;\nclaim k;\nevidence once from \"o\";\ncaveat gap consequence low;\ndecisions d limit 3;\nevent tick;\non tick reveal once supports k;\non tick commit d because enough retaining gap because once ;\n";

/// Whether the commitment `m__d@1` relies on `evidence`.
fn relies_on(session: &ReactiveSession, evidence: &str) -> bool {
    let saved: Value = serde_json::from_str(&session.save_json().unwrap()).unwrap();
    saved["graph"]["relations"]
        .as_array()
        .unwrap()
        .contains(&json!(["m__d@1", "relies_on", evidence]))
}

/// 1.4.0 began keeping a module's `once` bare after a `because` where the
/// item ends, once `retaining` has come, for `because once`; before it the
/// word was the module's own. A retention reason written with a space before
/// its end, `because once ;`, loaded from 1.1.0 to 1.3.0, and a citation
/// `because once , x` in a procedure whose earlier step retains a caveat
/// from rc.1 to 1.3.0; both are read so again, and the commitment relies on
/// the module's evidence `once`. Where the program declares an evidence
/// `once` that it reveals, 1.4.0 and 1.4.1 loaded the bundle too, reading
/// the program's, and so does this runtime.
#[test]
fn a_module_once_after_a_retained_caveat_is_its_own_as_before_1_4() {
    for (root, evidence) in [
        (ROOT.to_string(), "m__once"),
        (format!("{ROOT}evidence once from \"r\";\n"), "m__once"),
        (
            format!("{ROOT}evidence once from \"r\";\non go reveal once supports c;\n"),
            "once",
        ),
    ] {
        let mut session = load(&bundle(ONCE, &root));
        send(&mut session, "go");
        send(&mut session, "m__tick");
        assert!(relies_on(&session, evidence), "{root}");
    }
    let source = bundle(
        "module m;\nclaim k;\nevidence ev from \"o\";\ncaveat gap consequence low;\ndecisions d limit 3;\nstate once = 1;\nstate x = 2;\nstate t = 0;\nevent tick;\nproc p() {\n  reveal ev supports k;\n  commit d because enough retaining gap;\n  set t = once + x because once , x;\n};\non tick call p();\n",
        ROOT,
    );
    let mut session = load(&source);
    send(&mut session, "m__tick");
    assert_eq!(values(&session)["m__t"], json!(3.0));
}

/// Where 1.3.0 was the newest release that could load a bundle, it is read
/// as 1.3.0 linked it, with a module's `hold` its own: here an outcome, which
/// came in 1.3.0, beside the module's `because once ;`, which 1.4.0 refused.
/// 1.3.0 read the module's `hold` before `because` as the program's, so a
/// save it wrote restores so without a choice, since only releases from
/// 1.2.0 could have written it.
#[test]
fn a_bundle_1_3_0_was_the_newest_to_load_reads_as_1_3_0_linked_it() {
    let module = format!(
        "{ONCE}state hold = 7;\nstate hv = 0;\nfn add(a, b) = a + b;\non tick set hv = add( hold , because );\n"
    );
    let root = format!(
        "{ROOT}readings oc from e limit 4;\ndecisions fc limit 4;\nevent pr p min 0 max 1;\non pr commit fc because enough using p;\nevent rp q min 0 max 1;\non rp sample oc = q supports c;\non rp resolve fc = q because latest(oc);\nstate because = 3;\nstate hold = 11;\n"
    );
    let source = bundle(&module, &root);
    let mut session = load(&source);
    let mut unmarked: Value = serde_json::from_str(&session.save_json().unwrap()).unwrap();
    assert_eq!(
        unmarked.as_object_mut().unwrap().remove("module_bindings"),
        Some(json!("scoped"))
    );
    send(&mut session, "m__tick");
    assert_eq!(values(&session)["m__hv"], json!(10.0));
    assert!(relies_on(&session, "m__once"));

    let mut restored = ReactiveSession::restore_json(&source, &unmarked.to_string()).unwrap();
    send(&mut restored, "m__tick");
    assert_eq!(values(&restored)["m__hv"], json!(14.0));
    assert!(relies_on(&restored, "m__once"));
    let saved: Value = serde_json::from_str(&restored.save_json().unwrap()).unwrap();
    assert_eq!(saved["module_bindings"], json!("legacy-hold-capture"));
}

/// Where only releases before 1.2.0 could load a bundle, none read a
/// module's `hold` or `release` as the program's. A host that asks for that
/// reading gets the bundle as they read it where it would change nothing, as
/// in the first bundle, which has neither word: its saves need no marker, and
/// 1.4's reading, with its `changed ,r` naming the program's stream, is no
/// reading of it. Where it would change the reading, as in the second
/// bundle, asking for it is refused, and a save restores as 1.1.0 read it.
#[test]
fn asking_for_hold_capture_where_no_release_read_it_reads_as_the_releases_that_did() {
    let legacy = RestoreOptions {
        module_bindings: Some(ModuleBindings::LegacyHoldCapture),
    };
    // rc.3 to 1.0.3 load it: 1.1.0 refused a function body that calls
    // `in_basis`.
    let changed = bundle(
        "module m;\nclaim mc;\nevidence me from \"m\";\nreadings changed from me limit 2;\nreadings r from me limit 2;\ndecisions d limit 3 reopened by changed ,r;\nstate n = 0;\nevent pick;\non pick when not committed(d) commit d because enough using n;\nevent obs x min 0 max 10;\non obs sample changed = x supports mc;\n",
        &format!("{ROOT}readings changed from e limit 2;\nevent rootobs x min 0 max 10;\non rootobs sample changed = x supports c;\nfn in_basis(x, y) = x + y;\nfn g(x) = in_basis(x, x);\nstate w = 0;\non go set w = g(s);\n"),
    );
    let mut fresh = load(&changed);
    send(&mut fresh, "m__pick");
    let saved = fresh.save_json().unwrap();
    assert!(!saved.contains("module_bindings"), "{saved}");
    let mut chosen = ReactiveSession::restore_json_with_options(&changed, &saved, &legacy).unwrap();
    for session in [&mut fresh, &mut chosen] {
        session.dispatch_json("rootobs", "{\"x\":3}").unwrap();
        session.dispatch_json("m__obs", "{\"x\":3}").unwrap();
    }
    let continued = chosen.save_json().unwrap();
    assert_eq!(continued, fresh.save_json().unwrap());
    assert_eq!(
        ReactiveSession::restore_json(&changed, &continued)
            .unwrap()
            .save_json()
            .unwrap(),
        continued
    );

    // rc.1 to 1.1.0 load it: 1.2.0 refused a function body that calls
    // `on_hold`.
    let hold = bundle(
        "module m;\nclaim only_if;\nstate uo = 0;\nstate hold = 7;\nstate hv = 0;\nfn add(a, b) = a + b;\nevent tk;\non tk set uo = only_if(0, 2);\non tk set hv = add( hold , because );\n",
        &format!("{ROOT}fn m__only_if(c, v) = v;\nfn only_if(c, v) = v + 10;\nfn on_hold(x) = x + 1;\nfn b_call(x) = on_hold(x);\nstate q = 0;\non go set q = b_call(s);\nstate because = 3;\nstate hold = 11;\n"),
    );
    let mut fresh = load(&hold);
    send(&mut fresh, "m__tk");
    assert_eq!(values(&fresh)["m__uo"], json!(2.0));
    assert_eq!(values(&fresh)["m__hv"], json!(10.0));
    let mut unmarked: Value = serde_json::from_str(&fresh.save_json().unwrap()).unwrap();
    assert_eq!(
        unmarked.as_object_mut().unwrap().remove("module_bindings"),
        Some(json!("scoped"))
    );
    let unmarked = unmarked.to_string();
    // As 1.1.0 wrote it, a save without a marker restores with no choice.
    let mut restored = ReactiveSession::restore_json(&hold, &unmarked).unwrap();
    send(&mut restored, "m__tk");
    assert_eq!(values(&restored)["m__hv"], json!(10.0));
    assert_eq!(values(&restored)["m__uo"], json!(2.0));
    assert_eq!(
        ReactiveSession::restore_json_with_options(&hold, &unmarked, &legacy).unwrap_err(),
        "cannot restore save: no release from 1.2.0 to 1.4.1 could load this source, so its moduleBindings cannot be legacy-hold-capture"
    );
}
