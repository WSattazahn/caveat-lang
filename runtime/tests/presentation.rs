use caveat_runtime::map::CaveatMap;
use caveat_runtime::parser;
use caveat_runtime::presentation::{parse_directive, Number};

const WORLD: &str = "
    place court kind courtyard;
    place harbor kind harbor;
    place tower kind lighthouse;
    entity receiver kind transmitter at court;
    connect court to harbor;
    start_at court;
";

fn compile(extra: &str) -> Result<CaveatMap, String> {
    CaveatMap::from_source(&format!("{WORLD}\n{extra}"))
}

#[test]
fn presentation_is_source_authored_and_serializes_numeric_coordinates() {
    let map = compile(
        "position court at -1.5 3 0;
         position harbor at 6 1 12;
         position receiver at 2 3.2 -1;
         camera court from 18 20 30 toward -1.5 4 0;
         overview from 35 30 46 toward 0 6 1;
         route court to harbor via 2 3 7 then 4 2 10;",
    )
    .unwrap();
    let json = serde_json::to_value(&map.world.presentation).unwrap();
    assert_eq!(
        json["positions"][0]["position"],
        serde_json::json!([-1.5, 3.0, 0.0])
    );
    assert_eq!(json["positions"][2]["target"], "receiver");
    assert_eq!(
        json["cameras"][0]["position"],
        serde_json::json!([18.0, 20.0, 30.0])
    );
    assert_eq!(
        json["overview"]["target"],
        serde_json::json!([0.0, 6.0, 1.0])
    );
    assert_eq!(json["routes"][0]["points"].as_array().unwrap().len(), 2);
    assert_eq!(map.world.connections.len(), 1);
}

#[test]
fn presentation_resolves_forward_references_and_preserves_old_worlds() {
    let map = CaveatMap::from_source(&format!(
        "position court at 0 3 0; camera court from 1 8 9 toward 0 3 0; {WORLD}"
    ))
    .unwrap();
    assert_eq!(map.world.presentation.positions.len(), 1);
    let old = compile("").unwrap();
    assert!(old.world.presentation.is_empty());
    assert!(serde_json::to_value(&old.world)
        .unwrap()
        .get("presentation")
        .is_none());
}

#[test]
fn changing_only_source_changes_placement_views_and_routes() {
    let first = compile("position court at 0 3 0; position harbor at 6 1 12; camera court from 18 20 30 toward 0 5 0; route court to harbor via 2 3 7;").unwrap();
    let second = compile("position court at 10 3 0; position harbor at 6 1 12; camera court from 28 20 30 toward 10 5 0; route court to harbor via 12 3 7;").unwrap();
    assert_eq!(first.world.connections, second.world.connections);
    assert_eq!(first.world.action_plans, second.world.action_plans);
    assert_ne!(
        first.world.presentation.positions,
        second.world.presentation.positions
    );
    assert_ne!(
        first.world.presentation.cameras,
        second.world.presentation.cameras
    );
    assert_ne!(
        first.world.presentation.routes,
        second.world.presentation.routes
    );
}

#[test]
fn canonical_numbers_preserve_reflexive_ast_equality() {
    assert_eq!(
        Number::parse("1e1").unwrap(),
        Number::parse("10.0").unwrap()
    );
    assert_eq!(Number::parse("-0").unwrap(), Number::parse("0").unwrap());
    let program = parser::parse("position court at -1e2 +2.5 .25;").unwrap();
    assert_eq!(program, program.clone());
    assert!(parse_directive("claim safe_crossing").is_none());
}

#[test]
fn invalid_and_nonfinite_numbers_have_source_diagnostics() {
    for value in ["NaN", "inf", "-inf", "1e999", "north"] {
        let error = parser::parse(&format!("\nposition court at {value} 1 2;")).unwrap_err();
        assert!(error.contains("line 2, column 1"), "{error}");
        assert!(error.contains("finite number"), "{error}");
    }
}

#[test]
fn malformed_presentation_directives_are_rejected() {
    for directive in [
        "position court at 1 2",
        "position court at 1 2 3 4",
        "camera court from 1 2 3 toward 4 5",
        "overview from 1 2 3 to 4 5 6",
        "route court to harbor via",
        "route court to harbor via 1 2 3 then",
        "route court to harbor via 1 2 3 4 5 6",
    ] {
        assert!(parser::parse(directive).is_err(), "{directive}");
    }
}

#[test]
fn unknown_targets_and_entity_cameras_are_rejected() {
    for (directive, expected) in [
        ("position missing at 0 0 0;", "unknown place or entity"),
        (
            "camera receiver from 1 2 3 toward 0 0 0;",
            "requires a declared place",
        ),
        (
            "route receiver to harbor via 1 2 3;",
            "requires declared places",
        ),
    ] {
        let error = compile(directive).unwrap_err();
        assert!(error.contains(expected), "{error}");
    }
}

#[test]
fn duplicates_and_degenerate_cameras_are_rejected() {
    for (directive, expected) in [
        (
            "position court at 0 3 0; position court at 1 3 0;",
            "duplicate position",
        ),
        (
            "camera court from 1 2 3 toward 0 0 0; camera court from 3 2 1 toward 0 0 0;",
            "duplicate camera",
        ),
        (
            "overview from 1 2 3 toward 0 0 0; overview from 3 2 1 toward 0 0 0;",
            "only one overview",
        ),
        ("camera court from 1 2 3 toward 1 2 3;", "must differ"),
    ] {
        let error = compile(directive).unwrap_err();
        assert!(error.contains(expected), "{error}");
    }
}

#[test]
fn routes_cannot_create_topology_or_omit_endpoints() {
    for (directive, expected) in [
        ("route court to tower via 1 2 3;", "no declared connection"),
        ("route court to court via 1 2 3;", "must be distinct"),
        ("route court to harbor via 1 2 3;", "requires a position"),
        ("position court at 0 3 0; position harbor at 6 1 12; route court to harbor via 1 2 3; route harbor to court via 4 5 6;", "duplicate route"),
        ("route court to harbor via 1 2 3 then 1 2 3;", "repeat consecutive"),
    ] {
        let error = compile(directive).unwrap_err();
        assert!(error.contains(expected), "{error}");
    }
}
