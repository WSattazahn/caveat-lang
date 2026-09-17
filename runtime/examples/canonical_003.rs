use caveat_runtime::{Consequence, EpistemicGraph, NodeKind, Relation, StopReason};

fn main() {
    let mut g = EpistemicGraph::new();
    let ready = g.add(NodeKind::Claim { proposition: "release_ready".into() });
    let c1 = g.add_caveat("edge_case_1", Consequence::Low);
    let c2 = g.add_caveat("edge_case_2", Consequence::Low);
    let c3 = g.add_caveat("edge_case_3", Consequence::Low);

    g.relate(c1, Relation::Qualifies, ready);
    g.relate(c2, Relation::Qualifies, c1);
    g.relate(c3, Relation::Qualifies, c2);

    println!("qualification depth before commit={}", g.qualification_depth_from(ready));
    let commitment = g.commit_because("ship_v1", &[c1, c2, c3], StopReason::Enough);
    println!("commitment={:?}", g.nodes.get(&commitment).unwrap());
    println!("qualification depth after commit={}", g.qualification_depth_from(ready));
    println!("unresolved caveats retained={}", g.relations(Relation::Retains).filter(|e| e.from == commitment).count());
}
