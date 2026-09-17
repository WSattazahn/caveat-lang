use crate::ast::{Program,Statement};

#[derive(Debug,Clone,PartialEq,Eq)]
pub enum PendingInteraction{Investigate{name:String,options:Vec<String>,cost:u64,budget:u64},Choice{name:String,options:Vec<String>,budget:u64},Complete}

#[derive(Debug,Clone)]
pub struct Session{program:Program,cursor:usize}
impl Session{
 pub fn from_source(source:&str)->Result<Self,String>{Ok(Self{program:crate::parser::parse(source)?,cursor:0})}
 pub fn pending(&self)->Result<PendingInteraction,String>{
  for i in self.cursor..self.program.statements.len(){match &self.program.statements[i]{
   Statement::Inspect{investigation,cost,..}=>{let e=crate::eval::evaluate(&Program::new(self.program.statements[..i].to_vec()))?;let inv=e.investigations.get(investigation).ok_or_else(||format!("unknown investigation: {investigation}"))?;return Ok(PendingInteraction::Investigate{name:investigation.clone(),options:inv.options.clone(),cost:*cost,budget:e.resources.as_ref().map(|b|b.remaining).unwrap_or(0)})},
   Statement::Select{choice,..}=>{let e=crate::eval::evaluate(&Program::new(self.program.statements[..i].to_vec()))?;let cp=e.choices.get(choice).ok_or_else(||format!("unknown choice: {choice}"))?;return Ok(PendingInteraction::Choice{name:choice.clone(),options:cp.options.clone(),budget:e.resources.as_ref().map(|b|b.remaining).unwrap_or(0)})},_=>{}}
  }Ok(PendingInteraction::Complete)
 }
 pub fn apply(&mut self,selection:&str)->Result<(),String>{for i in self.cursor..self.program.statements.len(){match &mut self.program.statements[i]{Statement::Inspect{investigation,caveat,..}=>{let prefix=crate::eval::evaluate(&Program::new(self.program.statements[..i].to_vec()))?;let inv=prefix.investigations.get(investigation).ok_or("unknown investigation")?;if !inv.options.iter().any(|x|x==selection){return Err(format!("invalid investigation selection: {selection}"))}*caveat=selection.into();self.cursor=i+1;return Ok(())},Statement::Select{choice,option}=>{let prefix=crate::eval::evaluate(&Program::new(self.program.statements[..i].to_vec()))?;let cp=prefix.choices.get(choice).ok_or("unknown choice")?;if !cp.options.iter().any(|x|x==selection){return Err(format!("invalid choice selection: {selection}"))}*option=selection.into();self.cursor=i+1;return Ok(())},_=>{}}}Err("session is complete".into())}
 pub fn program(&self)->&Program{&self.program}
}
