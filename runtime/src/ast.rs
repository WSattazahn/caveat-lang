use crate::{Attention,Consequence,Relation,StopReason};
#[derive(Debug,Clone,PartialEq,Eq)]pub struct Program{pub statements:Vec<Statement>}
#[derive(Debug,Clone,PartialEq,Eq)]pub enum ConditionalAction{Reopen{commitment:String,because:String},Relate{from:String,relation:Relation,to:String}}
#[derive(Debug,Clone,PartialEq,Eq)]pub enum Statement{Budget{units:u64},Claim{name:String},Evidence{name:String,source:String},Caveat{name:String,consequence:Consequence},Relate{from:String,relation:Relation,to:String},Attention{caveat:String,state:Attention},Examine{caveat:String,cost:u64},Rule{name:String,premises:Vec<String>,conclusion:String},Infer{rule:String},Choice{name:String,options:Vec<String>,retaining:Vec<String>},Select{choice:String,option:String},WhenCommitted{action:String,then:ConditionalAction},Commit{action:String,reason:StopReason,retaining:Vec<String>},Reopen{commitment:String,because:String}}
impl Program{pub fn new(statements:Vec<Statement>)->Self{Self{statements}}}
