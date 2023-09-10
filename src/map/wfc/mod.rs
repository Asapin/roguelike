use std::collections::HashMap;

use self::{
    constraints::CompatibilityMatrix,
    modules::{load_from_byte_slice, Module},
    solver::Solver,
};
use crate::{components::Position, spawn::spawner::spawn_region};

use super::{
    generate_voronoi_spawn_regions,
    map::{Map, TileType},
    remove_unreachable_areas, MapBuilder,
};

rltk::embedded_resource!(WFC_MAP_1, "../../../resources/wfc_tiles.map");

const MAX_ENTITIES: u16 = 4;
const CHUNK_SIZE: u8 = 7;

mod constraints;
mod modules;
mod solver;

pub struct WaveformCollapseBuilder {
    modules: Vec<Module>,
    map: Map,
    starting_position: Position,
    noise_areas: HashMap<i32, Vec<usize>>,
}

impl MapBuilder for WaveformCollapseBuilder {
    fn get_map(&mut self) -> Map {
        self.map.clone()
    }

    fn get_starting_position(&mut self) -> Position {
        self.starting_position.clone()
    }

    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator) {
        // Skip 1 pixel on each side of the map
        // since these pixels are just part of the world border.
        let horizontal_chunks = (self.map.width - 2) / CHUNK_SIZE as u16;
        let vertical_chunks = (self.map.height - 2) / CHUNK_SIZE as u16;
        let compatibility_matrix = CompatibilityMatrix::build(&self.modules);
        let mut solver = Solver::new(
            compatibility_matrix.clone(),
            self.modules.len(),
            horizontal_chunks,
            vertical_chunks,
        );
        let mut attempt = 0;
        loop {
            attempt += 1;
            while !solver.is_collapsed() {
                solver.iterate(rng);
            }

            self.apply_solver(&solver, horizontal_chunks, vertical_chunks);
            if !solver.is_solved() {
                rltk::console::log("Solver couldn't solve current attempt. Dumping map's content and restarting attempt...");
                let file_name = format!("{}.map", attempt);
                self.map.dump_to_file(&file_name);
                solver = Solver::new(
                    compatibility_matrix.clone(),
                    self.modules.len(),
                    horizontal_chunks,
                    vertical_chunks,
                );
            } else {
                break;
            }
        }

        self.starting_position = Position {
            x: self.map.width / 2 - 1,
            y: self.map.height / 2 - 3,
        };

        let start_idx = self
            .map
            .index_from_xy(self.starting_position.x, self.starting_position.y);
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

impl WaveformCollapseBuilder {
    pub fn from_manual_tiles(new_depth: u32) -> Self {
        let modules = load_from_byte_slice(WFC_MAP_1, CHUNK_SIZE);
        Self {
            modules,
            map: Map::empty_map(new_depth),
            starting_position: Position { x: 0, y: 0 },
            noise_areas: HashMap::new(),
        }
    }

    fn apply_solver(&mut self, solver: &Solver, horizontal_chunks: u16, vertical_chunks: u16) {
        for chunk_y in 0..vertical_chunks {
            for chunk_x in 0..horizontal_chunks {
                let chunk_idx = (chunk_y * horizontal_chunks + chunk_x) as usize;
                let map_y_start = chunk_y * CHUNK_SIZE as u16 + 1;
                let map_x_start = chunk_x * CHUNK_SIZE as u16 + 1;

                if let Some(module_idx) = solver.collapsed.get(&chunk_idx) {
                    let module = &self.modules[*module_idx];

                    for pattern_y in 0..CHUNK_SIZE {
                        for pattern_x in 0..CHUNK_SIZE {
                            let pattern_tile_idx =
                                Module::tile_idx(pattern_x, pattern_y, CHUNK_SIZE);
                            let tile_idx = self.map.index_from_xy(
                                map_x_start + pattern_x as u16,
                                map_y_start + pattern_y as u16,
                            );
                            self.map.tiles[tile_idx] = module.pattern[pattern_tile_idx];
                        }
                    }
                } else {
                    // This block wasn't solved. Fill it with stairs for debugging.
                    for pattern_y in 0..CHUNK_SIZE {
                        for pattern_x in 0..CHUNK_SIZE {
                            let tile_idx = self.map.index_from_xy(
                                map_x_start + pattern_x as u16,
                                map_y_start + pattern_y as u16,
                            );
                            self.map.tiles[tile_idx] = TileType::DownStairs;
                        }
                    }
                }
            }
        }
    }
}
