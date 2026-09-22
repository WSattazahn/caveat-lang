use caveat_runtime::ast::{ActionStep, Statement};
use caveat_runtime::parser;

#[test]
fn quoted_prose_preserves_delimiters_comments_unicode_and_whitespace() {
    let program = parser::parse(
        r##"
        scene "Orbit; ∆ // radar #7 — 星";
        display mission "Read  carefully; // keep # both";
        evidence archive from "https://station.example/log;rev=2#entry  α";
        claim launch;
        "##,
    )
    .unwrap();
    assert_eq!(
        program.statements,
        vec![
            Statement::Scene {
                text: "Orbit; ∆ // radar #7 — 星".into(),
            },
            Statement::Display {
                symbol: "mission".into(),
                text: "Read  carefully; // keep # both".into(),
            },
            Statement::Evidence {
                name: "archive".into(),
                source: "https://station.example/log;rev=2#entry  α".into(),
            },
            Statement::Claim {
                name: "launch".into(),
            },
        ]
    );
}

#[test]
fn comments_are_whitespace_and_can_contain_unbalanced_syntax() {
    let program = parser::parse(
        r#"
        # A heading with an unmatched " and ;
        budget // the value continues on the next line; "
            9; # trailing comment
        claim// don't glue the keyword and its argument
            launch;
        // A comment at EOF needs no newline or semicolon
        "#,
    )
    .unwrap();
    assert_eq!(
        program.statements,
        vec![
            Statement::Budget { units: 9 },
            Statement::Claim {
                name: "launch".into(),
            },
        ]
    );
    assert!(parser::parse("# comments only; \"")
        .unwrap()
        .statements
        .is_empty());
}

#[test]
fn escapes_decode_for_scene_display_and_evidence() {
    let program = parser::parse(
        r#"scene "A \"signal;\"\nB\tC\\";
           display archive "line 1\r\nline 2";
           evidence archive from "captain's \"log\"\\raw";"#,
    )
    .unwrap();
    assert_eq!(
        program.statements,
        vec![
            Statement::Scene {
                text: "A \"signal;\"\nB\tC\\".into(),
            },
            Statement::Display {
                symbol: "archive".into(),
                text: "line 1\r\nline 2".into(),
            },
            Statement::Evidence {
                name: "archive".into(),
                source: "captain's \"log\"\\raw".into(),
            },
        ]
    );
}

#[test]
fn empty_and_multiline_strings_and_keyword_whitespace_work() {
    let program = parser::parse("scene\t\"\"; display\nbeacon\t\"one\ntwo\";").unwrap();
    assert_eq!(
        program.statements,
        vec![
            Statement::Scene { text: "".into() },
            Statement::Display {
                symbol: "beacon".into(),
                text: "one\ntwo".into(),
            },
        ]
    );
}

#[test]
fn action_headers_and_steps_accept_comment_boundaries() {
    let program = parser::parse(
        "action // name follows\n launch from dock to sky # plan follows\n steps\nobserve star, // then travel\n move sky;",
    )
    .unwrap();
    assert_eq!(
        program.statements,
        vec![Statement::ActionPlan {
            action: "launch".into(),
            from: "dock".into(),
            to: "sky".into(),
            requires_open: vec![],
            steps: vec![
                ActionStep::Observe {
                    symbol: "star".into(),
                },
                ActionStep::Move {
                    place: "sky".into(),
                },
            ],
        }]
    );
}

#[test]
fn legacy_final_semicolon_and_unquoted_evidence_remain_optional() {
    let program =
        parser::parse(";; evidence sensor from ship  radar; claim launch # done").unwrap();
    assert_eq!(
        program.statements,
        vec![
            Statement::Evidence {
                name: "sensor".into(),
                source: "ship radar".into(),
            },
            Statement::Claim {
                name: "launch".into(),
            },
        ]
    );
}

#[test]
fn unterminated_strings_report_the_opening_quote() {
    let error = parser::parse("# heading\n  scene \"never closed; # not a comment").unwrap_err();
    assert_eq!(error, "line 2, column 9: unterminated quoted string");

    let error = parser::parse("scene \"incomplete\\").unwrap_err();
    assert_eq!(
        error,
        "line 1, column 7: unterminated quoted string (incomplete escape)"
    );
}

#[test]
fn bad_escape_reports_unicode_character_column_not_byte_offset() {
    let error = parser::parse(r#"scene "星🛰\q";"#).unwrap_err();
    assert_eq!(error, "line 1, column 10: unknown string escape: \\q");
}

#[test]
fn malformed_statements_report_their_source_start() {
    for source in [
        "claim good;\n  budget nope;",
        "# skipped\n  claim;",
        "\n  scene unquoted;",
        "\n  scene \"one\" \"two\";",
        "\n  display signal \"ready\" trailing;",
        "\n  evidence radio from \"bridge\" claim stray;",
        "\n  claim alpha\nclaim beta;",
        "\n  connect alpha to",
    ] {
        let error = parser::parse(source).unwrap_err();
        assert!(error.starts_with("line 2, column 3:"), "{error}");
    }
}

#[test]
fn diagnostics_keep_locations_after_comments_and_crlf() {
    let error = parser::parse("# first\r\n// second\r\n\tbudget none;").unwrap_err();
    assert_eq!(error, "line 3, column 2: invalid resource amount: none");
}

#[test]
fn every_existing_example_and_game_still_parses() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    for directory in ["examples", "game"] {
        for entry in std::fs::read_dir(root.join(directory)).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().and_then(|ext| ext.to_str()) == Some("cav") {
                let source = std::fs::read_to_string(&path).unwrap();
                // Parse what a session parses: `for` blocks expanded first.
                let expanded = caveat_runtime::repeat::expand(&source)
                    .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
                parser::parse(&expanded)
                    .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
            }
        }
    }
}
