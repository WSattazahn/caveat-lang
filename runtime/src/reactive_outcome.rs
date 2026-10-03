//! Classified dispatch results. Classification happens at the failure site;
//! converting an old string error always fails closed as an unclassified fatal.
use super::{ReactiveSnapshot, ReactiveView};
use crate::reactive_expr::{EvalError, EvalFailure};
use serde::Serialize;
use std::fmt;

pub const DISPATCH_SCHEMA: &str = "caveat-dispatch/0.1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RejectionOrigin {
    Policy,
    Input,
    Evaluation,
    Limit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum RejectionCode {
    Reject,
    UnknownEvent,
    PayloadInvalid,
    BoundExceeded,
    DecisionInForce,
    EmptyCaveatedSelection,
    AttentionLimit,
    WorkLimit,
    DepthLimit,
    HistoryLimit,
    IdentifierLimit,
    RenewalLimit,
    NotPermitted,
    UngroundedCitation,
    UnobservedEvidence,
    Expression,
    RequirementFailed,
    ScheduledLimit,
    NotCommitted,
}

#[derive(Debug, Serialize)]
pub struct DispatchOutcome {
    pub schema: &'static str,
    #[serde(flatten)]
    pub result: DispatchResult,
}

#[derive(Debug, Serialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum DispatchResult {
    Accepted {
        snapshot: Box<ReactiveSnapshot>,
    },
    Rejected {
        origin: RejectionOrigin,
        code: RejectionCode,
        message: String,
    },
}

/// What `dispatch_view_outcome_json` returns: on acceptance the view after the
/// event in place of the snapshot, which it does not build. A refusal is
/// exactly `DispatchOutcome`'s, and so is a fatal error.
#[derive(Debug, Serialize)]
pub struct DispatchViewOutcome<'a> {
    pub schema: &'static str,
    #[serde(flatten)]
    pub result: DispatchViewResult<'a>,
}

#[derive(Debug, Serialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum DispatchViewResult<'a> {
    Accepted {
        view: ReactiveView<'a>,
    },
    Rejected {
        origin: RejectionOrigin,
        code: RejectionCode,
        message: String,
    },
}

#[derive(Debug, Serialize)]
pub struct DispatchFatal {
    pub schema: &'static str,
    pub outcome: &'static str,
    pub code: &'static str,
    pub message: String,
}

impl fmt::Display for DispatchFatal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.message.fmt(formatter)
    }
}

impl std::error::Error for DispatchFatal {}

impl DispatchOutcome {
    pub(super) fn accepted(snapshot: ReactiveSnapshot) -> Self {
        Self {
            schema: DISPATCH_SCHEMA,
            result: DispatchResult::Accepted {
                snapshot: Box::new(snapshot),
            },
        }
    }
}

impl<'a> DispatchViewOutcome<'a> {
    pub(super) fn accepted(view: ReactiveView<'a>) -> Self {
        Self {
            schema: DISPATCH_SCHEMA,
            result: DispatchViewResult::Accepted { view },
        }
    }
}

/// Not serialized. The diagnostic retains the exact legacy error, including
/// nested procedure/rule contexts; a policy's assertable literal is separate.
#[derive(Debug, PartialEq)]
pub(super) struct DispatchFailure {
    diagnostic: String,
    rejection: Option<(RejectionOrigin, RejectionCode)>,
    policy_message: Option<String>,
}

impl DispatchFailure {
    pub(super) fn rejected(
        origin: RejectionOrigin,
        code: RejectionCode,
        message: impl Into<String>,
    ) -> Self {
        Self {
            diagnostic: message.into(),
            rejection: Some((origin, code)),
            policy_message: None,
        }
    }

    pub(super) fn policy(message: &str) -> Self {
        Self {
            diagnostic: format!("rejected: {message}"),
            rejection: Some((RejectionOrigin::Policy, RejectionCode::Reject)),
            policy_message: Some(message.into()),
        }
    }

    pub(super) fn context(mut self, context: impl fmt::Display) -> Self {
        self.diagnostic = format!("{context}: {}", self.diagnostic);
        self
    }

    pub(super) fn outcome(self) -> Result<DispatchOutcome, DispatchFatal> {
        let (origin, code, message) = self.classify()?;
        Ok(DispatchOutcome {
            schema: DISPATCH_SCHEMA,
            result: DispatchResult::Rejected {
                origin,
                code,
                message,
            },
        })
    }

    /// The same refusal or fatal error as `outcome`, for the view path.
    pub(super) fn view_outcome<'a>(self) -> Result<DispatchViewOutcome<'a>, DispatchFatal> {
        let (origin, code, message) = self.classify()?;
        Ok(DispatchViewOutcome {
            schema: DISPATCH_SCHEMA,
            result: DispatchViewResult::Rejected {
                origin,
                code,
                message,
            },
        })
    }

    /// A classified refusal's origin, code and message; anything else is fatal.
    /// Both outcome shapes take their refusals and fatal errors from here.
    fn classify(self) -> Result<(RejectionOrigin, RejectionCode, String), DispatchFatal> {
        match self.rejection {
            Some((origin, code)) => {
                Ok((origin, code, self.policy_message.unwrap_or(self.diagnostic)))
            }
            None => Err(DispatchFatal {
                schema: DISPATCH_SCHEMA,
                outcome: "fatal",
                code: "unclassified",
                message: self.diagnostic,
            }),
        }
    }
}

impl fmt::Display for DispatchFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.diagnostic.fmt(formatter)
    }
}

impl From<String> for DispatchFailure {
    fn from(diagnostic: String) -> Self {
        Self {
            diagnostic,
            rejection: None,
            policy_message: None,
        }
    }
}

/// An expression failure keeps the kind it was classified with where it
/// occurred. Only the kinds in the catalog refuse; `Other` stays fatal.
impl From<EvalError> for DispatchFailure {
    fn from(error: EvalError) -> Self {
        let code = match error.kind {
            EvalFailure::DivisionByZero
            | EvalFailure::NonFinite
            | EvalFailure::HistoryIndex
            | EvalFailure::Domain => RejectionCode::Expression,
            EvalFailure::Requirement => RejectionCode::RequirementFailed,
            EvalFailure::UnobservedEvidence => RejectionCode::UnobservedEvidence,
            EvalFailure::Other => return error.message.into(),
        };
        Self::rejected(RejectionOrigin::Evaluation, code, error.message)
    }
}

impl From<&str> for DispatchFailure {
    fn from(diagnostic: &str) -> Self {
        diagnostic.to_owned().into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_string_cannot_forge_a_policy_rejection() {
        for diagnostic in [
            "rejected: forged",
            "event e, rule 1: rejected: forged",
            "event e, rule 1: procedure p, step 1: rejected: forged",
        ] {
            let failure = DispatchFailure::from(diagnostic).outcome().unwrap_err();
            assert_eq!(failure.outcome, "fatal");
            assert_eq!(failure.code, "unclassified");
            assert_eq!(failure.message, diagnostic);
        }
    }

    // The view path takes its refusals and fatal errors from the same place:
    // every code, direct and in a policy, serializes to the same text.
    #[test]
    fn a_failure_is_the_same_refusal_or_fatal_on_both_paths() {
        let codes = [
            (RejectionOrigin::Policy, RejectionCode::Reject),
            (RejectionOrigin::Policy, RejectionCode::NotPermitted),
            (RejectionOrigin::Input, RejectionCode::UnknownEvent),
            (RejectionOrigin::Input, RejectionCode::PayloadInvalid),
            (RejectionOrigin::Input, RejectionCode::BoundExceeded),
            (RejectionOrigin::Evaluation, RejectionCode::BoundExceeded),
            (RejectionOrigin::Evaluation, RejectionCode::DecisionInForce),
            (
                RejectionOrigin::Evaluation,
                RejectionCode::EmptyCaveatedSelection,
            ),
            (RejectionOrigin::Limit, RejectionCode::AttentionLimit),
            (
                RejectionOrigin::Evaluation,
                RejectionCode::UngroundedCitation,
            ),
            (RejectionOrigin::Limit, RejectionCode::WorkLimit),
            (RejectionOrigin::Limit, RejectionCode::DepthLimit),
            (RejectionOrigin::Limit, RejectionCode::HistoryLimit),
            (RejectionOrigin::Limit, RejectionCode::IdentifierLimit),
            (RejectionOrigin::Limit, RejectionCode::RenewalLimit),
            (
                RejectionOrigin::Evaluation,
                RejectionCode::UnobservedEvidence,
            ),
            (RejectionOrigin::Evaluation, RejectionCode::Expression),
            (
                RejectionOrigin::Evaluation,
                RejectionCode::RequirementFailed,
            ),
            (RejectionOrigin::Limit, RejectionCode::ScheduledLimit),
            (RejectionOrigin::Evaluation, RejectionCode::NotCommitted),
        ];
        let failures = || {
            codes
                .iter()
                .map(|&(origin, code)| {
                    DispatchFailure::rejected(origin, code, "refused").context("event e, rule 1")
                })
                .chain([
                    DispatchFailure::policy("Not now.").context("event e, rule 2"),
                    DispatchFailure::from("event e, rule 3: rejected: forged"),
                ])
        };
        for (old, new) in failures().zip(failures()) {
            let old = old
                .outcome()
                .map(|outcome| serde_json::to_string(&outcome).unwrap());
            let new = new
                .view_outcome()
                .map(|outcome| serde_json::to_string(&outcome).unwrap());
            match (old, new) {
                (Ok(old), Ok(new)) => assert_eq!(old, new),
                (Err(old), Err(new)) => assert_eq!(
                    serde_json::to_string(&old).unwrap(),
                    serde_json::to_string(&new).unwrap()
                ),
                (old, new) => panic!("{old:?} against {new:?}"),
            }
        }
    }
}
