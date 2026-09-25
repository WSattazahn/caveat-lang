//! `caveat check`: when each warning appears, and when it must stay quiet.
//! See spec/caveat-check-0.1.md.
use caveat_runtime::reactive::{check_source, CheckReport, ReactiveSession};

const DECLARATIONS: &str = r#"claim frost_risk;
evidence probe from "a soil probe";
evidence drone from "a survey drone";
evidence kite from "a kite camera";
evidence thaw from "a thaw report";
caveat uncalibrated consequence material;
uncalibrated qualifies drone;
readings soil from probe limit 12;
readings aerial from drone limit 12;
readings overhead from kite limit 12;
decisions uncover limit 4;
state last_probe = 0 min -40 max 60;
state last_drone = 0 min -40 max 60;
event probe_read celsius min -40 max 60;
event drone_read celsius min -40 max 60;
event kite_read reading min -40 max 60;
event decide;
event rethink;
"#;

/// The same two rules for each instrument, written out by hand.
const BY_HAND: &str = r#"on probe_read when celsius <= 2 sample soil = celsius supports frost_risk;
on probe_read when celsius > 2 sample soil = celsius opposes frost_risk;
on drone_read when celsius <= 2 sample aerial = celsius supports frost_risk;
on drone_read when celsius > 2 sample aerial = celsius opposes frost_risk;
"#;

fn check(source: &str) -> CheckReport {
    check_source(source).unwrap_or_else(|error| panic!("check failed: {error}\n{source}"))
}

fn codes(report: &CheckReport) -> Vec<(&str, usize)> {
    report
        .diagnostics
        .iter()
        .map(|diagnostic| (diagnostic.code, diagnostic.line))
        .collect()
}

/// The line of the first line of `source` that starts with `prefix`.
fn line_of(source: &str, prefix: &str) -> usize {
    source
        .lines()
        .position(|line| line.trim_start().starts_with(prefix))
        .unwrap_or_else(|| panic!("no line starts with {prefix}"))
        + 1
}

#[test]
fn rules_repeated_with_other_streams_are_reported_where_the_repeat_starts() {
    let source = format!("{DECLARATIONS}{BY_HAND}");
    let report = check(&source);
    assert_eq!(
        codes(&report),
        [("C001", line_of(&source, "on drone_read"))]
    );
    let warning = &report.diagnostics[0];
    assert_eq!(
        (warning.name, warning.severity),
        ("repeated-rules", "warning")
    );
    assert_eq!(warning.column, 1);
    assert_eq!(
        warning.message,
        format!(
            "the 2 rules on `drone_read` repeat the rules on `probe_read` (line {}), with `aerial` for `soil`",
            line_of(&source, "on probe_read")
        )
    );
    assert!(warning.suggestion.contains("candidates for one `proc`"));
    assert_eq!(warning.related[0].line, line_of(&source, "on probe_read"));
    assert!(report.suppressed.is_empty());
}

#[test]
fn numbers_and_event_parameters_may_differ_too() {
    let source = format!(
        "{DECLARATIONS}{BY_HAND}on kite_read when reading <= 3 sample overhead = reading supports frost_risk;\non kite_read when reading > 3 sample overhead = reading opposes frost_risk;\n"
    );
    let report = check(&source);
    // Each repeat points at the first block it repeats.
    assert_eq!(
        codes(&report),
        [
            ("C001", line_of(&source, "on drone_read")),
            ("C001", line_of(&source, "on kite_read"))
        ]
    );
    assert!(report.diagnostics[1]
        .message
        .ends_with("with `reading` for `celsius`, `overhead` for `soil`, other numbers"));
}

#[test]
fn nothing_is_reported_where_one_procedure_could_not_take_the_difference() {
    for rules in [
        // A state is not a procedure parameter.
        "on probe_read set last_probe = celsius;\non probe_read sample soil = celsius supports frost_risk;\non drone_read set last_drone = celsius;\non drone_read sample aerial = celsius supports frost_risk;\n",
        // The relation differs.
        "on probe_read sample soil = celsius supports frost_risk;\non probe_read set last_probe = 1;\non drone_read sample aerial = celsius opposes frost_risk;\non drone_read set last_probe = 1;\n",
        // A reading stream in one place, a decision series in the other.
        "on probe_read when history_count(soil) > 1 set last_probe = 1;\non probe_read when history_count(soil) > 2 set last_probe = 2;\non drone_read when history_count(uncover) > 1 set last_probe = 1;\non drone_read when history_count(uncover) > 2 set last_probe = 2;\n",
        // One rule each: a call is no shorter.
        "on probe_read sample soil = celsius supports frost_risk;\non drone_read sample aerial = celsius supports frost_risk;\n",
        // A different number of rules.
        "on probe_read sample soil = celsius supports frost_risk;\non probe_read set last_probe = 1;\non drone_read sample aerial = celsius supports frost_risk;\n",
        // The same name must stand for the same name throughout.
        "on probe_read sample soil = celsius supports frost_risk;\non probe_read sample soil = celsius opposes frost_risk;\non drone_read sample aerial = celsius supports frost_risk;\non drone_read sample overhead = celsius opposes frost_risk;\n",
        // Two names may not become one.
        "on probe_read sample soil = celsius supports frost_risk;\non probe_read sample aerial = celsius opposes frost_risk;\non drone_read sample overhead = celsius supports frost_risk;\non drone_read sample overhead = celsius opposes frost_risk;\n",
        // Different messages.
        "on probe_read when celsius > 60 reject \"probe\";\non probe_read sample soil = celsius supports frost_risk;\non drone_read when celsius > 60 reject \"drone\";\non drone_read sample aerial = celsius supports frost_risk;\n",
        // Already one procedure.
        "proc record(s readings, celsius) { when celsius <= 2 sample s = celsius supports frost_risk; when celsius > 2 sample s = celsius opposes frost_risk; };\non probe_read call record(soil, celsius);\non drone_read call record(aerial, celsius);\n",
    ] {
        let report = check(&format!("{DECLARATIONS}{rules}"));
        assert_eq!(codes(&report), [], "{rules}");
    }
}

#[test]
fn a_for_block_is_already_shared() {
    let source = r#"
place field kind field;
entity north kind plot at field;
entity south kind plot at field;
claim frost_risk;
for plot as $p {
    evidence $p_probe from "a probe";
    readings $p_soil from $p_probe limit 4;
    event $p_read celsius min -40 max 60;
    on $p_read when celsius <= 2 sample $p_soil = celsius supports frost_risk;
    on $p_read when celsius > 2 sample $p_soil = celsius opposes frost_risk;
};
"#;
    assert_eq!(codes(&check(source)), []);
}

const DECIDE: &str =
    "on decide when not committed(uncover) commit uncover because enough using latest(soil);\n";

#[test]
fn a_decision_on_readings_that_nothing_reopens_is_reported_at_its_declaration() {
    let source = format!("{DECLARATIONS}{DECIDE}");
    let report = check(&source);
    assert_eq!(
        codes(&report),
        [("C002", line_of(&source, "decisions uncover"))]
    );
    let warning = &report.diagnostics[0];
    assert_eq!(warning.name, "no-reopening-path");
    assert!(
        warning.message.starts_with(
            "`uncover` is decided on readings from `soil`, and no reopening path was detected"
        ),
        "{}",
        warning.message
    );
    assert!(!warning.message.contains("stale"));
    assert!(warning
        .suggestion
        .starts_with("Confirm that keeping `uncover` fixed is intended."));
    assert!(warning
        .suggestion
        .contains("`decisions uncover limit N reopened by soil`"));
}

#[test]
fn a_guard_on_readings_counts_and_so_does_a_procedure() {
    for rules in [
        "on decide when latest(soil) < 3 and not committed(uncover) commit uncover because enough;\n",
        "on decide when has_sample(aerial) and not committed(uncover) commit uncover because enough;\n",
        "proc decide_on(s readings, d decisions) { when not committed(d) commit d because enough using latest(s); };\non decide call decide_on(soil, uncover);\n",
    ] {
        let source = format!("{DECLARATIONS}{rules}");
        assert_eq!(
            codes(&check(&source)),
            [("C002", line_of(&source, "decisions uncover"))],
            "{rules}"
        );
    }
}

#[test]
fn every_reopening_path_keeps_it_quiet() {
    for reopening in [
        // A rule.
        "on rethink when not observed(thaw) reveal thaw opposes frost_risk;\non rethink when committed(uncover) and not reopened(uncover) reopen uncover because thaw;\n",
        // A procedure taking the series as a parameter.
        "proc reconsider(d decisions, s readings) { when committed(d) and not reopened(d) reopen d because latest(s); };\non probe_read sample soil = celsius supports frost_risk;\non probe_read call reconsider(uncover, soil);\n",
    ] {
        let source = format!("{DECLARATIONS}{DECIDE}{reopening}");
        assert_eq!(codes(&check(&source)), [], "{reopening}");
    }
    // A declared trigger.
    let source = format!(
        "{}{DECIDE}",
        DECLARATIONS.replace(
            "decisions uncover limit 4;",
            "decisions uncover limit 4 reopened by soil;"
        )
    );
    assert_eq!(codes(&check(&source)), []);
}

#[test]
fn a_decision_not_made_on_readings_is_not_reported() {
    for rules in [
        "on decide when not committed(uncover) commit uncover because enough using last_probe;\n",
        "on decide when not committed(uncover) commit uncover because enough;\n",
        // Declared but never made.
        "",
    ] {
        let report = check(&format!("{DECLARATIONS}{rules}"));
        assert_eq!(codes(&report), [], "{rules}");
    }
}

#[test]
fn a_decision_declared_in_a_for_block_is_reported_at_the_template() {
    let source = r#"place field kind field;
entity north kind plot at field;
entity south kind plot at field;
claim frost_risk;
event decide;
for plot as $p {
    evidence $p_probe from "a probe";
    readings $p_soil from $p_probe limit 4;
    decisions $p_cover limit 2;
    on decide when has_sample($p_soil) and not committed($p_cover) commit $p_cover because enough using latest($p_soil);
};
"#;
    let report = check(source);
    let line = line_of(source, "decisions $p_cover");
    assert_eq!(codes(&report), [("C002", line), ("C002", line)]);
    assert_eq!(report.diagnostics[0].column, 5);
    assert!(report.diagnostics[0].message.starts_with("`north_cover`"));
    assert!(report.diagnostics[1].message.starts_with("`south_cover`"));

    // Reopening inside the block keeps it quiet.
    let reopened = source.replace(
        "};\n",
        "    on decide when committed($p_cover) reopen $p_cover because latest($p_soil);\n};\n",
    );
    assert_eq!(codes(&check(&reopened)), []);
}

#[test]
fn an_allow_comment_on_the_line_above_moves_a_warning_to_suppressed() {
    let allowed = format!(
        "{}{}",
        DECLARATIONS.replace(
            "decisions uncover limit 4;",
            "# One decision per season, on purpose.\n# caveat check: allow no-reopening-path\ndecisions uncover limit 4;"
        ),
        BY_HAND.replace(
            "on drone_read when celsius <= 2",
            "// caveat check: allow C001\non drone_read when celsius <= 2"
        )
    ) + DECIDE;
    let report = check(&allowed);
    assert_eq!(codes(&report), []);
    assert_eq!(
        report
            .suppressed
            .iter()
            .map(|diagnostic| diagnostic.code)
            .collect::<Vec<_>>(),
        ["C002", "C001"]
    );

    // Only the line directly above, and only the check it names.
    for (comment, before) in [
        ("# caveat check: allow C001\n", "decisions uncover"),
        (
            "# caveat check: allow no-reopening-path\n\n",
            "decisions uncover",
        ),
        ("# caveat check: allow no-reopening\n", "decisions uncover"),
        ("# allow no-reopening-path\n", "decisions uncover"),
    ] {
        let source =
            format!("{DECLARATIONS}{DECIDE}").replacen(before, &format!("{comment}{before}"), 1);
        let report = check(&source);
        assert_eq!(report.diagnostics.len(), 1, "{comment:?}");
        assert!(report.suppressed.is_empty(), "{comment:?}");
    }
}

#[test]
fn allow_must_be_a_whole_word() {
    for comment in [
        "# caveat check: allowance C002",
        "# caveat check: allowC002",
        "// caveat check: allowed C002",
        "# caveat check: allow,C002",
    ] {
        let source = format!("{DECLARATIONS}{DECIDE}").replacen(
            "decisions uncover",
            &format!("{comment}\ndecisions uncover"),
            1,
        );
        let report = check(&source);
        assert_eq!(
            codes(&report),
            [("C002", line_of(&source, "decisions uncover"))],
            "{comment}"
        );
        assert!(report.suppressed.is_empty(), "{comment}");
    }
    // Names after the word, separated by commas or spaces, still work.
    for comment in [
        "# caveat check: allow C002",
        "# caveat check: allow C001, no-reopening-path",
        "//caveat check:   allow\tC002",
    ] {
        let source = format!("{DECLARATIONS}{DECIDE}").replacen(
            "decisions uncover",
            &format!("{comment}\ndecisions uncover"),
            1,
        );
        let report = check(&source);
        assert_eq!(codes(&report), [], "{comment}");
        assert_eq!(report.suppressed.len(), 1, "{comment}");
    }
}

#[test]
fn a_last_statement_without_its_semicolon_is_checked() {
    let source = format!("{DECLARATIONS}{}", DECIDE.trim_end().trim_end_matches(';'));
    assert_eq!(
        codes(&check(&source)),
        [("C002", line_of(&source, "decisions uncover"))]
    );
}

#[test]
fn a_program_that_does_not_load_is_an_error_not_a_report() {
    let broken = format!("{DECLARATIONS}on decide sample nowhere = 1 supports frost_risk;");
    let expected = ReactiveSession::from_source(&broken).err().unwrap();
    assert_eq!(check_source(&broken).unwrap_err(), expected);
    assert!(check_source(&format!(
        "{}\n{DECLARATIONS}",
        caveat_runtime::link::BUNDLE_MARKER
    ))
    .unwrap_err()
    .contains("not a bundle"));
}

#[test]
fn the_shipped_examples_check_clean() {
    for file in [
        "../examples/thermostat_history.cav",
        "../kit/templates/umbrella.cav",
        "../experiments/agent-ledger/ledger.cav",
        "../experiments/agent-ledger/ledger-identifiers.cav",
    ] {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(file);
        let source = std::fs::read_to_string(&path).unwrap();
        let report = check(&source);
        assert_eq!(codes(&report), [], "{file}: {:#?}", report.diagnostics);
    }
}
