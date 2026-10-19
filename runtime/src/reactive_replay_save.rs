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
/// The positional replay save: each event as its position among the
/// program's events, then its parameters in declared order.
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

    /// `[E, ARGUMENT, …]`: the event's position in `events`, which is in name
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

    fn from_positional(
        value: &Value,
        events: &BTreeMap<String, Vec<Parameter>>,
    ) -> Result<Self, String> {
        let malformed = || "an event entry is not [event, argument, …]".to_string();
        let items = value.as_array().ok_or_else(malformed)?;
        let (event, parameters) = items
            .first()
            .and_then(Value::as_u64)
            .and_then(|position| events.iter().nth(usize::try_from(position).ok()?))
            .ok_or_else(|| "an event entry names no event of this program".to_string())?;
        if items.len() != parameters.len() + 1 {
            return Err(format!(
                "event {event} takes {} arguments",
                parameters.len()
            ));
        }
        let payload = parameters
            .iter()
            .zip(&items[1..])
            .map(|(parameter, value)| match value {
                Value::Number(_) | Value::String(_) => Ok((parameter.name.clone(), value.clone())),
                _ => Err(malformed()),
            })
            .collect::<Result<_, _>>()?;
        Ok(Self {
            event: event.clone(),
            payload,
        })
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
        let positional = entry.to_positional(&self.events).to_string().len() + 1;
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
        struct Written<'a> {
            schema: &'static str,
            source_id: &'a str,
            source_sha256: &'a str,
            events: Vec<Value>,
        }
        let text = serde_json::to_string(&Written {
            schema: POSITIONAL_REPLAY_SAVE_SCHEMA,
            source_id: &self.source_id,
            source_sha256: &self.source_sha256,
            events: log
                .entries
                .iter()
                .map(|entry| entry.to_positional(&self.events))
                .collect(),
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
        let saved: Saved = serde_json::from_str(save).map_err(|error| refuse(error.to_string()))?;
        let mut session = Self::from_source_cached(source)?;
        let positional = saved.schema == POSITIONAL_REPLAY_SAVE_SCHEMA;
        if saved.schema != REPLAY_SAVE_SCHEMA && !positional {
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
        for (index, value) in saved.events.iter().enumerate() {
            let entry = if positional {
                ReplayEntry::from_positional(value, &session.events)
            } else {
                ReplayEntry::from_value(value)
            }
            .map_err(refuse)?;
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
