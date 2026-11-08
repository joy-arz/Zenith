//! Split 1.21.1 FoodProperties into the food, consumable and use-remainder
//! components consumed by the Rust engine. Inputs come from the running oracle.
use serde_json::{Map, Value, json};

pub fn components(
    old: &Value,
    animation: Option<&str>,
    use_ticks: Option<u32>,
) -> Result<Map<String, Value>, String> {
    let nutrition = old["nutrition"].as_u64().ok_or("food: no nutrition")?;
    let saturation = old["saturation"].as_f64().ok_or("food: no saturation")?;
    if saturation < 0.0 {
        return Err("food: negative saturation".into());
    }
    let seconds = use_ticks
        .map(|t| t as f64 / 20.0)
        .unwrap_or_else(|| old["seconds_to_eat"].as_f64().unwrap_or(1.6));
    if seconds <= 0.0 {
        return Err("food: non-positive use duration".into());
    }
    let animation = animation.unwrap_or("eat");
    if !matches!(animation, "eat" | "drink") {
        return Err(format!("food: unsupported use animation {animation}"));
    }
    let mut food = json!({"nutrition": nutrition, "saturation": saturation});
    if let Some(always) = old.get("can_always_eat") {
        if !always.is_boolean() {
            return Err("food: can_always_eat is not a boolean".into());
        }
        food["can_always_eat"] = always.clone();
    }
    let mut consume = json!({"consume_seconds": seconds, "animation": animation});
    if animation == "drink" {
        consume["sound"] = json!("minecraft:entity.generic.drink");
        consume["has_consume_particles"] = json!(false);
    }
    if let Some(effects) = old.get("effects") {
        let effects = effects.as_array().ok_or("food: effects is not a list")?;
        let mut upgraded = Vec::new();
        for entry in effects {
            let effect = entry
                .get("effect")
                .filter(|e| e.is_object())
                .ok_or("food: no effect instance")?;
            if effect["id"].as_str().is_none() {
                return Err("food: effect has no id".into());
            }
            let probability = entry
                .get("probability")
                .map_or(Some(1.0), Value::as_f64)
                .ok_or("food: effect probability is not numeric")?;
            if !(0.0..=1.0).contains(&probability) {
                return Err("food: effect probability outside 0..=1".into());
            }
            // One entry per old effect preserves separate random trials, including
            // repeated effects. Grouping them would change the probability contract.
            upgraded.push(json!({"type": "minecraft:apply_effects",
                "effects": [effect], "probability": probability}));
        }
        consume["on_consume_effects"] = json!(upgraded);
    }
    let mut patch = Map::from_iter([
        ("minecraft:food".into(), food),
        ("minecraft:consumable".into(), consume),
    ]);
    if let Some(remainder) = old.get("using_converts_to") {
        if !remainder.is_object() || remainder["id"].as_str().is_none() {
            return Err("food: invalid use remainder".into());
        }
        if remainder
            .get("count")
            .is_some_and(|c| c.as_u64() != Some(1))
            || remainder
                .get("components")
                .is_some_and(|c| c.as_object().is_none_or(|c| !c.is_empty()))
        {
            return Err(
                "food: component-bearing or multiple use remainders need native logic".into(),
            );
        }
        patch.insert("minecraft:use_remainder".into(), remainder.clone());
    }
    Ok(patch)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn food_keeps_absolute_saturation_timing_and_independent_effect_trials() {
        let got = components(&json!({"nutrition": 4, "saturation": 2.4,
            "can_always_eat": true, "seconds_to_eat": 0.8,
            "effects": [
                {"effect": {"id": "minecraft:speed", "duration": 160, "amplifier": 1}, "probability": 0.25},
                {"effect": {"id": "minecraft:poison", "duration": 60}, "probability": 0.5}
            ]}), None, None).unwrap();
        assert_eq!(
            got["minecraft:food"],
            json!({"nutrition": 4,
            "saturation": 2.4, "can_always_eat": true})
        );
        assert_eq!(
            got["minecraft:consumable"],
            json!({"consume_seconds": 0.8,
            "animation": "eat", "on_consume_effects": [
                {"type": "minecraft:apply_effects", "effects": [{"id": "minecraft:speed", "duration": 160, "amplifier": 1}], "probability": 0.25},
                {"type": "minecraft:apply_effects", "effects": [{"id": "minecraft:poison", "duration": 60}], "probability": 0.5}
            ]})
        );
    }

    #[test]
    fn observed_drink_duration_overrides_food_and_keeps_the_bottle() {
        let got = components(
            &json!({"nutrition": 6, "saturation": 1.2,
            "using_converts_to": {"id": "minecraft:glass_bottle"}}),
            Some("drink"),
            Some(40),
        )
        .unwrap();
        assert_eq!(
            got["minecraft:consumable"],
            json!({"consume_seconds": 2.0,
            "animation": "drink", "sound": "minecraft:entity.generic.drink",
            "has_consume_particles": false})
        );
        assert_eq!(
            got["minecraft:use_remainder"],
            json!({"id": "minecraft:glass_bottle"})
        );
    }

    #[test]
    fn unsupported_or_malformed_food_is_not_silently_downgraded() {
        for old in [
            json!({"nutrition": 4}),
            json!({"nutrition": 4, "saturation": -1}),
            json!({"nutrition": 4, "saturation": 1, "effects": [{"effect": {"id": "minecraft:speed"}, "probability": 2}]}),
            json!({"nutrition": 4, "saturation": 1, "using_converts_to": {"id": "minecraft:bowl", "count": 2}}),
            json!({"nutrition": 4, "saturation": 1, "using_converts_to": {"id": "minecraft:bowl", "components": {"minecraft:custom_name": "x"}}}),
        ] {
            assert!(components(&old, None, None).is_err(), "{old}");
        }
        assert!(components(&json!({"nutrition": 4, "saturation": 1}), Some("bow"), None).is_err());
        assert!(components(&json!({"nutrition": 4, "saturation": 1}), None, Some(0)).is_err());
    }
}
