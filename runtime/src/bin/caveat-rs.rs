use std::{env, fs, process};

fn main() {
    let mut args = env::args();
    let program = args.next().unwrap_or_else(|| "caveat-rs".to_string());
    let Some(path) = args.next() else {
        eprintln!("usage: {program} <source.cvr>");
        process::exit(2);
    };
    if args.next().is_some() {
        eprintln!("usage: {program} <source.cvr>");
        process::exit(2);
    }

    let source = match fs::read_to_string(&path) {
        Ok(source) => source,
        Err(error) => {
            eprintln!("{program}: could not read {path}: {error}");
            process::exit(1);
        }
    };

    match caveat_runtime::caveat_rs::transpile(&source) {
        Ok(rust) => print!("{rust}"),
        Err(error) => {
            eprintln!("{program}: {error}");
            process::exit(1);
        }
    }
}
