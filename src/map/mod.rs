use self::{map_builder::MapBuilder, random_builder::RandomMapBuilder};

pub mod map;
pub mod map_builder;
pub mod random_builder;

pub fn random_builder(new_depth: u32) -> impl MapBuilder {
    RandomMapBuilder::new(new_depth)
}
