use caveat_runtime::{ast::{Program, Statement}, eval, parser, NodeKind};
use std::{collections::HashMap, env, fs, io::{self, Write}, process};

fn shown(display: &HashMap<String, String>, symbol: &str) -> String {
    display.get(symbol).cloned().unwrap_or_else(|| symbol.replace('_', " "))
}

fn ask(options: &[String], display: &HashMap<String, String>, fallback: &str) -> String {
    for option in options { println!("  {option} — {}", shown(display, option)); }
    loop {
        print!("> ");
        io::stdout().flush().expect("stdout should be writable");
        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() { return fallback.into(); }
        let selection = input.trim().to_lowercase();
        if options.contains(&selection) { return selection; }
        if selection.is_empty() { return fallback.into(); }
        println!("Choose: {}", options.join(", "));
    }
}

fn fail(message: impl std::fmt::Display) -> ! { eprintln!("{message}"); process::exit(1) }

fn main() {
    let mut interactive = false;
    let mut path = None;
    for argument in env::args().skip(1) {
        if argument == "-i" || argument == "--interactive" { interactive = true; } else { path = Some(argument); }
    }
    let path = path.unwrap_or_else(|| { eprintln!("usage: caveat [-i] file.cav"); process::exit(2) });
    let source = fs::read_to_string(&path).unwrap_or_else(|error| fail(format!("cannot read {path}: {error}")));
    let mut program = parser::parse(&source).unwrap_or_else(|error| fail(format!("parse error: {error}")));

    if interactive {
        let interaction_indices: Vec<usize> = program.statements.iter().enumerate().filter_map(|(index, statement)| matches!(statement, Statement::Inspect { .. } | Statement::Select { .. }).then_some(index)).collect();
        for index in interaction_indices {
            let preview = eval::evaluate(&Program::new(program.statements[..index].to_vec())).unwrap_or_else(|error| fail(format!("evaluation error: {error}")));
            match program.statements[index].clone() {
                Statement::Inspect { investigation, caveat, cost } => {
                    let options = preview.investigations.get(&investigation).unwrap_or_else(|| fail(format!("unknown investigation: {investigation}"))).options.clone();
                    println!("\n=== Investigation ===");
                    if let Some(budget) = &preview.resources { println!("Attention remaining: {}", budget.remaining); }
                    println!("What do you investigate? (cost {cost})");
                    let selection = ask(&options, &preview.display, &caveat);
                    if let Statement::Inspect { caveat, .. } = &mut program.statements[index] { *caveat = selection; }
                }
                Statement::Select { choice, option } => {
                    let options = preview.choices.get(&choice).unwrap_or_else(|| fail(format!("unknown choice: {choice}"))).options.clone();
                    println!("\n=== {} ===", shown(&preview.display, &choice));
                    if let Some(budget) = &preview.resources { println!("Attention remaining: {}", budget.remaining); }
                    println!("What do you do?");
                    let selection = ask(&options, &preview.display, &option);
                    if let Statement::Select { option, .. } = &mut program.statements[index] { *option = selection; }
                }
                _ => unreachable!("interaction index must point to an interactive statement"),
            }
        }
    }

    let evaluation = eval::evaluate(&program).unwrap_or_else(|error| fail(format!("evaluation error: {error}")));
    println!("\n--- Decision record ---");
    for (name, id) in &evaluation.symbols {
        if let Some(NodeKind::Commitment { open, .. }) = evaluation.graph.nodes.get(id) {
            println!("{}: {}", shown(&evaluation.display, name), if *open { "REOPENED" } else { "committed" });
        }
    }
    if let Some(budget) = evaluation.resources { println!("Attention spent: {} / {}", budget.spent, budget.initial); }
}
