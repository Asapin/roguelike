use std::cmp::{max, min};

use rltk::{Algorithm2D, BaseMap, Point, RandomNumberGenerator};
use serde::{Deserialize, Serialize};
use specs::Entity;

use crate::rect::Rect;

#[derive(PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum TileType {
    Wall,
    Floor,
    DownStairs,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Map {
    pub width: u16,
    pub height: u16,
    pub window_width: u16,
    pub window_height: u16,
    pub tiles: Vec<TileType>,
    pub rooms: Vec<Rect>,
    pub revealed_tiles: Vec<bool>,
    pub visible_tiles: Vec<bool>,
    pub blocked: Vec<bool>,
    pub depth: u32,

    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    pub tile_content: Vec<Vec<Entity>>,
}

impl Map {
    pub fn new_map(
        &self,
        room_count: u8,
        min_size: u8,
        max_size: u8,
        rng: &mut RandomNumberGenerator,
    ) -> Self {
        Map::new_map_rooms_and_corridors(
            self.width,
            self.height,
            self.window_width,
            self.window_height,
            room_count,
            min_size,
            max_size,
            rng,
            self.depth + 1,
        )
    }

    pub fn new_map_rooms_and_corridors(
        width: u16,
        height: u16,
        window_width: u16,
        window_height: u16,
        room_count: u8,
        min_size: u8,
        max_size: u8,
        rng: &mut RandomNumberGenerator,
        new_depth: u32,
    ) -> Self {
        let map_dimensions = width as usize * height as usize;
        let tiles = vec![TileType::Wall; map_dimensions];
        let mut map = Self {
            width,
            height,
            window_width,
            window_height,
            tiles,
            rooms: Vec::new(),
            revealed_tiles: vec![false; map_dimensions],
            visible_tiles: vec![false; map_dimensions],
            blocked: vec![false; map_dimensions],
            tile_content: vec![Vec::new(); map_dimensions],
            depth: new_depth,
        };

        for _ in 0..room_count {
            let w = rng.range(min_size, max_size);
            let h = rng.range(min_size, max_size);
            let x = rng.roll_dice(1, (width - w as u16 - 1) as i32) - 1;
            let y = rng.roll_dice(1, (height - h as u16 - 1) as i32) - 1;
            let new_room = Rect::new(x as u16, y as u16, w, h);
            let intersects = map
                .rooms
                .iter()
                .any(|room: &Rect| room.intersect(&new_room));
            if intersects {
                continue;
            }

            map.apply_room(&new_room);
            if !map.rooms.is_empty() {
                let (new_x, new_y) = new_room.center();
                let (prev_x, prev_y) = map.rooms[map.rooms.len() - 1].center();
                if rng.rand::<bool>() {
                    map.apply_horizontal_tunnel(prev_x, new_x, prev_y);
                    map.apply_vertical_tunnel(prev_y, new_y, new_x);
                } else {
                    map.apply_vertical_tunnel(prev_y, new_y, new_x);
                    map.apply_horizontal_tunnel(prev_x, new_x, prev_y);
                }
            }

            map.rooms.push(new_room);
        }

        let stairs_position = map.rooms[map.rooms.len() - 1].center();
        let stairs_idx = map.index_from_xy(stairs_position.0, stairs_position.1);
        map.tiles[stairs_idx] = TileType::DownStairs;

        map
    }

    fn apply_room(&mut self, room: &Rect) {
        for y in room.y1 + 1..=room.y2 {
            for x in room.x1 + 1..=room.x2 {
                let idx = self.index_from_xy(x, y);
                self.tiles[idx] = TileType::Floor;
            }
        }
    }

    fn apply_horizontal_tunnel(&mut self, x1: u16, x2: u16, y: u16) {
        for x in min(x1, x2)..=max(x1, x2) {
            let idx = self.index_from_xy(x, y);
            if idx > 0 && idx < self.tiles.len() {
                self.tiles[idx] = TileType::Floor;
            }
        }
    }

    fn apply_vertical_tunnel(&mut self, y1: u16, y2: u16, x: u16) {
        for y in min(y1, y2)..=max(y1, y2) {
            let idx = self.index_from_xy(x, y);
            if idx > 0 && idx < self.tiles.len() {
                self.tiles[idx] = TileType::Floor;
            }
        }
    }

    pub fn index_from_xy(&self, x: u16, y: u16) -> usize {
        ((y * self.width) + x) as usize
    }

    pub fn xy_from_index(&self, idx: &usize) -> (u16, u16) {
        let x = idx % self.width as usize;
        let y = idx / self.width as usize;
        (x as u16, y as u16)
    }

    fn is_exit_valid(&self, x: u16, y: u16) -> bool {
        if x < 1 || x > self.width - 1 || y < 1 || y > self.height - 1 {
            return false;
        }
        let idx = self.index_from_xy(x, y);
        !self.blocked[idx]
    }

    pub fn populate_blocked(&mut self) {
        for (i, tile) in self.tiles.iter_mut().enumerate() {
            self.blocked[i] = tile == &TileType::Wall;
        }
    }

    pub fn clear_content_index(&mut self) {
        for content in self.tile_content.iter_mut() {
            content.clear();
        }
    }
}

impl Algorithm2D for Map {
    fn dimensions(&self) -> Point {
        Point::new(self.width, self.height)
    }
}

impl BaseMap for Map {
    fn is_opaque(&self, idx: usize) -> bool {
        self.tiles[idx] == TileType::Wall
    }

    fn get_available_exits(&self, idx: usize) -> rltk::SmallVec<[(usize, f32); 10]> {
        let mut exits = rltk::SmallVec::new();
        let x = (idx % self.width as usize) as u16;
        let y = (idx / self.width as usize) as u16;
        let w = self.width as usize;

        // Cardinal directions
        if self.is_exit_valid(x - 1, y) {
            exits.push((idx - 1, 1.0))
        }
        if self.is_exit_valid(x + 1, y) {
            exits.push((idx + 1, 1.0));
        }
        if self.is_exit_valid(x, y - 1) {
            exits.push((idx - w, 1.0));
        }
        if self.is_exit_valid(x, y + 1) {
            exits.push((idx + w, 1.0));
        }

        // Diagonal directions
        if self.is_exit_valid(x - 1, y - 1) {
            exits.push(((idx - w) - 1, 1.45));
        }
        if self.is_exit_valid(x + 1, y - 1) {
            exits.push(((idx - w) + 1, 1.45));
        }
        if self.is_exit_valid(x - 1, y + 1) {
            exits.push(((idx + w) - 1, 1.45));
        }
        if self.is_exit_valid(x + 1, y + 1) {
            exits.push(((idx + w) + 1, 1.45));
        }

        exits
    }

    fn get_pathing_distance(&self, idx1: usize, idx2: usize) -> f32 {
        let w = self.width as usize;
        let p1 = Point::new(idx1 % w, idx1 / w);
        let p2 = Point::new(idx2 % w, idx2 / w);
        rltk::DistanceAlg::Pythagoras.distance2d(p1, p2)
    }
}
