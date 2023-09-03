use std::collections::HashMap;

use rltk::{Point, RandomNumberGenerator};

use crate::{components::Position, spawn::spawner::spawn_region};

use super::{
    generate_voronoi_spawn_regions,
    map::{Map, TileType},
    remove_unreachable_areas, MapBuilder,
};

const MAX_ENTITIES: u16 = 4;

pub enum DLAAlgorithm {
    WalkInwards,
    WalkOutwards,
    CentralAttractor,
}

pub enum DLASymmetry {
    None,
    Horizontal,
    Vertical,
    Both,
}

pub struct DLABuilder {
    map: Map,
    starting_position: Position,
    noise_areas: HashMap<i32, Vec<usize>>,
    algorithm: DLAAlgorithm,
    brush_size: u16,
    symmetry: DLASymmetry,
    floor_percent: f32,
}

impl MapBuilder for DLABuilder {
    fn get_map(&mut self) -> Map {
        self.map.clone()
    }

    fn get_starting_position(&mut self) -> Position {
        self.starting_position.clone()
    }

    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator) {
        self.starting_position = Position {
            x: self.map.width / 2,
            y: self.map.height / 2,
        };

        // Carve a starting seed
        let start_idx = self
            .map
            .index_from_xy(self.starting_position.x, self.starting_position.y);
        self.map.tiles[start_idx] = TileType::Floor;
        self.map.tiles[start_idx - 1] = TileType::Floor;
        self.map.tiles[start_idx + 1] = TileType::Floor;
        self.map.tiles[start_idx - self.map.width as usize] = TileType::Floor;
        self.map.tiles[start_idx + self.map.width as usize] = TileType::Floor;

        // Random walker
        let total_tiles = self.map.width as usize * self.map.height as usize;
        let desired_floor_tiles = (self.floor_percent * total_tiles as f32) as usize;
        let mut floor_tile_count = self
            .map
            .tiles
            .iter()
            .filter(|tile| **tile == TileType::Floor)
            .count();

        while floor_tile_count < desired_floor_tiles {
            match self.algorithm {
                DLAAlgorithm::WalkInwards => self.walk_inwards(rng),
                DLAAlgorithm::WalkOutwards => self.walk_outwards(rng),
                DLAAlgorithm::CentralAttractor => self.central_attractor(rng),
            }
            floor_tile_count = self
                .map
                .tiles
                .iter()
                .filter(|tile| **tile == TileType::Floor)
                .count();
        }

        let exit_idx = remove_unreachable_areas(&mut self.map, start_idx);
        self.map.tiles[exit_idx] = TileType::DownStairs;
        self.noise_areas = generate_voronoi_spawn_regions(&self.map, rng);
    }

    fn spawn_entities(&mut self, ecs: &mut specs::World) {
        for (_, area) in self.noise_areas.iter() {
            spawn_region(ecs, &area, &self.map, MAX_ENTITIES);
        }
    }
}

impl DLABuilder {
    pub fn new_walk_inwards(new_depth: u32) -> Self {
        Self {
            map: Map::empty_map(new_depth),
            starting_position: Position { x: 0, y: 0 },
            noise_areas: HashMap::new(),
            algorithm: DLAAlgorithm::WalkInwards,
            brush_size: 1,
            symmetry: DLASymmetry::None,
            floor_percent: 0.25,
        }
    }

    pub fn new_walk_outwards(new_depth: u32) -> Self {
        Self {
            map: Map::empty_map(new_depth),
            starting_position: Position { x: 0, y: 0 },
            noise_areas: HashMap::new(),
            algorithm: DLAAlgorithm::WalkOutwards,
            brush_size: 2,
            symmetry: DLASymmetry::None,
            floor_percent: 0.25,
        }
    }

    pub fn new_central_attractor(new_depth: u32) -> Self {
        Self {
            map: Map::empty_map(new_depth),
            starting_position: Position { x: 0, y: 0 },
            noise_areas: HashMap::new(),
            algorithm: DLAAlgorithm::CentralAttractor,
            brush_size: 2,
            symmetry: DLASymmetry::None,
            floor_percent: 0.25,
        }
    }

    pub fn insectoid(new_depth: u32) -> Self {
        Self {
            map: Map::empty_map(new_depth),
            starting_position: Position { x: 0, y: 0 },
            noise_areas: HashMap::new(),
            algorithm: DLAAlgorithm::CentralAttractor,
            brush_size: 2,
            symmetry: DLASymmetry::Horizontal,
            floor_percent: 0.25,
        }
    }

    fn paint(&mut self, x: u16, y: u16) {
        match self.symmetry {
            DLASymmetry::None => self.apply_paint(x, y),
            DLASymmetry::Horizontal => {
                let center_x = self.map.width / 2;
                if x == center_x {
                    self.apply_paint(x, y);
                } else {
                    let dist_x = i32::abs(center_x as i32 - x as i32) as u16;
                    self.apply_paint(center_x + dist_x, y);
                    self.apply_paint(center_x - dist_x, y);
                }
            }
            DLASymmetry::Vertical => {
                let center_y = self.map.height / 2;
                if y == center_y {
                    self.apply_paint(x, y);
                } else {
                    let dist_y = i32::abs(center_y as i32 - y as i32) as u16;
                    self.apply_paint(x, center_y + dist_y);
                    self.apply_paint(x, center_y - dist_y);
                }
            }
            DLASymmetry::Both => {
                let center_x = self.map.width / 2;
                let center_y = self.map.height / 2;
                if x == center_x && y == center_y {
                    self.apply_paint(x, y);
                } else {
                    let dist_x = i32::abs(center_x as i32 - x as i32) as u16;
                    let dist_y = i32::abs(center_y as i32 - y as i32) as u16;
                    self.apply_paint(center_x + dist_x, y);
                    self.apply_paint(center_x - dist_x, y);
                    self.apply_paint(x, center_y + dist_y);
                    self.apply_paint(x, center_y - dist_y);
                }
            }
        }
    }

    fn apply_paint(&mut self, x: u16, y: u16) {
        match self.brush_size {
            1 => {
                let digger_idx = self.map.index_from_xy(x, y);
                self.map.tiles[digger_idx] = TileType::Floor;
            }
            _ => {
                let half_brush_size = self.brush_size / 2;
                for brush_y in y - half_brush_size..y + half_brush_size {
                    for brush_x in x - half_brush_size..x + half_brush_size {
                        if brush_x > 1
                            && brush_x < self.map.width - 1
                            && brush_y > 1
                            && brush_y < self.map.height - 1
                        {
                            let idx = self.map.index_from_xy(brush_x, brush_y);
                            self.map.tiles[idx] = TileType::Floor;
                        }
                    }
                }
            }
        }
    }

    fn walk_inwards(&mut self, rng: &mut rltk::RandomNumberGenerator) {
        let mut digger_x = (rng.roll_dice(1, self.map.width as i32 - 3) + 1) as u16;
        let mut digger_y = (rng.roll_dice(1, self.map.height as i32 - 3) + 1) as u16;
        let mut prev_x = digger_x;
        let mut prev_y = digger_y;
        let mut digger_idx = self.map.index_from_xy(digger_x, digger_y);
        while self.map.tiles[digger_idx] == TileType::Wall {
            prev_x = digger_x;
            prev_y = digger_y;
            (digger_x, digger_y) = self.random_stagger(rng, digger_x, digger_y);
            digger_idx = self.map.index_from_xy(digger_x, digger_y);
        }
        self.paint(prev_x, prev_y);
    }

    fn walk_outwards(&mut self, rng: &mut RandomNumberGenerator) {
        let mut digger_x = self.starting_position.x;
        let mut digger_y = self.starting_position.y;
        let mut digger_idx = self.map.index_from_xy(digger_x, digger_y);
        while self.map.tiles[digger_idx] == TileType::Floor {
            (digger_x, digger_y) = self.random_stagger(rng, digger_x, digger_y);
            digger_idx = self.map.index_from_xy(digger_x, digger_y);
        }
        self.paint(digger_x, digger_y);
    }

    fn central_attractor(&mut self, rng: &mut RandomNumberGenerator) {
        let mut digger_x = (rng.roll_dice(1, self.map.width as i32 - 3) + 1) as u16;
        let mut digger_y = (rng.roll_dice(1, self.map.height as i32 - 3) + 1) as u16;
        let mut prev_x = digger_x;
        let mut prev_y = digger_y;
        let mut digger_idx = self.map.index_from_xy(digger_x, digger_y);

        let mut path = rltk::line2d(
            rltk::LineAlg::Bresenham,
            Point::new(digger_x, digger_y),
            Point::new(self.starting_position.x, self.starting_position.y),
        );

        while self.map.tiles[digger_idx] == TileType::Wall && !path.is_empty() {
            prev_x = digger_x;
            prev_y = digger_y;
            digger_x = path[0].x as u16;
            digger_y = path[0].y as u16;
            path.remove(0);
            digger_idx = self.map.index_from_xy(digger_x, digger_y);
        }
        self.paint(prev_x, prev_y);
    }

    fn random_stagger(
        &self,
        rng: &mut RandomNumberGenerator,
        mut x: u16,
        mut y: u16,
    ) -> (u16, u16) {
        let stagger_direction = rng.roll_dice(1, 4);
        match stagger_direction {
            1 => {
                if x > 2 {
                    x -= 1;
                }
            }
            2 => {
                if x < self.map.width - 2 {
                    x += 1;
                }
            }
            3 => {
                if y > 2 {
                    y -= 1;
                }
            }
            _ => {
                if y < self.map.height - 2 {
                    y += 1;
                }
            }
        }
        (x, y)
    }
}
