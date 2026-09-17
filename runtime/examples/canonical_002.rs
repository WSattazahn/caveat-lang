use caveat_runtime::{Attention, Consequence, EpistemicGraph, NodeKind, Relation};

fn main() {
    let mut g = EpistemicGraph::new();
    let bridge = g.add(NodeKind::Claim {
        proposition: "bridge_safe".into(),
    });
    let load_test = g.add(NodeKind::Evidence {
        description: "static_load_test".into(),
        source: "lab".into(),
    });
    let paint = g.add_caveat("paint_color_variation", Consequence::Negligible);
    let crosswind = g.add_caveat("untested_crosswind", Consequence::Catastrophic);

    g.relate(load_test, Relation::Supports, bridge);
    g.relate(paint, Relation::Qualifies, bridge);
    g.relate(crosswind, Relation::Qualifies, bridge);

    g.set_attention(paint, Attention::Deferred);
    g.set_attention(crosswind, Attention::Examining);

    println!("paint: {:?}", g.nodes.get(&paint).unwrap());
    println!("crosswind: {:?}", g.nodes.get(&crosswind).unwrap());

    let commitment = g.commit("continue_review", &[paint, crosswind]);
    println!("commitment: {:?}", g.nodes.get(&commitment).unwrap());
    println!(
        "retained caveats={}",
        g.relations(Relation::Retains)
            .filter(|edge| edge.from == commitment)
            .count()
    );
}
