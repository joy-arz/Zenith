//! Shared per-texel geometry for a player skin’s outer layer.

/// One quad: the game's `Polygon` (model pixels, then UV in 0..1 of the
/// item's texture).
#[derive(Clone, Debug, PartialEq)]
pub struct Face {
    pub normal: [f32; 3],
    pub vertices: [[f32; 5]; 4],
}

/// The parts of `PlayerModel` that make up the outer layer, as the prepared
/// `entity-models.json` names them.
pub const OUTER_PARTS: [&str; 6] = [
    "hat",
    "jacket",
    "left_sleeve",
    "right_sleeve",
    "left_pants",
    "right_pants",
];

/// The size of a prepared skin image: `resample_64` always hands out this
/// many texels a side, wide and legacy skins alike.
pub const SIZE: usize = 64;

/// How far a texel is pushed out, in model units (1 = 1/16 block = one skin
/// texel): the depth the 3D Skin Layers mod uses by default.
pub const DEPTH: f32 = 1.0;

/// A texel this transparent is not drawn at all (`alpha` of 0–255); the ones
/// just above it stay, so a tinted glass-like layer keeps its translucency.
const ALPHA_CUTOFF: u8 = 8;

/// The slabs a face's texels make, or None for a face without a rectangular texel grid.
pub fn extrude_face(polygon: &Face, skin: &[u8], size: usize, depth: f32) -> Option<Vec<Face>> {
    let (u0, u1, v0, v1) = polygon
        .vertices
        .iter()
        .fold((f32::MAX, f32::MIN, f32::MAX, f32::MIN), |a, v| {
            (a.0.min(v[3]), a.1.max(v[3]), a.2.min(v[4]), a.3.max(v[4]))
        });
    let texels_u = ((u1 - u0) * size as f32).round() as i32;
    let texels_v = ((v1 - v0) * size as f32).round() as i32;
    if texels_u <= 0 || texels_v <= 0 || texels_u > 32 || texels_v > 32 {
        return None;
    }
    let (Some(a), Some(b), Some(c)) = (
        corner(polygon, u0, v0),
        corner(polygon, u1, v0),
        corner(polygon, u0, v1),
    ) else {
        return None;
    };
    let n = glam::Vec3::from_array(polygon.normal).normalize_or_zero();
    if n == glam::Vec3::ZERO {
        return None;
    }
    // The face's own axes, from its UV grid, so the skirt normals point away
    // from the slab no matter how the quad is wound.
    let du = (b - a).normalize_or_zero();
    let dv = (c - a).normalize_or_zero();
    let span_u = (u1 - u0).max(f32::EPSILON);
    let span_v = (v1 - v0).max(f32::EPSILON);
    let at = |u: f32, v: f32| a + (b - a) * ((u - u0) / span_u) + (c - a) * ((v - v0) / span_v);
    let base_u = (u0 * size as f32).round() as i32;
    let base_v = (v0 * size as f32).round() as i32;
    let opaque = |i: i32, j: i32| {
        if i < 0 || j < 0 || i >= texels_u || j >= texels_v {
            return false;
        }
        let x = (base_u + i) as usize;
        let y = (base_v + j) as usize;
        x < size
            && y < size
            && skin
                .get((y * size + x) * 4 + 3)
                .is_some_and(|alpha| *alpha > ALPHA_CUTOFF)
    };
    let mut out = Vec::with_capacity((texels_u * texels_v) as usize);
    let step = 1.0 / size as f32;
    for j in 0..texels_v {
        for i in 0..texels_u {
            if !opaque(i, j) {
                continue;
            }
            let (ua, va) = (u0 + i as f32 * step, v0 + j as f32 * step);
            let (ub, vb) = (ua + step, va + step);
            let (p00, p10) = (at(ua, va), at(ub, va));
            let (p11, p01) = (at(ub, vb), at(ua, vb));
            let lift = n * depth;
            out.push(quad(
                [p00 + lift, p10 + lift, p11 + lift, p01 + lift],
                [(ua, va), (ub, va), (ub, vb), (ua, vb)],
                polygon.normal,
            ));
            // Sides only where the neighbour is not painted: the ones between
            // two texels would be hidden inside the layer and cost fill rate.
            let (uc, vc) = ((ua + ub) * 0.5, (va + vb) * 0.5);
            let sides = [
                ((i - 1, j), -du, [p00, p01]),
                ((i + 1, j), du, [p10, p11]),
                ((i, j - 1), -dv, [p01, p11]),
                ((i, j + 1), dv, [p00, p10]),
            ];
            for ((ni, nj), normal, [s0, s1]) in sides {
                if opaque(ni, nj) {
                    continue;
                }
                let normal = normal.normalize_or_zero().to_array();
                let uv = (uc, vc);
                out.push(quad(
                    [s0, s1, s1 + lift, s0 + lift],
                    [uv, uv, uv, uv],
                    normal,
                ));
            }
        }
    }
    Some(out)
}

fn quad(positions: [glam::Vec3; 4], uvs: [(f32, f32); 4], normal: [f32; 3]) -> Face {
    Face {
        normal,
        vertices: std::array::from_fn(|k| {
            [
                positions[k].x,
                positions[k].y,
                positions[k].z,
                uvs[k].0,
                uvs[k].1,
            ]
        }),
    }
}

fn corner(polygon: &Face, u: f32, v: f32) -> Option<glam::Vec3> {
    polygon
        .vertices
        .iter()
        .find(|a| (a[3] - u).abs() < 1e-4 && (a[4] - v).abs() < 1e-4)
        .map(|a| glam::Vec3::new(a[0], a[1], a[2]))
}
