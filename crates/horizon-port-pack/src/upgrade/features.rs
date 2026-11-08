//! Configured and placed features.
use super::{Report, kind, states, vanilla};
use serde_json::{Map, Value, json};

/// Feature types 26.3 dropped for compositions of other features; a pack
/// using them needs vanilla's composition, which the upgrade does not
/// write.
const REPLACED_FEATURES: [&str; 12] = [
    "basalt_columns",
    "basalt_pillar",
    "coral_mushroom",
    "desert_well",
    "glowstone_blob",
    "kelp",
    "nether_forest_vegetation",
    "sea_pickle",
    "seagrass",
    "twisting_vines",
    "weeping_vines",
    "random_patch",
];

/// 26.2 block state provider types and their 26.3 names.
fn provider(t: &str) -> Option<&'static str> {
    Some(match t {
        "simple_state_provider" => "simple_state_provider",
        "weighted_state_provider" => "weighted",
        "rotated_block_provider" => "rotated",
        "randomized_int_state_provider" => "randomized_int",
        "rule_based_state_provider" => "rule_based",
        "noise_provider" => "noise",
        "dual_noise_provider" => "dual_noise",
        "noise_threshold_provider" => "noise_threshold",
        _ => return None,
    })
}

/// One placement modifier: `random_offset` became `offset` with one
/// provider per axis (x and z both from `xz_spread`, drawn in that order).
fn placement(m: &mut Value) {
    if kind(m) == Some("random_offset") {
        let xz = m.get("xz_spread").cloned().unwrap_or(json!(0));
        let y = m.get("y_spread").cloned().unwrap_or(json!(0));
        *m = json!({"type": "minecraft:offset", "x": xz.clone(), "y": y, "z": xz});
    }
}

/// A configured feature in 26.3's form: `config` unwrapped, block state
/// providers renamed, nested features and placements upgraded.
pub(super) fn feature(v: &Value, report: &mut Report) -> Value {
    let Value::Object(o) = v else {
        return v.clone();
    };
    let Some(t) = kind(v) else {
        return v.clone();
    };
    if let Some(patch) = random_patch(v, report) {
        return patch;
    }
    if REPLACED_FEATURES.contains(&t) {
        report.unknown(format!("feature {t} (written as data in 26.3)"));
    }
    let mut n = Map::new();
    n.insert("type".into(), o["type"].clone());
    if let Some(Value::Object(config)) = o.get("config") {
        for (k, x) in config {
            n.insert(k.clone(), x.clone());
        }
    }
    if t == "tree"
        && let Some(dirt) = n.remove("dirt_provider")
    {
        let force = n.remove("force_dirt").and_then(|value| value.as_bool()) == Some(true);
        let below_trunk = if force {
            dirt
        } else {
            json!({
                "type": "minecraft:rule_based",
                "rules": [{
                    "if_true": {"type": "minecraft:any_of", "predicates": [
                        {"type": "minecraft:not", "predicate": {
                            "type": "minecraft:matching_block_tag", "tag": "minecraft:dirt"
                        }},
                        {"type": "minecraft:matching_blocks",
                            "blocks": ["minecraft:grass_block", "minecraft:mycelium"]}
                    ]},
                    "then": dirt
                }]
            })
        };
        n.insert("below_trunk_provider".into(), below_trunk);
    }
    let mut v = Value::Object(n);
    states(&mut v);
    nested(&mut v, report);
    v
}

/// Legacy vanilla trees also appear as configured features in mod archives.
pub(crate) fn legacy_tree(v: &Value) -> Option<Value> {
    (kind(v) == Some("tree") && v.pointer("/config/dirt_provider").is_some())
        .then(|| feature(v, &mut Report::default()))
}

/// A legacy patch repeats its placed feature with three differences of
/// uniform draws. A symmetric trapezoid with zero plateau has that sampling
/// order, and placement evaluates the child before drawing the next offset.
pub(crate) fn random_patch(v: &Value, report: &mut Report) -> Option<Value> {
    if kind(v) != Some("random_patch") {
        return None;
    }
    let config = v.get("config")?;
    let tries = config.get("tries")?.as_i64()?;
    let horizontal = config.get("xz_spread")?.as_i64()?;
    let vertical = config.get("y_spread")?.as_i64()?;
    if !(1..=4096).contains(&tries)
        || !(0..=16).contains(&horizontal)
        || !(0..=16).contains(&vertical)
    {
        return None;
    }
    let triangle = |spread: i64| {
        json!({
            "type": "minecraft:trapezoid", "min": -spread, "max": spread, "plateau": 0
        })
    };
    let child = config.get("feature")?;
    let mut modifiers = vec![
        json!({"type": "minecraft:count", "count": tries}),
        json!({"type": "minecraft:offset",
            "x": triangle(horizontal), "y": triangle(vertical), "z": triangle(horizontal)}),
    ];
    let configured = if child.is_string() {
        json!({"type": "minecraft:sequence", "features": [child]})
    } else {
        child.get("feature")?;
        child.get("placement")?.as_array()?;
        let mut child = placed(child, report);
        modifiers.append(child.get_mut("placement")?.as_array_mut()?);
        child.get_mut("feature")?.take()
    };
    Some(json!({"type": "minecraft:sequence", "features": [{
        "feature": configured, "placement": modifiers
    }]}))
}

/// Legacy ore features already use the current rule tests and fields.
pub(crate) fn ore(v: &Value) -> Option<Value> {
    if !matches!(kind(v), Some("ore" | "scattered_ore")) {
        return None;
    }
    let mut config = v.get("config")?.as_object()?.clone();
    config.insert("type".into(), v["type"].clone());
    let mut value = Value::Object(config);
    states(&mut value);
    Some(value)
}

/// Inside a feature: block state providers, placed features and
/// configured features held inline.
fn nested(v: &mut Value, report: &mut Report) {
    match v {
        Value::Object(o) => {
            if let Some(t) = kind(&Value::Object(o.clone()))
                && let Some(to) = provider(t)
            {
                if t == "simple_state_provider" {
                    // A direct provider is a state object: a bare string
                    // would name a provider in its registry instead.
                    *v = match o.get("state").cloned().unwrap_or_default() {
                        id @ Value::String(_) => json!({"id": id}),
                        state => state,
                    };
                    return;
                }
                o.insert("type".into(), vanilla(to));
            }
            // An inline placed feature: {feature, placement}.
            if o.contains_key("placement") && o.contains_key("feature") {
                let placed = placed(&Value::Object(o.clone()), report);
                *v = placed;
                return;
            }
            for x in o.values_mut() {
                if x.get("config").is_some() && x.get("type").is_some() {
                    *x = feature(x, report);
                } else {
                    nested(x, report);
                }
            }
        }
        Value::Array(a) => a.iter_mut().for_each(|x| nested(x, report)),
        _ => {}
    }
}

/// A placed feature: its feature (an id or inline) and its placement
/// modifiers.
pub(super) fn placed(v: &Value, report: &mut Report) -> Value {
    let mut v = v.clone();
    states(&mut v);
    if let Some(f) = v.get("feature").cloned()
        && f.is_object()
    {
        v["feature"] = feature(&f, report);
    }
    if let Some(Value::Array(ms)) = v.get_mut("placement") {
        for m in ms.iter_mut() {
            placement(m);
        }
    }
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn random_patch_repeats_the_child_and_retains_its_filter() {
        let legacy = json!({
            "type": "minecraft:random_patch",
            "config": {
                "tries": 32, "xz_spread": 6, "y_spread": 2,
                "feature": {
                    "feature": "farm:plant",
                    "placement": [{"type": "minecraft:block_predicate_filter",
                        "predicate": {"type": "minecraft:matching_blocks", "blocks": "minecraft:air"}}]
                }
            }
        });
        let mut report = Report::default();
        let converted = feature(&legacy, &mut report);
        let child = &converted["features"][0];
        assert_eq!(converted["type"], "minecraft:sequence");
        assert_eq!(child["feature"], "farm:plant");
        assert_eq!(
            child["placement"][0],
            json!({"type": "minecraft:count", "count": 32})
        );
        assert_eq!(
            child["placement"][1],
            json!({"type": "minecraft:offset",
            "x": {"type": "minecraft:trapezoid", "min": -6, "max": 6, "plateau": 0},
            "y": {"type": "minecraft:trapezoid", "min": -2, "max": 2, "plateau": 0},
            "z": {"type": "minecraft:trapezoid", "min": -6, "max": 6, "plateau": 0}})
        );
        assert_eq!(
            child["placement"][2],
            legacy["config"]["feature"]["placement"][0]
        );
        assert!(report.unknown.is_empty());
    }
}
