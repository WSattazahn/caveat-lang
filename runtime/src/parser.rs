use crate::ast::{ActionStep, ConditionalAction, EpistemicCondition, Program, Statement};
use crate::{Attention, Consequence, Relation, StopReason};

/// Parse CAVEAT source. Diagnostics use one-based Unicode character positions.
pub fn parse(source: &str) -> Result<Program, String> {
    let statements = scan_statements(source)?
        .into_iter()
        .map(|(line, position)| {
            parse_statement(line.trim(), position).map_err(|message| position.error(message))
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Program::new(statements))
}

#[derive(Clone, Copy)]
pub(crate) struct Position {
    pub(crate) line: usize,
    pub(crate) column: usize,
}

impl Position {
    pub(crate) fn error(self, message: impl std::fmt::Display) -> String {
        format!("line {}, column {}: {message}", self.line, self.column)
    }

    pub(crate) fn advance(&mut self, ch: char) {
        if ch == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
    }
}

/// Semicolons and comment markers are syntax only outside quoted text.
/// Keeping comment boundaries as whitespace also prevents adjacent tokens merging.
fn scan_statements(source: &str) -> Result<Vec<(String, Position)>, String> {
    scan_statements_at(source, Position { line: 1, column: 1 })
}

pub(crate) fn scan_statements_at(
    source: &str,
    mut position: Position,
) -> Result<Vec<(String, Position)>, String> {
    let mut statements = Vec::new();
    let mut text = String::new();
    let mut start = None;
    let mut string_start = None;
    let mut escape_start: Option<Position> = None;
    let mut comment = false;
    let mut braces = Vec::new();
    let mut chars = source.chars().peekable();

    while let Some(ch) = chars.next() {
        if comment {
            if ch == '\n' {
                comment = false;
                text.push(ch);
            } else {
                // Preserve Unicode columns for diagnostics inside procedures.
                text.push(' ');
            }
        } else if string_start.is_some() {
            text.push(ch);
            if let Some(escape) = escape_start.take() {
                if !matches!(ch, '"' | '\\' | 'n' | 'r' | 't') {
                    return Err(escape.error(format!("unknown string escape: \\{ch}")));
                }
            } else if ch == '\\' {
                escape_start = Some(position);
            } else if ch == '"' {
                string_start = None;
            }
        } else if ch == '#' || (ch == '/' && chars.peek() == Some(&'/')) {
            comment = true;
            text.push(' ');
        } else if ch == ';' && braces.is_empty() {
            if let Some(statement_start) = start.take() {
                statements.push((std::mem::take(&mut text), statement_start));
            } else {
                text.clear();
            }
        } else {
            if !ch.is_whitespace() && start.is_none() {
                start = Some(position);
            }
            if ch == '"' {
                string_start = Some(position);
            } else if ch == '{' && text.split_whitespace().next() == Some("proc") {
                if braces.len() >= 64 {
                    return Err(position.error("source exceeds brace nesting limit 64"));
                }
                braces.push(position);
            } else if ch == '}'
                && text.split_whitespace().next() == Some("proc")
                && braces.pop().is_none()
            {
                return Err(position.error("unmatched closing brace"));
            }
            text.push(ch);
        }
        position.advance(ch);
    }

    if let Some(quote) = string_start {
        let detail = if escape_start.is_some() {
            "unterminated quoted string (incomplete escape)"
        } else {
            "unterminated quoted string"
        };
        return Err(quote.error(detail));
    }
    if let Some(brace) = braces.first() {
        return Err(brace.error("unterminated procedure body"));
    }
    // The original parser permitted an omitted final semicolon; retain that API.
    if let Some(statement_start) = start {
        statements.push((text, statement_start));
    }
    Ok(statements)
}

fn parse_statement(line: &str, position: Position) -> Result<Statement, String> {
    if let Some(directive) = crate::reactive::parse_directive_at(line, position) {
        return directive.map(Statement::Reactive);
    }
    if let Some(directive) = crate::presentation::parse_directive(line) {
        return directive.map(Statement::Presentation);
    }
    let words: Vec<&str> = line.split_whitespace().collect();
    let statement = if words.first() == Some(&"scene") {
        Statement::Scene {
            text: quoted(line.strip_prefix("scene").unwrap().trim())?,
        }
    } else if words.first() == Some(&"display") {
        parse_display(line)?
    } else if words.first() == Some(&"investigate") {
        parse_investigate(&words, line)?
    } else if words.first() == Some(&"reveal") {
        parse_reveal(&words, line)?
    } else if words.first() == Some(&"rule") {
        parse_rule(&words, line)?
    } else if words.first() == Some(&"choice") {
        parse_choice(&words)?
    } else if words.first() == Some(&"when_committed") {
        parse_when(&words, line)?
    } else if words.first() == Some(&"action") {
        parse_action(line)?
    } else {
        match words.as_slice() {
            ["require", action, kind, symbol] => Statement::Require {
                action: (*action).into(),
                condition: epistemic_condition(kind, symbol)?,
            },
            ["resolve", action, "as", outcome, "when", kind, symbol] => Statement::Resolve {
                action: (*action).into(),
                outcome: (*outcome).into(),
                condition: Some(epistemic_condition(kind, symbol)?),
            },
            ["resolve", action, "as", outcome, "otherwise"] => Statement::Resolve {
                action: (*action).into(),
                outcome: (*outcome).into(),
                condition: None,
            },
            ["budget", amount] => Statement::Budget {
                units: number(amount)?,
            },
            ["place", name, "kind", kind] => Statement::Place {
                name: (*name).into(),
                kind: (*kind).into(),
            },
            ["entity", name, "kind", kind, "at", at] => Statement::Entity {
                name: (*name).into(),
                kind: (*kind).into(),
                at: (*at).into(),
            },
            ["connect", from, "to", to] => Statement::Connect {
                from: (*from).into(),
                to: (*to).into(),
                via: None,
            },
            ["connect", from, "to", to, "via", via] => Statement::Connect {
                from: (*from).into(),
                to: (*to).into(),
                via: Some((*via).into()),
            },
            ["start_at", place] => Statement::StartAt {
                place: (*place).into(),
            },
            ["claim", name] => Statement::Claim {
                name: (*name).into(),
            },
            ["evidence", name, "from", rest @ ..] if !rest.is_empty() => Statement::Evidence {
                name: (*name).into(),
                source: evidence_source(line, rest)?,
            },
            ["caveat", name, "consequence", consequence] => Statement::Caveat {
                name: (*name).into(),
                consequence: parse_consequence(consequence)?,
            },
            [from, "supports", to] => relation(from, Relation::Supports, to),
            [from, "opposes", to] => relation(from, Relation::Opposes, to),
            [from, "qualifies", to] => relation(from, Relation::Qualifies, to),
            ["examine", name, "cost", cost] => Statement::Examine {
                caveat: (*name).into(),
                cost: number(cost)?,
            },
            ["examine", name] => Statement::Attention {
                caveat: (*name).into(),
                state: Attention::Examining,
            },
            ["defer", name] => Statement::Attention {
                caveat: (*name).into(),
                state: Attention::Deferred,
            },
            ["inspect", investigation, caveat, "cost", cost] => Statement::Inspect {
                investigation: (*investigation).into(),
                caveat: (*caveat).into(),
                cost: number(cost)?,
            },
            ["infer", rule] => Statement::Infer {
                rule: (*rule).into(),
            },
            ["converge", choice] => Statement::Converge {
                choice: (*choice).into(),
            },
            ["select", choice, option] => Statement::Select {
                choice: (*choice).into(),
                option: (*option).into(),
            },
            ["commit", action, "because", reason] => Statement::Commit {
                action: (*action).into(),
                reason: parse_reason(reason)?,
                retaining: Vec::new(),
            },
            ["commit", action, "because", reason, "retaining", rest @ ..] => Statement::Commit {
                action: (*action).into(),
                reason: parse_reason(reason)?,
                retaining: identifiers(rest),
            },
            ["reopen", commitment, "because", because] => Statement::Reopen {
                commitment: (*commitment).into(),
                because: (*because).into(),
            },
            _ => return Err(format!("cannot parse statement: {line}")),
        }
    };
    Ok(statement)
}

fn relation(from: &str, relation: Relation, to: &str) -> Statement {
    Statement::Relate {
        from: from.into(),
        relation,
        to: to.into(),
    }
}

fn epistemic_condition(kind: &str, symbol: &str) -> Result<EpistemicCondition, String> {
    match kind {
        "observed" => Ok(EpistemicCondition::Observed {
            symbol: symbol.into(),
        }),
        "examined" => Ok(EpistemicCondition::Examined {
            symbol: symbol.into(),
        }),
        _ => Err(format!(
            "unknown epistemic condition {kind}; expected observed or examined"
        )),
    }
}

fn parse_reveal(words: &[&str], line: &str) -> Result<Statement, String> {
    match words {
        ["reveal", caveat, "then", from, "supports", to] => Ok(Statement::Reveal {
            when_inspected: (*caveat).into(),
            from: (*from).into(),
            relation: Relation::Supports,
            to: (*to).into(),
        }),
        ["reveal", caveat, "then", from, "opposes", to] => Ok(Statement::Reveal {
            when_inspected: (*caveat).into(),
            from: (*from).into(),
            relation: Relation::Opposes,
            to: (*to).into(),
        }),
        _ => Err(format!("invalid reveal: {line}")),
    }
}

fn parse_investigate(words: &[&str], line: &str) -> Result<Statement, String> {
    match words {
        ["investigate", name, "options", rest @ ..] => Ok(Statement::Investigate {
            name: (*name).into(),
            options: identifiers(rest),
        }),
        _ => Err(format!("invalid investigate: {line}")),
    }
}

pub(crate) fn quoted(value: &str) -> Result<String, String> {
    let mut chars = value.chars();
    if chars.next() != Some('"') {
        return Err(format!("expected quoted text: {value}"));
    }
    let mut text = String::new();
    while let Some(ch) = chars.next() {
        match ch {
            '"' => {
                return if chars.all(char::is_whitespace) {
                    Ok(text)
                } else {
                    Err(format!("unexpected text after closing quote: {value}"))
                };
            }
            '\\' => {
                let escaped = match chars.next() {
                    Some('"') => '"',
                    Some('\\') => '\\',
                    Some('n') => '\n',
                    Some('r') => '\r',
                    Some('t') => '\t',
                    Some(other) => return Err(format!("unknown string escape: \\{other}")),
                    None => return Err("unterminated quoted string (incomplete escape)".into()),
                };
                text.push(escaped);
            }
            _ => text.push(ch),
        }
    }
    Err("unterminated quoted string".into())
}

fn evidence_source(line: &str, words: &[&str]) -> Result<String, String> {
    // Consume the three header tokens, preserving whitespace within quoted provenance.
    let mut rest = line;
    for _ in 0..3 {
        rest = rest.trim_start();
        let end = rest.find(char::is_whitespace).unwrap_or(rest.len());
        rest = &rest[end..];
    }
    let source = rest.trim();
    if source.starts_with('"') {
        quoted(source)
    } else {
        // Unquoted provenance was accepted by the original parser.
        Ok(words.join(" "))
    }
}

fn parse_display(line: &str) -> Result<Statement, String> {
    let rest = line.strip_prefix("display").unwrap().trim_start();
    let end = rest
        .find(char::is_whitespace)
        .ok_or_else(|| format!("display missing text: {line}"))?;
    Ok(Statement::Display {
        symbol: rest[..end].into(),
        text: quoted(rest[end..].trim())?,
    })
}

fn parse_action(line: &str) -> Result<Statement, String> {
    let rest = line
        .strip_prefix("action")
        .ok_or_else(|| format!("invalid action: {line}"))?;
    let tokens = rest.split_whitespace().collect::<Vec<_>>();
    let steps_index = tokens
        .iter()
        .enumerate()
        .skip(5)
        .find_map(|(index, word)| (*word == "steps").then_some(index))
        .ok_or_else(|| format!("action missing steps: {line}"))?;
    let words = &tokens[..steps_index];
    let steps_text = tokens[steps_index + 1..].join(" ");

    if words.len() < 5 || words.get(1) != Some(&"from") || words.get(3) != Some(&"to") {
        return Err(format!("invalid action header: {line}"));
    }

    let action = words[0].to_string();
    let from = words[2].to_string();
    let to = words[4].to_string();
    let requires_open = if words.len() == 5 {
        Vec::new()
    } else if words.get(5) == Some(&"requires_open") {
        identifiers(&words[6..])
    } else {
        return Err(format!("invalid action preconditions: {line}"));
    };

    let steps = steps_text
        .split(',')
        .map(str::trim)
        .filter(|step| !step.is_empty())
        .map(parse_action_step)
        .collect::<Result<Vec<_>, _>>()?;

    if steps.is_empty() {
        return Err(format!("action requires at least one step: {line}"));
    }

    Ok(Statement::ActionPlan {
        action,
        from,
        to,
        requires_open,
        steps,
    })
}

fn parse_action_step(step: &str) -> Result<ActionStep, String> {
    let words = step.split_whitespace().collect::<Vec<_>>();
    match words.as_slice() {
        ["inspect", entity] => Ok(ActionStep::Inspect {
            entity: (*entity).into(),
        }),
        ["operate", entity] => Ok(ActionStep::Operate {
            entity: (*entity).into(),
        }),
        ["open", entity] => Ok(ActionStep::Open {
            entity: (*entity).into(),
        }),
        ["through", entity] => Ok(ActionStep::Through {
            entity: (*entity).into(),
        }),
        ["move", place] => Ok(ActionStep::Move {
            place: (*place).into(),
        }),
        ["observe", symbol] => Ok(ActionStep::Observe {
            symbol: (*symbol).into(),
        }),
        ["stay"] => Ok(ActionStep::Stay),
        _ => Err(format!("invalid action step: {step}")),
    }
}

fn parse_when(words: &[&str], line: &str) -> Result<Statement, String> {
    match words {
        ["when_committed", action, "reopen", commitment, "because", because] => {
            Ok(Statement::WhenCommitted {
                action: (*action).into(),
                then: ConditionalAction::Reopen {
                    commitment: (*commitment).into(),
                    because: (*because).into(),
                },
            })
        }
        ["when_committed", action, from, "supports", to] => Ok(Statement::WhenCommitted {
            action: (*action).into(),
            then: ConditionalAction::Relate {
                from: (*from).into(),
                relation: Relation::Supports,
                to: (*to).into(),
            },
        }),
        ["when_committed", action, from, "opposes", to] => Ok(Statement::WhenCommitted {
            action: (*action).into(),
            then: ConditionalAction::Relate {
                from: (*from).into(),
                relation: Relation::Opposes,
                to: (*to).into(),
            },
        }),
        _ => Err(format!("invalid when_committed: {line}")),
    }
}

fn parse_rule(words: &[&str], line: &str) -> Result<Statement, String> {
    let arrow = words
        .iter()
        .position(|word| *word == "=>")
        .ok_or_else(|| format!("rule missing =>: {line}"))?;
    if words.len() < 5 || arrow <= 3 || arrow + 1 >= words.len() {
        return Err(format!("invalid rule: {line}"));
    }
    Ok(Statement::Rule {
        name: words[1].into(),
        premises: identifiers(&words[3..arrow]),
        conclusion: words[arrow + 1].into(),
    })
}

fn parse_choice(words: &[&str]) -> Result<Statement, String> {
    if words.len() < 4 || words.get(2) != Some(&"options") {
        return Err("invalid choice".into());
    }
    let retaining = words.iter().position(|word| *word == "retaining");
    let end = retaining.unwrap_or(words.len());
    Ok(Statement::Choice {
        name: words[1].into(),
        options: identifiers(&words[3..end]),
        retaining: retaining
            .map(|index| identifiers(&words[index + 1..]))
            .unwrap_or_default(),
    })
}

fn identifiers(values: &[&str]) -> Vec<String> {
    values
        .join(" ")
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .collect()
}

fn number(value: &str) -> Result<u64, String> {
    value
        .parse()
        .map_err(|_| format!("invalid resource amount: {value}"))
}

fn parse_consequence(value: &str) -> Result<Consequence, String> {
    match value {
        "negligible" => Ok(Consequence::Negligible),
        "low" => Ok(Consequence::Low),
        "material" => Ok(Consequence::Material),
        "high" => Ok(Consequence::High),
        "catastrophic" => Ok(Consequence::Catastrophic),
        _ => Err(format!("unknown consequence: {value}")),
    }
}

fn parse_reason(value: &str) -> Result<StopReason, String> {
    match value {
        "enough" => Ok(StopReason::Enough),
        "budget" => Ok(StopReason::BudgetExhausted),
        "deadline" => Ok(StopReason::Deadline),
        _ => Err(format!("unknown stop reason: {value}")),
    }
}
