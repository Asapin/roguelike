use rltk::{field_of_view, Point};
use specs::prelude::*;
use specs::{System, WriteStorage};

use crate::components::{Player, Position, Viewshed};
use crate::map::Map;

#[derive(Clone, Copy)]
pub struct VisibilitySystem;

impl<'a> System<'a> for VisibilitySystem {
    type SystemData = (
        WriteExpect<'a, Map>,
        Entities<'a>,
        WriteStorage<'a, Viewshed>,
        WriteStorage<'a, Position>,
        ReadStorage<'a, Player>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut map, entities, mut viewshed, pos, player) = data;
        for (ent, viewshed, pos) in (&entities, &mut viewshed, &pos).join() {
            if !viewshed.dirty {
                continue;
            }
            viewshed.dirty = false;
            viewshed.visible_tiles.clear();
            viewshed.visible_tiles =
                field_of_view(Point::new(pos.x, pos.y), viewshed.range as i32, &*map);
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
                }
            }
        }
    }
}
