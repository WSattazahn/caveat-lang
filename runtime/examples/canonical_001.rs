use caveat_runtime::{Consequence, EpistemicGraph, NodeKind, Relation};

fn main() {
    let mut g = EpistemicGraph::new();
    let cat = g.add(NodeKind::Claim {
        proposition: "creature_is_cat".into(),
    });
    let not_cat = g.add(NodeKind::Claim {
        proposition: "creature_is_not_cat".into(),
    });
    let photo = g.add(NodeKind::Evidence {
        description: "photo_1".into(),
        source: "camera".into(),
    });
    let label = g.add(NodeKind::Evidence {
        description: "label_1".into(),
        source: "display_label".into(),
    });
    let misplaced = g.add_caveat("label_may_be_misplaced", Consequence::Material);

    g.relate(photo, Relation::Supports, cat);
    g.relate(label, Relation::Supports, not_cat);
    g.relate(misplaced, Relation::Qualifies, label);

    let commitment = g.commit("treat_as_cat", &[not_cat, misplaced]);
    println!("COMMIT {:?}", g.nodes.get(&commitment).unwrap());

    let record = g.add(NodeKind::Evidence {
        description: "curator_record_7".into(),
        source: "archive".into(),
    });
    g.relate(record, Relation::Opposes, label);
    g.reopen(commitment, misplaced);

    println!("REOPEN {:?}", g.nodes.get(&commitment).unwrap());
    println!("nodes={} edges={}", g.nodes.len(), g.edges.len());
}
