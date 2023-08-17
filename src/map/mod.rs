use rltk::RandomNumberGenerator;
use specs::World;
use std::cmp::{max, min};

use crate::{components::Position, rect::Rect};

use self::{
    bsp_dungeon::BspDungeonBuilder,
    bsp_interior::BspInteriorBuilder,
    map::{Map, TileType},
};

pub mod bsp_dungeon;
pub mod bsp_interior;
pub mod map;

pub trait MapBuilder {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator);
    fn spawn_entities(&mut self, ecs: &mut World);
    fn get_map(&mut self) -> Map;
    fn get_starting_position(&mut self) -> Position;
}

pub fn random_builder(new_depth: u32, rng: &mut RandomNumberGenerator) -> Box<dyn MapBuilder> {
    let builder_idx = rng.roll_dice(1, 2) - 1;
    match builder_idx {
        1 => Box::new(BspDungeonBuilder::new(new_depth)),
        _ => Box::new(BspInteriorBuilder::new(new_depth)),
    }
}

pub fn apply_room_to_map(map: &mut Map, room: &Rect) {
    for y in room.y1 + 1..=room.y2 {
        for x in room.x1 + 1..=room.x2 {
            let idx = map.index_from_xy(x, y);
            map.tiles[idx] = TileType::Floor;
        }
    }
}

pub fn apply_horizontal_tunnel(map: &mut Map, x1: u16, x2: u16, y: u16) {
    let min_x = min(x1, x2);
    let max_x = max(x1, x2);
    for x in min_x..=max_x {
        let idx = map.index_from_xy(x, y);
        if idx > 0 && idx < map.tiles.len() {
            map.tiles[idx] = TileType::Floor;
        }
    }
}

pub fn apply_vertical_tunnel(map: &mut Map, y1: u16, y2: u16, x: u16) {
    let min_y = min(y1, y2);
    let max_y = max(y1, y2);
    for y in min_y..=max_y {
        let idx = map.index_from_xy(x, y);
        if idx > 0 && idx < map.tiles.len() {
            map.tiles[idx] = TileType::Floor;
        }
    }
}
