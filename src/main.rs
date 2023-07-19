use rltk::Point;
use specs::prelude::*;

use crate::{
    gamelog::GameLog,
    map::Map,
    state::{RunState, State},
    systems::Systems,
};

mod components;
mod gamelog;
mod gui;
mod map;
mod rect;
mod spawner;
mod state;
mod systems;

const MAP_WIDTH: u16 = 80;
const MAP_HEIGHT: u16 = 43;
const ROOM_COUNT: u8 = 30;
const MIN_ROOM_SIZE: u8 = 6;
const MAX_ROOM_SIZE: u8 = 10;
const MAX_MONSTERS: i32 = 4;
const MAX_ITEMS: i32 = 2;

fn main() -> rltk::BError {
    use rltk::RltkBuilder;
    let mut context = RltkBuilder::simple80x50()
        .with_title("Roguelike tutorial")
        .build()?;
    context.with_post_scanlines(true);

    let mut gs = State {
        ecs: World::new(),
        systems: Systems::new(),
    };
    components::register_components(&mut gs.ecs);

    let mut rng = rltk::RandomNumberGenerator::new();
    let map = Map::new_map_rooms_and_corridors(
        MAP_WIDTH,
        MAP_HEIGHT,
        80,
        50,
        ROOM_COUNT,
        MIN_ROOM_SIZE,
        MAX_ROOM_SIZE,
        &mut rng,
    );
    gs.ecs.insert(rng);

    let (player_x, player_y) = map.rooms[0].center();
    let player_entity = spawner::player(&mut gs.ecs, player_x, player_y);
    gs.ecs.insert(Point::new(player_x, player_y));
    gs.ecs.insert(player_entity);

    for room in map.rooms.iter().skip(1) {
        spawner::spawn_room(&mut gs.ecs, &map, room, MAX_MONSTERS, MAX_ITEMS);
    }
    gs.ecs.insert(map);
    gs.ecs.insert(GameLog {
        entries: vec!["Welcome to Rusty Roguelike".to_string()],
    });
    gs.ecs.insert(RunState::PreRun);
    rltk::main_loop(context, gs)
}
