//! Texts a program first learns while it runs, such as commit SHAs, each held
//! as a numeric handle. See spec/caveat-identifiers-0.1.md.
use std::collections::HashMap;

/// The largest `identifiers limit N` a program may declare.
pub const MAX_IDENTIFIER_LIMIT: usize = 65_536;
/// The longest identifier text, in bytes.
pub const MAX_IDENTIFIER_BYTES: usize = 1_024;
/// The most identifier text one session holds, in bytes.
pub const MAX_IDENTIFIER_TOTAL_BYTES: usize = 1 << 20;

/// A session's identifiers in handle order: `texts[0]` has handle 1.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct Identifiers {
    /// The declared limit, or 0 when the program declares no identifiers.
    limit: usize,
    texts: Vec<String>,
    handles: HashMap<String, usize>,
    bytes: usize,
}

impl Identifiers {
    pub(super) fn declared(limit: usize) -> Self {
        Self {
            limit,
            ..Self::default()
        }
    }

    pub(super) fn limit(&self) -> usize {
        self.limit
    }

    pub(super) fn texts(&self) -> &[String] {
        &self.texts
    }

    pub(super) fn len(&self) -> usize {
        self.texts.len()
    }

    /// The handle of `text`, if the session holds it.
    pub(super) fn handle(&self, text: &str) -> Option<usize> {
        self.handles.get(text).copied()
    }

    /// The text whose handle is `handle`. Handles are whole numbers from 1.
    pub(super) fn text(&self, handle: f64) -> Option<&str> {
        if handle.fract() != 0.0 || handle < 1.0 || handle > self.texts.len() as f64 {
            return None;
        }
        Some(&self.texts[handle as usize - 1])
    }

    /// Whether `text` may be an identifier at all: 1 to 1,024 bytes.
    pub(super) fn check_text(text: &str) -> Result<(), String> {
        if text.is_empty() || text.len() > MAX_IDENTIFIER_BYTES {
            return Err(format!(
                "an identifier must be 1 to {MAX_IDENTIFIER_BYTES} bytes of text"
            ));
        }
        Ok(())
    }

    /// Whether the session can also hold `new`, which it does not hold yet.
    pub(super) fn has_room_for(&self, new: &[String]) -> bool {
        let bytes = new.iter().map(String::len).sum::<usize>();
        self.texts.len() + new.len() <= self.limit
            && self.bytes + bytes <= MAX_IDENTIFIER_TOTAL_BYTES
    }

    /// Adds `text`, which the session does not hold, and returns its handle.
    pub(super) fn push(&mut self, text: String) -> usize {
        debug_assert!(!self.handles.contains_key(&text));
        self.bytes += text.len();
        self.handles.insert(text.clone(), self.texts.len() + 1);
        self.texts.push(text);
        self.texts.len()
    }

    /// Rebuilds a saved list against the program's declared limit, refusing a
    /// list a session could not have produced.
    pub(super) fn restore(limit: usize, texts: Vec<String>) -> Result<Self, String> {
        if !texts.is_empty() && limit == 0 {
            return Err("save holds identifiers but the program declares none".into());
        }
        let mut restored = Self::declared(limit);
        for text in texts {
            Self::check_text(&text).map_err(|error| format!("saved identifier: {error}"))?;
            if restored.handles.contains_key(&text) {
                return Err(format!("saved identifiers repeat {text:?}"));
            }
            if !restored.has_room_for(std::slice::from_ref(&text)) {
                return Err("saved identifiers exceed the program's limit".into());
            }
            restored.push(text);
        }
        Ok(restored)
    }
}
