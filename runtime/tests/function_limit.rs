//! F257: the 128-function limit in spec/caveat-reactive-0.2.md counts the
//! program's own `fn` declarations. The standard library prelude is not
//! counted against it, so a program may declare 128 functions of its own.
use caveat_runtime::reactive::ReactiveSession;
use caveat_runtime::web::WebReactiveSession;

const USER_FUNCTION_LIMIT: usize = 128;

fn program(functions: usize) -> String {
    let mut source = String::new();
    for index in 0..functions {
        source.push_str(&format!("fn f{index}() = {index};\n"));
    }
    let last = functions - 1;
    source.push_str(&format!(
        "state total = 0;\nevent go;\non go set total = total + f{last}() + abs(-1);\n"
    ));
    source
}

#[test]
fn f257_a_program_may_declare_128_functions_of_its_own() {
    let source = program(USER_FUNCTION_LIMIT);
    let mut game = ReactiveSession::from_source(&source)
        .unwrap_or_else(|error| panic!("128 user functions must load: {error}"));
    game.dispatch_json("go", "{}").unwrap();
    assert_eq!(game.snapshot().values["total"], 128.0);

    let saved = game.save_json().unwrap();
    let mut resumed = ReactiveSession::restore_json(&source, &saved).unwrap();
    resumed.dispatch_json("go", "{}").unwrap();
    assert_eq!(resumed.snapshot().values["total"], 256.0);

    let mut web = WebReactiveSession::new(&source).unwrap();
    web.dispatch("go", "{}").unwrap();
    WebReactiveSession::check(&source).unwrap();
}

#[test]
fn f257_the_129th_user_function_is_refused() {
    let source = program(USER_FUNCTION_LIMIT + 1);
    let Err(error) = ReactiveSession::from_source(&source) else {
        panic!("129 user functions must be refused");
    };
    assert!(error.contains("function limit 128"), "{error}");
    assert!(WebReactiveSession::new(&source).is_err());
}
