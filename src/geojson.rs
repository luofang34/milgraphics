//! GeoJSON of a render plan's geographic tier and labels, for map engines
//! that draw GeoJSON sources.

use serde_json::{Map, Value, json};

use crate::pick::{PickRef, PickTarget};
use crate::render::{GeoItem, GeoShape, Label, RenderPlan, TextAlign};
use crate::style::{Fill, Stroke};

#[cfg(test)]
mod tests;

/// A FeatureCollection with one feature per geographic item and one Point
/// feature per label.
///
/// Features carry their pick reference (`graphic`, `part`) as properties;
/// numeric feature IDs are the adapter's to assign. Dash patterns are named
/// (`dash`) because engines take dash arrays per layer, not per feature.
pub fn to_geojson(plan: &RenderPlan) -> Value {
    let mut features: Vec<Value> = plan.geo.iter().map(item_feature).collect();
    features.extend(plan.labels.iter().map(label_feature));
    json!({ "type": "FeatureCollection", "features": features })
}

/// Features for geographic items only, e.g. from [`crate::render::geographic`],
/// for engines that draw the geographic tier while labels and decorations
/// are drawn per view.
pub fn geographic_features(items: &[GeoItem]) -> Vec<Value> {
    items.iter().map(item_feature).collect()
}

fn item_feature(item: &GeoItem) -> Value {
    let geometry = match &item.shape {
        GeoShape::Lines(lines) => json!({ "type": "MultiLineString", "coordinates": lines }),
        GeoShape::Polygons(rings) => {
            let polygons: Vec<Value> = rings.iter().map(|r| json!([r])).collect();
            json!({ "type": "MultiPolygon", "coordinates": polygons })
        }
    };
    let mut props = Map::new();
    props.insert("graphic".into(), json!(item.pick.definition.as_str()));
    props.insert("part".into(), part(&item.pick));
    props.insert("role".into(), json!(format!("{:?}", item.role)));
    stroke_props(&mut props, item.stroke);
    if let Fill::Solid(c) = item.fill {
        props.insert("fill".into(), json!(c.to_hex()));
    }
    json!({ "type": "Feature", "geometry": geometry, "properties": props })
}

fn stroke_props(props: &mut Map<String, Value>, stroke: Option<Stroke>) {
    if let Some(s) = stroke {
        props.insert("stroke".into(), json!(s.color.to_hex()));
        props.insert("stroke-width".into(), json!(s.width_px));
        props.insert("dash".into(), json!(format!("{:?}", s.dash)));
        props.insert("dasharray".into(), json!(s.dash.array()));
        if s.dash.round_caps() {
            props.insert("line-cap".into(), json!("round"));
        }
    }
}

fn label_feature(label: &Label) -> Value {
    let align = match label.align {
        TextAlign::Left => "left",
        TextAlign::Center => "center",
        TextAlign::Right => "right",
    };
    json!({
        "type": "Feature",
        "geometry": { "type": "Point", "coordinates": [label.anchor.lon(), label.anchor.lat()] },
        "properties": {
            "graphic": label.pick.definition.as_str(),
            "part": part(&label.pick),
            "label": label.text,
            "rotation": label.rotation_deg,
            "align": align,
            "offset_em": label.offset_em,
            "font": label.font.family,
            "font_size_px": label.font.size_px,
            "bold": label.font.bold,
            "may_hide": label.may_hide,
        }
    })
}

fn part(pick: &PickRef) -> Value {
    match pick.target {
        PickTarget::Part(p) => json!(p.0),
        _ => Value::Null,
    }
}
