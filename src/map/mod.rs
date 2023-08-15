use rltk::RandomNumberGenerator;

use self::{map::Map, random_builder::RandomMapBuilder, map_builder::MapBuilder};

pub mod map;
pub mod map_builder;
pub mod random_builder;

pub fn generate_map(new_depth: u32, rng: &mut RandomNumberGenerator) -> Map {
    RandomMapBuilder::build(new_depth, rng)
}