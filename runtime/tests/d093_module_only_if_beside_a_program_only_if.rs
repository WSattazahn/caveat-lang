//! D093: a module of a bundle that declares `only_if` as something other
//! than a function, in a bundle whose program declares `fn only_if` as well
//! as `fn m__only_if`. From rc.1 to 1.3.0 the module's `only_if(…)` called
//! `fn m__only_if`, its own name flattened. 1.4.0 and 1.4.1 refused every
//! such bundle in which a function's body gives `only_if(…)` a number for
//! its condition ("source function m__f: expression requires boolean, found
//! number"). Main, where a function's body may call the program's own
//! `fn only_if` (D084), read the module's call as one of `fn only_if`, which
//! no release did. A bundle that 1.4 loads keeps its reading.
use caveat_runtime::reactive::ReactiveSession;
use serde_json::{json, Value};

const ROOT: &str = "use m;\nclaim c;\nevidence e from \"e\";\nstate s = 0;\nevent go;\non go set s = s + 1;\nfn m__only_if(c, v) = v + 100;\nfn only_if(c, v) = v + 10;\n";

fn bundle(module: &str, root: &str) -> String {
    format!(
        "#caveat-bundle 1\n#module m {}\n{module}#module main {}\n{root}",
        module.len(),
        root.len()
    )
}

fn load(source: &str) -> ReactiveSession {
    ReactiveSession::from_source(source).unwrap_or_else(|error| panic!("{error}\n{source}"))
}

fn values(session: &ReactiveSession) -> Value {
    serde_json::to_value(session.snapshot()).unwrap()["values"].clone()
}

fn send(session: &mut ReactiveSession, event: &str) {
    session
        .dispatch_json(event, "{}")
        .unwrap_or_else(|error| panic!("{event}: {error}"));
}

/// The module's function body calls `fn m__only_if`, as 1.3.0 did, and a
/// save of it restores to the same reading.
#[test]
fn a_module_function_body_calls_the_function_named_for_the_module() {
    let source = bundle(
        "module m;\nstate only_if = 0;\nevent tick;\nstate u = 0;\nfn f(x) = only_if(x, x);\non tick set u = f(2);\n",
        ROOT,
    );
    let mut session = load(&source);
    send(&mut session, "m__tick");
    assert_eq!(values(&session)["m__u"], json!(102.0));
    let saved = session.save_json().unwrap();
    assert!(!saved.contains("module_bindings"), "{saved}");
    let mut restored = ReactiveSession::restore_json(&source, &saved).unwrap();
    assert_eq!(restored.snapshot(), session.snapshot());
    send(&mut restored, "m__tick");
    assert_eq!(values(&restored)["m__u"], json!(102.0));
}

/// A function of the program that gives `only_if` a number made 1.4 refuse
/// the bundle too, so the module's rule calls `fn m__only_if` as 1.3.0's did,
/// and the program's own function body still calls `fn only_if`.
#[test]
fn a_module_rule_reads_as_before_1_4_beside_such_a_program_function() {
    let root = format!("{ROOT}fn g(x) = only_if(x, x);\nstate t = 0;\non go set t = g(s);\n");
    let source = bundle(
        "module m;\nstate only_if = 0;\nevent tick;\nstate u = 0;\non tick set u = only_if(2, 2);\n",
        &root,
    );
    let mut session = load(&source);
    send(&mut session, "go");
    send(&mut session, "m__tick");
    let values = values(&session);
    assert_eq!(values["m__u"], json!(102.0));
    assert_eq!(values["t"], json!(11.0));
}

/// Where 1.4 loads the bundle, its reading stands: a rule calls the program's
/// `fn only_if`, and an unused body may hold the form with a condition.
#[test]
fn a_bundle_that_1_4_loads_keeps_its_reading() {
    let rule = bundle(
        "module m;\nstate only_if = 0;\nevent tick;\nstate u = 0;\non tick set u = only_if(2, 2);\n",
        ROOT,
    );
    let mut session = load(&rule);
    send(&mut session, "m__tick");
    assert_eq!(values(&session)["m__u"], json!(12.0));

    let unused = bundle(
        "module m;\nstate only_if = 0;\nevent tick;\nstate u = 0;\nfn f(x) = only_if(x > 0, x);\non tick set u = only_if(2, 2);\n",
        ROOT,
    );
    let mut session = load(&unused);
    send(&mut session, "m__tick");
    assert_eq!(values(&session)["m__u"], json!(12.0));
}

/// Without `fn m__only_if` the module's call has nothing to reach, so the
/// bundle, which no release loaded, keeps the call of the program's own
/// function.
#[test]
fn without_the_function_named_for_the_module_the_program_function_is_called() {
    let source = bundle(
        "module m;\nstate only_if = 0;\nevent tick;\nstate u = 0;\nfn f(x) = only_if(x, x);\non tick set u = f(2);\n",
        "use m;\nclaim c;\nevidence e from \"e\";\nstate s = 0;\nevent go;\non go set s = s + 1;\nfn only_if(c, v) = v + 10;\n",
    );
    let mut session = load(&source);
    send(&mut session, "m__tick");
    assert_eq!(values(&session)["m__u"], json!(12.0));
}
