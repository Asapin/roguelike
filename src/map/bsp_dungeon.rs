use rltk::RandomNumberGenerator;

use crate::{components::Position, rect::Rect, spawn::spawner};

use super::{
    apply_horizontal_tunnel, apply_room_to_map, apply_vertical_tunnel,
    map::{Map, TileType},
    MapBuilder,
};

const MAX_ENTITIES: i32 = 4;
const MAX_ATTEMPTS: i32 = 15;
const MIN_LEAF_WIDTH: u16 = 6;
const MIN_LEAF_HEIGHT: u16 = 6;
const LEAF_BORDER: u16 = 1;
const MAX_ROOM_WIDTH: u16 = 10;
const MAX_ROOM_HEIGHT: u16 = 10;
// Min room size is (MIN_LEAF_WIDTH - LEAF_BORDER*2) x (MIN_LEAF_HEIGHT - LEAF_BORDER*2)

enum AvailableSplits {
    Both,
    Horizontal,
    Vertical,
    None,
}

#[derive(PartialEq)]
enum SelectedSplit {
    Horizontal,
    Vertical,
}

pub struct BspDungeonBuilder {
    map: Map,
    rooms: Vec<Rect>,
    leafs: Vec<Rect>,
}

impl MapBuilder for BspDungeonBuilder {
    fn get_map(&mut self) -> Map {
        self.map.clone()
    }

    fn get_starting_position(&mut self) -> Position {
        let (x, y) = self.rooms[0].center();
        Position { x, y }
    }

    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator) {
        // Start with a single map-sized rectangle
        self.leafs
            .push(Rect::new(2, 2, self.map.width - 5, self.map.height - 5));
        let mut index = 0;
        for _ in 0..MAX_ATTEMPTS {
            let leaf = &self.leafs[index];
            match BspDungeonBuilder::split(&leaf, MIN_LEAF_WIDTH, MIN_LEAF_HEIGHT, rng) {
                Some((leaf1, leaf2)) => {
                    self.leafs[index] = leaf2;
                    self.leafs.insert(index, leaf1);
                }
                None => {}
            };
            index = self.select_leaf_to_split();
        }

        for leaf in self.leafs.iter() {
            let room = BspDungeonBuilder::generate_room(leaf, MIN_LEAF_WIDTH, MIN_LEAF_HEIGHT, rng);
            apply_room_to_map(&mut self.map, &room);
            self.rooms.push(room);
        }

        self.generate_tunnels(rng);

        let (stairs_x, stairs_y) = self.rooms[self.rooms.len() - 1].center();
        let stairs_idx = self.map.index_from_xy(stairs_x, stairs_y);
        self.map.tiles[stairs_idx] = TileType::DownStairs;

        for tile in self.map.revealed_tiles.iter_mut() {
            *tile = true;
        }
    }

    fn spawn_entities(&mut self, ecs: &mut specs::World) {
        for room in self.rooms.iter().skip(1) {
            spawner::spawn_room(ecs, &self.map, room, MAX_ENTITIES);
        }
    }
}

impl BspDungeonBuilder {
    pub fn new(new_depth: u32) -> Self {
        Self {
            map: Map::empty_map(new_depth),
            rooms: Vec::new(),
            leafs: Vec::new(),
        }
    }

    fn available_splits(leaf: &Rect, min_width: u16, min_height: u16) -> AvailableSplits {
        let horizontal_split = leaf.height() >= min_height * 2;
        let vertical_split = leaf.width() >= min_width * 2;
        match (horizontal_split, vertical_split) {
            (false, false) => AvailableSplits::None,
            (false, true) => AvailableSplits::Vertical,
            (true, false) => AvailableSplits::Horizontal,
            (true, true) => AvailableSplits::Both,
        }
    }

    fn split(
        leaf: &Rect,
        min_width: u16,
        min_height: u16,
        rng: &mut RandomNumberGenerator,
    ) -> Option<(Rect, Rect)> {
        let available = BspDungeonBuilder::available_splits(leaf, min_width, min_height);
        let selected = match available {
            AvailableSplits::None => return None,
            AvailableSplits::Vertical => SelectedSplit::Vertical,
            AvailableSplits::Horizontal => SelectedSplit::Horizontal,
            AvailableSplits::Both => {
                if rng.rand() {
                    SelectedSplit::Vertical
                } else {
                    SelectedSplit::Horizontal
                }
            }
        };

        if selected == SelectedSplit::Horizontal {
            let random_height_limit = leaf.height() - min_height * 2;
            let leaf1_height = if random_height_limit == 0 {
                min_height
            } else {
                rng.roll_dice(1, random_height_limit as i32) as u16 + min_height
            };
            let leaf1 = Rect::new(leaf.x1, leaf.y1, leaf.width(), leaf1_height);
            let leaf2 = Rect::new(
                leaf.x1,
                leaf1.y2,
                leaf.width(),
                leaf.height() - leaf1_height,
            );
            Some((leaf1, leaf2))
        } else {
            let random_width_limit = leaf.width() - min_width * 2;
            let leaf1_width = if random_width_limit == 0 {
                min_width
            } else {
                rng.roll_dice(1, random_width_limit as i32) as u16 + min_width
            };
            let leaf1 = Rect::new(leaf.x1, leaf.y1, leaf1_width, leaf.height());
            let leaf2 = Rect::new(leaf1.x2, leaf.y1, leaf.width() - leaf1_width, leaf.height());
            Some((leaf1, leaf2))
        }
    }

    fn generate_room(
        leaf: &Rect,
        min_width: u16,
        min_height: u16,
        rng: &mut RandomNumberGenerator,
    ) -> Rect {
        let leaf_width = leaf.width();
        let leaf_height = leaf.height();

        // Room's max width and size are limited to 10 tiles each
        let room_width = if leaf_width == min_width {
            min_width - LEAF_BORDER * 2
        } else {
            let width = rng.roll_dice(1, (leaf_width - min_width) as i32) as u16 + min_width
                - LEAF_BORDER * 2;
            u16::min(width, MAX_ROOM_WIDTH)
        };
        let room_height = if leaf_height == min_height {
            min_height - LEAF_BORDER * 2
        } else {
            let height = rng.roll_dice(1, (leaf_height - min_height) as i32) as u16 + min_height
                - LEAF_BORDER * 2;
            u16::min(height, MAX_ROOM_HEIGHT)
        };

        let width_margin = leaf_width - room_width - LEAF_BORDER * 2;
        let height_margin = leaf_height - room_height - LEAF_BORDER * 2;

        let x_offset = if width_margin == 0 {
            0
        } else {
            rng.roll_dice(1, width_margin as i32) as u16
        };

        let y_offset = if height_margin == 0 {
            0
        } else {
            rng.roll_dice(1, height_margin as i32) as u16
        };

        Rect::new(
            leaf.x1 + x_offset + LEAF_BORDER,
            leaf.y1 + y_offset + LEAF_BORDER,
            room_width,
            room_height,
        )
    }

    // Split the biggest leaf
    fn select_leaf_to_split(&self) -> usize {
        let mut max_area = 0;
        let mut idx = 0;
        for (i, leaf) in self.leafs.iter().enumerate() {
            let leaf_area = leaf.area();
            if leaf_area > max_area {
                max_area = leaf_area;
                idx = i;
            }
        }
        idx
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
