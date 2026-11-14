//! Which releases could have loaded a bundle as it loads here, and so how it
//! is read (finding D088). Releases linked bundles in different ways
//! (`link::Reading`), and a bundle is read as the newest release that could
//! load it linked it: a release could load it as it linked it only when the
//! program uses no form that came after that release, and no function body
//! that a release from that one on refused.
//!
//! What the loaded program keeps is looked at: its declarations, rules,
//! procedures as they loaded, bindings, defines and state initial values,
//! and every function body (`reactive_expr::validate_functions_reading`).
//! A template no call specializes is not, nor a nested `for` block or an
//! across-members form, which leave nothing once expanded, nor an expansion
//! larger than a release allowed; the same places finding D089 lists for
//! `may_load_before_rc12`.

use super::{
    BindingExpression, DecisionChange, Effect, EvidenceSelector, ModuleBindings, ReactiveSession,
    Release,
};
use crate::link::{self, Reading};
use crate::reactive_expr::Expr;
use std::cell::RefCell;

/// A bundle as each line of releases linked it, and the programs those
/// readings load. With a module's `hold` and `release` its own, a bundle is
/// read as the newest release that could load it linked it, corrected so;
/// with the program's, as the newest from 1.2.0 to 1.4.1 that could, where a
/// save says such a release wrote it (finding D091).
pub(super) struct Readings<'a> {
    source: &'a str,
    now: Result<String, String>,
    v1_4: Result<String, String>,
    v1_3: Result<String, String>,
    /// 1.3.x corrected: as 1.3.x linked it, with a module's `hold` and
    /// `release` its own.
    v1_3_scoped: Result<String, String>,
    v1_2: Result<String, String>,
    v1_1: Result<String, String>,
    v1_0: Result<String, String>,
    /// 1.1.0 and the releases before it, with the program's `hold` and
    /// `release` as from 1.2.0: no release read a bundle so. Compared with
    /// their readings, they say whether the program's `hold` and `release`
    /// would change how those releases read it.
    v1_1_captured: Result<String, String>,
    v1_0_captured: Result<String, String>,
    /// Whether a module's `hold` or `release` reads differently where the
    /// program declares the name, so that a save says which reading wrote
    /// it.
    pub(super) affected: bool,
    /// The programs loaded so far, by linked text.
    loaded: RefCell<Vec<(String, Result<ReactiveSession, String>)>>,
}

impl<'a> Readings<'a> {
    pub(super) fn new(source: &'a str) -> Self {
        let now = link::link_reading(source, Reading::NOW);
        let v1_4 = link::link_reading(source, Reading::V1_4);
        // The older readings link a bundle as these do unless a module
        // declares `only_if`, or a word that a later minor's clause keeps by
        // position (`changed`, `resolve`, `materialized`, `strict`, `every`,
        // `once`), whose names are then in the linked text, or `irreversible`
        // or `forecloses`, which these refuse.
        let older = now.as_ref().map_or(true, |text| {
            [
                "only_if",
                "changed",
                "resolve",
                "materialized",
                "strict",
                "every",
                "once",
            ]
            .iter()
            .any(|word| text.contains(word))
        });
        let (v1_3, v1_3_scoped, v1_2, v1_1, v1_0, v1_1_captured, v1_0_captured) = if older {
            (
                link::link_reading(source, Reading::V1_3),
                link::link_reading(source, Reading::V1_3.scoped()),
                link::link_reading(source, Reading::V1_2),
                link::link_reading(source, Reading::V1_1),
                link::link_reading(source, Reading::V1_0),
                link::link_reading(source, Reading::V1_1.captured()),
                link::link_reading(source, Reading::V1_0.captured()),
            )
        } else {
            (
                v1_4.clone(),
                now.clone(),
                v1_4.clone(),
                now.clone(),
                now.clone(),
                v1_4.clone(),
                v1_4.clone(),
            )
        };
        // Compare link text, not current values: two readings can have
        // identical initial state and diverge only on a later event. Even
        // when one of them does not load today, saves say which wrote them,
        // so a later compiler cannot make a save ambiguous.
        let affected = now.as_ref().ok() != v1_4.as_ref().ok()
            || v1_3_scoped.as_ref().ok() != v1_3.as_ref().ok()
            || v1_1.as_ref().ok() != v1_1_captured.as_ref().ok()
            || v1_0.as_ref().ok() != v1_0_captured.as_ref().ok();
        Self {
            source,
            now,
            v1_4,
            v1_3,
            v1_3_scoped,
            v1_2,
            v1_1,
            v1_0,
            v1_1_captured,
            v1_0_captured,
            affected,
            loaded: RefCell::default(),
        }
    }

    /// The session of the bundle as `bindings` reads it, saying so in its
    /// saves where the readings differ.
    pub(super) fn load(&self, bindings: ModuleBindings) -> Result<ReactiveSession, String> {
        let linked = match bindings {
            ModuleBindings::Scoped => self.scoped_reading(),
            ModuleBindings::LegacyHoldCapture => self.legacy_reading().ok_or_else(|| {
                "cannot restore save: no release from 1.2.0 to 1.4.1 could load this source, so its moduleBindings cannot be legacy-hold-capture".to_string()
            })?,
        };
        let mut session = self.program(linked)?;
        session.module_bindings = self.affected.then_some(bindings);
        Ok(session)
    }

    /// With a module's names its own: as 1.4 read it, corrected, where 1.4
    /// could load it and it loads so; else as 1.3.x read it, corrected,
    /// likewise; else 1.2.0, whose reading corrected is 1.1.0's for a bundle
    /// 1.2.0 could load; else as 1.1.0 or the releases before it read it,
    /// where they could load it; else as this runtime reads it.
    pub(super) fn scoped_reading(&self) -> &Result<String, String> {
        if self.by_1_4() && self.program(&self.now).is_ok() {
            &self.now
        } else if self.by_1_3() && self.program(&self.v1_3_scoped).is_ok() {
            &self.v1_3_scoped
        } else if self.by_1_2() && self.program(&self.v1_1).is_ok() || self.by_1_1() {
            &self.v1_1
        } else if self.by_1_0() {
            &self.v1_0
        } else {
            &self.now
        }
    }

    /// With the program's `hold` and `release` where it declares the name:
    /// as 1.4 read it where it could load it, else as 1.3.x read it where
    /// they could, else as 1.2.0 read it where it could, else, where no
    /// release could load it, as 1.4 links it. Where only releases before
    /// 1.2.0 could load it, none read it so: it is read as they read it
    /// where the program's `hold` and `release` would change nothing, and
    /// is `None` where they would.
    pub(super) fn legacy_reading(&self) -> Option<&Result<String, String>> {
        if self.by_1_4() {
            Some(&self.v1_4)
        } else if self.by_1_3() {
            Some(&self.v1_3)
        } else if self.by_1_2() {
            Some(&self.v1_2)
        } else if self.by_1_1() {
            (self.v1_1.as_ref().ok() == self.v1_1_captured.as_ref().ok()).then_some(&self.v1_1)
        } else if self.by_1_0() {
            (self.v1_0.as_ref().ok() == self.v1_0_captured.as_ref().ok()).then_some(&self.v1_0)
        } else {
            Some(&self.v1_4)
        }
    }

    /// Whether a release before 1.2.0 could have loaded the bundle and
    /// written a save of it, and whether one from 1.2.0 to 1.4.1 could.
    pub(super) fn writers(&self) -> (bool, bool) {
        (
            self.by_1_1() || self.by_1_0(),
            self.by_1_4() || self.by_1_3() || self.by_1_2(),
        )
    }

    fn by_1_4(&self) -> bool {
        self.could_load(&self.v1_4, Release::V1_4, Release::V1_4)
    }

    fn by_1_3(&self) -> bool {
        self.could_load(&self.v1_3, Release::V1_3, Release::V1_3)
    }

    fn by_1_2(&self) -> bool {
        self.could_load(&self.v1_2, Release::V1_2, Release::V1_2)
    }

    fn by_1_1(&self) -> bool {
        self.could_load(&self.v1_1, Release::V1_1, Release::V1_1)
    }

    fn by_1_0(&self) -> bool {
        self.could_load(&self.v1_0, Release::Before1_1, Release::Before1_1)
    }

    /// Whether `linked` loads as a program a release from `first` to `last`
    /// could have loaded.
    fn could_load(&self, linked: &Result<String, String>, first: Release, last: Release) -> bool {
        self.program(linked).is_ok_and(|session| {
            let (from, to) = session.release_bounds();
            from.max(first) <= to.min(last)
        })
    }

    /// The program `linked` loads.
    fn program(&self, linked: &Result<String, String>) -> Result<ReactiveSession, String> {
        let linked = linked.as_ref().map_err(String::clone)?;
        if let Some((_, program)) = self.loaded.borrow().iter().find(|(text, _)| text == linked) {
            return program.clone();
        }
        let program = crate::parser::parse(linked)
            .and_then(|program| ReactiveSession::load_program(self.source, program, None));
        self.loaded
            .borrow_mut()
            .push((linked.clone(), program.clone()));
        program
    }
}

/// The first release that reads an effect: its form, and the expressions
/// it holds. The match is exhaustive, so a new effect must say which release
/// first read it.
fn effect_first_release(effect: &Effect) -> Release {
    let all = |expressions: &[Expr]| {
        expressions
            .iter()
            .map(Expr::first_release)
            .max()
            .unwrap_or(Release::Before1_1)
    };
    match effect {
        Effect::Call { arguments, .. } => all(arguments),
        Effect::Sample {
            value,
            stance,
            note,
            with,
            ..
        } => {
            let mut first = value.first_release();
            if stance.is_none() || note.is_some() {
                first = first.max(Release::V1_1);
            }
            if let Some(note) = note {
                first = first.max(note.first_release());
            }
            if let Some(with) = with {
                first = first.max(Release::Later);
                if let Some(condition) = &with.condition {
                    first = first.max(condition.first_release());
                }
            }
            first
        }
        Effect::Emit { .. }
        | Effect::Reject { .. }
        | Effect::Renew { .. }
        | Effect::Observe { .. }
        | Effect::Reveal { .. }
        | Effect::Examine { .. } => Release::Before1_1,
        Effect::Qualify {
            after,
            index,
            latest,
            ..
        } => after
            .as_ref()
            .map_or(Release::Before1_1, Expr::first_release)
            .max(target_first_release(index.as_deref(), *latest)),
        Effect::Unqualify { index, latest, .. } => target_first_release(index.as_deref(), *latest),
        Effect::Withdraw { target, .. } => selector_first_release(target),
        Effect::Unset { .. } => Release::Later,
        Effect::Set { value, because, .. } => {
            let mut first = value.first_release();
            if let Some(because) = because {
                first = first.max(all(because));
            }
            first
        }
        Effect::Commit {
            using,
            citing,
            permission,
            reasons,
            ..
        } => {
            let mut first = all(citing);
            if !citing.is_empty() || !reasons.is_empty() {
                first = first.max(Release::V1_1);
            }
            if reasons.iter().any(|(_, _, _, once)| *once) {
                first = first.max(Release::V1_4);
            }
            for (_, _, read, _) in reasons {
                first = first.max(read.first_release());
            }
            if let Some(using) = using {
                first = first.max(using.first_release());
            }
            if let Some(permission) = permission {
                first = first.max(selector_first_release(&permission.grant));
                if let Some(scope) = &permission.scope {
                    first = first.max(scope.first_release());
                }
            }
            first
        }
        Effect::Reopen { because, form, .. } => {
            let first = selector_first_release(because);
            match form {
                DecisionChange::Reopen => first,
                DecisionChange::Hold | DecisionChange::Release => first.max(Release::V1_2),
            }
        }
        Effect::Resolve(resolution) => {
            let mut first = Release::V1_3
                .max(resolution.value.first_release())
                .max(selector_first_release(&resolution.because));
            if let Some(revision) = &resolution.revision {
                first = first.max(revision.first_release());
            }
            first
        }
    }
}

/// The first release that reads what a `qualify` or `unqualify` names: an
/// occurrence or `latest(…)` from 1.1.0.
fn target_first_release(index: Option<&Expr>, latest: bool) -> Release {
    let mut first = Release::Before1_1;
    if let Some(index) = index {
        first = first.max(Release::V1_1).max(index.first_release());
    }
    if latest {
        first = first.max(Release::V1_1);
    }
    first
}

/// The first release that reads what an effect names as its evidence:
/// `occurrence(…)` and `reading(…)` from 1.1.0, `every` from 1.4.0.
fn selector_first_release(selector: &EvidenceSelector) -> Release {
    match selector {
        EvidenceSelector::Named(_)
        | EvidenceSelector::Latest(_)
        | EvidenceSelector::Caveated { .. } => Release::Before1_1,
        EvidenceSelector::Occurrence { index, .. } => Release::V1_1.max(index.first_release()),
        EvidenceSelector::Every(_) => Release::V1_4,
    }
}

impl ReactiveSession {
    /// The releases that could have loaded this program as it linked here:
    /// from the first that reads every form it keeps, to the last before one
    /// that refused a function body this runtime reads again (finding D088).
    /// The declarations: a reopening `changed` trigger came in 1.1.0, an
    /// irreversible series in 1.2.0, an agent, a reversible series that
    /// forecloses and `irreversible strict` in 1.4.0, and a threshold, a
    /// state declared `unset` and `work skips other routes` after 1.4.1.
    pub(super) fn release_bounds(&self) -> (Release, Release) {
        let (mut first, last) = self.loaded_releases;
        let mut declared = |release: Release, declares: bool| {
            if declares {
                first = first.max(release);
            }
        };
        declared(
            Release::V1_1,
            self.reopening_triggers
                .iter()
                .any(|(_, triggers, _)| triggers.iter().any(|trigger| trigger.changed)),
        );
        declared(Release::V1_2, !self.irreversible.is_empty());
        declared(
            Release::V1_4,
            !self.agents.is_empty()
                || !self.foreclosures.is_empty()
                || !self.strict_irreversible.is_empty(),
        );
        declared(
            Release::Later,
            self.has_thresholds
                || self.routed_work
                || self.ranges.values().any(|range| range.unset),
        );
        for rule in self.rules.iter() {
            first = first
                .max(rule.condition.first_release())
                .max(effect_first_release(&rule.effect));
        }
        for procedure in self.procedures.values() {
            for step in &procedure.body {
                first = first
                    .max(step.condition.first_release())
                    .max(effect_first_release(&step.effect));
            }
        }
        for binding in self.binding_rules.iter() {
            first = first.max(binding.condition.first_release());
            if let BindingExpression::Expression(value) = &binding.value {
                first = first.max(value.first_release());
            }
            for citation in binding.because.iter().flatten() {
                first = first.max(citation.first_release());
            }
        }
        for (_, expression) in self.define_rules.iter() {
            first = first.max(expression.first_release());
        }
        (first, last)
    }
}

#[cfg(test)]
mod tests {
    use super::super::{ModuleBindings, ReactiveSession, RestoreOptions};

    const ROOT: &str =
        "use m;\nclaim c;\nevidence e from \"e\";\nstate s = 0;\nevent go;\non go set s = s + 1;\n";

    fn bundle(module: &str, root: &str) -> String {
        format!(
            "#caveat-bundle 1\n#module m {}\n{module}#module main {}\n{root}",
            module.len(),
            root.len()
        )
    }

    /// A save `bindings` writes, unmarked, as a release before this runtime
    /// wrote it.
    fn unmarked(
        source: &str,
        bindings: ModuleBindings,
    ) -> (ReactiveSession, super::super::ReactiveSave) {
        let mut session = ReactiveSession::from_source_with_bindings(source, bindings)
            .unwrap_or_else(|error| panic!("{error}\n{source}"));
        session.dispatch_json("go", "{}").unwrap();
        let mut saved = session.save().unwrap();
        assert_eq!(saved.module_bindings, Some(bindings), "{source}");
        saved.module_bindings = None;
        (session, saved)
    }

    /// Only releases that bound a module's `hold` as the program's could
    /// load these bundles: 1.4.0 and 1.4.1 for the first two, with the
    /// `only_if` form and `irreversible strict`, and 1.2.0 and 1.3.0 for the
    /// last, whose `hold` the module declares as a claim. A fresh session
    /// does not inherit the capture, so it does not load them, but an
    /// unmarked save restores as those releases read it, without a choice
    /// (finding D091).
    #[test]
    fn a_save_only_releases_that_captured_hold_could_write_restores_without_a_choice() {
        let module = "module m;\nstate only_if = 0;\nstate hold = 0;\nstate u = 0;\nevent tick;\non tick set u = only_if( hold , because );\n";
        let captured = format!("{ROOT}state because = 1;\ndefine hold = s > 0;\n");
        for source in [
            bundle(
                module,
                &format!("{captured}fn m__only_if(c, v) = v;\non go set s = only_if(s > 0, 1);\n"),
            ),
            bundle(
                module,
                &format!("{captured}fn m__only_if(c, v) = v + 100;\ndecisions dd limit 1 irreversible strict;\n"),
            ),
            bundle(
                "module m;\nstate only_if = 0;\nclaim hold;\nstate u = 0;\nstate t = 0;\nevent tick;\non tick set u = only_if(1, 2);\nevent tap;\non tap set t = max(1, hold ) because nothing;\n",
                &format!("{ROOT}fn m__only_if(c, v) = v;\nstate hold = 5;\n"),
            ),
        ] {
            assert!(ReactiveSession::from_source(&source).is_err(), "{source}");
            let (session, saved) = unmarked(&source, ModuleBindings::LegacyHoldCapture);
            let restored = ReactiveSession::restore(&source, &saved).unwrap();
            assert_eq!(restored.snapshot(), session.snapshot());
            assert_eq!(
                restored.save().unwrap().module_bindings,
                Some(ModuleBindings::LegacyHoldCapture)
            );
        }
    }

    /// From 1.2.0 to 1.4.1 a module could not declare `irreversible`, so
    /// only releases before 1.2.0, which bound a module's `hold` as its own,
    /// could load this bundle: an unmarked save restores so without a
    /// choice, and the program's `hold` is no reading of it, so asking for
    /// that is refused.
    #[test]
    fn a_save_only_releases_before_1_2_could_write_restores_without_a_choice() {
        let source = bundle(
            "module m;\nevidence me from \"m\";\nreadings irreversible from me limit 2;\ndecisions d limit 3 reopened by irreversible;\nstate hold = 0;\nstate t = 0 min 0 max 4;\nevent tap;\non tap set t = max(1, hold ) because nothing;\n",
            &format!("{ROOT}state hold = 5;\n"),
        );
        let (session, saved) = unmarked(&source, ModuleBindings::Scoped);
        let mut restored = ReactiveSession::restore(&source, &saved).unwrap();
        assert_eq!(restored.snapshot(), session.snapshot());
        restored.dispatch_json("m__tap", "{}").unwrap();
        let values = serde_json::to_value(restored.snapshot()).unwrap()["values"].clone();
        assert_eq!(values["m__t"], serde_json::json!(1.0));
        let legacy = RestoreOptions {
            module_bindings: Some(ModuleBindings::LegacyHoldCapture),
        };
        assert_eq!(
            ReactiveSession::restore_with_options(&source, &saved, &legacy).unwrap_err(),
            "cannot restore save: no release from 1.2.0 to 1.4.1 could load this source, so its moduleBindings cannot be legacy-hold-capture"
        );
    }
}
