use caveat_runtime::caveat_rs::transpile;
use std::{fs, process::Command, time::{SystemTime, UNIX_EPOCH}};

#[test]
fn example_transpiles_to_rust_that_rustc_accepts() {
    let source = include_str!("../../examples/route.cvr");
    let rust = transpile(source).expect("CAVEAT-RS example should transpile");

    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be after Unix epoch")
        .as_nanos();
    let directory = std::env::temp_dir().join(format!(
        "caveat-rs-{}-{nonce}",
        std::process::id()
    ));
    fs::create_dir_all(&directory).expect("temporary directory should be created");

    let source_path = directory.join("generated.rs");
    let output_path = directory.join("generated.rlib");
    fs::write(&source_path, rust).expect("generated Rust should be written");

    let output = Command::new("rustc")
        .arg("--edition=2021")
        .arg("--crate-type=lib")
        .arg(&source_path)
        .arg("-o")
        .arg(&output_path)
        .output()
        .expect("rustc should be available during runtime tests");

    if let Err(error) = fs::remove_dir_all(&directory) {
        eprintln!("could not remove CAVEAT-RS test directory: {error}");
    }

    assert!(
        output.status.success(),
        "generated Rust did not compile:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
