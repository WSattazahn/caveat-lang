use crate::ast::{Program, Statement};
use crate::{Attention, Consequence, Relation, StopReason};

pub fn parse(source: &str) -> Result<Program, String> {
    let mut statements = Vec::new();
    for raw in source.split(';') {
        let line = raw.trim();
        if line.is_empty() { continue; }
        let words: Vec<&str> = line.split_whitespace().collect();
        let stmt = match words.as_slice() {
            ["claim", name] => Statement::Claim { name: (*name).into() },
            ["evidence", name, "from", rest @ ..] if !rest.is_empty() => Statement::Evidence { name: (*name).into(), source: rest.join(" ").trim_matches('"').into() },
            ["caveat", name, "consequence", level] => Statement::Caveat { name: (*name).into(), consequence: consequence(level)? },
            [from, "supports", to] => Statement::Relate { from: (*from).into(), relation: Relation::Supports, to: (*to).into() },
            [from, "opposes", to] => Statement::Relate { from: (*from).into(), relation: Relation::Opposes, to: (*to).into() },
            [from, "qualifies", to] => Statement::Relate { from: (*from).into(), relation: Relation::Qualifies, to: (*to).into() },
            ["examine", name] => Statement::Attention { caveat: (*name).into(), state: Attention::Examining },
            ["defer", name] => Statement::Attention { caveat: (*name).into(), state: Attention::Deferred },
            ["commit", action, "because", reason] => Statement::Commit { action: (*action).into(), reason: stop_reason(reason)?, retaining: vec![] },
            ["commit", action, "because", reason, "retaining", rest @ ..] => Statement::Commit { action: (*action).into(), reason: stop_reason(reason)?, retaining: rest.join(" ").split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect() },
            ["reopen", commitment, "because", because] => Statement::Reopen { commitment: (*commitment).into(), because: (*because).into() },
            _ => return Err(format!("cannot parse statement: {line}")),
        };
        statements.push(stmt);
    }
    Ok(Program::new(statements))
}

fn consequence(s: &str) -> Result<Consequence, String> { match s { "negligible"=>Ok(Consequence::Negligible), "low"=>Ok(Consequence::Low), "material"=>Ok(Consequence::Material), "high"=>Ok(Consequence::High), "catastrophic"=>Ok(Consequence::Catastrophic), _=>Err(format!("unknown consequence: {s}")) } }
fn stop_reason(s: &str) -> Result<StopReason, String> { match s { "enough"=>Ok(StopReason::Enough), "budget"=>Ok(StopReason::BudgetExhausted), "deadline"=>Ok(StopReason::Deadline), _=>Err(format!("unknown stop reason: {s}")) } }

#[cfg(test)]
mod tests {
 use super::*;
 #[test] fn parses_eternal_caveat_program(){let src="claim release_ready; caveat edge_1 consequence low; edge_1 qualifies release_ready; commit ship_v1 because enough retaining edge_1;";let p=parse(src).unwrap();assert_eq!(p.statements.len(),4);}
}
