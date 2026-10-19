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
    /// `["event"]`, or `["event", {payload}]`.
    fn to_value(&self) -> Value {
        let mut entry = vec![Value::String(self.event.clone())];
        if !self.payload.is_empty() {
            entry.push(Value::Object(self.payload.clone()));
        }
        Value::Array(entry)
    }

    /// `[E, ARGUMENT, …]`, written into the save without its brackets: the event's position in `events`, which is in name
    /// order as the interface lists them, then each parameter in declared
    /// order, written as in the payload.
    fn to_positional(&self, events: &BTreeMap<String, Vec<Parameter>>) -> Value {
        let position = events.keys().position(|name| *name == self.event);
        let mut entry = vec![Value::from(position.unwrap_or(usize::MAX))];
        for parameter in events.get(&self.event).into_iter().flatten() {
            entry.push(
                self.payload
                    .get(&parameter.name)
                    .cloned()
                    .unwrap_or(Value::Null),
            );
        }
        Value::Array(entry)
    }

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

/// A parameter as JSON: a whole number within 2^53 without a fraction, so
/// `1` rather than `1.0`, and any other value as `serde_json` writes an `f64`,
/// `-0.0` included. Both read back as the same `f64`. An accepted event's
/// parameters are finite.
fn number(value: f64) -> Value {
    const EXACT: f64 = 9_007_199_254_740_992.0;
    if value.fract() == 0.0 && value.abs() < EXACT && !(value == 0.0 && value.is_sign_negative()) {
        return Value::from(value as i64);
    }
    serde_json::Number::from_f64(value).map_or(Value::Null, Value::Number)
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

/// The events a session accepted since its program loaded, while their
/// replay save fits.
#[derive(Debug, Clone)]
pub(super) struct ReplayLog {
    entries: Vec<ReplayEntry>,
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
        if self.replay.is_none() {
            return;
        }
        let identifier = |name: &String| {
            self.events.get(event).is_some_and(|signature| {
                signature.iter().any(|parameter| {
                    parameter.name == *name
                        && matches!(parameter.domain, ParameterDomain::Identifier { .. })
                })
            })
        };
        let payload = parameters
            .iter()
            .map(|(name, value)| {
                let value = match self.identifiers.text(*value) {
                    Some(text) if identifier(name) => Value::String(text.to_string()),
                    _ => number(*value),
                };
                (name.clone(), value)
            })
            .collect();
        let entry = ReplayEntry {
            event: event.to_string(),
            payload,
        };
        // Without its brackets, and with the comma before it.
        let positional = entry.to_positional(&self.events).to_string().len() - 1;
        let named = entry.to_value().to_string().len() + 1;
        let Some(log) = self.replay.as_mut() else {
            return;
        };
        // A positional entry is never longer than its named one, so the
        // positional log outlasts the named one.
        if log.positional_bytes + positional > REPLAY_SAVE_MAX_BYTES {
            self.replay = None;
            return;
        }
        let log = Arc::make_mut(log);
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
            events: Vec<Value>,
        }
        let text = serde_json::to_string(&Written {
            schema: REPLAY_SAVE_SCHEMA,
            source_id: &self.source_id,
            source_sha256: &self.source_sha256,
            events: log.entries.iter().map(ReplayEntry::to_value).collect(),
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
            events: Value,
        }
        let values: Vec<Value> = log
            .entries
            .iter()
            .flat_map(|entry| match entry.to_positional(&self.events) {
                Value::Array(items) => items,
                other => vec![other],
            })
            .collect();
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
    pub(super) fn restore_replay_json(source: &str, save: &str) -> Result<Self, String> {
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
            events: Vec<Value>,
        }
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Positional {
            #[allow(dead_code)]
            schema: String,
            sha256: String,
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
        let mut session = Self::from_source_cached(source)?;
        let mut entries = Vec::new();
        if let Some(saved) = positional {
            if base64url(&session.source_sha256).as_deref() != Some(saved.sha256.as_str()) {
                return Err(refuse("its source digest is not this source's".into()));
            }
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
            if saved.schema != REPLAY_SAVE_SCHEMA {
                return Err(refuse(format!(
                    "schema {} is not {REPLAY_SAVE_SCHEMA}",
                    saved.schema
                )));
            }
            if saved.source_id != session.source_id {
                return Err(refuse("it belongs to a different program".into()));
            }
            if saved.source_sha256 != session.source_sha256 {
                return Err(refuse("its source digest is not this source's".into()));
            }
            for value in &saved.events {
                entries.push(ReplayEntry::from_value(value).map_err(refuse)?);
            }
        }
        for (index, entry) in entries.into_iter().enumerate() {
            let payload = Value::Object(entry.payload).to_string();
            session
                .dispatch_classified(&entry.event, &payload)
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
