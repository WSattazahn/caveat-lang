//! Repetition 0.1: `for KIND as $NAME { ... };`

use caveat_runtime::reactive::{BindingValue, ReactiveSession};
use caveat_runtime::{link, repeat};

fn statements(source: &str) -> Vec<String> {
    source
        .split(';')
        .map(|statement| statement.split_whitespace().collect::<Vec<_>>().join(" "))
        .filter(|statement| !statement.is_empty())
        .collect()
}

const ENTITIES: &str = "entity reef_one kind reef at harbor;\n\
     entity reef_two kind reef at harbor;\n\
     entity reef_three kind reef at harbor;\n";

#[test]
fn a_block_expands_to_exactly_what_a_hand_unrolled_source_says() {
    // The shapes are the ones game/light_the_way.cav actually repeats.
    let block = format!(
        "{ENTITIES}\
         for reef as $r {{\n\
           evidence $r_reading from \"$r\";\n\
           state $r_distance = 100;\n\
           cue $r_ring ring $r \"#ffe3a0\" 0.75;\n\
           on tick when contact == $index set hit_x = $r.x;\n\
           bind $r.reveal.opacity = 0;\n\
           bind $r.reveal.opacity = 0.25 when observed($r_reading);\n\
         }};\n"
    );
    let mut hand = String::from(ENTITIES);
    for (index, reef) in ["reef_one", "reef_two", "reef_three"].iter().enumerate() {
        hand.push_str(&format!(
            "evidence {reef}_reading from \"{reef}\";\n\
             state {reef}_distance = 100;\n\
             cue {reef}_ring ring {reef} \"#ffe3a0\" 0.75;\n\
             on tick when contact == {} set hit_x = {reef}.x;\n\
             bind {reef}.reveal.opacity = 0;\n\
             bind {reef}.reveal.opacity = 0.25 when observed({reef}_reading);\n",
            index + 1
        ));
    }

    let expanded = repeat::expand(&block).expect("block expands");
    assert_eq!(
        statements(&expanded),
        statements(&hand),
        "expansion must produce the statements a hand-unrolled source has"
    );
}

#[test]
fn expansion_is_member_major() {
    // Order is meaning: `on` rules fire in source order and the last matching
    // `bind` wins, so the whole body is emitted per member.
    let expanded = repeat::expand(&format!(
        "{ENTITIES}for reef as $r {{ claim $r_seen; claim $r_charted; }};"
    ))
    .expect("expands");
    assert_eq!(
        statements(&expanded)[3..],
        statements(
            "claim reef_one_seen; claim reef_one_charted;\
             claim reef_two_seen; claim reef_two_charted;\
             claim reef_three_seen; claim reef_three_charted;"
        )
    );
}

#[test]
fn a_source_without_a_block_is_returned_unchanged() {
    let source = "claim mist; entity reef_one kind reef at harbor; budget 2;";
    assert_eq!(repeat::expand(source).expect("no-op"), source);
}

#[test]
fn every_existing_game_expands_to_itself() {
    // Repetition must be invisible to sources that do not use it, including
    // the policy whose bytes Vessel pins.
    for name in [
        "../game/light_the_way.cav",
        "../game/the_last_beacon.cav",
        "../game/moon_garden.cav",
        "../game/missing_dumpling.cav",
        "../examples/slime_glow_ability.cav",
    ] {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(name);
        let source = std::fs::read_to_string(&path).expect("game source reads");
        assert_eq!(
            repeat::expand(&source).expect("expands"),
            source,
            "{name} must expand to itself byte for byte"
        );
    }
}

#[test]
fn a_block_is_rejected_rather_than_guessed_at() {
    for (source, expected) in [
        (
            format!("{ENTITIES}for buoy as $b {{ claim $b_seen; }};"),
            "no entity is declared `kind buoy`",
        ),
        (
            format!("{ENTITIES}for reef as $r {{ claim $typo_seen; }};"),
            "$typo_seen is not bound",
        ),
        (
            format!("{ENTITIES}for reef as r {{ claim r_seen; }};"),
            "r is not a binding; write $name",
        ),
        (
            format!("{ENTITIES}for reef as $index {{ claim $index; }};"),
            "$index is provided by the block",
        ),
        (
            format!("{ENTITIES}for reef as $r {{ for reef as $q {{ claim $q; }}; }};"),
            "for blocks do not nest",
        ),
        (
            format!("{ENTITIES}for reef {{ claim x; }};"),
            "expected `for KIND as $NAME",
        ),
    ] {
        let message = repeat::expand(&source).expect_err("must be rejected");
        assert!(
            message.contains(expected),
            "expected {expected}, got: {message}"
        );
    }
}

#[test]
fn the_expanded_program_and_the_hand_written_one_run_identically() {
    // The point of a compile-time expansion is that the runtime cannot tell.
    let shared = "scene \"Chart the reefs.\";\n\
         claim clear_course;\n\
         state charted = 0 min 0 max 9;\n\
         event sweep;\n\
         bind hud.charted = charted;\n";
    let entities = "place harbor kind harbor;\n\
         entity reef_one kind reef at harbor;\n\
         entity reef_two kind reef at harbor;\n";
    let body = |reef: &str, index: usize| {
        format!(
            "evidence {reef}_reading from \"{reef}\";\n\
             on sweep when charted == {} reveal {reef}_reading opposes clear_course;\n\
             on sweep when charted == {} set charted = charted + 1;\n",
            index - 1,
            index - 1
        )
    };

    let hand = format!(
        "{shared}{entities}{}{}",
        body("reef_one", 1),
        body("reef_two", 2)
    );
    let looped = format!(
        "{shared}{entities}\
         for reef as $r {{\n\
           evidence $r_reading from \"$r\";\n\
           on sweep when charted == $index - 1 reveal $r_reading opposes clear_course;\n\
           on sweep when charted == $index - 1 set charted = charted + 1;\n\
         }};\n"
    );

    let mut one = ReactiveSession::from_source(&hand).expect("hand-written runs");
    let mut two = ReactiveSession::from_source(&looped).expect("looped runs");
    assert_eq!(one.snapshot().bindings, two.snapshot().bindings);
    for _ in 0..3 {
        let left = one.dispatch_json("sweep", "{}");
        let right = two.dispatch_json("sweep", "{}");
        assert_eq!(left.is_ok(), right.is_ok());
        assert_eq!(
            one.snapshot().bindings,
            two.snapshot().bindings,
            "the two layouts diverged"
        );
    }
    assert_eq!(
        one.snapshot().bindings["hud"]["charted"],
        BindingValue::Number(2.0),
        "both reefs should have been charted"
    );
    assert_eq!(
        one.snapshot().relations.len(),
        two.snapshot().relations.len()
    );
}

#[test]
fn a_block_reproduces_the_real_reef_rules_in_light_the_way() {
    // Checked against the shipped game rather than a fixture: every statement
    // the block generates must already appear in the hand-unrolled source.
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../game/light_the_way.cav");
    let game = std::fs::read_to_string(&path).expect("light_the_way reads");
    let entities: String = game
        .lines()
        .filter(|line| line.starts_with("entity reef_"))
        .map(|line| format!("{line}\n"))
        .collect();
    assert_eq!(entities.lines().count(), 6, "six reefs are declared");

    let block = format!(
        "{entities}\
         for reef as $r {{\n\
           evidence $r_reading from \"$r\";\n\
           on tick when phase == 1 and paused == 0 and light_on == 1 and distance(aim_x, aim_z, $r.x, $r.z) <= beam_radius and not observed($r_reading) reveal $r_reading opposes clear_course;\n\
           on tick when phase == 1 and paused == 0 set $r_distance = distance(boat_x, boat_z, $r.x, $r.z);\n\
           on tick when phase == 1 and paused == 0 and contact == $index set hit_x = $r.x;\n\
           bind $r.reveal.opacity = 0;\n\
         }};\n"
    );
    let expanded = repeat::expand(&block).expect("block expands");
    let shipped = statements(&game);
    let generated = statements(&expanded);
    assert_eq!(
        generated.len(),
        6 + 6 * 5,
        "six members times five statements"
    );

    let mut checked = 0;
    for statement in generated.iter().filter(|s| !s.starts_with("entity ")) {
        assert!(
            shipped.contains(statement),
            "generated a statement the shipped game does not have:\n  {statement}"
        );
        checked += 1;
    }
    assert_eq!(
        checked, 30,
        "every generated rule was matched against the game"
    );
}

#[test]
fn a_block_survives_linking_into_a_module() {
    // Expansion runs per part before names are rewritten, so the generated
    // names are scoped to the module that wrote the block.
    let module = format!(
        "module reefs;\n{ENTITIES}for reef as $r {{ claim $r_seen; evidence $r_reading from \"$r\"; $r_reading supports $r_seen; }};\n"
    );
    let text = link::bundle(&[
        link::BundlePart {
            name: "reefs".into(),
            source: module,
        },
        link::BundlePart {
            name: "main".into(),
            source: "use reefs;\nbudget 1;\nclaim here;\n".into(),
        },
    ]);
    let linked = link::link(&text).expect("bundle links");
    assert!(
        linked.contains("claim reefs__reef_one_seen;"),
        "generated names belong to the module that wrote the block: {linked}"
    );
    assert!(
        linked.contains("reefs__reef_two_reading supports reefs__reef_two_seen;"),
        "{linked}"
    );
    let evaluation =
        caveat_runtime::eval::evaluate(&caveat_runtime::parser::parse(&linked).expect("parses"))
            .expect("evaluates");
    assert!(evaluation.symbols.contains_key("reefs__reef_three_seen"));
}
