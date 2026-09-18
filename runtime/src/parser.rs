use crate::ast::{ConditionalAction, Program, Statement};
use crate::{Attention, Consequence, Relation, StopReason};

pub fn parse(source: &str) -> Result<Program, String> {
    let mut statements = Vec::new();
    for raw in source.split(';') {
        let line = raw.trim();
        if line.is_empty() {
            continue;
        }
        let words: Vec<&str> = line.split_whitespace().collect();
        let statement = if line.starts_with("scene ") {
            Statement::Scene {
                text: quoted(line, "scene ")?,
            }
        } else if line.starts_with("display ") {
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
        } else {
            match words.as_slice() {
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
                ["claim", name] => Statement::Claim {
                    name: (*name).into(),
                },
                ["evidence", name, "from", rest @ ..] if !rest.is_empty() => Statement::Evidence {
                    name: (*name).into(),
                    source: rest.join(" ").trim_matches('"').into(),
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
                ["select", choice, option] => Statement::Select {
                    choice: (*choice).into(),
                    option: (*option).into(),
                },
                ["commit", action, "because", reason] => Statement::Commit {
                    action: (*action).into(),
                    reason: parse_reason(reason)?,
                    retaining: Vec::new(),
                },
                ["commit", action, "because", reason, "retaining", rest @ ..] => {
                    Statement::Commit {
                        action: (*action).into(),
                        reason: parse_reason(reason)?,
                        retaining: identifiers(rest),
                    }
                }
                ["reopen", commitment, "because", because] => Statement::Reopen {
                    commitment: (*commitment).into(),
                    because: (*because).into(),
                },
                _ => return Err(format!("cannot parse statement: {line}")),
            }
        };
        statements.push(statement);
    }
    Ok(Program::new(statements))
}

fn relation(from: &str, relation: Relation, to: &str) -> Statement {
    Statement::Relate {
        from: from.into(),
        relation,
        to: to.into(),
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

fn quoted(line: &str, prefix: &str) -> Result<String, String> {
    let value = line.strip_prefix(prefix).unwrap().trim();
    if value.len() >= 2 && value.starts_with('"') && value.ends_with('"') {
        Ok(value[1..value.len() - 1].to_string())
    } else {
        Err(format!("expected quoted text: {line}"))
    }
}

fn parse_display(line: &str) -> Result<Statement, String> {
    let rest = line.strip_prefix("display ").unwrap();
    let mut parts = rest.splitn(2, ' ');
    let symbol = parts.next().unwrap();
    let text = parts
        .next()
        .ok_or_else(|| format!("display missing text: {line}"))?;
    Ok(Statement::Display {
        symbol: symbol.into(),
        text: quoted(&format!("x {text}"), "x ")?,
    })
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
