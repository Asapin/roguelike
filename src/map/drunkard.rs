use std::collections::HashMap;

use rltk::console;

use crate::{components::Position, spawn::spawner::spawn_region};

use super::{
    generate_voronoi_spawn_regions,
    map::{Map, TileType},
    paint, remove_unreachable_areas, MapBuilder, Symmetry,
};

const MAX_ENTITIES: u16 = 4;

#[derive(PartialEq)]
pub enum DrunkSpawnMode {
    StartingPoint,
    Random,
}

pub struct DrunkardSettings {
    pub spawn_mode: DrunkSpawnMode,
    pub drunken_lifetime: i32,
    pub floor_percent: f32,
    pub brush_size: u16,
    pub symmetry: Symmetry,
}

pub struct DrunkardsWalkBuilder {
    map: Map,
    starting_position: Position,
    noise_areas: HashMap<i32, Vec<usize>>,
    settings: DrunkardSettings,
}

impl MapBuilder for DrunkardsWalkBuilder {
    fn get_map(&mut self) -> Map {
        self.map.clone()
    }

    fn get_starting_position(&mut self) -> Position {
        self.starting_position.clone()
    }

    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator) {
        // Set a central starting point
        self.starting_position = Position {
            x: self.map.width / 2,
            y: self.map.height / 2,
        };
        let start_idx = self
            .map
            .index_from_xy(self.starting_position.x, self.starting_position.y);
        self.map.tiles[start_idx] = TileType::Floor;

        let total_tiles = self.map.tiles.len();
        let desired_floor_tiles = (self.settings.floor_percent * total_tiles as f32) as usize;
        let mut floor_tile_count = self
            .map
            .tiles
            .iter()
            .filter(|tile| **tile == TileType::Floor)
            .count();
        let mut digger_count = 0;
        let mut active_digger_count = 0;

        while floor_tile_count < desired_floor_tiles {
            let mut did_something = false;
            let mut drunk_x;
            let mut drunk_y;
            match self.settings.spawn_mode {
                DrunkSpawnMode::StartingPoint => {
                    drunk_x = self.starting_position.x;
                    drunk_y = self.starting_position.y;
                }
                DrunkSpawnMode::Random => {
                    if digger_count == 0 {
                        drunk_x = self.starting_position.x;
                        drunk_y = self.starting_position.y;
                    } else {
                        drunk_x = rng.roll_dice(1, self.map.width as i32 - 3) as u16 + 1;
                        drunk_y = rng.roll_dice(1, self.map.height as i32 - 3) as u16 + 1;
                    }
                }
            }
            let mut drunk_life = self.settings.drunken_lifetime;

            while drunk_life > 0 {
                let drunk_idx = self.map.index_from_xy(drunk_x, drunk_y);
                if self.map.tiles[drunk_idx] == TileType::Wall {
                    did_something = true;
                }
                paint(
                    &mut self.map,
                    &self.settings.symmetry,
                    self.settings.brush_size,
                    drunk_x,
                    drunk_y,
                );

                match rng.roll_dice(1, 4) {
                    1 => {
                        if drunk_x > 2 {
                            drunk_x -= 1;
                        }
                    }
                    2 => {
                        if drunk_x < self.map.width - 2 {
                            drunk_x += 1;
                        }
                    }
                    3 => {
                        if drunk_y > 2 {
                            drunk_y -= 1;
                        }
                    }
                    _ => {
                        if drunk_y < self.map.height - 2 {
                            drunk_y += 1;
                        }
                    }
                }

                drunk_life -= 1;
            }
            if did_something {
                active_digger_count += 1;
            }

            digger_count += 1;
            floor_tile_count = self
                .map
                .tiles
                .iter()
                .filter(|tile| **tile == TileType::Floor)
                .count();
        }

        console::log(format!(
            "{} dwarves gave up their sobriety, of whom {} actually found a wall.",
            digger_count, active_digger_count
        ));

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

impl DrunkardsWalkBuilder {
    pub fn open_area(new_depth: u32) -> Self {
        Self {
            map: Map::empty_map(new_depth),
            starting_position: Position { x: 0, y: 0 },
            noise_areas: HashMap::new(),
            settings: DrunkardSettings {
                spawn_mode: DrunkSpawnMode::StartingPoint,
                drunken_lifetime: 400,
                floor_percent: 0.5,
                brush_size: 1,
                symmetry: Symmetry::None,
            },
        }
    }

    pub fn open_halls(new_depth: u32) -> Self {
        Self {
            map: Map::empty_map(new_depth),
            starting_position: Position { x: 0, y: 0 },
            noise_areas: HashMap::new(),
            settings: DrunkardSettings {
                spawn_mode: DrunkSpawnMode::Random,
                drunken_lifetime: 400,
                floor_percent: 0.5,
                brush_size: 1,
                symmetry: Symmetry::None,
            },
        }
    }

    pub fn winding_passages(new_depth: u32) -> Self {
        Self {
            map: Map::empty_map(new_depth),
            starting_position: Position { x: 0, y: 0 },
            noise_areas: HashMap::new(),
            settings: DrunkardSettings {
                spawn_mode: DrunkSpawnMode::Random,
                drunken_lifetime: 100,
                floor_percent: 0.4,
                brush_size: 1,
                symmetry: Symmetry::None,
            },
        }
    }

    pub fn fat_passages(new_depth: u32) -> Self {
        Self {
            map: Map::empty_map(new_depth),
            starting_position: Position { x: 0, y: 0 },
            noise_areas: HashMap::new(),
            settings: DrunkardSettings {
                spawn_mode: DrunkSpawnMode::Random,
                drunken_lifetime: 100,
                floor_percent: 0.4,
                brush_size: 2,
                symmetry: Symmetry::None,
            },
        }
    }

    pub fn fearful_symmetry(new_depth: u32) -> Self {
        Self {
            map: Map::empty_map(new_depth),
            starting_position: Position { x: 0, y: 0 },
            noise_areas: HashMap::new(),
            settings: DrunkardSettings {
                spawn_mode: DrunkSpawnMode::Random,
                drunken_lifetime: 100,
                floor_percent: 0.4,
                brush_size: 1,
                symmetry: Symmetry::Both,
            },
        }
    }
}
