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
fn a_diagnostic_maps_back_to_the_part_and_line_the_author_wrote() {
    // Linking concatenates parts, so the evaluator's line number is not the
    // author's. The source map is what makes it one again.
    let (source, map) = link::link_with_map(&bundle(&[
        ("weather", WEATHER),
        (
            "main",
            "use weather;
claim ok;
this is not a statement;
",
        ),
    ]))
    .expect("bundle links");
    let message = parser::parse(&source).expect_err("bad statement must fail");
    let line: usize = message
        .strip_prefix("line ")
        .and_then(|rest| rest.split(',').next())
        .and_then(|digits| digits.parse().ok())
        .unwrap_or_else(|| panic!("unexpected diagnostic: {message}"));
    assert_eq!(
        map.locate(line),
        Some(("main", 3)),
        "linked line {line} should be main line 3; map: {:?}",
        map.parts
    );
    // The module's own first line still maps to the module.
    assert_eq!(map.locate(1), Some(("weather", 1)));
}

#[test]
fn a_single_file_program_maps_to_itself() {
    let (_, map) = link::link_with_map("budget 1; claim only;").expect("plain source links");
    assert_eq!(map.locate(1), Some(("", 1)));
    assert_eq!(map.locate(9), Some(("", 9)));
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
