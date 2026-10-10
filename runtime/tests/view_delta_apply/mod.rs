//! Applies a View 0.2 delta to a full view, written from
//! spec/caveat-view-0.2.md ("The delta") and not from the runtime's delta
//! builder, so the gate checks one against the other. Shared by
//! tests/view_v2.rs and examples/view_delta_sweep.rs.
#![allow(dead_code)]
use serde_json::{Map, Value};

/// `apply(V, D)`, or why `D` does not apply to `V`.
pub fn apply(view: &Value, delta: &Value) -> Result<Value, String> {
    let view = view.as_object().ok_or("the view is not an object")?;
    let delta = delta.as_object().ok_or("the delta is not an object")?;
    let field = |name: &str| delta.get(name).ok_or(format!("the delta has no {name}"));
    if field("schema")? != "caveat-reactive-view-delta/0.2" {
        return Err("not a View 0.2 delta".into());
    }
    if view.get("schema").and_then(Value::as_str) != Some("caveat-reactive-view/0.2") {
        return Err("not a View 0.2 view".into());
    }
    if field("since")? != view.get("sequence").ok_or("the view has no sequence")? {
        return Err("since is not the view's sequence".into());
    }
    let mut next = view.clone();
    next.insert("sequence".into(), field("sequence")?.clone());
    next.insert("last_event".into(), field("last_event")?.clone());
    next.insert("cues".into(), field("cues")?.clone());
    next.insert("effects".into(), field("effects")?.clone());
    for name in ["bindings", "binding_explanations"] {
        let value = properties(&next[name], field(name)?)?;
        next.insert(name.into(), value);
    }
    let value = commitments(&next["commitments"], field("commitments")?)?;
    next.insert("commitments".into(), value);
    for name in ["commitment_grounds", "decision_series"] {
        let value = named(&next[name], field(name)?)?;
        next.insert(name.into(), value);
    }
    for name in ["decision_journal", "relations"] {
        let value = list(&next[name], field(name)?)?;
        next.insert(name.into(), value);
    }
    Ok(Value::Object(next))
}

fn parts<'a>(change: &'a Value, set: &str) -> Result<(&'a Value, &'a Vec<Value>), String> {
    let change = change.as_object().ok_or("a change is not an object")?;
    let set = change.get(set).ok_or("a change has no set or appended")?;
    let removed = change
        .get("removed")
        .and_then(Value::as_array)
        .ok_or("a change has no removed list")?;
    Ok((set, removed))
}

fn properties(current: &Value, change: &Value) -> Result<Value, String> {
    let mut targets = current.as_object().cloned().unwrap_or_default();
    let (set, removed) = parts(change, "set")?;
    for (target, values) in set.as_object().ok_or("set is not an object")? {
        let entry = targets
            .entry(target.clone())
            .or_insert_with(|| Value::Object(Map::new()));
        let properties = entry.as_object_mut().ok_or("a target is not an object")?;
        for (property, value) in values
            .as_object()
            .ok_or("a target's set is not an object")?
        {
            properties.insert(property.clone(), value.clone());
        }
    }
    for pair in removed {
        let pair = pair
            .as_array()
            .filter(|pair| pair.len() == 2)
            .ok_or("removed holds a non-pair")?;
        let (target, property) = (
            pair[0].as_str().ok_or("bad target")?,
            pair[1].as_str().ok_or("bad property")?,
        );
        let properties = targets
            .get_mut(target)
            .and_then(Value::as_object_mut)
            .ok_or(format!("removed names a missing target {target}"))?;
        properties.remove(property).ok_or(format!(
            "removed names a missing property {target}.{property}"
        ))?;
        if properties.is_empty() {
            targets.remove(target);
        }
    }
    Ok(Value::Object(targets))
}

fn commitments(current: &Value, change: &Value) -> Result<Value, String> {
    let action = |commitment: &Value| {
        commitment
            .get("action")
            .and_then(Value::as_str)
            .map(String::from)
    };
    let mut list = current
        .as_array()
        .cloned()
        .ok_or("commitments is not a list")?;
    let (set, removed) = parts(change, "set")?;
    for name in removed {
        let name = name.as_str().ok_or("removed holds a non-name")?;
        let before = list.len();
        list.retain(|commitment| action(commitment).as_deref() != Some(name));
        if list.len() == before {
            return Err(format!("removed names a missing commitment {name}"));
        }
    }
    for commitment in set.as_array().ok_or("set is not a list")? {
        let name = action(commitment).ok_or("a commitment has no action")?;
        match list
            .iter_mut()
            .find(|known| action(known).as_deref() == Some(&name))
        {
            Some(known) => *known = commitment.clone(),
            None => list.push(commitment.clone()),
        }
    }
    list.sort_by(|a, b| {
        action(a)
            .unwrap_or_default()
            .as_bytes()
            .cmp(action(b).unwrap_or_default().as_bytes())
    });
    Ok(Value::Array(list))
}

fn named(current: &Value, change: &Value) -> Result<Value, String> {
    let mut members = current.as_object().cloned().unwrap_or_default();
    let (set, removed) = parts(change, "set")?;
    for (name, value) in set.as_object().ok_or("set is not an object")? {
        members.insert(name.clone(), value.clone());
    }
    for name in removed {
        let name = name.as_str().ok_or("removed holds a non-name")?;
        members
            .remove(name)
            .ok_or(format!("removed names a missing member {name}"))?;
    }
    Ok(Value::Object(members))
}

fn list(current: &Value, change: &Value) -> Result<Value, String> {
    let entries = current.as_array().ok_or("not a list")?;
    let (appended, removed) = parts(change, "appended")?;
    let mut positions = Vec::with_capacity(removed.len());
    for position in removed {
        let position = position.as_u64().ok_or("removed holds a non-position")? as usize;
        if position >= entries.len() || positions.last().is_some_and(|last| *last >= position) {
            return Err(format!(
                "removed position {position} is out of range or order"
            ));
        }
        positions.push(position);
    }
    let mut next = entries
        .iter()
        .enumerate()
        .filter(|(position, _)| positions.binary_search(position).is_err())
        .map(|(_, entry)| entry.clone())
        .collect::<Vec<_>>();
    next.extend(
        appended
            .as_array()
            .ok_or("appended is not a list")?
            .iter()
            .cloned(),
    );
    Ok(Value::Array(next))
}

/// JSON equality as the spec counts it: object members in any order, arrays
/// in order, and `-0` distinct from `0`.
pub fn same(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Number(x), Value::Number(y)) => {
            if x.is_f64() || y.is_f64() {
                x.as_f64().map(f64::to_bits) == y.as_f64().map(f64::to_bits)
            } else {
                x == y
            }
        }
        (Value::Array(x), Value::Array(y)) => {
            x.len() == y.len() && x.iter().zip(y).all(|(p, q)| same(p, q))
        }
        (Value::Object(x), Value::Object(y)) => {
            x.len() == y.len()
                && x.iter()
                    .all(|(key, p)| y.get(key).is_some_and(|q| same(p, q)))
        }
        _ => a == b,
    }
}
