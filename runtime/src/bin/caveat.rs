use caveat_runtime::{
    ast::{EpistemicCondition, Program, Statement},
    eval,
    game_session::{GamePending, GameSession, GameSnapshot},
    parser, NodeKind,
};
use std::{
    collections::HashMap,
    env, fs,
    io::{self, Write},
    process,
};

fn shown(display: &HashMap<String, String>, symbol: &str) -> String {
    display
        .get(symbol)
        .cloned()
        .unwrap_or_else(|| symbol.replace('_', " "))
}

fn ask(options: &[String], display: &HashMap<String, String>, fallback: &str) -> String {
    for option in options {
        println!("  {option} — {}", shown(display, option));
    }
    loop {
        print!("> ");
        io::stdout().flush().expect("stdout should be writable");
        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            return fallback.into();
        }
        let selection = input.trim().to_lowercase();
        if options.contains(&selection) {
            return selection;
        }
        if selection.is_empty() {
            return fallback.into();
        }
        println!("Choose: {}", options.join(", "));
    }
}

fn fail(message: impl std::fmt::Display) -> ! {
    eprintln!("{message}");
    process::exit(1)
}

fn game_label(snapshot: &GameSnapshot, symbol: &str) -> String {
    let label = snapshot
        .labels
        .get(symbol)
        .cloned()
        .unwrap_or_else(|| symbol.replace('_', " "));
    format!("{label} [{symbol}]")
}

fn game_condition(snapshot: &GameSnapshot, condition: &EpistemicCondition) -> String {
    let kind = match condition {
        EpistemicCondition::Observed { .. } => "observed",
        EpistemicCondition::Examined { .. } => "examined",
    };
    format!("{kind} {}", game_label(snapshot, condition.symbol()))
}

fn play_game(source: &str) -> Result<(), String> {
    let mut session = GameSession::from_source(source)?;
    let mut snapshot = session.snapshot()?;
    let mut scene_count = 0;
    let mut discovery_count = 0;
    println!("\n=== CAVEAT game ===");
    println!("Enter an option number or action ID. Type quit to stop.");
    loop {
        for scene in snapshot.scenes.iter().skip(scene_count) {
            println!("\n{scene}");
        }
        scene_count = snapshot.scenes.len();
        println!("\n--- Turn {} ---", snapshot.turn);
        println!(
            "Location: {}",
            game_label(&snapshot, &snapshot.state.current_place)
        );
        if let Some(budget) = &snapshot.budget {
            println!(
                "Attention remaining: {} (spent {} / {})",
                budget.remaining, budget.spent, budget.initial
            );
        }
        for discovery in snapshot.discoveries.iter().skip(discovery_count) {
            println!(
                "Evidence: {} {} {}",
                game_label(&snapshot, &discovery.evidence),
                discovery.relation,
                game_label(&snapshot, &discovery.target)
            );
        }
        discovery_count = snapshot.discoveries.len();
        for symbol in &snapshot.symbols {
            if symbol.attention.as_deref() == Some("examined") {
                println!("Examined: {}", game_label(&snapshot, &symbol.name));
            }
        }
        for commitment in &snapshot.commitments {
            println!(
                "Decision: {} — {}",
                game_label(&snapshot, &commitment.action),
                if commitment.open {
                    "REOPENED"
                } else {
                    "committed"
                }
            );
            if !commitment.retained.is_empty() {
                println!(
                    "  Retained uncertainty: {}",
                    commitment
                        .retained
                        .iter()
                        .map(|symbol| game_label(&snapshot, symbol))
                        .collect::<Vec<_>>()
                        .join(", ")
                );
            }
            for evidence in &commitment.reopened_by {
                println!("  Reopened by: {}", game_label(&snapshot, evidence));
            }
        }
        if let Some(outcome) = &snapshot.outcome {
            println!("Outcome: {}", game_label(&snapshot, &outcome.id));
            if !outcome.basis.is_empty() {
                println!(
                    "  Basis at decision: {}",
                    outcome
                        .basis
                        .iter()
                        .map(|condition| game_condition(&snapshot, condition))
                        .collect::<Vec<_>>()
                        .join(", ")
                );
            }
        }
        for blocked in &snapshot.blocked_actions {
            println!(
                "Unavailable: {} — requires {}",
                game_label(&snapshot, &blocked.action),
                blocked
                    .reasons
                    .iter()
                    .map(|condition| game_condition(&snapshot, condition))
                    .collect::<Vec<_>>()
                    .join("; ")
            );
        }
        let options = match &snapshot.pending {
            GamePending::Complete => {
                println!("Game complete after {} actions.", snapshot.turn);
                return Ok(());
            }
            GamePending::Blocked { name, reason, .. } => {
                return Err(format!(
                    "blocked at {}: {reason}",
                    game_label(&snapshot, name)
                ));
            }
            GamePending::Investigate {
                name,
                options,
                cost,
                ..
            } => {
                println!(
                    "\n{} (investigation cost {cost})",
                    game_label(&snapshot, name)
                );
                options
            }
            GamePending::Choice { name, options, .. } => {
                println!("\n{}", game_label(&snapshot, name));
                options
            }
        };
        for (index, option) in options.iter().enumerate() {
            println!("  {}. {}", index + 1, game_label(&snapshot, option));
        }
        let next = loop {
            print!("> ");
            io::stdout()
                .flush()
                .map_err(|error| format!("cannot write prompt: {error}"))?;
            let mut input = String::new();
            let read = io::stdin()
                .read_line(&mut input)
                .map_err(|error| format!("cannot read selection: {error}"))?;
            if read == 0 || input.trim().eq_ignore_ascii_case("quit") {
                println!("Stopped after {} actions.", snapshot.turn);
                return Ok(());
            }
            let input = input.trim();
            let selection = options
                .iter()
                .find(|option| option.as_str() == input)
                .or_else(|| {
                    input
                        .parse::<usize>()
                        .ok()
                        .and_then(|number| number.checked_sub(1))
                        .and_then(|index| options.get(index))
                });
            let Some(selection) = selection else {
                println!(
                    "Selection is not available: {input:?}. Choose 1–{} or an available action ID.",
                    options.len()
                );
                continue;
            };
            match session.apply(selection) {
                Ok(next) => break next,
                Err(error) => println!("Action failed; state unchanged: {error}"),
            }
        };
        snapshot = next;
    }
}

fn main() {
    let mut interactive = false;
    let mut game = false;
    let mut path = None;
    for argument in env::args().skip(1) {
        if argument == "-i" || argument == "--interactive" {
            interactive = true;
        } else if argument == "--game" {
            game = true;
        } else {
            path = Some(argument);
        }
    }
    let path = path.unwrap_or_else(|| {
        eprintln!("usage: caveat [-i | --game] file.cav");
        process::exit(2)
    });
    if game && interactive {
        eprintln!("choose either --game or --interactive");
        process::exit(2);
    }
    let source = fs::read_to_string(&path)
        .unwrap_or_else(|error| fail(format!("cannot read {path}: {error}")));
    if game {
        play_game(&source).unwrap_or_else(|error| fail(format!("game error: {error}")));
        return;
    }
    let mut program =
        parser::parse(&source).unwrap_or_else(|error| fail(format!("parse error: {error}")));

    if interactive {
        let interaction_indices: Vec<usize> = program
            .statements
            .iter()
            .enumerate()
            .filter_map(|(index, statement)| {
                matches!(
                    statement,
                    Statement::Inspect { .. } | Statement::Select { .. }
                )
                .then_some(index)
            })
            .collect();
        for index in interaction_indices {
            let preview = eval::evaluate(&Program::new(program.statements[..index].to_vec()))
                .unwrap_or_else(|error| fail(format!("evaluation error: {error}")));
            match program.statements[index].clone() {
                Statement::Inspect {
                    investigation,
                    caveat,
                    cost,
                } => {
                    let options = preview
                        .investigations
                        .get(&investigation)
                        .unwrap_or_else(|| fail(format!("unknown investigation: {investigation}")))
                        .options
                        .clone();
                    println!("\n=== Investigation ===");
                    if let Some(budget) = &preview.resources {
                        println!("Attention remaining: {}", budget.remaining);
                    }
                    println!("What do you investigate? (cost {cost})");
                    let selection = ask(&options, &preview.display, &caveat);
                    if let Statement::Inspect { caveat, .. } = &mut program.statements[index] {
                        *caveat = selection;
                    }
                }
                Statement::Select { choice, option } => {
                    let options = preview
                        .choices
                        .get(&choice)
                        .unwrap_or_else(|| fail(format!("unknown choice: {choice}")))
                        .options
                        .clone();
                    println!("\n=== {} ===", shown(&preview.display, &choice));
                    if let Some(budget) = &preview.resources {
                        println!("Attention remaining: {}", budget.remaining);
                    }
                    println!("What do you do?");
                    let selection = ask(&options, &preview.display, &option);
                    if let Statement::Select { option, .. } = &mut program.statements[index] {
                        *option = selection;
                    }
                }
                _ => unreachable!("interaction index must point to an interactive statement"),
            }
        }
    }

    let evaluation =
        eval::evaluate(&program).unwrap_or_else(|error| fail(format!("evaluation error: {error}")));
    println!("\n--- Decision record ---");
    for (name, id) in &evaluation.symbols {
        if let Some(NodeKind::Commitment { open, .. }) = evaluation.graph.nodes.get(id) {
            println!(
                "{}: {}",
                shown(&evaluation.display, name),
                if *open { "REOPENED" } else { "committed" }
            );
        }
    }
    if let Some(budget) = evaluation.resources {
        println!("Attention spent: {} / {}", budget.spent, budget.initial);
    }
}
