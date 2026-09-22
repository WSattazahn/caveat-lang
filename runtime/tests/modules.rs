//! Draft 0.5 module linking.

use caveat_runtime::link::{self, BundlePart};
use caveat_runtime::reactive::BindingValue;
use caveat_runtime::{eval, parser, Attention, Consequence, NodeKind};

fn part(name: &str, source: &str) -> BundlePart {
    BundlePart {
        name: name.into(),
        source: source.into(),
    }
}

fn bundle(parts: &[(&str, &str)]) -> String {
    let parts: Vec<BundlePart> = parts
        .iter()
        .map(|(name, source)| part(name, source))
        .collect();
    link::bundle(&parts)
}

fn linked(parts: &[(&str, &str)]) -> String {
    link::link(&bundle(parts)).expect("bundle should link")
}

fn error(parts: &[(&str, &str)]) -> String {
    link::link(&bundle(parts)).expect_err("bundle should be rejected")
}

fn evaluate(source: &str) -> eval::Evaluation {
    eval::evaluate(&parser::parse(source).expect("linked source should parse"))
        .expect("linked program should evaluate")
}

const WEATHER: &str = "module weather;\n\
     claim low_visibility;\n\
     evidence fog_bank from \"lookout report\";\n\
     caveat fog_refraction consequence material;\n\
     fog_bank supports low_visibility;\n";

#[test]
fn a_source_without_a_bundle_marker_is_unchanged() {
    let single = "budget 2; claim mist; caveat drift consequence low; examine drift cost 1;";
    assert_eq!(link::link(single).expect("plain source links"), single);
    let parts = link::split_bundle(single).expect("plain source splits");
    assert_eq!(parts.len(), 1);
    assert_eq!(parts[0].source, single);
    assert!(parts[0].name.is_empty());
}

#[test]
fn bundle_headers_round_trip_exactly() {
    let text = bundle(&[("weather", WEATHER), ("main", "use weather;\n")]);
    assert!(text.starts_with(link::BUNDLE_MARKER));
    let parts = link::split_bundle(&text).expect("bundle splits");
    assert_eq!(parts.len(), 2);
    assert_eq!(parts[0].name, "weather");
    assert_eq!(parts[0].source, WEATHER);
    assert_eq!(parts[1].source, "use weather;\n");
    // Splitting is byte-exact, so rebuilding reproduces the bundle.
    assert_eq!(link::bundle(&parts), text);
}

#[test]
fn a_module_declaration_becomes_a_flat_name_the_program_can_reach() {
    let source = linked(&[
        ("weather", WEATHER),
        (
            "main",
            "use weather;\nbudget 4;\nexamine weather::fog_refraction cost 2;\n",
        ),
    ]);
    assert!(
        !source.contains("module weather") && !source.contains("use weather"),
        "link-time statements must not reach the evaluator: {source}"
    );
    assert!(
        source.contains("claim weather__low_visibility;"),
        "{source}"
    );
    assert!(
        source.contains("weather__fog_bank supports weather__low_visibility;"),
        "{source}"
    );
    assert!(
        source.contains("examine weather__fog_refraction cost 2;"),
        "{source}"
    );

    let evaluation = evaluate(&source);
    let caveat = evaluation.symbols["weather__fog_refraction"];
    assert!(
        matches!(
            evaluation.graph.nodes[&caveat],
            NodeKind::Caveat {
                consequence: Consequence::Material,
                attention: Attention::Examined,
                ..
            }
        ),
        "{:?}",
        evaluation.graph.nodes[&caveat]
    );
    let ledger = evaluation.resources.expect("budget is recorded");
    assert_eq!(ledger.spent, 2);
}

#[test]
fn importing_does_not_observe_or_examine() {
    // The program imports the module and does nothing else.
    let source = linked(&[("weather", WEATHER), ("main", "use weather;\nbudget 4;\n")]);
    let evaluation = evaluate(&source);
    let caveat = evaluation.symbols["weather__fog_refraction"];
    assert!(
        !matches!(
            evaluation.graph.nodes[&caveat],
            NodeKind::Caveat {
                attention: Attention::Examined,
                ..
            }
        ),
        "use must not examine an imported caveat: {:?}",
        evaluation.graph.nodes[&caveat]
    );
    let ledger = evaluation.resources.expect("budget is recorded");
    assert_eq!(ledger.spent, 0, "importing spends no attention");
}

#[test]
fn a_module_reached_by_two_paths_is_linked_once() {
    let source = linked(&[
        ("base", "module base;\nclaim shared;\n"),
        (
            "left",
            "module left;\nuse base;\nevidence port from \"port watch\";\nport supports base::shared;\n",
        ),
        (
            "right",
            "module right;\nuse base;\nevidence starboard from \"starboard watch\";\nstarboard supports base::shared;\n",
        ),
        ("main", "use left;\nuse right;\nbudget 1;\n"),
    ]);
    assert_eq!(
        source.matches("claim base__shared;").count(),
        1,
        "a diamond import must not duplicate the declaration: {source}"
    );

    let evaluation = evaluate(&source);
    let shared = evaluation.symbols["base__shared"];
    let supporting = evaluation
        .graph
        .edges
        .iter()
        .filter(|edge| edge.to == shared)
        .count();
    assert_eq!(
        supporting, 2,
        "both modules must support the same node, not one each"
    );
}

#[test]
fn quoted_text_and_comments_are_not_rewritten() {
    let source = linked(&[
        ("weather", WEATHER),
        (
            "main",
            "use weather;\n\
             # weather::fog_refraction is named in this comment\n\
             claim note;\n\
             evidence log from \"see weather::fog_refraction in the log\";\n\
             log supports note;\n",
        ),
    ]);
    assert!(
        source.contains("\"see weather::fog_refraction in the log\""),
        "string contents must survive linking: {source}"
    );
    assert!(
        source.contains("# weather::fog_refraction is named in this comment"),
        "comments must survive linking: {source}"
    );
    // The program's own names stay bare.
    assert!(source.contains("claim note;"), "{source}");
    evaluate(&source);
}

#[test]
fn a_module_may_not_execute() {
    for (statement, expected) in [
        ("budget 3;", "attention is spent by the program"),
        ("examine fog_refraction cost 1;", "an examination"),
        ("defer fog_refraction;", "an attention statement"),
        ("commit hold because enough;", "a commitment"),
    ] {
        let module = format!("{WEATHER}{statement}\n");
        let message = error(&[("weather", &module), ("main", "use weather;\n")]);
        assert!(
            message.contains(expected),
            "{statement} should be rejected with {expected}, got: {message}"
        );
    }
}

#[test]
fn unresolvable_and_unauthorised_names_are_rejected_before_execution() {
    let cases: Vec<(Vec<(&str, &str)>, &str)> = vec![
        (
            vec![("main", "use missing;\n")],
            "imports a module that is not in the bundle: missing",
        ),
        (
            vec![
                ("weather", WEATHER),
                ("main", "claim here;\nevidence e from \"s\";\ne supports weather::low_visibility;\n"),
            ],
            "without `use weather;`",
        ),
        (
            vec![
                ("weather", WEATHER),
                ("main", "use weather;\nclaim here;\nevidence e from \"s\";\ne supports weather::absent;\n"),
            ],
            "module weather does not declare absent",
        ),
        (
            vec![
                ("a", "module a;\nuse b;\nclaim from_a;\n"),
                ("b", "module b;\nuse a;\nclaim from_b;\n"),
                ("main", "use a;\n"),
            ],
            "import cycle",
        ),
        (
            vec![
                ("weather", "module weather;\nclaim low__visibility;\n"),
                ("main", "use weather;\n"),
            ],
            "reserved for linked names",
        ),
        (
            vec![
                ("weather", WEATHER),
                ("main", "module main;\nclaim here;\n"),
            ],
            "is the program and must not declare a module",
        ),
    ];
    for (parts, expected) in cases {
        let message = error(&parts);
        assert!(
            message.contains(expected),
            "expected {expected}, got: {message}"
        );
    }
}

#[test]
fn a_bundle_header_must_match_the_text_that_follows() {
    let text = format!(
        "{}\n#module weather 9999\nmodule weather;\n",
        link::BUNDLE_MARKER
    );
    let message = link::split_bundle(&text).expect_err("a short part must be rejected");
    assert!(message.contains("declares 9999 bytes"), "{message}");

    let duplicate = format!(
        "{}\n#module weather 16\nmodule weather;\n#module weather 16\nmodule weather;\n",
        link::BUNDLE_MARKER
    );
    let message = link::split_bundle(&duplicate).expect_err("duplicates must be rejected");
    assert!(message.contains("duplicate module in bundle"), "{message}");
}

#[test]
fn a_malformed_part_is_named_with_the_line_the_author_wrote() {
    // Linking parses each part on its own, so the line number is already the
    // one in that file rather than one in the concatenated output.
    let message = link::link(&bundle(&[
        ("weather", WEATHER),
        (
            "main",
            "use weather;
claim ok;
this is not a statement;
",
        ),
    ]))
    .expect_err("a malformed program must be rejected");
    assert!(
        message.contains("the program main") && message.contains("line 3"),
        "expected the program and line 3, got: {message}"
    );

    let message = link::link(&bundle(&[
        (
            "weather",
            &format!(
                "{WEATHER}also not a statement;
"
            ),
        ),
        (
            "main",
            "use weather;
",
        ),
    ]))
    .expect_err("a malformed module must be rejected");
    assert!(
        message.contains("module weather") && message.contains("line 6"),
        "expected module weather and line 6, got: {message}"
    );
}

#[test]
fn the_prelude_links_as_an_ordinary_module() {
    // spec/caveat-0.5-draft.md section 1: the standard library stops being a
    // special case in reactive.rs and becomes a module like any other.
    let prelude = format!("module prelude;\n{}", include_str!("../prelude.cav"));
    let source = linked(&[
        ("prelude", &prelude),
        (
            "main",
            "use prelude;\nfn headroom(metres) = prelude::clamp(metres / 400, 0, 1);\n",
        ),
    ]);
    assert!(
        source.contains("fn prelude__clamp(value, lower, upper)"),
        "prelude functions are namespaced: {source}"
    );
    assert!(
        source.contains("prelude__min(prelude__max(value, lower), upper)"),
        "a module's internal calls resolve to its own names: {source}"
    );
    assert!(
        source.contains("fn headroom(metres) = prelude__clamp(metres / 400, 0, 1);"),
        "the program's call is rewritten, its own name is not: {source}"
    );
    // The comments in prelude.cav survive, and `text(...)`/`floor(...)` are
    // runtime primitives, not module names, so they must stay bare.
    assert!(
        source.contains("fn prelude__number_text(value) = text(round(value));"),
        "{source}"
    );

    // And it still runs: the linked functions evaluate through the ordinary
    // reactive session, with no prelude-specific handling.
    let program = parser::parse(&source).expect("linked prelude should parse");
    assert!(
        program.statements.len() > 10,
        "expected the whole library plus the program"
    );
}

const DISCOVERY: &str = "module discovery;\n\
     claim mushroom_discovered;\n\
     evidence first_mushroom from \"first authored absorption\";\n\
     caveat single_absorption consequence material;\n\
     single_absorption qualifies first_mushroom;\n";

const GLOW_PROGRAM: &str = "use discovery;\n\
     state learned = 0 min 0 max 1;\n\
     event absorb;\n\
     proc learn() {\n\
       reveal discovery::first_mushroom supports discovery::mushroom_discovered;\n\
       set learned = qualified(1, discovery::first_mushroom);\n\
     };\n\
     on absorb when not observed(discovery::first_mushroom) call learn();\n\
     bind ability.learned = learned == 1;\n";

#[test]
fn a_multi_module_bundle_runs_as_one_program_with_one_graph() {
    let text = bundle(&[("discovery", DISCOVERY), ("main", GLOW_PROGRAM)]);
    let mut session =
        caveat_runtime::reactive::ReactiveSession::from_source(&text).expect("bundle runs");

    let before = session.snapshot();
    assert_eq!(
        before.bindings["ability"]["learned"],
        BindingValue::Bool(false)
    );

    session.dispatch_json("absorb", "{}").expect("absorb");
    let after = session.snapshot();
    assert_eq!(
        after.bindings["ability"]["learned"],
        BindingValue::Bool(true),
        "the program's rule fired on the module's evidence"
    );

    // Re-absorbing is still guarded by the imported evidence being observed.
    session
        .dispatch_json("absorb", "{}")
        .expect("second absorb");
    assert_eq!(
        session.snapshot().bindings["ability"]["learned"],
        BindingValue::Bool(true)
    );
}

#[test]
fn a_bundle_is_identified_by_its_own_bytes_not_by_the_linked_output() {
    // spec/caveat-0.5-draft.md section 5: the bundle is the artifact, so saves
    // and byte-pinned receipts keep referring to what the author shipped.
    let text = bundle(&[("discovery", DISCOVERY), ("main", GLOW_PROGRAM)]);
    let from_bundle = caveat_runtime::reactive::ReactiveSession::from_source(&text)
        .expect("bundle runs")
        .snapshot()
        .source_id;
    let again = caveat_runtime::reactive::ReactiveSession::from_source(&text)
        .expect("bundle runs")
        .snapshot()
        .source_id;
    assert_eq!(from_bundle, again, "identity is stable for the same bundle");

    let linked_text = link::link(&text).expect("bundle links");
    let from_linked = caveat_runtime::reactive::ReactiveSession::from_source(&linked_text)
        .expect("linked source runs")
        .snapshot()
        .source_id;
    assert_ne!(
        from_bundle, from_linked,
        "identity must follow the bundle, not the linker's output"
    );
}

// ---- loading from disk ----

use caveat_runtime::link::disk;
use std::path::{Path, PathBuf};

/// Fixtures live under the gitignored target directory, keyed by process, so
/// concurrent test binaries cannot sweep each other's files.
fn fixture_root(test: &str) -> PathBuf {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("target/module-fixtures")
        .join(format!("{}-{test}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("fixture directory");
    root
}

fn write(root: &Path, name: &str, contents: &str) -> PathBuf {
    let path = root.join(name);
    std::fs::write(&path, contents).expect("fixture file");
    path
}

fn repo(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join(relative)
}

#[test]
fn an_on_disk_program_loads_links_and_runs_its_modules() {
    let entry = repo("examples/modules/crossing.cav");
    let text = disk::load(&entry).expect("crossing loads");
    let parts = link::split_bundle(&text).expect("bundle splits");
    assert_eq!(
        parts
            .iter()
            .map(|part| part.name.as_str())
            .collect::<Vec<_>>(),
        vec!["weather", "crossing"],
        "modules come first, the program last"
    );

    let source = link::link(&text).expect("crossing links");
    let evaluation = evaluate(&source);

    // The commitment retains an imported caveat, and attention came out of the
    // program's single budget.
    let ledger = evaluation.resources.clone().expect("budget is recorded");
    assert_eq!((ledger.spent, ledger.initial, ledger.remaining), (2, 6, 4));
    assert!(
        evaluation.symbols.contains_key("enter_channel"),
        "the program's own names stay bare: {:?}",
        evaluation.symbols.keys().collect::<Vec<_>>()
    );

    // Imported evidence keeps the module's provenance, not the importer's.
    let forecast = evaluation.symbols["weather__dawn_forecast"];
    match &evaluation.graph.nodes[&forecast] {
        NodeKind::Evidence { source, .. } => {
            assert_eq!(source, "harbour forecast issued 05:00")
        }
        other => panic!("expected evidence, got {other:?}"),
    }

    // `display` in a module follows its symbol.
    assert_eq!(
        evaluation.display["weather__forecast_age"],
        "The forecast is over an hour old."
    );

    // Examining in the program is what made the imported caveat examined.
    let aged = evaluation.symbols["weather__forecast_age"];
    assert!(
        matches!(
            evaluation.graph.nodes[&aged],
            NodeKind::Caveat {
                attention: Attention::Examined,
                ..
            }
        ),
        "{:?}",
        evaluation.graph.nodes[&aged]
    );
    // The module's other caveat was never paid for.
    let refraction = evaluation.symbols["weather__refraction"];
    assert!(
        !matches!(
            evaluation.graph.nodes[&refraction],
            NodeKind::Caveat {
                attention: Attention::Examined,
                ..
            }
        ),
        "an unexamined imported caveat must stay unexamined"
    );
}

#[test]
fn a_program_without_imports_is_loaded_byte_for_byte() {
    // Vessel pins the sha256 of this file, so loading must not touch it.
    let entry = repo("examples/slime_glow_ability.cav");
    let loaded = disk::load(&entry).expect("single-file program loads");
    let raw = std::fs::read_to_string(&entry).expect("fixture reads");
    assert_eq!(
        loaded, raw,
        "a program with no imports is returned unchanged"
    );
    assert!(!loaded.starts_with(link::BUNDLE_MARKER));
}

#[test]
fn a_use_cannot_name_a_path() {
    let root = fixture_root("escape");
    for name in ["../secret", "sub/secret", "C:/secret", ".."] {
        let entry = write(&root, "main.cav", &format!("use {name};\nclaim here;\n"));
        let message = disk::load(&entry).expect_err("a path must be rejected");
        assert!(
            message.contains("is not a module name"),
            "{name} should be rejected as a name, got: {message}"
        );
    }
}

#[test]
fn a_missing_or_mislabelled_module_file_is_named_in_the_error() {
    let root = fixture_root("resolution");

    let entry = write(&root, "main.cav", "use weather;\nclaim here;\n");
    let message = disk::load(&entry).expect_err("a missing module must be reported");
    assert!(message.contains("cannot read"), "{message}");
    assert!(message.contains("weather.cav"), "{message}");

    write(&root, "weather.cav", "module climate;\nclaim fog;\n");
    let message = disk::load(&entry).expect_err("a mismatched name must be reported");
    assert!(
        message.contains("declares `module climate;` but is imported as weather"),
        "{message}"
    );

    write(&root, "weather.cav", "claim fog;\n");
    let message = disk::load(&entry).expect_err("a missing module header must be reported");
    assert!(
        message.contains("must begin with `module weather;`"),
        "{message}"
    );
}

#[test]
fn imports_are_followed_through_modules() {
    let root = fixture_root("transitive");
    write(&root, "base.cav", "module base;\nclaim shared;\n");
    write(
        &root,
        "middle.cav",
        "module middle;\nuse base;\nevidence relay from \"relay\";\nrelay supports base::shared;\n",
    );
    let entry = write(&root, "main.cav", "use middle;\nbudget 1;\n");

    let text = disk::load(&entry).expect("transitive load");
    let parts = link::split_bundle(&text).expect("splits");
    let names: Vec<&str> = parts.iter().map(|part| part.name.as_str()).collect();
    assert!(
        names.contains(&"base"),
        "a module reached only through another module must still be loaded: {names:?}"
    );

    let evaluation = evaluate(&link::link(&text).expect("links"));
    assert!(evaluation.symbols.contains_key("base__shared"));
}

#[test]
fn a_module_may_not_shadow_a_parameter_with_a_declaration() {
    // The rewriter is lexical, so a declared name that is also a parameter
    // would be rewritten inside the body and break the parameter reference.
    let module = "module scale;\n\
         claim value;\n\
         fn double(value) = value + value;\n";
    let message = error(&[("scale", module), ("main", "use scale;\nbudget 1;\n")]);
    assert!(
        message.contains("value"),
        "a declaration shadowing a parameter must be rejected, got: {message}"
    );
}

// ---- reactive declarations in modules (draft 0.5 section 8) ----

const GLOW_MODULE: &str = "module glow;\n\
     claim mushroom_discovered;\n\
     evidence first_mushroom from \"first authored absorption\";\n\
     caveat single_absorption consequence material;\n\
     single_absorption qualifies first_mushroom;\n\
     state learned = 0 min 0 max 1;\n\
     event absorb;\n\
     proc learn() {\n\
       reveal first_mushroom supports mushroom_discovered;\n\
       set learned = qualified(1, first_mushroom);\n\
     };\n\
     on absorb when not observed(first_mushroom) call learn();\n\
     bind ability.learned = learned == 1;\n\
     bind ability.text = \"Not discovered\";\n\
     bind ability.text = \"Mushroom absorbed\" when learned == 1;\n";

#[test]
fn a_module_may_own_state_events_rules_and_bindings() {
    let text = bundle(&[
        ("glow", GLOW_MODULE),
        (
            "main",
            "use glow;\n\
             state escaped = 0 min 0 max 1;\n\
             event escape;\n\
             on escape set escaped = 1;\n\
             bind objective.complete = escaped == 1;\n",
        ),
    ]);
    let mut session =
        caveat_runtime::reactive::ReactiveSession::from_source(&text).expect("bundle runs");

    let before = session.snapshot();
    assert_eq!(
        before.bindings["ability"]["learned"],
        BindingValue::Bool(false)
    );
    assert_eq!(
        before.bindings["ability"]["text"],
        BindingValue::Text("Not discovered".into())
    );

    // The host dispatches the module's event by its linked name.
    session.dispatch_json("glow__absorb", "{}").expect("absorb");
    let after = session.snapshot();
    assert_eq!(
        after.bindings["ability"]["learned"],
        BindingValue::Bool(true)
    );
    assert_eq!(
        after.bindings["ability"]["text"],
        BindingValue::Text("Mushroom absorbed".into()),
        "the cascade inside one module still resolves in source order"
    );

    // The program's own event and binding are untouched by the module.
    assert_eq!(
        after.bindings["objective"]["complete"],
        BindingValue::Bool(false)
    );
    session.dispatch_json("escape", "{}").expect("escape");
    assert_eq!(
        session.snapshot().bindings["objective"]["complete"],
        BindingValue::Bool(true)
    );
}

#[test]
fn a_binding_target_is_a_host_name_and_is_not_rewritten() {
    // The host reads `ability.learned`. If linking renamed the target,
    // modularising a policy would silently break its consumer.
    let source = linked(&[("glow", GLOW_MODULE), ("main", "use glow;\n")]);
    assert!(
        source.contains("bind ability.learned = glow__learned == 1;"),
        "target stays, state flattens: {source}"
    );
    assert!(
        !source.contains("glow__ability"),
        "a binding target must not be namespaced: {source}"
    );
}

#[test]
fn two_parts_may_not_write_the_same_binding_or_state_cell() {
    // Across parts the order is the linker's, which nobody authored, so a
    // second writer is refused rather than silently losing to link order.
    let message = error(&[
        ("glow", GLOW_MODULE),
        ("main", "use glow;\nbind ability.learned = 1 == 1;\n"),
    ]);
    assert!(
        message.contains("ability.learned is bound by both")
            && message.contains("module glow")
            && message.contains("the program main"),
        "{message}"
    );

    let shared = "module shared;\nstate level = 0 min 0 max 9;\nevent bump;\non bump set level = level + 1;\n";
    let message = error(&[
        ("shared", shared),
        (
            "main",
            "use shared;\nevent nudge;\non nudge set shared::level = 9;\n",
        ),
    ]);
    assert!(
        message.contains("shared__level is assigned by both"),
        "{message}"
    );
}

#[test]
fn each_part_keeps_its_own_binding_cascade() {
    // Different properties of the same host target may come from different
    // parts; only the same property is a conflict.
    let text = bundle(&[
        ("glow", GLOW_MODULE),
        (
            "main",
            "use glow;\nbind ability.name = \"Glow\";\nbind objective.text = \"Absorb one.\";\n",
        ),
    ]);
    let session =
        caveat_runtime::reactive::ReactiveSession::from_source(&text).expect("bundle runs");
    let snapshot = session.snapshot();
    assert_eq!(
        snapshot.bindings["ability"]["name"],
        BindingValue::Text("Glow".into())
    );
    assert_eq!(
        snapshot.bindings["ability"]["text"],
        BindingValue::Text("Not discovered".into())
    );
}

#[test]
fn a_program_can_route_its_own_event_to_a_module_procedure() {
    // Keeps the host-facing event name in the program while the behaviour
    // lives in the module.
    let text = bundle(&[
        ("glow", GLOW_MODULE),
        (
            "main",
            "use glow;\nevent absorb;\non absorb when not observed(glow::first_mushroom) call glow::learn();\n",
        ),
    ]);
    let mut session =
        caveat_runtime::reactive::ReactiveSession::from_source(&text).expect("bundle runs");
    session
        .dispatch_json("absorb", "{}")
        .expect("the program's own event name still works");
    assert_eq!(
        session.snapshot().bindings["ability"]["learned"],
        BindingValue::Bool(true)
    );
}

#[test]
fn the_split_slime_glow_policy_behaves_exactly_like_the_single_file() {
    // examples/modules/{glow,slime_glow}.cav is examples/slime_glow_ability.cav
    // split in two. If linking changed anything the host can see, these two
    // sessions would diverge.
    use caveat_runtime::reactive::ReactiveSession;

    let single = std::fs::read_to_string(repo("examples/slime_glow_ability.cav"))
        .expect("the single-file policy reads");
    let split =
        disk::load(&repo("examples/modules/slime_glow.cav")).expect("the split policy loads");

    let mut one = ReactiveSession::from_source(&single).expect("single-file session");
    let mut two = ReactiveSession::from_source(&split).expect("split session");
    assert_eq!(
        one.snapshot().bindings,
        two.snapshot().bindings,
        "the two layouts must start identical"
    );

    // Every host event, including the ones that must do nothing yet.
    for event in [
        "toggle_glow",
        "cave_entered",
        "glow_renderer_ready",
        "absorb_mushroom",
        "toggle_glow",
        "toggle_glow",
        "absorb_mushroom",
        "clearing_started",
        "ruin_escaped",
    ] {
        let first = one.dispatch_json(event, "{}");
        let second = two.dispatch_json(event, "{}");
        assert_eq!(
            first.is_ok(),
            second.is_ok(),
            "{event} succeeded in one layout and not the other"
        );
        assert_eq!(
            one.snapshot().bindings,
            two.snapshot().bindings,
            "bindings diverged after {event}"
        );
    }

    let end = one.snapshot();
    assert_eq!(
        end.bindings["ability"]["text"],
        BindingValue::Text("Glow on".into()),
        "the sequence should end with Glow learned and on"
    );
    assert_eq!(
        end.bindings["objective"]["text"],
        BindingValue::Text("You escaped into the garden.".into())
    );

    // Same graph size, just differently spelled names.
    let (left, right) = (one.snapshot(), two.snapshot());
    assert_eq!(left.symbols.len(), right.symbols.len(), "symbol count");
    assert_eq!(
        left.relations.len(),
        right.relations.len(),
        "relation count"
    );
    assert_ne!(
        left.source_id, right.source_id,
        "different bytes, so different identities"
    );
}

#[test]
fn the_cli_emits_a_bundle_that_runs() {
    // scripts/build-web.mjs publishes exactly this stdout as the game's .cav,
    // so what the CLI writes has to be loadable program text.
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_caveat"))
        .args(["--link", "examples/modules/slime_glow.cav"])
        .current_dir(repo("."))
        .output()
        .expect("caveat --link runs");
    assert!(
        output.status.success(),
        "caveat --link failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let emitted = String::from_utf8(output.stdout).expect("bundle is text");
    assert!(emitted.starts_with(link::BUNDLE_MARKER), "{emitted:.120}");

    let mut session =
        caveat_runtime::reactive::ReactiveSession::from_source(&emitted).expect("bundle runs");
    session
        .dispatch_json("glow_renderer_ready", "{}")
        .expect("renderer");
    session
        .dispatch_json("absorb_mushroom", "{}")
        .expect("absorb");
    assert_eq!(
        session.snapshot().bindings["ability"]["text"],
        BindingValue::Text("Glow on".into())
    );

    // It is also byte-identical to what the library produces, so the CLI is
    // not a second implementation.
    let loaded = disk::load(&repo("examples/modules/slime_glow.cav")).expect("library load");
    assert_eq!(emitted, loaded);
}

#[test]
fn a_late_failure_inside_a_module_rolls_back_the_whole_event() {
    // spec/caveat-0.5-draft.md section 4 claims a call across a module
    // boundary is one transaction. CAVEAT_ESSENCE.md: "A failed event
    // publishes none of its numeric changes, graph changes, or presentation
    // cues." A module must not be able to half-commit.
    use caveat_runtime::reactive::ReactiveSession;

    let module = "module ledger;\n\
         claim counted;\n\
         evidence tally from \"tally\";\n\
         state total = 0 min 0 max 9;\n\
         proc record(amount) {\n\
           set total = total + 1;\n\
           reveal tally supports counted;\n\
           set total = total + amount;\n\
         };\n\
         bind hud.total = total;\n";
    let program = "use ledger;\n\
         event add;\n\
         on add call ledger::record(1);\n";
    let text = bundle(&[("ledger", module), ("main", program)]);

    let mut session = ReactiveSession::from_source(&text).expect("bundle runs");
    session.dispatch_json("add", "{}").expect("first add");
    let after_one = session.snapshot();
    assert_eq!(
        after_one.bindings["hud"]["total"],
        BindingValue::Number(2.0)
    );
    assert_eq!(after_one.relations.len(), 1, "the reveal landed");

    // `total` is capped at 9. Three more adds reach 8, and the fourth would
    // pass the cap partway through the procedure — after the first `set` and
    // after the `reveal`.
    for _ in 0..3 {
        session.dispatch_json("add", "{}").expect("add");
    }
    let before = session.snapshot();
    assert_eq!(before.bindings["hud"]["total"], BindingValue::Number(8.0));

    let failed = session.dispatch_json("add", "{}");
    assert!(failed.is_err(), "exceeding the cap must fail the event");
    let after = session.snapshot();
    assert_eq!(
        after.bindings, before.bindings,
        "a module's half-finished numeric change must not be published"
    );
    assert_eq!(
        after.relations.len(),
        before.relations.len(),
        "a module's graph change must not survive a failed event"
    );
    assert_eq!(after.sequence, before.sequence, "no event was recorded");
}

// ---- the rewriter only substitutes in reference positions ----

/// Every slot where a word is *not* a reference to a symbol, with a module
/// that declares a name colliding with it. Before this was enumerated, each
/// one was found by a bug rather than by the list.
#[test]
fn a_word_that_is_not_a_reference_is_never_rewritten() {
    let cases: Vec<(&str, &str, &str)> = vec![
        (
            "a caveat's consequence",
            "module m;\nclaim thing;\ncaveat doubt consequence material;\n",
            "caveat m__doubt consequence material;",
        ),
        (
            "an entity's kind",
            "module m;\nclaim reef;\nplace harbor kind harbor;\nentity reef_one kind reef at harbor;\n",
            "entity m__reef_one kind reef at m__harbor;",
        ),
        (
            "a place's kind",
            "module m;\nclaim cove;\nplace harbor kind cove;\n",
            "place m__harbor kind cove;",
        ),
        (
            "a relation word",
            "module m;\nclaim a;\nevidence b from \"s\";\nb supports a;\n",
            "m__b supports m__a;",
        ),
        (
            "a state's bounds",
            "module m;\nfn min(a, b) = if(a < b, a, b);\nstate level = min(3, 4) min 0 max 9;\n",
            "state m__level = m__min(3, 4) min 0 max 9;",
        ),
        (
            "a binding property",
            "module m;\nstate learned = 0 min 0 max 1;\nbind ability.learned = learned == 1;\n",
            "bind ability.learned = m__learned == 1;",
        ),
    ];
    for (slot, module, expected) in cases {
        let source = linked(&[("m", module), ("main", "use m;\nbudget 1;\n")]);
        assert!(
            source.contains(expected),
            "{slot}: expected `{expected}` in:\n{source}"
        );
    }
}

#[test]
fn a_module_may_not_declare_a_word_the_grammar_uses() {
    for word in [
        "material",
        "supports",
        "consequence",
        "kind",
        "when",
        "because",
        "observed",
    ] {
        let module = format!("module m;\nclaim {word};\n");
        let message = error(&[("m", &module), ("main", "use m;\n")]);
        assert!(
            message.contains(&format!("declares {word}, which the grammar already uses")),
            "{word}: {message}"
        );
    }
}

#[test]
fn a_module_may_shadow_a_function_only_with_a_function() {
    // `fn abs` in a module is its own abs; `claim abs` would make every
    // `abs(...)` in that module resolve to a claim.
    let source = linked(&[
        (
            "m",
            "module m;\nfn abs(value) = value * value;\nfn scaled(v) = abs(v) + 1;\n",
        ),
        ("main", "use m;\nbudget 1;\n"),
    ]);
    assert!(source.contains("fn m__abs(value)"), "{source}");
    assert!(
        source.contains("fn m__scaled(v) = m__abs(v) + 1;"),
        "a module's call resolves to its own function: {source}"
    );

    let message = error(&[("m", "module m;\nclaim abs;\n"), ("main", "use m;\n")]);
    assert!(message.contains("which is already a function"), "{message}");
}

#[test]
fn the_reserved_list_does_not_drift_from_the_standard_library() {
    // The standard library is source, not a hand-kept list here. If a prelude
    // function is added and not listed, a module could declare its name as a
    // claim and silently break every call to it.
    let prelude = include_str!("../prelude.cav");
    for line in prelude.lines() {
        let Some(rest) = line.strip_prefix("fn ") else {
            continue;
        };
        let name = rest.split('(').next().expect("a function name").trim();
        assert!(
            caveat_runtime::link::is_callable(name),
            "prelude function {name} is missing from link.rs CALLABLE"
        );
    }
}
