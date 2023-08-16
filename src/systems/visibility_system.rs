use rltk::field_of_view;
use specs::prelude::*;
use specs::{System, WriteStorage};

use crate::components::{Hidden, Name, Player, Position, Viewshed};
use crate::gamelog::GameLog;
use crate::map::map::Map;

#[derive(Clone, Copy)]
pub struct VisibilitySystem;

impl<'a> System<'a> for VisibilitySystem {
    type SystemData = (
        WriteExpect<'a, Map>,
        Entities<'a>,
        WriteStorage<'a, Viewshed>,
        WriteStorage<'a, Position>,
        ReadStorage<'a, Player>,
        WriteStorage<'a, Hidden>,
        WriteExpect<'a, rltk::RandomNumberGenerator>,
        WriteExpect<'a, GameLog>,
        ReadStorage<'a, Name>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut map, entities, mut viewshed, pos, player, mut hidden, mut rng, mut gamelog, names) =
            data;
        for (ent, viewshed, pos) in (&entities, &mut viewshed, &pos).join() {
            if !viewshed.dirty {
                continue;
            }
            viewshed.dirty = false;
            viewshed.visible_tiles.clear();
            viewshed.visible_tiles = field_of_view(pos.into(), viewshed.range as i32, &*map);
            viewshed.visible_tiles.retain(|p| {
                p.x >= 0 && p.x < map.width as i32 && p.y >= 0 && p.y < map.height as i32
            });

            // If this is the player, reveal what the can see
            let p: Option<&Player> = player.get(ent);
            if p.is_some() {
                for t in map.visible_tiles.iter_mut() {
                    *t = false;
                }
                for vis in viewshed.visible_tiles.iter() {
                    let idx = map.index_from_xy(vis.x as u16, vis.y as u16);
                    map.revealed_tiles[idx] = true;
                    map.visible_tiles[idx] = true;

                    // Chance to reveal hidden things
                    for e in map.tile_content[idx].iter() {
                        if let Some(_hidden) = hidden.get(*e) {
                            if rng.roll_dice(1, 24) == 1 {
                                if let Some(name) = names.get(*e) {
                                    gamelog
                                        .entries
                                        .push(format!("You spotted a {}.", name.name));
                                }
                                hidden.remove(*e);
                            }
                        }
                    }
                }
            }
        }
    }
}
