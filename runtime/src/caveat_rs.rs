use std::collections::HashSet;

const PRELUDE: &str = r#"#[must_use = "a provisional value preserves a commitment and its unresolved caveats"]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Provisional<T> {
    pub value: T,
    pub retained: Vec<&'static str>,
    pub open: bool,
    pub reopen_causes: Vec<&'static str>,
}

impl<T> Provisional<T> {
    pub fn new(value: T, retained: &[&'static str]) -> Self {
        Self {
            value,
            retained: retained.to_vec(),
            open: false,
            reopen_causes: Vec::new(),
        }
    }

    pub fn reopen(&mut self, because: &'static str) {
        self.open = true;
        self.reopen_causes.push(because);
    }
}
"#;

#[derive(Debug)]
struct FunctionState {
    name: String,
    provisional_return: bool,
    caveats: HashSet<String>,
    provisional_bindings: HashSet<String>,
    saw_commit: bool,
    brace_depth: i32,
}

/// Transpile the executable CAVEAT-RS 0.1 bootstrap profile into ordinary Rust.
///
/// Rust remains responsible for ownership, borrowing, lifetimes, and ordinary type checking.
/// This pass adds static checks for CAVEAT-RS commitments, retained caveats, and reopening.
pub fn transpile(source: &str) -> Result<String, String> {
    let mut output = String::new();
    output.push_str(PRELUDE);
    output.push('\n');

    let mut function: Option<FunctionState> = None;

    for (index, original) in source.lines().enumerate() {
        let line_number = index + 1;
        let trimmed = original.trim();

        if function.is_none() && is_function_signature(trimmed) {
            let depth = brace_delta(original);
            if depth <= 0 {
                return Err(at_line(
                    line_number,
                    "CAVEAT-RS 0.1 requires a one-line function signature that opens with {",
                ));
            }

            let name = function_name(trimmed).ok_or_else(|| {
                at_line(
                    line_number,
                    "could not determine function name from signature",
                )
            })?;
            let provisional_return = has_provisional_return(trimmed);
            function = Some(FunctionState {
                name,
                provisional_return,
                caveats: HashSet::new(),
                provisional_bindings: HashSet::new(),
                saw_commit: false,
                brace_depth: depth,
            });
            output.push_str(&rewrite_types(original));
            output.push('\n');
            continue;
        }

        if let Some(state) = function.as_mut() {
            let indent = leading_whitespace(original);
            let rewritten = rewrite_function_line(trimmed, indent, state, line_number)?;
            output.push_str(&rewritten);
            output.push('\n');

            state.brace_depth += brace_delta(original);
            if state.brace_depth < 0 {
                return Err(at_line(line_number, "unbalanced closing brace"));
            }
            if state.brace_depth == 0 {
                validate_finished_function(state, line_number)?;
                function = None;
            }
        } else {
            output.push_str(&rewrite_types(original));
            output.push('\n');
        }
    }

    if let Some(state) = function {
        return Err(format!(
            "function {} is missing a closing brace",
            state.name
        ));
    }

    Ok(output)
}

fn rewrite_function_line(
    trimmed: &str,
    indent: &str,
    state: &mut FunctionState,
    line_number: usize,
) -> Result<String, String> {
    if trimmed.starts_with("caveat ") {
        let name = parse_caveat(trimmed)
            .ok_or_else(|| at_line(line_number, "malformed caveat declaration"))?;
        if !state.caveats.insert(name.to_string()) {
            return Err(at_line(
                line_number,
                &format!("duplicate caveat {name} in function {}", state.name),
            ));
        }
        return Ok(format!("{indent}// CAVEAT-RS caveat {name};"));
    }

    if trimmed.starts_with("reopen ") {
        let (target, because) = parse_reopen(trimmed)
            .ok_or_else(|| at_line(line_number, "malformed reopen statement"))?;

        require_declared_caveat(state, because, line_number)?;
        if !state.provisional_bindings.contains(target) {
            return Err(at_line(
                line_number,
                &format!(
                    "{target} is not a provisional binding created by let {target} = commit ..."
                ),
            ));
        }

        return Ok(format!("{indent}{target}.reopen(\"{because}\");"));
    }

    if trimmed.starts_with("let ") && trimmed.contains(" = commit ") {
        if !state.provisional_return {
            return Err(at_line(
                line_number,
                "commit is only valid in a function returning provisional<...> in bootstrap 0.1",
            ));
        }

        let (binding, expression, retained) = parse_bound_commit(trimmed)
            .ok_or_else(|| at_line(line_number, "malformed bound commitment"))?;
        require_identifier(binding, line_number)?;
        require_retained_caveats(state, &retained, line_number)?;

        if !state.provisional_bindings.insert(binding.to_string()) {
            return Err(at_line(
                line_number,
                &format!("provisional binding {binding} is already defined"),
            ));
        }

        state.saw_commit = true;
        return Ok(format!(
            "{indent}let mut {binding} = Provisional::new({expression}, &{});",
            retained_literal(&retained)
        ));
    }

    if trimmed.starts_with("commit ") {
        if !state.provisional_return {
            return Err(at_line(
                line_number,
                "commit is only valid in a function returning provisional<...> in bootstrap 0.1",
            ));
        }

        let (expression, retained) =
            parse_commit(trimmed).ok_or_else(|| at_line(line_number, "malformed commitment"))?;
        require_retained_caveats(state, &retained, line_number)?;
        state.saw_commit = true;

        return Ok(format!(
            "{indent}return Provisional::new({expression}, &{});",
            retained_literal(&retained)
        ));
    }

    Ok(rewrite_types_with_indent(trimmed, indent))
}

fn parse_caveat(line: &str) -> Option<&str> {
    let name = line.strip_prefix("caveat ")?.strip_suffix(';')?.trim();
    valid_identifier(name).then_some(name)
}

fn parse_bound_commit(line: &str) -> Option<(&str, &str, Vec<&str>)> {
    let rest = line.strip_prefix("let ")?;
    let (binding, commitment) = rest.split_once(" = commit ")?;
    let (expression, retained) = parse_commit_body(commitment.strip_suffix(';')?)?;
    Some((binding.trim(), expression, retained))
}

fn parse_commit(line: &str) -> Option<(&str, Vec<&str>)> {
    let body = line.strip_prefix("commit ")?.strip_suffix(';')?;
    parse_commit_body(body)
}

fn parse_commit_body(body: &str) -> Option<(&str, Vec<&str>)> {
    let (expression, retained) = body.split_once(" retaining ")?;
    let expression = expression.trim();
    if expression.is_empty() {
        return None;
    }

    let retained = retained
        .split(',')
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .collect::<Vec<_>>();
    if retained.is_empty() || retained.iter().any(|name| !valid_identifier(name)) {
        return None;
    }

    Some((expression, retained))
}

fn parse_reopen(line: &str) -> Option<(&str, &str)> {
    let body = line.strip_prefix("reopen ")?.strip_suffix(';')?;
    let (target, because) = body.split_once(" because ")?;
    let target = target.trim();
    let because = because.trim();
    (valid_identifier(target) && valid_identifier(because)).then_some((target, because))
}

fn require_retained_caveats(
    state: &FunctionState,
    retained: &[&str],
    line_number: usize,
) -> Result<(), String> {
    for caveat in retained {
        require_declared_caveat(state, caveat, line_number)?;
    }
    Ok(())
}

fn require_declared_caveat(
    state: &FunctionState,
    caveat: &str,
    line_number: usize,
) -> Result<(), String> {
    if state.caveats.contains(caveat) {
        Ok(())
    } else {
        Err(at_line(
            line_number,
            &format!(
                "caveat {caveat} has not been declared in function {}",
                state.name
            ),
        ))
    }
}

fn require_identifier(identifier: &str, line_number: usize) -> Result<(), String> {
    if valid_identifier(identifier) {
        Ok(())
    } else {
        Err(at_line(
            line_number,
            &format!("invalid CAVEAT-RS identifier {identifier}"),
        ))
    }
}

fn validate_finished_function(state: &FunctionState, line_number: usize) -> Result<(), String> {
    if state.provisional_return && !state.saw_commit {
        return Err(at_line(
            line_number,
            &format!(
                "function {} returns provisional<...> but constructs no commitment",
                state.name
            ),
        ));
    }
    Ok(())
}

fn retained_literal(retained: &[&str]) -> String {
    let entries = retained
        .iter()
        .map(|name| format!("\"{name}\""))
        .collect::<Vec<_>>()
        .join(", ");
    format!("[{entries}]")
}

fn is_function_signature(line: &str) -> bool {
    line.starts_with("fn ") || line.starts_with("pub fn ")
}

fn function_name(signature: &str) -> Option<String> {
    let after_fn = signature
        .strip_prefix("fn ")
        .or_else(|| signature.strip_prefix("pub fn "))?;
    let name = after_fn.split_once('(')?.0.trim();
    valid_identifier(name).then(|| name.to_string())
}

fn has_provisional_return(signature: &str) -> bool {
    signature
        .split_once("->")
        .is_some_and(|(_, return_type)| return_type.trim_start().starts_with("provisional<"))
}

fn rewrite_types(line: &str) -> String {
    line.replace("provisional<", "Provisional<")
}

fn rewrite_types_with_indent(trimmed: &str, indent: &str) -> String {
    format!("{indent}{}", rewrite_types(trimmed))
}

fn leading_whitespace(line: &str) -> &str {
    let length = line.len() - line.trim_start().len();
    &line[..length]
}

fn valid_identifier(value: &str) -> bool {
    let mut chars = value.chars();
    match chars.next() {
        Some(first) if first == '_' || first.is_ascii_alphabetic() => {
            chars.all(|character| character == '_' || character.is_ascii_alphanumeric())
        }
        _ => false,
    }
}

fn at_line(line_number: usize, message: &str) -> String {
    format!("line {line_number}: {message}")
}

fn brace_delta(line: &str) -> i32 {
    let mut delta = 0;
    let mut chars = line.chars().peekable();
    let mut in_string = false;
    let mut in_char = false;
    let mut escaped = false;

    while let Some(character) = chars.next() {
        if escaped {
            escaped = false;
            continue;
        }

        if in_string {
            match character {
                '\\' => escaped = true,
                '"' => in_string = false,
                _ => {}
            }
            continue;
        }

        if in_char {
            match character {
                '\\' => escaped = true,
                '\'' => in_char = false,
                _ => {}
            }
            continue;
        }

        if character == '/' && chars.peek() == Some(&'/') {
            break;
        }

        match character {
            '"' => in_string = true,
            '\'' => in_char = true,
            '{' => delta += 1,
            '}' => delta -= 1,
            _ => {}
        }
    }

    delta
}

#[cfg(test)]
mod tests {
    use super::transpile;

    #[test]
    fn provisional_commitment_and_reopening_transpile() {
        let source = r#"fn route(sensor_ok: bool) -> provisional<bool> {
    caveat stale_sensor;
    caveat camera_gap;
    let decision = commit sensor_ok retaining stale_sensor, camera_gap;
    reopen decision because camera_gap;
    return decision;
}"#;

        let rust = transpile(source).expect("program should transpile");
        assert!(rust.contains("fn route(sensor_ok: bool) -> Provisional<bool>"));
        assert!(rust.contains(
            "let mut decision = Provisional::new(sensor_ok, &[\"stale_sensor\", \"camera_gap\"]);"
        ));
        assert!(rust.contains("decision.reopen(\"camera_gap\");"));
    }

    #[test]
    fn direct_commit_becomes_return() {
        let source = r#"fn choose(ok: bool) -> provisional<bool> {
    caveat stale_sensor;
    commit ok retaining stale_sensor;
}"#;

        let rust = transpile(source).expect("program should transpile");
        assert!(rust.contains("return Provisional::new(ok, &[\"stale_sensor\"]);"));
    }

    #[test]
    fn undefined_retained_caveat_is_rejected() {
        let source = r#"fn choose(ok: bool) -> provisional<bool> {
    commit ok retaining missing;
}"#;

        let error = transpile(source).expect_err("undefined caveat should fail");
        assert!(error.contains("caveat missing has not been declared"));
    }

    #[test]
    fn ordinary_function_cannot_commit() {
        let source = r#"fn choose(ok: bool) -> bool {
    caveat stale_sensor;
    commit ok retaining stale_sensor;
}"#;

        let error = transpile(source).expect_err("ordinary return should fail");
        assert!(error.contains("only valid in a function returning provisional<...>"));
    }

    #[test]
    fn reopen_requires_a_commitment_binding() {
        let source = r#"fn choose(ok: bool) -> provisional<bool> {
    caveat stale_sensor;
    reopen decision because stale_sensor;
    commit ok retaining stale_sensor;
}"#;

        let error = transpile(source).expect_err("unknown binding should fail");
        assert!(error.contains("is not a provisional binding"));
    }

    #[test]
    fn provisional_function_requires_commitment() {
        let source = r#"fn choose(ok: bool) -> provisional<bool> {
    return ok;
}"#;

        let error = transpile(source).expect_err("missing commit should fail");
        assert!(error.contains("constructs no commitment"));
    }

    #[test]
    fn braces_inside_strings_do_not_end_function() {
        let source = r#"fn describe(ok: bool) -> provisional<bool> {
    caveat formatting;
    let text = "{not a block}";
    let decision = commit ok retaining formatting;
    return decision;
}"#;

        let rust = transpile(source).expect("string braces should be ignored");
        assert!(rust.contains("let text = \"{not a block}\";"));
    }
}
