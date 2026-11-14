//! The releases of 1.x, as far as they read programs differently: at each
//! minor that added a form, or changed how a bundle links its modules.
//!
//! A bundle that releases linked in different ways is read as the newest
//! release that could load it linked it (finding D088). That release is
//! found from the program as each reading loads it: the first release that
//! reads every form it uses (`Expr::first_release`, the effects and the
//! declarations in `reactive_releases.rs`), and the last that read every
//! function body as this runtime reads it again, which later releases
//! refused (`reactive_expr::validate_functions_reading`). Each form's first
//! release was found by loading a program that uses it on every release from
//! rc.1 to 1.4.1.

/// A line of releases that read programs alike, oldest first.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Release {
    /// rc.1 to 1.0.3: no form these lines name. Forms the release
    /// candidates added are read alike from 1.0.0 on.
    Before1_1,
    /// 1.1.0: `changed` triggers, occurrences, readings and `latest(…)` as
    /// targets, past revisions, `in_basis`, `citing`, retention reasons,
    /// neutral samples and notes.
    V1_1,
    /// 1.2.0: holds, irreversible series and what they foreclose, nested
    /// `for` blocks.
    V1_2,
    /// 1.3.0: outcomes.
    V1_3,
    /// 1.4.0 and 1.4.1: agents, `withdraw every`, `because once`,
    /// `only_if`, `carries_any`, foreclosure by a series that can be undone,
    /// `irreversible strict`.
    V1_4,
    /// After 1.4.1: what this runtime reads that no release did.
    Later,
}
