use crate::rect::Rect;

use super::{
    map::{Map, TileType},
    map_builder::{apply_horizontal_tunnel, apply_room_to_map, apply_vertical_tunnel, MapBuilder},
};

const ROOM_COUNT: u8 = 30;
const MIN_ROOM_SIZE: u8 = 6;
const MAX_ROOM_SIZE: u8 = 10;

pub struct RandomMapBuilder;

impl MapBuilder for RandomMapBuilder {
    fn build(new_depth: u32, rng: &mut rltk::RandomNumberGenerator) -> Map {
        let mut map = Map::empty_map(new_depth);

        for _ in 0..ROOM_COUNT {
            let w = rng.range(MIN_ROOM_SIZE, MAX_ROOM_SIZE);
            let h = rng.range(MIN_ROOM_SIZE, MAX_ROOM_SIZE);
            let x = rng.roll_dice(1, (map.width - w as u16 - 1) as i32) - 1;
            let y = rng.roll_dice(1, (map.height - h as u16 - 1) as i32) - 1;
            let new_room = Rect::new(x as u16, y as u16, w, h);
            let intersects = map
                .rooms
                .iter()
                .any(|room: &Rect| room.intersect(&new_room));
            if intersects {
                continue;
            }

            apply_room_to_map(&mut map, &new_room);
            if !map.rooms.is_empty() {
                let (new_x, new_y) = new_room.center();
                let (prev_x, prev_y) = map.rooms[map.rooms.len() - 1].center();
                if rng.rand::<bool>() {
                    apply_horizontal_tunnel(&mut map, prev_x, new_x, prev_y);
                    apply_vertical_tunnel(&mut map, prev_y, new_y, new_x);
                } else {
                    apply_vertical_tunnel(&mut map, prev_y, new_y, new_x);
                    apply_horizontal_tunnel(&mut map, prev_x, new_x, prev_y);
                }
            }

            map.rooms.push(new_room);
        }

        let stairs_position = map.rooms[map.rooms.len() - 1].center();
        let stairs_idx = map.index_from_xy(stairs_position.0, stairs_position.1);
        map.tiles[stairs_idx] = TileType::DownStairs;

        map
    }
}
