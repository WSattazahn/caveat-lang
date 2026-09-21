use caveat_runtime::{Consequence, EpistemicGraph, NodeKind, Relation};

#[test]
fn qualifications_reach_decisions_that_relied_on_evidence_without_asserting_truth() {
    let mut graph = EpistemicGraph::new();
    let claim = graph.add(NodeKind::Claim {
        proposition: "The forecast current is mild".into(),
    });
    let forecast = graph.add(NodeKind::Evidence {
        description: "Morning forecast".into(),
        source: "Tidal archive".into(),
    });
    let caveat = graph.add_caveat("Surge may be unmeasured", Consequence::High);
    graph.relate(forecast, Relation::Supports, claim);
    graph.relate(caveat, Relation::Qualifies, forecast);
    let decision = graph.commit("Hold course", &[caveat]);
    graph.relate(decision, Relation::ReliesOn, forecast);

    let impact = &graph.qualification_impacts(caveat)[0];
    assert!(impact.affected.contains(&forecast));
    assert!(impact.affected.contains(&claim));
    assert!(impact.affected.contains(&decision));
    assert_eq!(graph.relations(Relation::Supports).count(), 1);
    assert_eq!(graph.relations(Relation::ReliesOn).count(), 1);
    assert_eq!(graph.relations(Relation::Retains).count(), 1);

    // Reopening leaves both the original evidence dependency and its caveat.
    let reading = graph.add(NodeKind::Evidence {
        description: "Contrary foam drift".into(),
        source: "Light observation".into(),
    });
    graph.relate(reading, Relation::Opposes, claim);
    graph.reopen(decision, reading);
    assert_eq!(graph.relations(Relation::ReliesOn).count(), 1);
    assert_eq!(graph.relations(Relation::Retains).count(), 1);
    assert!(graph.qualification_impacts(caveat)[0]
        .affected
        .contains(&decision));
}
