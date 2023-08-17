use rltk::RandomNumberGenerator;

use crate::{components::Position, rect::Rect, spawn::spawner};

use super::{
    apply_horizontal_tunnel, apply_room_to_map, apply_vertical_tunnel,
    map::{Map, TileType},
    MapBuilder,
};

const MAX_ENTITIES: i32 = 4;
const MIN_LEAF_WIDTH: u16 = 8;
const MIN_LEAF_HEIGHT: u16 = 8;

pub struct BspInteriorBuilder {
    map: Map,
    rooms: Vec<Rect>,
    leafs: Vec<Rect>,
}

impl MapBuilder for BspInteriorBuilder {
    fn get_map(&mut self) -> Map {
        self.map.clone()
    }

    fn get_starting_position(&mut self) -> Position {
        let (x, y) = self.rooms[0].center();
        Position { x, y }
    }

    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator) {
        let root_leaf = Rect::new(0, 0, self.map.width - 2, self.map.height - 2);
        self.leafs.push(root_leaf);
        self.split_leafs(MIN_LEAF_WIDTH, MIN_LEAF_HEIGHT, rng);

        for leaf in self.leafs.iter() {
            self.rooms.push(leaf.clone());
            apply_room_to_map(&mut self.map, leaf);
        }

        self.generate_tunnels(rng);

        let (stairs_x, stairs_y) = self.rooms[self.rooms.len() - 1].center();
        let stairs_idx = self.map.index_from_xy(stairs_x, stairs_y);
        self.map.tiles[stairs_idx] = TileType::DownStairs;
    }

    fn spawn_entities(&mut self, ecs: &mut specs::World) {
        for room in self.rooms.iter().skip(1) {
            spawner::spawn_room(ecs, &self.map, room, MAX_ENTITIES);
        }
    }
}

impl BspInteriorBuilder {
    pub fn new(new_depth: u32) -> Self {
        Self {
            map: Map::empty_map(new_depth),
            rooms: Vec::new(),
            leafs: Vec::new(),
        }
    }

    fn split_leafs(&mut self, min_width: u16, min_height: u16, rng: &mut RandomNumberGenerator) {
        let leaf = match self.leafs.pop() {
            None => return,
            Some(leaf) => leaf,
        };

        let width = leaf.width();
        let height = leaf.height();
        let half_width = width / 2;
        let half_height = height / 2;

        if rng.rand() {
            // Horizontal split
            let leaf1 = Rect::new(leaf.x1, leaf.y1, half_width - 1, height);
            self.leafs.push(leaf1);
            if half_width > min_width {
                self.split_leafs(min_width, min_height, rng);
            }
            let leaf2 = Rect::new(leaf.x1 + half_width, leaf.y1, half_width, height);
            self.leafs.push(leaf2);
            if half_width > min_width {
                self.split_leafs(min_width, min_height, rng);
            }
        } else {
            // Vertical split
            let leaf1 = Rect::new(leaf.x1, leaf.y1, width, half_height - 1);
            self.leafs.push(leaf1);
            if half_height > min_height {
                self.split_leafs(min_width, min_height, rng);
            }
            let leaf2 = Rect::new(leaf.x1, leaf.y1 + half_height, width, half_height);
            self.leafs.push(leaf2);
            if half_height > min_height {
                self.split_leafs(min_width, min_height, rng);
            }
        }
    }

    fn generate_tunnels(&mut self, rng: &mut RandomNumberGenerator) {
        for i in 1..self.rooms.len() {
            let (x1, y1) = self.rooms[i - 1].center();
            let (x2, y2) = self.rooms[i].center();

            if rng.rand() {
                apply_horizontal_tunnel(&mut self.map, x1, x2, y1);
                apply_vertical_tunnel(&mut self.map, y1, y2, x2);
            } else {
                apply_vertical_tunnel(&mut self.map, y1, y2, x2);
                apply_horizontal_tunnel(&mut self.map, x1, x2, y1);
            }
        }
    }
}
