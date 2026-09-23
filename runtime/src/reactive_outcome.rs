//! Classified dispatch results. Classification happens at the failure site;
//! converting an old string error always fails closed as an unclassified fatal.
use super::ReactiveSnapshot;
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
    WorkLimit,
    DepthLimit,
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

/// Not serialized. The diagnostic retains the exact legacy error, including
/// nested procedure/rule contexts; a policy's assertable literal is separate.
#[derive(Debug)]
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
        match self.rejection {
            Some((origin, code)) => Ok(DispatchOutcome {
                schema: DISPATCH_SCHEMA,
                result: DispatchResult::Rejected {
                    origin,
                    code,
                    message: self.policy_message.unwrap_or(self.diagnostic),
                },
            }),
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
}
