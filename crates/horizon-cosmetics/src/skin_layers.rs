//! Outer skin geometry for the launcher preview.

use crate::{Part, Texture};
pub use horizon_platform::skin_layers::{DEPTH, OUTER_PARTS, SIZE, extrude_face};

/// Apply the same per-texel depth as the game to the preview model.
pub fn extrude_preview(model: &mut Part, skin: &Texture) {
    if skin.width != SIZE as u32 || skin.height != SIZE as u32 || skin.rgba.len() != SIZE * SIZE * 4
    {
        return;
    }
    extrude_part(model, &skin.rgba);
}

fn extrude_part(part: &mut Part, skin: &[u8]) {
    if OUTER_PARTS.contains(&part.name.as_str()) {
        part.faces = part
            .faces
            .drain(..)
            .flat_map(|face| extrude_face(&face, skin, SIZE, DEPTH).unwrap_or_else(|| vec![face]))
            .collect();
    }
    for child in &mut part.children {
        extrude_part(child, skin);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preview_extrudes_a_painted_hat_texel_and_preserves_the_head() {
        let mut model = crate::preview::player_model(false);
        let base = model.children[0].faces.clone();
        let mut texture = Texture {
            width: 64,
            height: 64,
            rgba: vec![0; 64 * 64 * 4],
        };
        texture.rgba[(8 * 64 + 40) * 4 + 3] = 255;
        extrude_preview(&mut model, &texture);
        assert_eq!(model.children[0].faces, base);
        let hat = &model.children[0].children[0];
        assert_eq!(hat.faces.len(), 5);
        assert!(
            hat.faces[0]
                .vertices
                .iter()
                .all(|vertex| vertex[2].abs() > 5.)
        );
    }
}
