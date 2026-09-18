use crate::ast::{Program, Statement};
use crate::eval::{Evaluation, EventKind};
use crate::NodeId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PendingInteraction {
    Investigate {
        name: String,
        options: Vec<String>,
        cost: u64,
        budget: u64,
    },
    Choice {
        name: String,
        options: Vec<String>,
        budget: u64,
    },
    Complete,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Discovery {
    pub because: String,
    pub evidence: String,
    pub relation: String,
    pub target: String,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitmentFeedback {
    pub action: String,
    pub retained: Vec<String>,
    pub reopened_by: Vec<String>,
}
#[derive(Debug, Clone)]
pub struct Session {
    program: Program,
    cursor: usize,
    last_discoveries: Vec<Discovery>,
    last_commitment: Option<CommitmentFeedback>,
}

impl Session {
    pub fn from_source(source: &str) -> Result<Self, String> {
        Ok(Self {
            program: crate::parser::parse(source)?,
            cursor: 0,
            last_discoveries: Vec::new(),
            last_commitment: None,
        })
    }

    fn next_interaction_index(&self) -> Option<usize> {
        (self.cursor..self.program.statements.len())
            .find(|&index| is_interaction(&self.program.statements[index]))
    }

    fn evaluate_before(&self, index: usize) -> Result<Evaluation, String> {
        crate::eval::evaluate(&Program::new(self.program.statements[..index].to_vec()))
    }

    pub fn pending(&self) -> Result<PendingInteraction, String> {
        let Some(index) = self.next_interaction_index() else {
            return Ok(PendingInteraction::Complete);
        };
        let evaluation = self.evaluate_before(index)?;
        let budget = evaluation
            .resources
            .as_ref()
            .map(|ledger| ledger.remaining)
            .unwrap_or(0);
        match &self.program.statements[index] {
            Statement::Inspect {
                investigation,
                cost,
                ..
            } => {
                let value = evaluation
                    .investigations
                    .get(investigation)
                    .ok_or_else(|| format!("unknown investigation: {investigation}"))?;
                Ok(PendingInteraction::Investigate {
                    name: investigation.clone(),
                    options: value.options.clone(),
                    cost: *cost,
                    budget,
                })
            }
            Statement::Select { choice, .. } => {
                let value = evaluation
                    .choices
                    .get(choice)
                    .ok_or_else(|| format!("unknown choice: {choice}"))?;
                Ok(PendingInteraction::Choice {
                    name: choice.clone(),
                    options: value.options.clone(),
                    budget,
                })
            }
            _ => unreachable!("next_interaction_index only returns interactive statements"),
        }
    }

    pub fn apply(&mut self, selection: &str) -> Result<(), String> {
        self.last_discoveries.clear();
        self.last_commitment = None;
        let index = self.next_interaction_index().ok_or("session is complete")?;
        let prefix = self.evaluate_before(index)?;
        self.validate_and_set_selection(index, selection, &prefix)?;
        let next = (index + 1..self.program.statements.len())
            .find(|&candidate| is_interaction(&self.program.statements[candidate]))
            .unwrap_or(self.program.statements.len());
        let evaluation =
            crate::eval::evaluate(&Program::new(self.program.statements[..next].to_vec()))?;
        self.collect_feedback(selection, &evaluation);
        self.cursor = index + 1;
        Ok(())
    }

    fn validate_and_set_selection(
        &mut self,
        index: usize,
        selection: &str,
        evaluation: &Evaluation,
    ) -> Result<(), String> {
        match self.program.statements[index].clone() {
            Statement::Inspect { investigation, .. } => {
                let value = evaluation
                    .investigations
                    .get(&investigation)
                    .ok_or_else(|| format!("unknown investigation: {investigation}"))?;
                if !value.options.iter().any(|option| option == selection) {
                    return Err(format!("invalid investigation selection: {selection}"));
                }
                if let Statement::Inspect { caveat, .. } = &mut self.program.statements[index] {
                    *caveat = selection.into();
                }
            }
            Statement::Select { choice, .. } => {
                let value = evaluation
                    .choices
                    .get(&choice)
                    .ok_or_else(|| format!("unknown choice: {choice}"))?;
                if !value.options.iter().any(|option| option == selection) {
                    return Err(format!("invalid choice selection: {selection}"));
                }
                if let Statement::Select { option, .. } = &mut self.program.statements[index] {
                    *option = selection.into();
                }
            }
            _ => unreachable!("selection target must be interactive"),
        }
        Ok(())
    }

    fn collect_feedback(&mut self, selection: &str, evaluation: &Evaluation) {
        let name = |id: NodeId| symbol_name(evaluation, id);
        let mut committed_id = None;
        for event in &evaluation.history {
            match &event.kind {
                EventKind::Revealed {
                    because,
                    from,
                    relation,
                    to,
                } if name(*because) == selection => self.last_discoveries.push(Discovery {
                    because: name(*because),
                    evidence: name(*from),
                    relation: format!("{relation:?}").to_lowercase(),
                    target: name(*to),
                }),
                EventKind::Committed {
                    commitment,
                    retained,
                    ..
                } if name(*commitment) == selection => {
                    committed_id = Some(*commitment);
                    self.last_commitment = Some(CommitmentFeedback {
                        action: selection.into(),
                        retained: retained.iter().map(|id| name(*id)).collect(),
                        reopened_by: Vec::new(),
                    });
                }
                _ => {}
            }
        }
        if let (Some(commitment_id), Some(feedback)) = (committed_id, self.last_commitment.as_mut())
        {
            for event in &evaluation.history {
                if let EventKind::Reopened {
                    commitment,
                    because,
                } = event.kind
                {
                    if commitment == commitment_id {
                        feedback.reopened_by.push(name(because));
                    }
                }
            }
        }
    }

    pub fn discoveries(&self) -> &[Discovery] {
        &self.last_discoveries
    }
    pub fn commitment(&self) -> Option<&CommitmentFeedback> {
        self.last_commitment.as_ref()
    }
    pub fn program(&self) -> &Program {
        &self.program
    }
}

fn is_interaction(statement: &Statement) -> bool {
    matches!(
        statement,
        Statement::Inspect { .. } | Statement::Select { .. }
    )
}
fn symbol_name(evaluation: &Evaluation, id: NodeId) -> String {
    evaluation
        .symbols
        .iter()
        .find_map(|(name, value)| (*value == id).then(|| name.clone()))
        .unwrap_or_else(|| id.to_string())
}


#[cfg(test)]
mod tests {
    use super::Session;

    const SOURCE: &str = r#"
budget 2;
claim cat_near_pavilion;
evidence first_evidence from "first_source";
evidence second_evidence from "second_source";
caveat first_clue consequence material;
caveat second_clue consequence low;
investigate first_check options first_clue;
inspect first_check first_clue cost 1;
reveal first_clue then first_evidence supports cat_near_pavilion;
choice route options go retaining first_clue;
select route go;
investigate second_check options second_clue;
inspect second_check second_clue cost 1;
reveal second_clue then second_evidence supports cat_near_pavilion;
"#;

    #[test]
    fn feedback_reports_only_discoveries_from_the_current_interaction() {
        let mut session = Session::from_source(SOURCE).expect("session should parse");

        session.apply("first_clue").expect("first clue should apply");
        assert_eq!(session.discoveries().len(), 1);
        assert_eq!(session.discoveries()[0].because, "first_clue");

        session.apply("go").expect("choice should apply");
        assert!(
            session.discoveries().is_empty(),
            "a later choice must not replay an earlier discovery"
        );
        assert_eq!(
            session
                .commitment()
                .expect("choice should report commitment")
                .action,
            "go"
        );

        session.apply("second_clue").expect("second clue should apply");
        assert_eq!(session.discoveries().len(), 1);
        assert_eq!(session.discoveries()[0].because, "second_clue");
    }
}
