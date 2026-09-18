use caveat_runtime::map::{to_json_pretty, CaveatMap, MAP_SCHEMA};
use std::{env, fs, process};

fn fail(message: impl std::fmt::Display) -> ! {
    eprintln!("{message}");
    process::exit(1)
}

fn load(path: &str) -> CaveatMap {
    let source =
        fs::read_to_string(path).unwrap_or_else(|error| fail(format!("cannot read {path}: {error}")));
    CaveatMap::from_source(&source).unwrap_or_else(|error| fail(format!("map error: {error}")))
}

fn usage(program: &str) -> ! {
    eprintln!(
        "usage:\n  {program} map FILE.cav\n  {program} inspect FILE.cav SYMBOL\n  {program} trace FILE.cav SYMBOL\n  {program} actions FILE.cav\n  {program} validate FILE.cav"
    );
    process::exit(2)
}

fn main() {
    let mut args = env::args();
    let program = args.next().unwrap_or_else(|| "caveat-map".into());
    let Some(command) = args.next() else {
        usage(&program);
    };
    let Some(path) = args.next() else {
        usage(&program);
    };

    match command.as_str() {
        "map" => {
            if args.next().is_some() {
                usage(&program);
            }
            let map = load(&path);
            println!(
                "{}",
                map.to_json_pretty()
                    .unwrap_or_else(|error| fail(format!("JSON error: {error}")))
            );
        }
        "inspect" => {
            let Some(symbol) = args.next() else {
                usage(&program);
            };
            if args.next().is_some() {
                usage(&program);
            }
            let map = load(&path);
            println!(
                "{}",
                to_json_pretty(&map.inspect(&symbol))
                    .unwrap_or_else(|error| fail(format!("JSON error: {error}")))
            );
        }
        "trace" => {
            let Some(symbol) = args.next() else {
                usage(&program);
            };
            if args.next().is_some() {
                usage(&program);
            }
            let map = load(&path);
            println!(
                "{}",
                to_json_pretty(&map.trace(&symbol))
                    .unwrap_or_else(|error| fail(format!("JSON error: {error}")))
            );
        }
        "actions" => {
            if args.next().is_some() {
                usage(&program);
            }
            let map = load(&path);
            println!(
                "{}",
                to_json_pretty(&map.actions)
                    .unwrap_or_else(|error| fail(format!("JSON error: {error}")))
            );
        }
        "validate" => {
            if args.next().is_some() {
                usage(&program);
            }
            let map = load(&path);
            println!(
                "{{\"valid\":true,\"schema\":\"{}\",\"symbols\":{},\"relations\":{},\"actions\":{}}}",
                MAP_SCHEMA,
                map.symbols.len(),
                map.relations.len(),
                map.actions.len()
            );
        }
        _ => usage(&program),
    }
}
