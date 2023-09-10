use std::collections::HashSet;

use crate::map::map::TileType;

#[derive(PartialEq, Eq, Hash)]
pub struct Module {
    pub pattern: Vec<TileType>,
    north_exits: u64,
    south_exits: u64,
    east_exits: u64,
    west_exits: u64,
}

impl Module {
    pub fn new(pattern: Vec<TileType>, chunk_size: u8) -> Self {
        let mut north_exits = 0;
        let mut south_exits = 0;
        let mut east_exits = 0;
        let mut west_exits = 0;
        for i in 0..chunk_size {
            let north_idx = Self::tile_idx(i, 0, chunk_size);
            if pattern[north_idx] == TileType::Floor {
                north_exits |= 1 << i;
            }

            let south_idx = Self::tile_idx(i, chunk_size - 1, chunk_size);
            if pattern[south_idx] == TileType::Floor {
                south_exits |= 1 << i;
            }

            let west_idx = Self::tile_idx(0, i, chunk_size);
            if pattern[west_idx] == TileType::Floor {
                west_exits |= 1 << i;
            }

            let east_idx = Self::tile_idx(chunk_size - 1, i, chunk_size);
            if pattern[east_idx] == TileType::Floor {
                east_exits |= 1 << i;
            }
        }

        Self {
            pattern,
            north_exits,
            south_exits,
            east_exits,
            west_exits,
        }
    }

    pub fn tile_idx(x: u8, y: u8, chunk_size: u8) -> usize {
        (y as usize * chunk_size as usize) + x as usize
    }

    pub fn compatible_on_north_side(&self, other: &Self) -> bool {
        self.north_exits == other.south_exits || self.north_exits & other.south_exits > 0
    }

    pub fn compatible_on_south_side(&self, other: &Self) -> bool {
        self.south_exits == other.north_exits || self.south_exits & other.north_exits > 0
    }

    pub fn compatible_on_west_side(&self, other: &Self) -> bool {
        self.west_exits == other.east_exits || self.west_exits & other.east_exits > 0
    }

    pub fn compatible_on_east_side(&self, other: &Self) -> bool {
        self.east_exits == other.west_exits || self.east_exits & other.west_exits > 0
    }
}

pub fn load_from_byte_slice(data: &[u8], chunk_size: u8) -> Vec<Module> {
    let mut tiles = Vec::with_capacity(data.len());
    for tile in data {
        match tile {
            b'#' => {
                tiles.push(TileType::Wall);
            }
            b' ' => {
                tiles.push(TileType::Floor);
            }
            _ => {}
        }
    }

    convert_tiles_to_modules(&tiles, chunk_size)
}

fn convert_tiles_to_modules(tiles: &[TileType], chunk_size: u8) -> Vec<Module> {
    let chunk_area = chunk_size as usize * chunk_size as usize;
    let number_of_chunks = tiles.len() / chunk_area;
    let line_length = number_of_chunks * chunk_size as usize;
    let mut modules = HashSet::new();
    for chunk_idx in 0..number_of_chunks {
        let x_start = chunk_idx * chunk_size as usize;
        let x_end = x_start + chunk_size as usize;

        let mut pattern = Vec::with_capacity(chunk_area);
        for y in 0..chunk_size {
            for x in x_start..x_end {
                let tile_idx = y as usize * line_length + x;
                pattern.push(tiles[tile_idx]);
            }
        }
        let module = Module::new(pattern, chunk_size);
        modules.insert(module);

        // Flip horizontally
        let mut pattern = Vec::with_capacity(chunk_area);
        for y in 0..chunk_size {
            for x in (x_start..x_end).rev() {
                let tile_idx = y as usize * line_length + x;
                pattern.push(tiles[tile_idx]);
            }
        }
        let module = Module::new(pattern, chunk_size);
        modules.insert(module);

        // Flip vertically
        let mut pattern = Vec::with_capacity(chunk_area);
        for y in (0..chunk_size).rev() {
            for x in x_start..x_end {
                let tile_idx = y as usize * line_length + x;
                pattern.push(tiles[tile_idx]);
            }
        }
        let module = Module::new(pattern, chunk_size);
        modules.insert(module);

        // Flip both sides
        let mut pattern = Vec::with_capacity(chunk_area);
        for y in (0..chunk_size).rev() {
            for x in (x_start..x_end).rev() {
                let tile_idx = y as usize * line_length + x;
                pattern.push(tiles[tile_idx]);
            }
        }
        let module = Module::new(pattern, chunk_size);
        modules.insert(module);
    }

    let mut result = Vec::new();
    result.extend(modules.into_iter());
    result
}
