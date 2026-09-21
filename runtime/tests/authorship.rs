//! Authorship 0.1: which part of a program asserted a thing.

use caveat_runtime::link::{self, BundlePart};
use caveat_runtime::reactive::ReactiveSession;
use caveat_runtime::{eval, parser, Relation};

fn part(name: &str, source: &str) -> BundlePart {
    BundlePart {
        name: name.into(),
        source: source.into(),
    }
}

fn evaluate(source: &str) -> eval::Evaluation {
    eval::evaluate(&parser::parse(source).expect("parses")).expect("evaluates")
}

const WEATHER: &str = "module weather;\n\
     claim low_visibility;\n\
     evidence fog_bank from \"lookout report, 06:40\";\n\
     caveat fog_refraction consequence material;\n\
     fog_bank supports low_visibility;\n";

const PROGRAM: &str = "use weather;\n\
     budget 4;\n\
     claim hold_the_channel;\n\
     evidence helm_check from \"helm, before the turn\";\n\
     helm_check supports hold_the_channel;\n";

fn linked() -> String {
    link::link(&link::bundle(&[
        part("weather", WEATHER),
        part("crossing", PROGRAM),
    ]))
    .expect("bundle links")
}

#[test]
fn a_symbol_knows_which_part_declared_it_without_reading_its_name() {
    let evaluation = evaluate(&linked());
    let origin = |symbol: &str| {
        evaluation
            .graph
            .origin(evaluation.symbols[symbol])
            .map(str::to_string)
    };
    assert_eq!(origin("weather__fog_bank").as_deref(), Some("weather"));
    assert_eq!(
        origin("weather__low_visibility").as_deref(),
        Some("weather")
    );
    assert_eq!(origin("helm_check").as_deref(), Some("crossing"));
    assert_eq!(origin("hold_the_channel").as_deref(), Some("crossing"));
}

#[test]
fn world_provenance_and_program_provenance_are_different_questions() {
    // `from` says where the claim came from in the world. `origin` says where
    // the assertion came from in the program. Both are recorded, separately.
    let text = link::bundle(&[
        part("weather", WEATHER),
        part("crossing", &format!("{PROGRAM}event look;\n")),
    ]);
    let session = ReactiveSession::from_source(&text).expect("runs");
    let snapshot = session.snapshot();
    let symbol = snapshot
        .symbols
        .iter()
        .find(|symbol| symbol.name == "weather__fog_bank")
        .expect("the imported evidence is in the snapshot");
    assert_eq!(symbol.source.as_deref(), Some("lookout report, 06:40"));
    assert_eq!(symbol.origin.as_deref(), Some("weather"));

    // The importing program declares its own evidence about a different thing.
    let mine = snapshot
        .symbols
        .iter()
        .find(|symbol| symbol.name == "helm_check")
        .expect("the program's evidence is in the snapshot");
    assert_eq!(mine.source.as_deref(), Some("helm, before the turn"));
    assert_eq!(mine.origin.as_deref(), Some("crossing"));
}

#[test]
fn a_declared_edge_records_the_part_that_asserted_it() {
    let evaluation = evaluate(&linked());
    let edge = |from: &str, to: &str| {
        evaluation
            .graph
            .edges
            .iter()
            .find(|edge| edge.from == evaluation.symbols[from] && edge.to == evaluation.symbols[to])
            .unwrap_or_else(|| panic!("no edge {from} -> {to}"))
    };
    assert_eq!(
        edge("weather__fog_bank", "weather__low_visibility")
            .origin
            .as_deref(),
        Some("weather")
    );
    assert_eq!(
        edge("helm_check", "hold_the_channel").origin.as_deref(),
        Some("crossing")
    );
}

#[test]
fn a_shared_node_is_attributed_to_the_part_that_declared_it() {
    // In a diamond, two modules assert edges into one claim a third declared.
    // The claim belongs to its declarer; each edge belongs to its asserter.
    let source = link::link(&link::bundle(&[
        part("base", "module base;\nclaim shared;\n"),
        part(
            "left",
            "module left;\nuse base;\nevidence port from \"port watch\";\nport supports base::shared;\n",
        ),
        part(
            "right",
            "module right;\nuse base;\nevidence starboard from \"starboard watch\";\nstarboard supports base::shared;\n",
        ),
        part("main", "use left;\nuse right;\nbudget 1;\n"),
    ]))
    .expect("bundle links");
    let evaluation = evaluate(&source);
    let shared = evaluation.symbols["base__shared"];
    assert_eq!(evaluation.graph.origin(shared), Some("base"));

    let mut asserters: Vec<&str> = evaluation
        .graph
        .edges
        .iter()
        .filter(|edge| edge.to == shared && edge.relation == Relation::Supports)
        .filter_map(|edge| edge.origin.as_deref())
        .collect();
    asserters.sort();
    assert_eq!(
        asserters,
        vec!["left", "right"],
        "each supporting edge belongs to the module that asserted it"
    );
}

#[test]
fn a_single_file_program_is_unattributed_rather_than_defaulted() {
    // Nothing declared an origin, so nothing is claimed. An invented default
    // would be a worse answer than none.
    let evaluation = evaluate("claim mist; evidence haze from \"deck\"; haze supports mist;");
    assert_eq!(evaluation.graph.origin(evaluation.symbols["mist"]), None);
    assert!(evaluation.graph.origins.is_empty());
    assert!(evaluation
        .graph
        .edges
        .iter()
        .all(|edge| edge.origin.is_none()));
}

#[test]
fn the_record_is_in_the_linked_text_not_a_side_channel() {
    let source = linked();
    assert!(source.contains("origin weather;"), "{source:.200}");
    assert!(source.contains("origin crossing;"), "{source:.200}");
    // And it is an ordinary statement: a hand-written program can carry one.
    let evaluation = evaluate("origin survey; claim mist;");
    assert_eq!(
        evaluation.graph.origin(evaluation.symbols["mist"]),
        Some("survey")
    );
}

#[test]
fn an_effect_that_reveals_at_runtime_is_not_attributed_to_the_last_part_read() {
    // Declarations are over by then. Inheriting whichever part happened to be
    // read last would be a confident wrong answer.
    let module = "module glow;\n\
         claim discovered;\n\
         evidence first_mushroom from \"first absorption\";\n\
         state learned = 0 min 0 max 1;\n\
         proc learn() {\n\
           reveal first_mushroom supports discovered;\n\
           set learned = 1;\n\
         };\n";
    let program = "use glow;\nevent absorb;\non absorb call glow::learn();\n";
    let text = link::bundle(&[part("glow", module), part("main", program)]);

    let mut session = ReactiveSession::from_source(&text).expect("runs");
    let before = session.snapshot().relations.len();
    session.dispatch_json("absorb", "{}").expect("absorb");
    let after = session.snapshot();
    assert_eq!(after.relations.len(), before + 1, "the reveal landed");

    // The nodes it connects are still attributed; the edge itself is not.
    let symbol = |name: &str| {
        after
            .symbols
            .iter()
            .find(|symbol| symbol.name == name)
            .unwrap_or_else(|| panic!("{name} is in the snapshot"))
    };
    assert_eq!(
        symbol("glow__first_mushroom").origin.as_deref(),
        Some("glow")
    );
    assert_eq!(symbol("glow__discovered").origin.as_deref(), Some("glow"));
}

#[test]
fn every_existing_game_still_records_nothing() {
    // Authorship must be invisible to a program that never declares one.
    for name in [
        "../game/light_the_way.cav",
        "../game/the_last_beacon.cav",
        "../examples/slime_glow_ability.cav",
    ] {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(name);
        let source = std::fs::read_to_string(&path).expect("reads");
        assert_eq!(
            link::link(&source).expect("links"),
            source,
            "{name} must link to itself"
        );
    }
}
