use crate::{components::Position, rect::Rect, spawn::spawner};

use super::{
    map::{Map, TileType},
    map_builder::{apply_horizontal_tunnel, apply_room_to_map, apply_vertical_tunnel, MapBuilder},
};

const ROOM_COUNT: u8 = 30;
const MIN_ROOM_SIZE: u8 = 6;
const MAX_ROOM_SIZE: u8 = 10;
const MAX_ENTITIES: i32 = 4;

pub struct RandomMapBuilder {
    map: Map,
    rooms: Vec<Rect>,
}

impl MapBuilder for RandomMapBuilder {
    fn new(new_depth: u32) -> Self {
        Self {
            map: Map::empty_map(new_depth),
            rooms: Vec::new(),
        }
    }

    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator) {
        let mut map = &mut self.map;
        for _ in 0..ROOM_COUNT {
            let w = rng.range(MIN_ROOM_SIZE, MAX_ROOM_SIZE);
            let h = rng.range(MIN_ROOM_SIZE, MAX_ROOM_SIZE);
            let x = rng.roll_dice(1, (map.width - w as u16 - 1) as i32) - 1;
            let y = rng.roll_dice(1, (map.height - h as u16 - 1) as i32) - 1;
            let new_room = Rect::new(x as u16, y as u16, w, h);
            let intersects = self
                .rooms
                .iter()
                .any(|room: &Rect| room.intersect(&new_room));
            if intersects {
                continue;
            }

            apply_room_to_map(map, &new_room);
            if !self.rooms.is_empty() {
                let (new_x, new_y) = new_room.center();
                let (prev_x, prev_y) = self.rooms[self.rooms.len() - 1].center();
                if rng.rand::<bool>() {
                    apply_horizontal_tunnel(&mut map, prev_x, new_x, prev_y);
                    apply_vertical_tunnel(&mut map, prev_y, new_y, new_x);
                } else {
                    apply_vertical_tunnel(&mut map, prev_y, new_y, new_x);
                    apply_horizontal_tunnel(&mut map, prev_x, new_x, prev_y);
                }
            }

            self.rooms.push(new_room);
        }

        let stairs_position = self.rooms[self.rooms.len() - 1].center();
        let stairs_idx = map.index_from_xy(stairs_position.0, stairs_position.1);
        map.tiles[stairs_idx] = TileType::DownStairs;
    }

    fn spawn_entities(&mut self, ecs: &mut specs::World) {
        // Generate new mobs and items
        for room in self.rooms.iter().skip(1) {
            spawner::spawn_room(ecs, &self.map, room, MAX_ENTITIES);
        }
    }

    fn get_map(&mut self) -> Map {
        self.map.clone()
    }

    fn get_starting_position(&mut self) -> Position {
        let (x, y) = self.rooms[0].center();
        Position { x, y }
    }
}
