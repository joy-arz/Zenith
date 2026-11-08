//! Server-side use and arrival of Allthemodium's crafted teleport pads.
use super::{Dim, Game, server_travel::LevelTeleport};
use horizon_client::{Result, blocks::Fluid};
use horizon_core::geometry::Vec3;

const PAD: &str = "allthemodium:teleport_pad";

impl Game<'_> {
    pub(super) fn use_allthemodium_pad(&mut self, pos: [i32; 3]) -> bool {
        if self.block_name(pos).as_ref() != PAD || self.spectator() {
            return false;
        }
        let Some(conn) = self.lan_acting else {
            return false;
        };
        let destination = match self.dim.id() {
            "minecraft:overworld" => "allthemodium:mining",
            "allthemodium:mining" => "minecraft:overworld",
            "minecraft:the_nether" => "allthemodium:the_other",
            "allthemodium:the_other" => "minecraft:the_nether",
            "minecraft:the_end" => "allthemodium:the_beyond",
            "allthemodium:the_beyond" => "minecraft:the_end",
            _ => return false,
        };
        if self.level_shapes.contains(destination)
            && !self.awaiting_server_landing(conn)
            && !self.level_teleports.iter().any(|trip| trip.conn == conn)
        {
            self.level_teleports.push(LevelTeleport {
                conn,
                dim: Dim::from_id(destination),
                at: [
                    f64::from(pos[0]) + 0.5,
                    f64::from(pos[1]),
                    f64::from(pos[2]) + 0.5,
                ],
                rotation: None,
                vehicle: None,
                cause: Some("unknown"),
                pad: true,
            });
        }
        true
    }

    pub(super) fn allthemodium_pad_arrival(&mut self, target: Vec3) -> Result<Option<Vec3>> {
        let registry = self.pack.assets.registry;
        let Some(default) = registry.default_state(PAD) else {
            return Ok(None);
        };
        let mut state = registry.state(default).unwrap().clone();
        state.properties.insert("spawned".into(), "true".into());
        let Some(spawned) = registry.id(&state) else {
            return Ok(None);
        };
        let [x, y, z] = [target.x, target.y, target.z].map(|coord| coord.floor() as i32);
        let shape = self.world.shape;
        let bottom = shape.min_y;
        let top = shape.max_y() - 3;
        let block = |y| self.world.block(x, y, z);
        let air = |y| self.res.facts().is_air(block(y));
        let headroom = |y| air(y + 1) && air(y + 2);
        let void = self.dim.id() == "allthemodium:the_beyond";
        let ceiling = if self.dim == Dim::Nether {
            top.min(bottom + shape.logical_height - 3)
        } else {
            top
        };
        let existing = (bottom..=ceiling)
            .filter(|&y| {
                matches!(block(y), state if state == default || state == spawned) && headroom(y)
            })
            .min_by_key(|at| (at - y).abs());
        let found = if existing.is_some() {
            existing
        } else if void {
            let at = y.clamp(bottom, top);
            (air(at) && headroom(at)).then_some(at)
        } else {
            (bottom + 1..=ceiling).rev().find(|&at| {
                let support = self.res.facts().get(block(at - 1));
                support.fluid == Fluid::None && support.full_collision && air(at) && headroom(at)
            })
        };
        let Some(at) = found else {
            return Ok(None);
        };
        if existing.is_none() {
            self.level_set([x, at, z], spawned)?;
        }
        Ok(Some(Vec3::new(
            f64::from(x) + 0.5,
            f64::from(at) + 0.25,
            f64::from(z) + 0.5,
        )))
    }
}
