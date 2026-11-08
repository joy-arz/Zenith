//! Allthemodium's stand-alone dimension data, with the biome feature additions
//! that NeoForge would apply before the generator reads the biomes.
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

const ROOT: &str = "data/allthemodium/";
pub(crate) const TELEPORT_PAD: &str = "allthemodium:teleport_pad";

pub(crate) fn piglich() -> Value {
    serde_json::json!({
        "template": horizon_mods::content::PIGLICH,
        "attributes": {
            "minecraft:armor": 24.0,
            "minecraft:armor_toughness": 24.0,
            "minecraft:attack_damage": 12.0,
            "minecraft:max_health": 9999.0,
            "minecraft:movement_speed": f64::from(0.21_f32),
        }
    })
}

pub(crate) fn pad_template(
    id: &str,
    block: &crate::oracle::Block,
) -> Option<crate::oracle::Choice> {
    if id != TELEPORT_PAD || block.default != "spawned=false" {
        return None;
    }
    let state = block.states.get(&block.default)?;
    let properties = serde_json::json!({
        "hardness": state.hardness,
        "explosion_resistance": block.explosion_resistance,
        "requires_tool": state.requires_tool,
        "friction": block.friction,
        "map_color": state.map_color_id,
        "occludes": false,
    });
    Some(crate::oracle::Choice {
        template: "minecraft:stone".into(),
        properties: properties.as_object()?.clone(),
        differences: Vec::new(),
    })
}

pub(crate) fn pad_shape(block: &crate::oracle::Block, declaration: &mut Value) {
    if let Some(state) = block.states.get(&block.default) {
        declaration["state_properties"] = serde_json::json!({"spawned":{"type":"boolean"}});
        declaration["default_state"] = serde_json::json!({"spawned":"false"});
        declaration["shapes"] = serde_json::json!({"default": state.outline});
        declaration["collision_shapes"] = serde_json::json!({"default": state.collision});
    }
}

pub(crate) fn dimension_source(path: &str) -> bool {
    let Some(path) = path.strip_prefix(ROOT) else {
        return false;
    };
    path.ends_with(".json")
        && [
            "dimension/",
            "dimension_type/",
            "worldgen/biome/",
            "worldgen/noise_settings/",
            "worldgen/density_function/",
            "worldgen/configured_feature/",
            "worldgen/placed_feature/",
            "tags/worldgen/biome/",
            "neoforge/biome_modifier/allthemodium/dim_ores/",
            "neoforge/biome_modifier/allthemodium/the_other/",
        ]
        .iter()
        .any(|prefix| path.starts_with(prefix))
}

fn holders(value: &Value) -> impl Iterator<Item = &str> {
    value.as_str().into_iter().chain(
        value
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(Value::as_str),
    )
}

fn expand_biomes(
    id: &str,
    files: &BTreeMap<String, Value>,
    visited: &mut BTreeSet<String>,
    targets: &mut BTreeSet<String>,
) {
    if let Some(tag) = id.strip_prefix("#allthemodium:") {
        if visited.insert(tag.to_owned())
            && let Some(file) = files.get(&format!("{ROOT}tags/worldgen/biome/{tag}.json"))
        {
            for child in holders(&file["values"]) {
                expand_biomes(child, files, visited, targets);
            }
        }
    } else if let Some(biome) = id.strip_prefix("allthemodium:") {
        targets.insert(biome.to_owned());
    }
}

/// The converted world pack, kept separate from the resource tables that the
/// content root also uses for vanilla biomes.
pub(crate) fn world_pack(files: BTreeMap<String, Vec<u8>>) -> BTreeMap<String, Vec<u8>> {
    if !files.contains_key(&format!("{ROOT}dimension/mining.json")) {
        return BTreeMap::new();
    }
    let files: BTreeMap<String, Value> = files
        .into_iter()
        .filter_map(|(path, bytes)| Some((path, serde_json::from_slice(&bytes).ok()?)))
        .collect();
    let mut pack = BTreeMap::new();
    for (path, value) in &files {
        let rel = path.strip_prefix(ROOT).unwrap_or_default();
        if rel == "tags/worldgen/biome/has_structure/ancient_pyramid_biomes.json"
            || [
                "dimension/",
                "dimension_type/",
                "worldgen/biome/",
                "worldgen/noise_settings/",
                "worldgen/density_function/",
            ]
            .iter()
            .any(|prefix| rel.starts_with(prefix))
        {
            pack.insert(path.clone(), value.clone());
        }
    }
    for (path, modifier) in &files {
        if !path.starts_with(&format!("{ROOT}neoforge/biome_modifier/"))
            || modifier["type"] != "neoforge:add_features"
            || modifier.get("neoforge:conditions").is_some()
        {
            continue;
        }
        let step = match modifier["step"].as_str() {
            Some("underground_ores") => 6,
            Some("vegetal_decoration") => 9,
            _ => continue,
        };
        let mut targets = BTreeSet::new();
        let mut visited = BTreeSet::new();
        for id in holders(&modifier["biomes"]) {
            expand_biomes(id, &files, &mut visited, &mut targets);
        }
        for id in holders(&modifier["features"]) {
            let Some(feature) = id.strip_prefix("allthemodium:") else {
                continue;
            };
            let placed_path = format!("{ROOT}worldgen/placed_feature/{feature}.json");
            let Some(placed) = files.get(&placed_path) else {
                continue;
            };
            pack.insert(placed_path, placed.clone());
            if let Some(configured) = placed["feature"]
                .as_str()
                .and_then(|id| id.strip_prefix("allthemodium:"))
            {
                let path = format!("{ROOT}worldgen/configured_feature/{configured}.json");
                if let Some(config) = files.get(&path) {
                    pack.insert(path, config.clone());
                }
            }
            for target in &targets {
                let Some(biome) = pack.get_mut(&format!("{ROOT}worldgen/biome/{target}.json"))
                else {
                    continue;
                };
                let Some(steps) = biome["features"].as_array_mut() else {
                    continue;
                };
                if steps.len() <= step {
                    steps.resize_with(step + 1, || Value::Array(Vec::new()));
                }
                if let Some(features) = steps[step].as_array_mut() {
                    let feature = Value::from(id);
                    if !features.contains(&feature) {
                        features.push(feature);
                    }
                }
            }
        }
    }
    let pack = pack
        .into_iter()
        .map(|(path, value)| (path, value.to_string().into_bytes()))
        .collect();
    crate::upgrade::upgrade_pack(&pack).0
}
