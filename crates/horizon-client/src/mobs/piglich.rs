//! The Piglich's cuboid geometry uses the same baked parts as vanilla mobs.
use super::{
    Part, Polygon,
    anim::{Poses, State},
    keyframes::{Animation, Channel, Keyframe, Target},
    model_layers::{
        CubeDeformation, CubeListBuilder, LayerDefinition, MeshDefinition, ModelPart, PartPose,
    },
};
use serde::Deserialize;
use serde_json::Value;
use std::{collections::HashMap, fs, path::Path};

pub(super) const MODEL: &str = "piglich#main";
pub(super) const TEXTURE: &str = "allthemodium:entity/piglich";

#[derive(Deserialize)]
struct Document {
    #[serde(rename = "minecraft:geometry")]
    geometry: Vec<Geometry>,
}

#[derive(Deserialize)]
struct Geometry {
    description: TextureSize,
    bones: Vec<Bone>,
}

#[derive(Deserialize)]
struct TextureSize {
    texture_width: u32,
    texture_height: u32,
}

#[derive(Deserialize)]
struct Bone {
    name: String,
    parent: Option<String>,
    pivot: [f32; 3],
    #[serde(default)]
    rotation: [f32; 3],
    #[serde(default)]
    mirror: bool,
    #[serde(default)]
    cubes: Vec<Cube>,
}

#[derive(Deserialize)]
struct Cube {
    origin: [f32; 3],
    size: [f32; 3],
    uv: [i32; 2],
    pivot: Option<[f32; 3]>,
    #[serde(default)]
    rotation: [f32; 3],
    mirror: Option<bool>,
    #[serde(default)]
    inflate: f32,
}

fn pose(pivot: [f32; 3], parent: [f32; 3], rotation: [f32; 3]) -> PartPose {
    let [x, y, z] = std::array::from_fn(|i| pivot[i] - parent[i]);
    let [rx, ry, rz] = rotation.map(f32::to_radians);
    PartPose::offset_and_rotation(x, -y, z, rx, ry, rz)
}

impl Cube {
    fn bake(&self, builder: CubeListBuilder, pivot: [f32; 3], mirror: bool) -> CubeListBuilder {
        let [x, y, z] = std::array::from_fn(|i| self.origin[i] - pivot[i]);
        builder
            .tex_offs(self.uv[0], self.uv[1])
            .set_mirror(self.mirror.unwrap_or(mirror))
            .add_box_with(
                [x, -y - self.size[1], z],
                self.size,
                CubeDeformation::new(self.inflate),
            )
    }
}

fn part(name: &str, model: &ModelPart) -> Part {
    let p = model.store_pose();
    Part {
        name: name.into(),
        pose: [
            p.x, p.y, p.z, p.x_rot, p.y_rot, p.z_rot, p.x_scale, p.y_scale, p.z_scale,
        ],
        polygons: model
            .polygons
            .iter()
            .map(|p| Polygon {
                normal: p.normal,
                vertices: p.vertices.map(|v| [v.pos[0], v.pos[1], v.pos[2], v.u, v.v]),
                cull_back: true,
            })
            .collect(),
        children: model
            .children()
            .map(|(name, model)| part(name, model))
            .collect(),
    }
}

pub(super) fn model(root: &Path) -> Option<Part> {
    let assets = root.join("assets/allthemodium");
    if !assets.join("textures/entity/piglich.png").is_file() {
        return None;
    }
    let bytes = fs::read(assets.join("geo/piglich_anim.geo.json")).ok()?;
    let document: Document = serde_json::from_slice(&bytes).ok()?;
    let g = document.geometry.first()?;
    let TextureSize {
        texture_width,
        texture_height,
    } = g.description;
    if !(1..=4096).contains(&texture_width)
        || !(1..=4096).contains(&texture_height)
        || g.bones.is_empty()
        || g.bones.len() > 64
    {
        return None;
    }
    let mut mesh = MeshDefinition::new();
    let mut bones = HashMap::new();
    // The mesh renderer puts the ground 24 texels below its origin.
    let ground = [0.0, 24.0, 0.0];
    for bone in &g.bones {
        if bones.contains_key(bone.name.as_str()) || bone.cubes.len() > 64 {
            return None;
        }
        let (parent, parent_pivot) = match &bone.parent {
            Some(name) => *bones.get(name.as_str())?,
            None => (mesh.root(), ground),
        };
        let mut builder = CubeListBuilder::create();
        for cube in bone.cubes.iter().filter(|c| c.rotation == [0.0; 3]) {
            builder = cube.bake(builder, bone.pivot, bone.mirror);
        }
        let id = mesh.add_or_replace_child(
            parent,
            &bone.name,
            builder,
            pose(bone.pivot, parent_pivot, bone.rotation),
        );
        bones.insert(bone.name.as_str(), (id, bone.pivot));
        for (index, cube) in bone
            .cubes
            .iter()
            .enumerate()
            .filter(|(_, c)| c.rotation != [0.0; 3])
        {
            let pivot = cube.pivot.unwrap_or(bone.pivot);
            mesh.add_or_replace_child(
                id,
                &format!("{}/cube/{index}", bone.name),
                cube.bake(CubeListBuilder::create(), pivot, bone.mirror),
                pose(pivot, bone.pivot, cube.rotation),
            );
        }
    }
    Some(part(
        "",
        &LayerDefinition::create(mesh, texture_width, texture_height).bake_root(),
    ))
}

// The source uses numerical vectors and the easing attached to the
// destination key. Bake the curves to the existing linear keyframe format.
fn eased(t: f32, kind: Option<&str>) -> f32 {
    use std::f32::consts::{FRAC_PI_2, PI};
    match kind {
        Some("easeInSine") => 1.0 - (FRAC_PI_2 * t).cos(),
        Some("easeOutSine") => (FRAC_PI_2 * t).sin(),
        Some("easeInOutSine") => 0.5 - (PI * t).cos() * 0.5,
        Some("easeInQuad") => t * t,
        Some("easeOutQuad") => t * (2.0 - t),
        Some("easeInOutQuad") if t < 0.5 => 2.0 * t * t,
        Some("easeInOutQuad") => 1.0 - 2.0 * (1.0 - t).powi(2),
        Some("easeInOutCubic") if t < 0.5 => 4.0 * t.powi(3),
        Some("easeInOutCubic") => 1.0 - 4.0 * (1.0 - t).powi(3),
        _ => t,
    }
}

fn channel(name: &str, source: &Value) -> Option<Channel> {
    let target = match name {
        "position" => Target::Position,
        "rotation" => Target::Rotation,
        "scale" => Target::Scale,
        _ => return None,
    };
    let mut keys = Vec::new();
    for (time, frame) in source.as_object().filter(|keys| keys.len() <= 128)? {
        let time = time.parse::<f32>().ok().filter(|t| t.is_finite())?;
        let [x, y, z] = serde_json::from_value::<[f32; 3]>(frame["vector"].clone()).ok()?;
        let vector = match target {
            Target::Position => [x, -y, z],
            Target::Rotation => [x, y, z].map(f32::to_radians),
            Target::Scale => [x - 1.0, y - 1.0, z - 1.0],
        };
        keys.push((time, vector, frame["easing"].as_str()));
    }
    keys.sort_by(|a, b| a.0.total_cmp(&b.0));
    let &(time, v, _) = keys.first()?;
    let key = |time, v| Keyframe {
        time,
        pre: v,
        post: v,
        catmull: false,
    };
    let mut keyframes = vec![key(time, v)];
    for pair in keys.windows(2) {
        let [(start, a, _), (end, b, easing)] = [pair[0], pair[1]];
        if end <= start {
            return None;
        }
        let subdivisions = if easing.is_some() { 8 } else { 1 };
        for i in 1..=subdivisions {
            let t = i as f32 / subdivisions as f32;
            let p = eased(t, easing);
            keyframes.push(key(
                start + (end - start) * t,
                std::array::from_fn(|j| a[j] + (b[j] - a[j]) * p),
            ));
        }
    }
    Some(Channel { target, keyframes })
}

pub(super) fn animations(root: &Path) -> impl Iterator<Item = (String, Animation)> {
    let file = root.join("assets/allthemodium/animations/piglich.animation.json");
    let doc = fs::read(file)
        .ok()
        .and_then(|b| serde_json::from_slice::<Value>(&b).ok());
    ["idle", "walk", "run", "meleeattack"]
        .into_iter()
        .filter_map(move |name| {
            let id = format!("{name}.piglich.nik");
            let a = doc.as_ref()?.get("animations")?.get(&id)?;
            let length = a["animation_length"].as_f64()? as f32;
            if length <= 0.0 || !length.is_finite() {
                return None;
            }
            let bones = a["bones"]
                .as_object()
                .filter(|bones| bones.len() <= 64)?
                .iter()
                .filter_map(|(name, bone)| {
                    let channels = bone
                        .as_object()?
                        .iter()
                        .filter_map(|(name, c)| channel(name, c))
                        .collect();
                    Some((name.clone(), channels))
                })
                .collect();
            Some((
                id,
                Animation {
                    length,
                    looping: a["loop"].as_bool().unwrap_or(false),
                    bones,
                },
            ))
        })
}

pub(super) fn animate(s: &State, p: &mut Poses, animations: &HashMap<String, Animation>) {
    let (key, time) = if s.swing > 0.0 {
        ("meleeattack.piglich.nik", s.swing * 1000.0)
    } else if s.speed > 0.5 {
        ("run.piglich.nik", s.pos * 95.0)
    } else if s.speed > 0.025 {
        ("walk.piglich.nik", s.pos * 95.0)
    } else {
        ("idle.piglich.nik", s.age * 50.0)
    };
    if let Some(animation) = animations.get(key) {
        animation.apply(time as i64, 1.0, p);
    }
    let head = p.get("head");
    head[3] += s.x_rot.to_radians();
    head[4] += s.y_rot.to_radians();
}
