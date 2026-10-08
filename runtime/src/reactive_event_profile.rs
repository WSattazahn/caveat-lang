//! Native event-level attribution. Phase boundaries preserve production scope
//! and drop order; there are no per-record hooks. The origins build tracks the
//! same phase locations without reading a clock. Outside is an allocation tag,
//! not a timed phase: the harness reports outer elapsed minus measured phases.
use super::ReactiveSession;
use serde::Serialize;
use std::cell::Cell;
#[cfg(not(feature = "event-phase-origins"))]
use std::time::Instant;

pub const EVENT_PHASE_COUNT: usize = 11;
pub const EVENT_PHASE_NAMES: [&str; EVENT_PHASE_COUNT] = [
    "outside",
    "transaction_preparation",
    "evaluation",
    "collection",
    "compaction",
    "archive_construction_and_removal",
    "binding_evaluation",
    "final_settlement",
    "commit_cleanup",
    "rollback_cleanup",
    "apply_wrapper",
];

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(usize)]
pub enum EventPhase {
    Outside = 0,
    TransactionPreparation = 1,
    Evaluation = 2,
    Collection = 3,
    Compaction = 4,
    ArchiveConstructionAndRemoval = 5,
    BindingEvaluation = 6,
    FinalSettlement = 7,
    CommitCleanup = 8,
    RollbackCleanup = 9,
    ApplyWrapper = 10,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize)]
pub struct EventPhaseProfile {
    pub clock_enabled: bool,
    /// Exclusive elapsed slices, including resumed parent scopes. Outside is
    /// unclocked; its zero is not a claim that wrapper/harness residual is free.
    pub phase_ns: [u64; EVENT_PHASE_COUNT],
    /// Entries include resumed phases and the final transition to Outside.
    pub phase_entries: [u64; EVENT_PHASE_COUNT],
    pub transitions: u64,
}

impl EventPhaseProfile {
    const ZERO: Self = Self {
        clock_enabled: !cfg!(feature = "event-phase-origins"),
        phase_ns: [0; EVENT_PHASE_COUNT],
        phase_entries: [0; EVENT_PHASE_COUNT],
        transitions: 0,
    };
}

impl Default for EventPhaseProfile {
    fn default() -> Self {
        Self::ZERO
    }
}

#[derive(Copy, Clone)]
struct Recorder {
    enabled: bool,
    profile: EventPhaseProfile,
    #[cfg(not(feature = "event-phase-origins"))]
    since: Option<Instant>,
}

impl Recorder {
    const OFF: Self = Self {
        enabled: false,
        profile: EventPhaseProfile::ZERO,
        #[cfg(not(feature = "event-phase-origins"))]
        since: None,
    };
}

thread_local! {
    // Const Cell TLS has no allocator-using initializer or destructor. The
    // allocator reads only ACTIVE, never the larger recorder or a RefCell.
    static ACTIVE: Cell<EventPhase> = const { Cell::new(EventPhase::Outside) };
    static RECORDER: Cell<Recorder> = const { Cell::new(Recorder::OFF) };
}

/// Allocation-free phase query. TLS teardown/unavailability maps to Outside;
/// the allocator must not panic or initialize allocation-backed diagnostic data.
pub fn current_event_phase_index() -> usize {
    ACTIVE.try_with(Cell::get).map_or(0, |phase| phase as usize)
}

impl ReactiveSession {
    /// Start an event record before apply, outside all runtime profiling scopes.
    /// The recorder is thread local and never belongs to transactional state.
    pub fn begin_event_phase_profile() {
        ACTIVE.set(EventPhase::Outside);
        RECORDER.set(Recorder {
            enabled: true,
            ..Recorder::OFF
        });
    }

    /// Read and disable attribution after apply and the harness elapsed/peak
    /// captures. Rejected dispatch work survives destruction of staged state.
    pub fn take_event_phase_profile() -> EventPhaseProfile {
        transition(EventPhase::Outside);
        RECORDER.replace(Recorder::OFF).profile
    }
}

fn transition(next: EventPhase) -> Option<EventPhase> {
    let mut recorder = RECORDER.get();
    if !recorder.enabled {
        return None;
    }
    let previous = ACTIVE.get();
    if previous == next {
        return Some(previous);
    }
    #[cfg(not(feature = "event-phase-origins"))]
    {
        // One timestamp closes the old slice and starts the next. Hook work
        // before/after that boundary belongs to the adjacent old/new slice;
        // nested parent resumptions never overlap their child phases.
        let now = Instant::now();
        if previous != EventPhase::Outside {
            if let Some(since) = recorder.since {
                recorder.profile.phase_ns[previous as usize] +=
                    now.duration_since(since).as_nanos() as u64;
            }
        }
        recorder.since = (next != EventPhase::Outside).then_some(now);
    }
    recorder.profile.phase_entries[next as usize] += 1;
    recorder.profile.transitions += 1;
    RECORDER.set(recorder);
    ACTIVE.set(next);
    Some(previous)
}

/// Declare before all measured locals so implicit return/error destruction is
/// still attributed before the guard restores the caller's active phase.
pub(super) struct Scope {
    previous: Option<EventPhase>,
}

impl Scope {
    pub(super) fn enter(phase: EventPhase) -> Self {
        Self {
            previous: transition(phase),
        }
    }

    pub(super) fn switch(&mut self, phase: EventPhase) {
        if self.previous.is_some() {
            transition(phase);
        }
    }
}

impl Drop for Scope {
    fn drop(&mut self) {
        if let Some(previous) = self.previous {
            transition(previous);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn entries(profile: &EventPhaseProfile, phase: EventPhase) -> u64 {
        profile.phase_entries[phase as usize]
    }

    fn capture(session: &mut ReactiveSession, event: &str) -> (bool, EventPhaseProfile) {
        ReactiveSession::begin_event_phase_profile();
        let result = session.apply(event, &BTreeMap::new());
        assert_eq!(current_event_phase_index(), EventPhase::Outside as usize);
        let profile = ReactiveSession::take_event_phase_profile();
        assert_eq!(
            profile.clock_enabled,
            !cfg!(feature = "event-phase-origins")
        );
        assert_eq!(profile.phase_ns[EventPhase::Outside as usize], 0);
        if !profile.clock_enabled {
            assert_eq!(profile.phase_ns, [0; EVENT_PHASE_COUNT]);
        }
        (result.is_err(), profile)
    }

    #[test]
    fn event_profile_disabled_until_begin_and_take_resets() {
        ReactiveSession::take_event_phase_profile();
        let mut session = ReactiveSession::from_source("event idle;").unwrap();
        session.apply("idle", &BTreeMap::new()).unwrap();
        assert_eq!(
            ReactiveSession::take_event_phase_profile(),
            EventPhaseProfile::default()
        );
        let (rejected, profile) = capture(&mut session, "idle");
        assert!(!rejected);
        assert_eq!(profile.transitions, 11);
        assert_eq!(entries(&profile, EventPhase::Collection), 1);
        assert_eq!(entries(&profile, EventPhase::Compaction), 0);
        assert_eq!(
            entries(&profile, EventPhase::ArchiveConstructionAndRemoval),
            0
        );
        assert_eq!(entries(&profile, EventPhase::FinalSettlement), 1);
        assert_eq!(entries(&profile, EventPhase::CommitCleanup), 1);
        assert_eq!(entries(&profile, EventPhase::RollbackCleanup), 0);
        assert_eq!(
            ReactiveSession::take_event_phase_profile(),
            EventPhaseProfile::default()
        );
    }

    #[test]
    fn event_profile_early_input_and_rule_errors_close_only_entered_phases() {
        let mut session = ReactiveSession::from_source(
            "state number = 1; event refuse; on refuse set number = 2; on refuse reject \"stop\";",
        )
        .unwrap();
        let saved = session.save_json().unwrap();
        let (rejected, early) = capture(&mut session, "unknown");
        assert!(rejected);
        assert_eq!(early.transitions, 4);
        assert_eq!(entries(&early, EventPhase::Evaluation), 0);
        assert_eq!(entries(&early, EventPhase::RollbackCleanup), 0);
        let (rejected, refusal) = capture(&mut session, "refuse");
        assert!(rejected);
        assert_eq!(entries(&refusal, EventPhase::Evaluation), 1);
        assert_eq!(entries(&refusal, EventPhase::Collection), 0);
        assert_eq!(entries(&refusal, EventPhase::BindingEvaluation), 0);
        assert_eq!(entries(&refusal, EventPhase::FinalSettlement), 0);
        assert_eq!(entries(&refusal, EventPhase::RollbackCleanup), 1);
        assert_eq!(entries(&refusal, EventPhase::CommitCleanup), 0);
        assert_eq!(session.save_json().unwrap(), saved);
    }

    #[test]
    fn event_profile_success_and_binding_rollback_preserve_undrained_archive() {
        let source =
            include_str!("../../experiments/departure-gate/collector-fixtures/release-mutual.cav");
        let mut session = ReactiveSession::from_source(source).unwrap();
        for _ in 0..4 {
            session.apply("cycle", &BTreeMap::new()).unwrap();
        }
        let (rejected, success) = capture(&mut session, "release");
        assert!(!rejected);
        assert_eq!(success.transitions, 13);
        for phase in [
            EventPhase::Collection,
            EventPhase::Compaction,
            EventPhase::ArchiveConstructionAndRemoval,
            EventPhase::BindingEvaluation,
            EventPhase::FinalSettlement,
            EventPhase::CommitCleanup,
        ] {
            assert_eq!(entries(&success, phase), 1, "{phase:?}");
        }
        assert_eq!(
            entries(&success, EventPhase::Evaluation),
            2,
            "caller resumes after departure"
        );
        assert_eq!(
            entries(&success, EventPhase::TransactionPreparation),
            2,
            "caller resumes after run_event"
        );
        for _ in 0..4 {
            session.apply("cycle", &BTreeMap::new()).unwrap();
        }
        let saved = session.save_json().unwrap();
        let archive = serde_json::to_string(&session.archive).unwrap();
        assert!(session.undrained() > 0);
        let (rejected, failure) = capture(&mut session, "fail");
        assert!(rejected);
        assert_eq!(failure.transitions, 12);
        assert_eq!(
            entries(&failure, EventPhase::ArchiveConstructionAndRemoval),
            1
        );
        assert_eq!(entries(&failure, EventPhase::BindingEvaluation), 1);
        assert_eq!(entries(&failure, EventPhase::FinalSettlement), 0);
        assert_eq!(entries(&failure, EventPhase::RollbackCleanup), 1);
        assert_eq!(entries(&failure, EventPhase::CommitCleanup), 0);
        assert_eq!(session.save_json().unwrap(), saved);
        assert_eq!(serde_json::to_string(&session.archive).unwrap(), archive);
    }

    #[test]
    fn event_profile_guard_restores_parent_after_local_error_cleanup() {
        struct Witness<'a>(&'a Cell<usize>);
        impl Drop for Witness<'_> {
            fn drop(&mut self) {
                self.0.set(current_event_phase_index());
            }
        }
        fn rejected(observed: &Cell<usize>) -> Result<(), ()> {
            let mut guard = Scope::enter(EventPhase::TransactionPreparation);
            let _staged = Witness(observed);
            {
                let _nested = Scope::enter(EventPhase::Evaluation);
            }
            guard.switch(EventPhase::RollbackCleanup);
            Err(())
        }
        let observed = Cell::new(usize::MAX);
        ReactiveSession::begin_event_phase_profile();
        {
            let _wrapper = Scope::enter(EventPhase::ApplyWrapper);
            assert!(rejected(&observed).is_err());
            assert_eq!(observed.get(), EventPhase::RollbackCleanup as usize);
            assert_eq!(
                current_event_phase_index(),
                EventPhase::ApplyWrapper as usize
            );
        }
        let profile = ReactiveSession::take_event_phase_profile();
        assert_eq!(entries(&profile, EventPhase::RollbackCleanup), 1);
        assert_eq!(current_event_phase_index(), EventPhase::Outside as usize);
    }
}
