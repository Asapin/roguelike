use std::collections::HashMap;

use rltk::{FastNoise, RandomNumberGenerator};

use crate::{components::Position, spawn::spawner::spawn_region};

use super::{
    map::{Map, TileType},
    MapBuilder,
};

const GENERATION_ITERATIONS: u8 = 15;
const MAX_ENTITIES: u16 = 6;

pub struct CellularAutomataBuilder {
    map: Map,
    starting_position: Position,
    noise_areas: HashMap<i32, Vec<usize>>,
}

impl MapBuilder for CellularAutomataBuilder {
    fn get_map(&mut self) -> Map {
        self.map.clone()
    }

    fn get_starting_position(&mut self) -> Position {
        self.starting_position.clone()
    }

    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator) {
        // Try until generated succesfully
        while !self.try_generating(rng) {
            continue;
        }

        self.polish_and_generate_exit();
        self.generate_noise_areas(rng);
    }

    fn spawn_entities(&mut self, ecs: &mut specs::World) {
        for (_, area) in self.noise_areas.iter() {
            spawn_region(ecs, &area, &self.map, MAX_ENTITIES)
        }
    }
}

impl CellularAutomataBuilder {
    pub fn new(new_depth: u32) -> Self {
        Self {
            map: Map::empty_map(new_depth),
            starting_position: Position { x: 1, y: 1 },
            noise_areas: HashMap::new(),
        }
    }

    fn success(probability: u8, rng: &mut RandomNumberGenerator) -> bool {
        let roll = rng.roll_dice(1, 100);
        roll <= probability as i32
    }

    fn try_generating(&mut self, rng: &mut RandomNumberGenerator) -> bool {
        self.generate_random_pattern(rng);
        for _ in 0..GENERATION_ITERATIONS {
            self.run_automata();
        }

        // Start in the middle and probe tiles to left and right
        // until we find an open space
        let start_idx = self
            .map
            .index_from_xy(self.map.width / 2, self.map.height / 2);
        let mut delta = 0;
        let idx = loop {
            if self.map.tiles[start_idx - delta] == TileType::Floor {
                break start_idx - delta;
            }
            if self.map.tiles[start_idx + delta] == TileType::Floor {
                break start_idx + delta;
            }
            delta += 1;
            if delta == self.map.tiles.len() / 2 {
                // Generation failed
                return false;
            }
        };

        let (x, y) = self.map.xy_from_index(&idx);
        self.starting_position = Position { x, y };

        true
    }

    fn generate_random_pattern(&mut self, rng: &mut RandomNumberGenerator) {
        for y in 1..self.map.height - 1 {
            for x in 1..self.map.width - 1 {
                let idx = self.map.index_from_xy(x, y);
                if CellularAutomataBuilder::success(55, rng) {
                    self.map.tiles[idx] = TileType::Floor;
                } else {
                    self.map.tiles[idx] = TileType::Wall;
                }
            }
        }
    }

    fn run_automata(&mut self) {
        let mut new_generation = self.map.tiles.clone();

        for y in 1..self.map.height - 1 {
            for x in 1..self.map.width - 1 {
                let mut neighbors = 0;
                let upper_row = self.map.index_from_xy(x, y - 1);
                let middle_row = self.map.index_from_xy(x, y);
                let lower_row = self.map.index_from_xy(x, y + 1);

                if self.map.tiles[upper_row - 1] == TileType::Wall {
                    neighbors += 1;
                }
                if self.map.tiles[upper_row] == TileType::Wall {
                    neighbors += 1;
                }
                if self.map.tiles[upper_row + 1] == TileType::Wall {
                    neighbors += 1;
                }

                if self.map.tiles[middle_row - 1] == TileType::Wall {
                    neighbors += 1;
                }
                if self.map.tiles[middle_row + 1] == TileType::Wall {
                    neighbors += 1;
                }

                if self.map.tiles[lower_row - 1] == TileType::Wall {
                    neighbors += 1;
                }
                if self.map.tiles[lower_row] == TileType::Wall {
                    neighbors += 1;
                }
                if self.map.tiles[lower_row + 1] == TileType::Wall {
                    neighbors += 1;
                }

                if neighbors > 4 || neighbors == 0 {
                    new_generation[middle_row] = TileType::Wall;
                } else {
                    new_generation[middle_row] = TileType::Floor;
                }
            }
        }

        self.map.tiles = new_generation;
    }

    fn polish_and_generate_exit(&mut self) {
        let starting_pos = self.starting_position;
        let start_idx = self.map.index_from_xy(starting_pos.x, starting_pos.y);
        let poi: Vec<usize> = vec![start_idx];
        let dijkstra_map =
            rltk::DijkstraMap::new(self.map.width, self.map.height, &poi, &self.map, 200.0);
        let (mut exit_idx, mut exit_distance) = (start_idx, 0.0f32);
        for (i, tile) in self.map.tiles.iter_mut().enumerate() {
            if *tile != TileType::Floor {
                continue;
            }
            let distance_to_start = dijkstra_map.map[i];
            if distance_to_start == f32::MAX {
                // We can't pathfind to this tile - so make it a wall
                *tile = TileType::Wall;
            } else {
                if distance_to_start > exit_distance {
                    exit_distance = distance_to_start;
                    exit_idx = i;
                }
            }
        }

        self.map.tiles[exit_idx] = TileType::DownStairs;
    }

    fn generate_noise_areas(&mut self, rng: &mut RandomNumberGenerator) {
        let mut noise = FastNoise::seeded(rng.roll_dice(1, u16::MAX as i32) as u64);
        noise.set_noise_type(rltk::NoiseType::Cellular);
        noise.set_frequency(0.08);
        noise.set_cellular_distance_function(rltk::CellularDistanceFunction::Manhattan);

        for y in 1..self.map.height - 1 {
            for x in 1..self.map.width - 1 {
                let idx = self.map.index_from_xy(x, y);
                if self.map.tiles[idx] != TileType::Floor {
                    continue;
                }
                let cell_value_f = noise.get_noise(x as f32, y as f32) * 10240.0;
                let cell_value = cell_value_f as i32;

                match self.noise_areas.get_mut(&cell_value) {
                    Some(entry) => {
                        entry.push(idx);
                    }
                    None => {
                        self.noise_areas.insert(cell_value, vec![idx]);
                    }
                };
            }
        }
    }
}
