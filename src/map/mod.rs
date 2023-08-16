use rltk::RandomNumberGenerator;

use crate::components::Position;

use self::{map::Map, map_builder::MapBuilder, random_builder::RandomMapBuilder};

pub mod map;
pub mod map_builder;
pub mod random_builder;

pub fn generate_map(new_depth: u32, rng: &mut RandomNumberGenerator) -> (Map, Position) {
    RandomMapBuilder::build(new_depth, rng)
}
