use components::{Player, Position, Renderable};
use rltk::{Point, RGB};
use specs::prelude::*;

use crate::{
    components::{BlocksTile, CombatStats, Monster, Name, SufferDamage, Viewshed, WantsToMelee},
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
mod state;
mod systems;

const MAP_WIDTH: u16 = 80;
const MAP_HEIGHT: u16 = 43;
const ROOM_COUNT: u8 = 30;
const MIN_ROOM_SIZE: u8 = 6;
const MAX_ROOM_SIZE: u8 = 10;

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
    gs.ecs.register::<Position>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<Player>();
    gs.ecs.register::<Viewshed>();
    gs.ecs.register::<Monster>();
    gs.ecs.register::<Name>();
    gs.ecs.register::<BlocksTile>();
    gs.ecs.register::<CombatStats>();
    gs.ecs.register::<WantsToMelee>();
    gs.ecs.register::<SufferDamage>();

    let map = Map::new_map_rooms_and_corridors(
        MAP_WIDTH,
        MAP_HEIGHT,
        ROOM_COUNT,
        MIN_ROOM_SIZE,
        MAX_ROOM_SIZE,
    );
    let (player_x, player_y) = map.rooms[0].center();
    let player_entity = gs
        .ecs
        .create_entity()
        .with(Position {
            x: player_x,
            y: player_y,
        })
        .with(Renderable {
            glyph: rltk::to_cp437('@'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
        })
        .with(Player {})
        .with(Viewshed {
            visible_tiles: Vec::new(),
            range: 8,
            dirty: true,
        })
        .with(Name {
            name: "Player".to_string(),
        })
        .with(CombatStats {
            max_hp: 30,
            hp: 30,
            defense: 2,
            power: 5,
        })
        .build();

    let mut rng = rltk::RandomNumberGenerator::new();
    for (idx, room) in map.rooms.iter().skip(1).enumerate() {
        let (x, y) = room.center();
        if rng.rand::<bool>() {
            let _ = new_goblin(&mut gs.ecs, x, y, idx);
        } else {
            let _ = new_orc(&mut gs.ecs, x, y, idx);
        }
    }

    gs.ecs.insert(Point::new(player_x, player_y));
    gs.ecs.insert(player_entity);
    gs.ecs.insert(map);
    gs.ecs.insert(GameLog {
        entries: vec!["Welcome to Rusty Roguelike".to_string()],
    });
    gs.ecs.insert(RunState::PreRun);
    rltk::main_loop(context, gs)
}

fn new_orc(ecs: &mut World, x: u16, y: u16, idx: usize) -> Entity {
    let glyph = rltk::to_cp437('o');
    let name = Name {
        name: format!("Orc #{}", idx),
    };
    let position = Position { x, y };
    generic_mob(ecs, name, position, glyph)
}

fn new_goblin(ecs: &mut World, x: u16, y: u16, idx: usize) -> Entity {
    let glyph = rltk::to_cp437('g');
    let name = Name {
        name: format!("Goblin #{}", idx),
    };
    let position = Position { x, y };
    generic_mob(ecs, name, position, glyph)
}

fn generic_mob(ecs: &mut World, name: Name, position: Position, glyph: u16) -> Entity {
    ecs.create_entity()
        .with(position)
        .with(Renderable {
            glyph,
            fg: RGB::named(rltk::RED),
            bg: RGB::named(rltk::BLACK),
        })
        .with(Viewshed {
            visible_tiles: Vec::new(),
            range: 8,
            dirty: true,
        })
        .with(Monster {})
        .with(name)
        .with(BlocksTile {})
        .with(CombatStats {
            max_hp: 16,
            hp: 16,
            defense: 1,
            power: 4,
        })
        .build()
}
