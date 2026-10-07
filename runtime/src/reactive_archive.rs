//! Content-addressed provenance handover. Live values retain only root digests;
//! transient expression DAGs are exported after the event commits.
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ArchiveProvenance {
    pub kind: &'static str,
    pub id: String,
    pub history: String,
    #[serde(flatten)]
    pub operation: ArchiveOperation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "operation", rename_all = "snake_case")]
pub enum ArchiveOperation {
    Record {
        source_id: String,
        record: String,
        departed_at: u64,
    },
    Union {
        parents: Vec<String>,
    },
}

#[derive(Debug)]
struct Pending {
    entry: ArchiveProvenance,
    parents: Vec<ArchiveTrace>,
}

/// Pending children exist only during evaluation; settled roots serialize as
/// one digest and cannot keep earlier DAG nodes alive after archive drain.
#[derive(Debug, Clone)]
pub struct ArchiveTrace {
    id: String,
    pending: Option<Arc<Pending>>,
}

impl PartialEq for ArchiveTrace {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl Eq for ArchiveTrace {}

impl Serialize for ArchiveTrace {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.id)
    }
}
impl<'de> Deserialize<'de> for ArchiveTrace {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let id = String::deserialize(deserializer)?;
        if id.len() != 64
            || !id
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            return Err(serde::de::Error::custom(
                "archive_ref must be a lowercase SHA-256 digest",
            ));
        }
        Ok(Self { id, pending: None })
    }
}

fn digest(value: &impl Serialize) -> String {
    let bytes = serde_json::to_vec(value).expect("archive identities serialize");
    format!("{:x}", Sha256::digest(bytes))
}
const DOMAIN: &str = "caveat-archive-provenance/0.1";

impl ArchiveTrace {
    pub(crate) fn record(source_id: &str, history: &str, record: &str, departed_at: u64) -> Self {
        let id = digest(&(DOMAIN, "record", source_id, history, record, departed_at));
        let entry = ArchiveProvenance {
            kind: "provenance",
            id: id.clone(),
            history: history.into(),
            operation: ArchiveOperation::Record {
                source_id: source_id.into(),
                record: record.into(),
                departed_at,
            },
        };
        Self {
            id,
            pending: Some(Arc::new(Pending {
                entry,
                parents: Vec::new(),
            })),
        }
    }

    pub(crate) fn join(&self, other: &Self, history: &str) -> Self {
        if self.id == other.id {
            return self.clone();
        }
        let parents = if self.id < other.id {
            vec![self.clone(), other.clone()]
        } else {
            vec![other.clone(), self.clone()]
        };
        let ids = parents
            .iter()
            .map(|parent| parent.id.clone())
            .collect::<Vec<_>>();
        let id = digest(&(DOMAIN, "union", history, &ids));
        let entry = ArchiveProvenance {
            kind: "provenance",
            id: id.clone(),
            history: history.into(),
            operation: ArchiveOperation::Union { parents: ids },
        };
        Self {
            id,
            pending: Some(Arc::new(Pending { entry, parents })),
        }
    }

    pub(crate) fn settle(
        &mut self,
        nodes: &mut Vec<ArchiveProvenance>,
        seen: &mut BTreeSet<String>,
    ) {
        let mut pending = self.pending.take().into_iter().collect::<Vec<_>>();
        while let Some(node) = pending.pop() {
            if !seen.insert(node.entry.id.clone()) {
                continue;
            }
            for parent in &node.parents {
                if let Some(child) = &parent.pending {
                    pending.push(Arc::clone(child));
                }
            }
            nodes.push(node.entry.clone());
        }
    }

    pub(crate) fn is_settled(&self) -> bool {
        self.pending.is_none()
    }
}

// Expression trees can be long within the bounded event budget. Releasing a
// transient DAG must not recurse through thousands of Arc destructors.
impl Drop for ArchiveTrace {
    fn drop(&mut self) {
        let mut pending = self.pending.take().into_iter().collect::<Vec<_>>();
        while let Some(node) = pending.pop() {
            if let Ok(mut unique) = Arc::try_unwrap(node) {
                for parent in &mut unique.parents {
                    if let Some(node) = parent.pending.take() {
                        pending.push(node);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_deep_temporary_archive_dag_settles_and_drops_without_recursive_destruction() {
        let mut root = ArchiveTrace::record("source", "s", "s@1", 1);
        for number in 2..=4096 {
            root = root.join(
                &ArchiveTrace::record("source", "s", &format!("s@{number}"), number),
                "s",
            );
        }
        let duplicate = root.clone();
        let mut nodes = Vec::new();
        root.settle(&mut nodes, &mut BTreeSet::new());
        assert!(root.is_settled());
        assert_eq!(nodes.len(), 8191);
        drop(duplicate);
    }
}
