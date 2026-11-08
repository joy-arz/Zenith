//! Data rules for the HarvestCraft 2 crops and the shared hanging-fruit tree layout.
//! The support sets and canopy coordinates are observations of ATM10 8.5's mods.

use crate::oracle::{Block, Choice};
use serde_json::{Map, Value, json};
use std::path::Path;

const CROPS: &str = "pamhc2crops";
const TREES: &str = "pamhc2trees";
const SOIL: [&str; 10] = [
    "minecraft:coarse_dirt",
    "minecraft:dirt",
    "minecraft:farmland",
    "minecraft:grass_block",
    "minecraft:moss_block",
    "minecraft:mud",
    "minecraft:muddy_mangrove_roots",
    "minecraft:mycelium",
    "minecraft:podzol",
    "minecraft:rooted_dirt",
];
const LEAVES: [&str; 10] = [
    "minecraft:acacia_leaves",
    "minecraft:azalea_leaves",
    "minecraft:birch_leaves",
    "minecraft:cherry_leaves",
    "minecraft:dark_oak_leaves",
    "minecraft:flowering_azalea_leaves",
    "minecraft:jungle_leaves",
    "minecraft:mangrove_leaves",
    "minecraft:oak_leaves",
    "minecraft:spruce_leaves",
];

pub(crate) fn support_tags(id: &str) -> Vec<(&'static str, Vec<&'static str>)> {
    match id {
        CROPS => {
            let mut ground: Vec<_> = SOIL
                .into_iter()
                .filter(|n| *n != "minecraft:farmland")
                .collect();
            ground.extend([
                "minecraft:red_sand",
                "minecraft:sand",
                "minecraft:suspicious_sand",
            ]);
            vec![("garden_support", ground)]
        }
        TREES => vec![
            ("sapling_support", SOIL.to_vec()),
            ("fruit_support", LEAVES.to_vec()),
        ],
        _ => Vec::new(),
    }
}

fn is_garden(id: &str, name: &str, block: &Block) -> bool {
    id == CROPS
        && name.ends_with("garden")
        && block
            .class
            .first()
            .is_some_and(|class| class.ends_with(".GardenBlock"))
}

pub(crate) fn garden_template(id: &str, name: &str, block: &Block) -> Option<Choice> {
    if !is_garden(id, name, block) {
        return None;
    }
    let state = block.states.get(&block.default)?;
    let properties = json!({
        "hardness": state.hardness,
        "explosion_resistance": block.explosion_resistance,
        "requires_tool": state.requires_tool,
        "map_color": state.map_color_id,
        "sounds": "minecraft:wheat",
        "random_ticks": true,
        "occludes": false
    });
    Some(Choice {
        template: "minecraft:poppy".into(),
        properties: properties.as_object()?.clone(),
        differences: Vec::new(),
    })
}

/// Configure the observed shapes and support on top of a carried template.
pub(crate) fn block(id: &str, name: &str, block: &Block, inside: &Path, declaration: &mut Value) {
    let is_fruit = id == TREES && name.starts_with("pam") && block.properties.contains_key("age");
    let is_crop = id == CROPS && name.ends_with("crop") && block.properties.contains_key("age");
    let is_sapling = id == TREES && name.ends_with("_sapling");
    if is_garden(id, name, block) {
        if let Some(state) = block.states.get(&block.default) {
            declaration["shapes"] = json!({"default":state.outline});
            declaration["collision_shapes"] = json!({"default":state.collision});
        }
        declaration["properties"]["plant_support"] =
            json!({"direction":"below","tag":"#pamhc2crops:garden_support"});
    }
    if is_fruit || is_crop {
        let mut outline = Map::new();
        let mut collision = Map::new();
        for (key, state) in &block.states {
            let Some(age) = key.strip_prefix("age=") else {
                return;
            };
            outline.insert(age.into(), state.outline.clone());
            collision.insert(age.into(), state.collision.clone());
        }
        declaration["shapes"] = json!({"property":"age","values":outline});
        declaration["collision_shapes"] = json!({"property":"age","values":collision});
    }
    if is_fruit {
        declaration["properties"]["occludes"] = json!(false);
        if block
            .class
            .first()
            .is_some_and(|class| class.ends_with(".BlockPamLogFruit"))
        {
            declaration["properties"]["plant_support"] = json!({"direction":"none"});
        } else {
            declaration["properties"]["plant_support"] =
                json!({"direction":"above","tag":"#pamhc2trees:fruit_support"});
        }
    }
    if is_sapling {
        declaration["properties"]["plant_support"] =
            json!({"direction":"below","tag":"#pamhc2trees:sapling_support"});
        let name = name.trim_end_matches("_sapling");
        let name = if name == "guava" { "gauva" } else { name };
        let path = inside.join(format!(
            "data/{TREES}/worldgen/configured_feature/{name}.json"
        ));
        if std::fs::read(path)
            .ok()
            .and_then(|bytes| serde_json::from_slice::<Value>(&bytes).ok())
            .is_some_and(|feature| {
                matches!(
                    feature["foliage_placer"]["type"].as_str(),
                    Some("horizon:layered_foliage_placer" | "horizon:fruit_spruce_foliage_placer")
                )
            })
        {
            declaration["properties"]["tree_feature"] = json!(format!("{TREES}:{name}"));
        }
    }
}

pub(crate) fn tree(id: &str, name: &str, feature: &Value) -> Option<Value> {
    if id != TREES || feature["type"] != "minecraft:tree" {
        return None;
    }
    let mut config = feature["config"].as_object()?.clone();
    let foliage_type = config.get("foliage_placer")?["type"].as_str()?;
    if !foliage_type.starts_with("pamhc2trees:")
        || !foliage_type.ends_with("_fruit_placer")
        || config.get("trunk_placer")?["type"] != "minecraft:straight_trunk_placer"
        || config.remove("force_dirt")? != false
    {
        return None;
    }
    let soil = config.remove("dirt_provider")?;
    if soil["state"]["id"]
        .as_str()
        .or_else(|| soil["state"]["Name"].as_str())
        != Some("minecraft:dirt")
    {
        return None;
    }
    config.insert("type".into(), feature["type"].clone());
    config.insert(
        "below_trunk_provider".into(),
        json!("minecraft:soil_beneath_tree"),
    );
    for provider in ["trunk_provider", "foliage_provider"] {
        let state = config.get(provider)?.get("state")?.clone();
        config.insert(provider.into(), state);
    }
    let fruit = if name == "gauva" { "guava" } else { name };
    let fruit = json!({"id":format!("{TREES}:pam{fruit}"),"properties":{"age":"0"}});
    let mut layers = vec![
        json!({"y":1,"radius":0}),
        json!({"y":0,"radius":0}),
        json!({"y":-1,"radius":1}),
        json!({"y":-2,"radius":1}),
        json!({"y":-3,"radius":2}),
    ];
    if matches!(name, "cinnamon" | "maple" | "paperbark") {
        let trunk = config.get_mut("trunk_placer")?.as_object_mut()?;
        trunk.insert(
            "type".into(),
            json!("horizon:segmented_straight_trunk_placer"),
        );
        trunk.insert("top_height".into(), json!(3));
        trunk.insert("base_provider".into(), fruit.clone());
    } else if name != "pinenut" {
        layers.push(json!({"y":-4,"radius":1,"provider":fruit.clone()}));
    }
    let foliage = if name == "pinenut" {
        json!({
            "type":"horizon:fruit_spruce_foliage_placer", "radius":2, "offset":0,
            "trunk_height":12, "fruit_provider":fruit
        })
    } else {
        json!({
        "type": "horizon:layered_foliage_placer",
        "radius": 0,
        "offset": 0,
        "layers": layers
        })
    };
    config.insert("foliage_placer".into(), foliage);
    Some(Value::Object(config))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn guava_keeps_the_original_feature_id_and_places_the_registered_fruit() {
        let legacy = json!({
            "type": "minecraft:tree",
            "config": {
                "force_dirt": false,
                "dirt_provider": {"state": {"Name": "minecraft:dirt"}},
                "foliage_provider": {"state": {"Name": "minecraft:jungle_leaves"}},
                "trunk_provider": {"state": {"Name": "minecraft:jungle_log"}},
                "trunk_placer": {"type": "minecraft:straight_trunk_placer"},
                "foliage_placer": {"type": "pamhc2trees:guava_fruit_placer"}
            }
        });
        let feature = tree(TREES, "gauva", &legacy).unwrap();
        assert_eq!(
            feature["trunk_provider"],
            json!({"Name":"minecraft:jungle_log"})
        );
        assert_eq!(
            feature["below_trunk_provider"],
            "minecraft:soil_beneath_tree"
        );
        assert_eq!(
            feature["foliage_placer"]["layers"][5]["provider"]["id"],
            "pamhc2trees:pamguava"
        );
        assert!(tree("other", "gauva", &legacy).is_none());
    }
}
