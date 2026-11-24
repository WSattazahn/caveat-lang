//! The replay save: the events a session accepted since its program loaded,
//! restored by dispatching them again. See spec/caveat-replay-save-0.1.md and,
//! for the positional form, spec/caveat-replay-save-0.2.md.
//!
//! A session keeps the events it accepts while they fit in
//! `REPLAY_SAVE_MAX_BYTES`, and `save_compact_json` writes them in place of the
//! state save when that is smaller. Restoring loads the source and dispatches
//! each event as a host would, so the restored session is the one those events
//! make. A save is data a host may hand back altered: an entry that is
//! malformed, or an event the program does not accept, refuses the save.
use super::*;
use serde_json::Value;

pub const REPLAY_SAVE_SCHEMA: &str = "caveat-replay-save/0.1";
/// The positional replay save: the source's digest in base64url, and each
/// event as its position among the program's events followed by its
/// parameters in declared order, all in one list.
pub const POSITIONAL_REPLAY_SAVE_SCHEMA: &str = "caveat-replay-save/0.2";
/// The largest replay save a session writes or restores, in either form. Past
/// it, the session drops the events it kept and its compact saves are state
/// saves.
pub const REPLAY_SAVE_MAX_BYTES: usize = 4096;

/// One accepted event: its name and its payload as `dispatch` takes it, each
/// identifier parameter as its text.
#[derive(Debug, Clone, PartialEq)]
pub(super) struct ReplayEntry {
    event: String,
    payload: serde_json::Map<String, Value>,
}

impl ReplayEntry {
    /// The entry that begins `items`, and how many values it takes: the
    /// event's position, then one value for each of its parameters.
    fn from_positional(
        items: &[Value],
        events: &BTreeMap<String, Vec<Parameter>>,
    ) -> Result<(Self, usize), String> {
        let (event, parameters) = items
            .first()
            .and_then(Value::as_u64)
            .and_then(|position| events.iter().nth(usize::try_from(position).ok()?))
            .ok_or_else(|| {
                "an entry does not begin with the position of an event of this program".to_string()
            })?;
        let Some(values) = items.get(1..=parameters.len()) else {
            return Err(format!(
                "event {event} takes {} arguments",
                parameters.len()
            ));
        };
        let payload = parameters
            .iter()
            .zip(values)
            .map(|(parameter, value)| match value {
                Value::Number(_) | Value::String(_) => Ok((parameter.name.clone(), value.clone())),
                _ => Err(format!(
                    "an argument of event {event} is not a number or a text"
                )),
            })
            .collect::<Result<_, _>>()?;
        Ok((
            Self {
                event: event.clone(),
                payload,
            },
            parameters.len() + 1,
        ))
    }

    fn from_value(value: &Value) -> Result<Self, String> {
        let malformed = || "an event entry is not [event] or [event, payload]".to_string();
        let items = value.as_array().ok_or_else(malformed)?;
        let Some(event) = items.first().and_then(Value::as_str) else {
            return Err(malformed());
        };
        let payload = match (items.len(), items.get(1)) {
            (1, None) => serde_json::Map::new(),
            (2, Some(Value::Object(payload))) if !payload.is_empty() => payload.clone(),
            _ => return Err(malformed()),
        };
        Ok(Self {
            event: event.to_string(),
            payload,
        })
    }
}

/// A hex digest as base64url without padding, as the positional replay save
/// writes it; `None` if it is not hex.
fn base64url(hex: &str) -> Option<String> {
    const DIGITS: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let bytes = (0..hex.len())
        .step_by(2)
        .map(|at| u8::from_str_radix(hex.get(at..at + 2)?, 16).ok())
        .collect::<Option<Vec<u8>>>()?;
    let mut text = String::new();
    for chunk in bytes.chunks(3) {
        let word = chunk
            .iter()
            .fold(0u32, |word, byte| word << 8 | u32::from(*byte))
            << (8 * (3 - chunk.len()));
        for digit in 0..=chunk.len() {
            text.push(char::from(DIGITS[(word >> (18 - 6 * digit) & 63) as usize]));
        }
    }
    Some(text)
}

use std::io::Write;

/// A parameter value kept in a compact replay log without allocating a
/// `serde_json::Value` or repeating parameter names on every event.
#[derive(Debug, Clone, PartialEq)]
enum ReplayValue {
    Int(i64),
    Float(f64),
    Text(String),
}

impl ReplayValue {
    fn from_number(value: f64) -> Self {
        const EXACT: f64 = 9_007_199_254_740_992.0;
        if value.fract() == 0.0
            && value.abs() < EXACT
            && !(value == 0.0 && value.is_sign_negative())
        {
            Self::Int(value as i64)
        } else {
            Self::Float(value)
        }
    }

    fn to_value(&self) -> Value {
        match self {
            Self::Int(value) => Value::from(*value),
            Self::Float(value) => {
                serde_json::Number::from_f64(*value).map_or(Value::Null, Value::Number)
            }
            Self::Text(text) => Value::String(text.clone()),
        }
    }

    fn json_len(&self) -> usize {
        match self {
            Self::Int(value) => signed_len(*value),
            Self::Float(value) => {
                let mut counter = ByteCounter(0);
                let _ = serde_json::to_writer(&mut counter, value);
                counter.0
            }
            Self::Text(text) => json_str_len(text),
        }
    }
}

struct ByteCounter(usize);

impl Write for ByteCounter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0 += buf.len();
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn unsigned_len(mut value: u64) -> usize {
    let mut digits = 1;
    while value >= 10 {
        value /= 10;
        digits += 1;
    }
    digits
}

fn signed_len(value: i64) -> usize {
    if value < 0 {
        1 + unsigned_len(value.unsigned_abs())
    } else {
        unsigned_len(value as u64)
    }
}

fn json_str_len(text: &str) -> usize {
    if text.bytes().all(|b| b >= 0x20 && b != b'"' && b != b'\\') {
        return text.len() + 2;
    }
    let mut counter = ByteCounter(0);
    let _ = serde_json::to_writer(&mut counter, text);
    counter.0
}

#[derive(Debug, Clone)]
struct CompactEntry {
    event_index: usize,
    values: Vec<ReplayValue>,
}

impl CompactEntry {
    fn to_named_value(&self, events: &BTreeMap<String, Vec<Parameter>>) -> Value {
        let Some((event, parameters)) = events.iter().nth(self.event_index) else {
            return Value::Null;
        };
        let mut entry = vec![Value::String(event.clone())];
        if !self.values.is_empty() {
            let mut map = serde_json::Map::with_capacity(self.values.len());
            for (parameter, value) in parameters.iter().zip(&self.values) {
                map.insert(parameter.name.clone(), value.to_value());
            }
            entry.push(Value::Object(map));
        }
        Value::Array(entry)
    }

    fn push_positional(&self, out: &mut Vec<Value>) {
        out.push(Value::from(self.event_index));
        for value in &self.values {
            out.push(value.to_value());
        }
    }
}

/// The events a session accepted since its program loaded, while their
/// replay save fits.
#[derive(Debug, Clone)]
pub(super) struct ReplayLog {
    entries: Vec<CompactEntry>,
    /// The bytes the entries take in a replay save, separators included,
    /// while they still fit; `None` once they no longer do.
    bytes: Option<usize>,
    /// The bytes they take in a positional replay save.
    positional_bytes: usize,
}

impl Default for ReplayLog {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            bytes: Some(0),
            positional_bytes: 0,
        }
    }
}

impl ReactiveSession {
    /// Keep an event the session has just accepted with `parameters`, or stop
    /// keeping events once they no longer fit in a replay save.
    pub(super) fn record_replay(&mut self, event: &str, parameters: &BTreeMap<String, f64>) {
        let Some(log) = self.replay.as_ref() else {
            return;
        };
        let Some((event_index, (_, signature))) = self
            .events
            .iter()
            .enumerate()
            .find(|(_, (name, _))| name.as_str() == event)
        else {
            return;
        };
        let track_named = log.bytes.is_some();
        let mut values = Vec::with_capacity(signature.len());
        // Positional entry `[E, v0, v1, ...]` without brackets plus comma:
        // `1 + len(E) + sum(1 + len(v))`.
        let mut positional = 1 + unsigned_len(event_index as u64);
        // Named entry `["event"]` + comma = `3 + len("event")`, or
        // `["event",{"k":v,...}]` + comma = `5 + len("event") + sum(len("k") + 2 + len(v))`.
        let mut named = if track_named {
            (if signature.is_empty() { 3 } else { 5 }) + json_str_len(event)
        } else {
            0
        };
        for parameter in signature {
            let raw = parameters[&parameter.name];
            let value = match parameter.domain {
                ParameterDomain::Identifier { .. } => match self.identifiers.text(raw) {
                    Some(text) => ReplayValue::Text(text.to_string()),
                    None => ReplayValue::from_number(raw),
                },
                _ => ReplayValue::from_number(raw),
            };
            let vlen = value.json_len();
            positional += 1 + vlen;
            if track_named {
                named += json_str_len(&parameter.name) + 2 + vlen;
            }
            values.push(value);
        }
        let entry = CompactEntry {
            event_index,
            values,
        };
        #[cfg(debug_assertions)]
        {
            let mut pos_values = Vec::with_capacity(entry.values.len() + 1);
            entry.push_positional(&mut pos_values);
            debug_assert_eq!(
                positional,
                Value::Array(pos_values).to_string().len() - 1,
                "positional replay length mismatch"
            );
            if track_named {
                debug_assert_eq!(
                    named,
                    entry.to_named_value(&self.events).to_string().len() + 1,
                    "named replay length mismatch"
                );
            }
        }
        // A positional entry is never longer than its named one, so the
        // positional log outlasts the named one.
        if log.positional_bytes + positional > REPLAY_SAVE_MAX_BYTES {
            self.replay = None;
            return;
        }
        let log = Arc::make_mut(self.replay.as_mut().expect("checked above"));
        log.entries.push(entry);
        log.positional_bytes += positional;
        log.bytes = log
            .bytes
            .map(|bytes| bytes + named)
            .filter(|bytes| *bytes <= REPLAY_SAVE_MAX_BYTES);
    }

    /// The replay save, while the session holds every event it accepted since
    /// its program loaded and the save fits.
    fn replay_json(&self) -> Option<String> {
        let log = self.replay.as_ref()?;
        log.bytes?;
        #[derive(Serialize)]
        struct Written<'a> {
            schema: &'static str,
            source_id: &'a str,
            source_sha256: &'a str,
            #[serde(skip_serializing_if = "Option::is_none")]
            module_bindings: Option<ModuleBindings>,
            events: Vec<Value>,
        }
        let text = serde_json::to_string(&Written {
            schema: REPLAY_SAVE_SCHEMA,
            source_id: &self.source_id,
            source_sha256: &self.source_sha256,
            module_bindings: self.module_bindings,
            events: log
                .entries
                .iter()
                .map(|entry| entry.to_named_value(&self.events))
                .collect(),
        })
        .ok()?;
        (text.len() <= REPLAY_SAVE_MAX_BYTES).then_some(text)
    }

    /// The replay save when the session holds one and it is smaller than the
    /// state save, otherwise the state save. Either restores with
    /// `restore_json`.
    pub fn save_compact_json(&self) -> Result<String, String> {
        let state = self.save_json()?;
        Ok(match self.replay_json() {
            Some(replay) if replay.len() < state.len() => replay,
            _ => state,
        })
    }

    /// The positional replay save, while the session holds every event it
    /// accepted since its program loaded and the save fits.
    fn positional_replay_json(&self) -> Option<String> {
        let log = self.replay.as_ref()?;
        #[derive(Serialize)]
        struct Written {
            schema: &'static str,
            sha256: String,
            #[serde(skip_serializing_if = "Option::is_none")]
            module_bindings: Option<ModuleBindings>,
            events: Value,
        }
        let mut values: Vec<Value> = Vec::new();
        for entry in &log.entries {
            entry.push_positional(&mut values);
        }
        // When every value is a whole number below 36, one base-36 digit each.
        let digits: Option<String> = values
            .iter()
            .map(|value| {
                value
                    .as_u64()
                    .and_then(|digit| char::from_digit(u32::try_from(digit).ok()?, 36))
            })
            .collect();
        let text = serde_json::to_string(&Written {
            schema: POSITIONAL_REPLAY_SAVE_SCHEMA,
            sha256: base64url(&self.source_sha256)?,
            module_bindings: self.module_bindings,
            events: digits.map_or(Value::Array(values), Value::String),
        })
        .ok()?;
        (text.len() <= REPLAY_SAVE_MAX_BYTES).then_some(text)
    }

    /// `save_compact_json` with the positional replay save in place of the
    /// named one: it holds more events, so it is a replay save for longer.
    pub fn save_compact_positional_json(&self) -> Result<String, String> {
        let state = self.save_json()?;
        Ok(match self.positional_replay_json() {
            Some(replay) if replay.len() < state.len() => replay,
            _ => state,
        })
    }

    /// Whether `save` names the replay save's schema. A text longer than a
    /// replay save can be is one only if it begins as the runtime writes one,
    /// so that restoring it says why it is refused; anything else is read as a
    /// state save.
    pub(super) fn is_replay_save(save: &str) -> bool {
        if save.len() > REPLAY_SAVE_MAX_BYTES {
            return [REPLAY_SAVE_SCHEMA, POSITIONAL_REPLAY_SAVE_SCHEMA]
                .iter()
                .any(|schema| {
                    let written = format!("{{\"schema\":\"{schema}\"");
                    save.trim_start().starts_with(&written)
                });
        }
        #[derive(Deserialize)]
        struct Schema {
            #[serde(default)]
            schema: Option<String>,
        }
        serde_json::from_str::<Schema>(save)
            .ok()
            .and_then(|save| save.schema)
            .is_some_and(|schema| {
                schema == REPLAY_SAVE_SCHEMA || schema == POSITIONAL_REPLAY_SAVE_SCHEMA
            })
    }

    /// A session of `source` with the replay save's events dispatched again.
    /// An event the program does not accept refuses the save.
    pub(super) fn restore_replay_json(
        source: &str,
        save: &str,
        options: &RestoreOptions,
    ) -> Result<Self, String> {
        let refuse = |error: String| format!("cannot restore save: {error}");
        if save.len() > REPLAY_SAVE_MAX_BYTES {
            return Err(refuse(format!(
                "a replay save is at most {REPLAY_SAVE_MAX_BYTES} bytes"
            )));
        }
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Saved {
            schema: String,
            source_id: String,
            source_sha256: String,
            #[serde(default, deserialize_with = "deserialize_module_bindings")]
            module_bindings: Option<ModuleBindings>,
            events: Vec<Value>,
        }
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Positional {
            #[allow(dead_code)]
            schema: String,
            sha256: String,
            #[serde(default, deserialize_with = "deserialize_module_bindings")]
            module_bindings: Option<ModuleBindings>,
            events: Value,
        }
        #[derive(Deserialize)]
        struct Schema {
            schema: String,
        }
        let schema = serde_json::from_str::<Schema>(save)
            .map_err(|error| refuse(error.to_string()))?
            .schema;
        // Each form is read whole before the source loads, as 0.1 always was.
        let (positional, named) = if schema == POSITIONAL_REPLAY_SAVE_SCHEMA {
            let saved: Positional =
                serde_json::from_str(save).map_err(|error| refuse(error.to_string()))?;
            (Some(saved), None)
        } else {
            let saved: Saved =
                serde_json::from_str(save).map_err(|error| refuse(error.to_string()))?;
            (None, Some(saved))
        };
        let saved_bindings = positional
            .as_ref()
            .and_then(|save| save.module_bindings)
            .or_else(|| named.as_ref().and_then(|save| save.module_bindings));
        // Whose save this is: what does not depend on the program the source
        // is read as, so it is checked before a historical reading is asked
        // for too.
        let identity = |source_id: &str, source_sha256: &str| -> Result<(), String> {
            if let Some(saved) = &positional {
                if base64url(source_sha256).as_deref() != Some(saved.sha256.as_str()) {
                    return Err(refuse("its source digest is not this source's".into()));
                }
            }
            if let Some(saved) = &named {
                if saved.schema != REPLAY_SAVE_SCHEMA {
                    return Err(refuse(format!(
                        "schema {} is not {REPLAY_SAVE_SCHEMA}",
                        saved.schema
                    )));
                }
                if saved.source_id != source_id {
                    return Err(refuse("it belongs to a different program".into()));
                }
                if saved.source_sha256 != source_sha256 {
                    return Err(refuse("its source digest is not this source's".into()));
                }
            }
            Ok(())
        };
        // Releases wrote Replay Save 0.1 from 1.1.0 on and 0.2 from 1.2.0 on.
        let format = if positional.is_some() {
            Release::V1_2..=Release::Later
        } else {
            Release::V1_1..=Release::Later
        };
        let mut session = Self::load_for_restore(source, saved_bindings, options, format, || {
            identity(&source_identity(source), &source_digest(source))
        })?;
        identity(&session.source_id, &session.source_sha256)?;
        let mut entries = Vec::new();
        if let Some(saved) = positional {
            let values = match saved.events {
                Value::Array(values) => values,
                Value::String(digits) => digits
                    .chars()
                    .map(|digit| {
                        digit
                            .to_digit(36)
                            .filter(|_| !digit.is_ascii_uppercase())
                            .map(Value::from)
                            .ok_or_else(|| refuse(format!("{digit:?} is not a base-36 digit")))
                    })
                    .collect::<Result<_, _>>()?,
                _ => {
                    return Err(refuse(
                        "its events are not a list or a text of digits".into(),
                    ))
                }
            };
            let mut rest = values.as_slice();
            while !rest.is_empty() {
                let (entry, taken) =
                    ReplayEntry::from_positional(rest, &session.events).map_err(refuse)?;
                entries.push(entry);
                rest = &rest[taken..];
            }
        }
        if let Some(saved) = named {
            for value in &saved.events {
                entries.push(ReplayEntry::from_value(value).map_err(refuse)?);
            }
        }
        for (index, entry) in entries.into_iter().enumerate() {
            let payload = Value::Object(entry.payload).to_string();
            session
                .dispatch_classified(&entry.event, &payload, None)
                .map_err(|failure| {
                    refuse(format!(
                        "event {} ({}) is not accepted: {failure}",
                        index + 1,
                        entry.event
                    ))
                })?;
        }
        // The archive is not part of a save: what those events departed was
        // the saved session's to drain.
        session.archive.clear();
        Ok(session)
    }
}
