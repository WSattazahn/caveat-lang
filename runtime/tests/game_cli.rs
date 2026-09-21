use std::{
    fs,
    process::{Command, Output, Stdio},
    sync::atomic::{AtomicUsize, Ordering},
};

const GAME: &str = r#"
scene "A dock with an uncertain route.";
budget 1;
claim safe;
evidence reading from "lens recorder";
caveat lens consequence material;
caveat noise consequence low;
place dock kind corridor;
place exit kind chamber;
entity gate kind door at dock;
entity console kind console at dock;
connect dock to exit via gate;
start_at dock;
action lens from dock to dock steps inspect console, observe reading;
action noise from dock to dock steps inspect console;
action launch from dock to exit steps open gate, through gate;
action wait from dock to dock steps stay;
require launch observed reading;
require launch examined lens;
resolve launch as informed when observed reading;
resolve launch as uncertain otherwise;
resolve wait as waited otherwise;
display lens "Check the lens";
display launch "Open the gate";
display informed "The instrument informed this departure.";
display waited "The gate stayed closed.";
investigate scan options lens, noise;
inspect scan lens cost 1;
reveal lens then reading supports safe;
choice departure options launch, wait retaining lens, noise;
select departure launch;
"#;

const LEGACY: &str = r#"
budget 1;
caveat risk consequence material;
investigate scan options risk;
inspect scan risk cost 1;
choice departure options go retaining risk;
select departure go;
"#;

fn run(source: &str, arguments: &[&str], input: &str) -> Output {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let file = std::env::temp_dir().join(format!(
        "caveat-game-cli-{}-{}.cav",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let input_file = file.with_extension("stdin");
    fs::write(&file, source).unwrap();
    // Some modes finish without reading stdin. Prepare input before launching
    // so an early exit cannot race a parent-side pipe write (BrokenPipe).
    fs::write(&input_file, input).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_caveat"))
        .args(arguments)
        .arg(&file)
        .stdin(Stdio::from(fs::File::open(&input_file).unwrap()))
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .unwrap();
    fs::remove_file(file).unwrap();
    fs::remove_file(input_file).unwrap();
    output
}

fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).unwrap()
}

#[test]
fn native_game_executes_numbered_actions_and_reports_reached_knowledge_and_outcome() {
    let output = run(GAME, &["--game"], "99\nlaunch\n1\n1\n");
    assert!(output.status.success(), "{:?}", output);
    let text = stdout(&output);
    assert!(text.contains("1. Check the lens [lens]"));
    assert_eq!(text.matches("Selection is not available:").count(), 2);
    assert!(text.contains("Evidence: reading [reading] supports safe [safe]"));
    assert!(text.contains("Examined: Check the lens [lens]"));
    assert!(text.contains("Location: exit [exit]"));
    assert!(text.contains("Outcome: The instrument informed this departure. [informed]"));
    assert!(text.contains("Basis at decision: observed reading [reading]"));
    assert!(text.contains("Retained uncertainty: Check the lens [lens], noise [noise]"));
    assert!(text.contains("Attention remaining: 0 (spent 1 / 1)"));
    assert!(text.contains("Game complete after 2 actions."));
    assert_eq!(text.matches("--- Turn ").count(), 3);
    assert!(!text.contains("[uncertain]"));
    assert!(output.stderr.is_empty());
}

#[test]
fn native_game_shows_missing_requirements_and_rejects_blocked_action_without_effects() {
    let output = run(GAME, &["--game"], "noise\nlaunch\nwait\n");
    assert!(output.status.success(), "{:?}", output);
    let text = stdout(&output);
    assert!(text.contains("Unavailable: Open the gate [launch] — requires observed reading [reading]; examined Check the lens [lens]"));
    assert!(text.contains("Selection is not available: \"launch\""));
    assert!(text.contains("Outcome: The gate stayed closed. [waited]"));
    assert!(!text.contains("Location: exit [exit]"));
    assert!(!text.contains("Evidence:"));
    assert!(!text.contains("[informed]"));
    assert!(text.contains("Game complete after 2 actions."));
}

#[test]
fn native_game_eof_and_quit_do_not_choose_scripted_defaults() {
    for (input, expected_turn) in [("", 0), ("noise\nquit\n", 1)] {
        let output = run(GAME, &["--game"], input);
        assert!(output.status.success(), "{:?}", output);
        let text = stdout(&output);
        assert!(text.contains(&format!("Stopped after {expected_turn} actions.")));
        assert!(!text.contains("Outcome:"));
        assert!(!text.contains("Game complete"));
        assert!(!text.contains("Location: exit [exit]"));
    }
}

#[test]
fn native_game_reports_fully_blocked_and_invalid_source_as_errors() {
    let blocked = GAME.replace("options launch, wait", "options launch");
    let output = run(&blocked, &["--game"], "noise\n");
    assert_eq!(output.status.code(), Some(1));
    assert!(stdout(&output).contains("requires observed reading [reading]"));
    assert!(String::from_utf8_lossy(&output.stderr).contains("game error: blocked at departure"));
    assert!(!stdout(&output).contains("Game complete"));

    let output = run("not_a_statement;", &["--game"], "");
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("game error:"));
}

#[test]
fn legacy_evaluation_and_interactive_modes_keep_their_existing_behavior() {
    for arguments in [vec![], vec!["-i"], vec!["--interactive"]] {
        let output = run(LEGACY, &arguments, "risk\ngo\n");
        assert!(output.status.success(), "{:?}", output);
        let text = stdout(&output);
        assert!(text.contains("--- Decision record ---"));
        assert!(text.contains("go: committed"));
        assert!(text.contains("Attention spent: 1 / 1"));
        assert!(!text.contains("CAVEAT game"));
    }
    let output = Command::new(env!("CARGO_BIN_EXE_caveat"))
        .arg("--game")
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("usage:"));
    let output = run(LEGACY, &["-i", "--game"], "");
    assert_eq!(output.status.code(), Some(2));
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("choose either --game or --interactive")
    );
}
