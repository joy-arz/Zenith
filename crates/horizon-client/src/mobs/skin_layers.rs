//! 3D skin layers (off by default): the player's outer skin layer drawn as
//! its own pixels, each opaque texel extruded out of the model as a small
//! slab (`threeDSkinLayers` in horizon-extras.txt, Options → Skin Customization).
//!
//! Vanilla draws the outer layer as one box inflated by 0.25 px, so a hat,
//! jacket, sleeves and trousers read as a flat skin over the body. Here each
//! texel of those boxes that the skin paints becomes a slab one texel deep:
//! the face pushed out along the box's normal, its sides filled in where the
//! neighbouring texel is transparent. Hair, hoods and hems then have a
//! silhouette of their own, and the gaps between painted texels stay open.
//!
//! The geometry is built once per (skin texture, slim/wide) from the decoded
//! 64×64 RGBA skin and cached by the game; the model tree, part names and the
//! texture's normalised UVs are the ones the renderer already uses, so both
//! the CPU and the GPU (`super::skin`) pose paths draw it unchanged.
use super::{Part, Polygon};

pub use horizon_cosmetics::skin_layers::{DEPTH, SIZE};
use horizon_cosmetics::skin_layers::{OUTER_PARTS, extrude_face};

/// `model` with every outer-layer part replaced by its extruded geometry,
/// or `None` when the model has no outer layer to extrude.
pub fn extrude(model: &Part, skin: &[u8], size: usize, depth: f32) -> Option<Part> {
    if skin.len() < size * size * 4 {
        return None;
    }
    let mut changed = false;
    let mut out = model.clone();
    explode(&mut out, skin, size, depth, &mut changed);
    changed.then_some(out)
}

fn explode(part: &mut Part, skin: &[u8], size: usize, depth: f32, changed: &mut bool) {
    if OUTER_PARTS.contains(&part.name.as_str()) {
        let mut polygons = Vec::with_capacity(part.polygons.len() * 8);
        for polygon in &part.polygons {
            let face = horizon_cosmetics::Face {
                normal: polygon.normal,
                vertices: polygon.vertices,
            };
            match extrude_face(&face, skin, size, depth) {
                Some(faces) => {
                    *changed = true;
                    polygons.extend(faces.into_iter().map(|face| Polygon {
                        normal: face.normal,
                        vertices: face.vertices,
                        cull_back: false,
                    }));
                }
                None => polygons.push(polygon.clone()),
            }
        }
        part.polygons = polygons;
    }
    for child in &mut part.children {
        explode(child, skin, size, depth, changed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A 2×2-texel face facing +Z, its four corners at the given depth.
    fn face() -> Polygon {
        Polygon {
            normal: [0.0, 0.0, 1.0],
            vertices: [
                [-1.0, -1.0, 0.5, 0.0, 0.0],
                [1.0, -1.0, 0.5, 0.5, 0.0],
                [1.0, 1.0, 0.5, 0.5, 0.5],
                [-1.0, 1.0, 0.5, 0.0, 0.5],
            ],
            cull_back: false,
        }
    }

    fn skin(alpha: [u8; 4]) -> Vec<u8> {
        let mut skin = vec![0; 64 * 64 * 4];
        for (i, a) in alpha.iter().enumerate() {
            skin[i * 4 + 3] = *a;
        }
        skin
    }

    #[test]
    fn one_slab_and_four_sides_for_a_single_painted_texel() {
        let model = Part {
            name: "hat".into(),
            pose: [0.0; 9],
            polygons: vec![face()],
            children: Vec::new(),
        };
        let out = extrude(&model, &skin([255, 0, 0, 0]), 64, 1.0).expect("a hat");
        // One face plus four skirts.
        assert_eq!(out.polygons.len(), 5);
        let top = out.polygons[0].vertices;
        for v in top {
            assert!(v[2] > 1.4, "the slab is pushed out: {v:?}");
        }
    }

    #[test]
    fn a_transparent_face_draws_nothing() {
        let model = Part {
            name: "jacket".into(),
            pose: [0.0; 9],
            polygons: vec![face()],
            children: Vec::new(),
        };
        let out = extrude(&model, &skin([0, 0, 0, 0]), 64, 1.0).expect("a jacket");
        assert!(out.polygons.is_empty());
    }

    #[test]
    fn a_part_that_is_not_an_outer_layer_is_untouched() {
        let model = Part {
            name: "head".into(),
            pose: [0.0; 9],
            polygons: vec![face()],
            children: Vec::new(),
        };
        assert!(extrude(&model, &skin([255, 255, 255, 255]), 64, 1.0).is_none());
    }
}
