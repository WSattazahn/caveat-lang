//! Source-authored world coordinates, views and routes.
//!
//! Presentation changes never authorize a physical move or epistemic effect.
//! Routes must refer to an edge already declared in the semantic world graph.

use crate::map::MapWorld;
use serde::{Serialize, Serializer};
use std::collections::HashSet;

/// A canonical finite JSON number with reflexive equality.
///
/// The private representation avoids introducing NaN into the Eq-based AST
/// and transaction snapshots. Serialization still emits JSON numbers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Number(String);

impl Number {
    pub fn parse(token: &str) -> Result<Self, String> {
        let value = token
            .parse::<f64>()
            .map_err(|_| format!("presentation coordinate must be a finite number: {token}"))?;
        if !value.is_finite() {
            return Err(format!(
                "presentation coordinate must be a finite number: {token}"
            ));
        }
        Ok(Self(if value == 0.0 {
            "0".into()
        } else {
            value.to_string()
        }))
    }

    pub fn value(&self) -> f64 {
        // Only Number::parse can construct the private, canonical string.
        self.0.parse().expect("validated presentation number")
    }
}

impl Serialize for Number {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_f64(self.value())
    }
}

pub type Point = [Number; 3];

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Placement {
    pub target: String,
    pub position: Point,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Camera {
    pub place: String,
    pub position: Point,
    pub target: Point,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct View {
    pub position: Point,
    pub target: Point,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Route {
    pub from: String,
    pub to: String,
    /// Intermediate world-space points; source endpoint placements are added
    /// by renderers. This does not create a connection between the endpoints.
    pub points: Vec<Point>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum Directive {
    Position(Placement),
    Camera(Camera),
    Overview(View),
    Route(Route),
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct Presentation {
    pub positions: Vec<Placement>,
    pub cameras: Vec<Camera>,
    pub overview: Option<View>,
    pub routes: Vec<Route>,
}

impl Presentation {
    pub fn is_empty(&self) -> bool {
        self.positions.is_empty()
            && self.cameras.is_empty()
            && self.overview.is_none()
            && self.routes.is_empty()
    }
}

fn point(tokens: &[&str]) -> Result<Point, String> {
    if tokens.len() != 3 {
        return Err("presentation coordinates require exactly X Y Z".into());
    }
    Ok([
        Number::parse(tokens[0])?,
        Number::parse(tokens[1])?,
        Number::parse(tokens[2])?,
    ])
}

/// Return None for another language statement, or the parsed presentation
/// directive/error. The main parser attaches line and column diagnostics.
pub fn parse_directive(line: &str) -> Option<Result<Directive, String>> {
    let words = line.split_whitespace().collect::<Vec<_>>();
    let keyword = words.first().copied()?;
    if !matches!(keyword, "position" | "camera" | "overview" | "route") {
        return None;
    }
    Some((|| match words.as_slice() {
        ["position", target, "at", x, y, z] => Ok(Directive::Position(Placement {
            target: (*target).into(),
            position: point(&[x, y, z])?,
        })),
        ["camera", place, "from", x, y, z, "toward", tx, ty, tz] => Ok(Directive::Camera(Camera {
            place: (*place).into(),
            position: point(&[x, y, z])?,
            target: point(&[tx, ty, tz])?,
        })),
        ["overview", "from", x, y, z, "toward", tx, ty, tz] => Ok(Directive::Overview(View {
            position: point(&[x, y, z])?,
            target: point(&[tx, ty, tz])?,
        })),
        ["route", from, "to", to, "via", rest @ ..] => {
            let mut points = Vec::new();
            let mut remaining = rest;
            loop {
                if remaining.len() < 3 {
                    return Err("route requires X Y Z after via and each then".into());
                }
                points.push(point(&remaining[..3])?);
                remaining = &remaining[3..];
                if remaining.is_empty() {
                    break;
                }
                if remaining[0] != "then" {
                    return Err("route points must be separated by then".into());
                }
                remaining = &remaining[1..];
            }
            Ok(Directive::Route(Route {
                from: (*from).into(),
                to: (*to).into(),
                points,
            }))
        }
        _ => Err(format!("invalid {keyword} presentation directive")),
    })())
}

/// Resolve all presentation references against the authoritative world.
/// Declarations may appear in any source order; routes cannot add edges.
pub fn build_presentation(
    directives: &[Directive],
    world: &MapWorld,
) -> Result<Presentation, String> {
    let place_ids = world
        .places
        .iter()
        .map(|place| place.id.as_str())
        .collect::<HashSet<_>>();
    let entity_ids = world
        .entities
        .iter()
        .map(|entity| entity.id.as_str())
        .collect::<HashSet<_>>();
    let mut result = Presentation::default();
    let mut positioned = HashSet::new();
    let mut cameras = HashSet::new();
    let mut routes = HashSet::new();

    for directive in directives {
        match directive {
            Directive::Position(placement) => {
                if !place_ids.contains(placement.target.as_str())
                    && !entity_ids.contains(placement.target.as_str())
                {
                    return Err(format!(
                        "position references unknown place or entity {}",
                        placement.target
                    ));
                }
                if !positioned.insert(placement.target.as_str()) {
                    return Err(format!("duplicate position for {}", placement.target));
                }
                result.positions.push(placement.clone());
            }
            Directive::Camera(camera) => {
                if !place_ids.contains(camera.place.as_str()) {
                    return Err(format!("camera requires a declared place {}", camera.place));
                }
                if !cameras.insert(camera.place.as_str()) {
                    return Err(format!("duplicate camera for {}", camera.place));
                }
                validate_view(&camera.position, &camera.target)?;
                result.cameras.push(camera.clone());
            }
            Directive::Overview(view) => {
                if result.overview.is_some() {
                    return Err("world can declare only one overview".into());
                }
                validate_view(&view.position, &view.target)?;
                result.overview = Some(view.clone());
            }
            Directive::Route(route) => {
                if !place_ids.contains(route.from.as_str())
                    || !place_ids.contains(route.to.as_str())
                {
                    return Err(format!(
                        "route requires declared places {} and {}",
                        route.from, route.to
                    ));
                }
                if route.from == route.to {
                    return Err("route endpoints must be distinct places".into());
                }
                if !world.connections.iter().any(|edge| {
                    (edge.from == route.from && edge.to == route.to)
                        || (edge.to == route.from && edge.from == route.to)
                }) {
                    return Err(format!(
                        "route {} to {} has no declared connection",
                        route.from, route.to
                    ));
                }
                let key = if route.from < route.to {
                    (&route.from, &route.to)
                } else {
                    (&route.to, &route.from)
                };
                if !routes.insert(key) {
                    return Err(format!(
                        "duplicate route between {} and {}",
                        route.from, route.to
                    ));
                }
                if route.points.is_empty() {
                    return Err("route requires at least one intermediate point".into());
                }
                if route.points.windows(2).any(|pair| pair[0] == pair[1]) {
                    return Err("route cannot repeat consecutive intermediate points".into());
                }
                result.routes.push(route.clone());
            }
        }
    }
    for route in &result.routes {
        for endpoint in [&route.from, &route.to] {
            if !positioned.contains(endpoint.as_str()) {
                return Err(format!("route endpoint {endpoint} requires a position"));
            }
        }
    }
    Ok(result)
}

fn validate_view(position: &Point, target: &Point) -> Result<(), String> {
    if position == target {
        return Err("camera position and target must differ".into());
    }
    Ok(())
}
