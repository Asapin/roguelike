use std::collections::HashSet;

use rltk::{Algorithm2D, BaseMap, FontCharType, Point};
use serde::{Deserialize, Serialize};
use specs::Entity;

use crate::rect::Rect;

pub const WINDOW_WIDTH: u16 = 80;
pub const WINDOW_HEIGHT: u16 = 50;
pub const MAP_WIDTH: u16 = 80;
pub const MAP_HEIGHT: u16 = 43;

#[derive(PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum TileType {
    Wall,
    Floor,
    DownStairs,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Map {
    pub window_width: u16,
    pub window_height: u16,
    pub width: u16,
    pub height: u16,
    pub tiles: Vec<TileType>,
    pub rooms: Vec<Rect>,
    pub revealed_tiles: Vec<bool>,
    pub visible_tiles: Vec<bool>,
    pub blocked: Vec<bool>,
    pub depth: u32,
    pub bloodstains: HashSet<usize>,

    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    pub tile_content: Vec<Vec<Entity>>,
}

impl Map {
    pub fn empty_map(new_depth: u32) -> Self {
        let width = MAP_WIDTH;
        let height = MAP_HEIGHT;
        let map_dimensions = width as usize * height as usize;
        Self {
            window_width: WINDOW_WIDTH,
            window_height: WINDOW_HEIGHT,
            width,
            height,
            tiles: vec![TileType::Wall; map_dimensions],
            rooms: Vec::new(),
            revealed_tiles: vec![false; map_dimensions],
            visible_tiles: vec![false; map_dimensions],
            blocked: vec![true; map_dimensions],
            depth: new_depth,
            bloodstains: HashSet::new(),
            tile_content: vec![Vec::new(); map_dimensions],
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

    pub fn wall_glyph(&self, x: u16, y: u16) -> FontCharType {
        let mut mask: u8 = 0;
        if y > 0 && self.is_revealed_and_wall(x, y - 1) {
            mask |= 0b0000_0001;
        }
        if y < self.height - 1 && self.is_revealed_and_wall(x, y + 1) {
            mask |= 0b0000_0010;
        }
        if x > 0 && self.is_revealed_and_wall(x - 1, y) {
            mask |= 0b0000_0100;
        }
        if x < self.width - 1 && self.is_revealed_and_wall(x + 1, y) {
            mask |= 0b0000_1000;
        }
        match mask {
            0b0000_0000 => 9, // Pillar because we can't see neighbors
            0b0000_0001 | 0b0000_0010 | 0b0000_0011 => 186, // Walls to the north and/or south
            0b0000_0100 => 205, // Wall only to the west
            0b0000_0101 => 188, // Wall to the noth and west
            0b0000_0110 => 187, // Wall to the south and west
            0b0000_0111 => 185, // Wall to the north, south and west
            0b0000_1000 => 205, // Wall only to the east
            0b0000_1001 => 200, // Wall to the north and east
            0b0000_1010 => 201, // Wall to the south and east
            0b0000_1011 => 204, // Wall to the north, south and east
            0b0000_1100 => 205, // Wall to the east and west
            0b0000_1101 => 202, // Wall to the east, west and south
            0b0000_1110 => 203, // Wall to the east, west and north
            0b0000_1111 => 206, // Wall on all sides
            _ => 35,          // Should never happen
        }
    }

    fn is_revealed_and_wall(&self, x: u16, y: u16) -> bool {
        let idx = self.index_from_xy(x, y);
        self.tiles[idx] == TileType::Wall && self.revealed_tiles[idx]
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
