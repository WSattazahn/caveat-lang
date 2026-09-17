use crate::ast::{Program,Statement}; use crate::{Attention,Consequence,Relation,StopReason};
pub fn parse(source:&str)->Result<Program,String>{let mut s=vec![];for raw in source.split(';'){let l=raw.trim();if l.is_empty(){continue}let w:Vec<&str>=l.split_whitespace().collect();let x=match w.as_slice(){
["budget",n]=>Statement::Budget{units:num(n)?},
["claim",n]=>Statement::Claim{name:(*n).into()},
["evidence",n,"from",rest@..]if !rest.is_empty()=>Statement::Evidence{name:(*n).into(),source:rest.join(" ").trim_matches('"').into()},
["caveat",n,"consequence",c]=>Statement::Caveat{name:(*n).into(),consequence:cons(c)?},
[f,"supports",t]=>Statement::Relate{from:(*f).into(),relation:Relation::Supports,to:(*t).into()},[f,"opposes",t]=>Statement::Relate{from:(*f).into(),relation:Relation::Opposes,to:(*t).into()},[f,"qualifies",t]=>Statement::Relate{from:(*f).into(),relation:Relation::Qualifies,to:(*t).into()},
["examine",n,"cost",c]=>Statement::Examine{caveat:(*n).into(),cost:num(c)?},["examine",n]=>Statement::Attention{caveat:(*n).into(),state:Attention::Examining},["defer",n]=>Statement::Attention{caveat:(*n).into(),state:Attention::Deferred},
["commit",a,"because",r]=>Statement::Commit{action:(*a).into(),reason:reason(r)?,retaining:vec![]},["commit",a,"because",r,"retaining",rest@..]=>Statement::Commit{action:(*a).into(),reason:reason(r)?,retaining:rest.join(" ").split(',').map(|x|x.trim().to_string()).filter(|x|!x.is_empty()).collect()},["reopen",c,"because",b]=>Statement::Reopen{commitment:(*c).into(),because:(*b).into()},_=>return Err(format!("cannot parse statement: {l}"))};s.push(x)}Ok(Program::new(s))}
fn num(s:&str)->Result<u64,String>{s.parse().map_err(|_|format!("invalid resource amount: {s}"))}
fn cons(s:&str)->Result<Consequence,String>{match s{"negligible"=>Ok(Consequence::Negligible),"low"=>Ok(Consequence::Low),"material"=>Ok(Consequence::Material),"high"=>Ok(Consequence::High),"catastrophic"=>Ok(Consequence::Catastrophic),_=>Err(format!("unknown consequence: {s}"))}}
fn reason(s:&str)->Result<StopReason,String>{match s{"enough"=>Ok(StopReason::Enough),"budget"=>Ok(StopReason::BudgetExhausted),"deadline"=>Ok(StopReason::Deadline),_=>Err(format!("unknown stop reason: {s}"))}}
