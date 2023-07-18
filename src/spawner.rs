use rltk::{RandomNumberGenerator, RGB};
use specs::prelude::*;

use crate::{
    components::{
        BlocksTile, CombatStats, Item, Monster, Name, Player, Position, Renderable, RestoreHealth,
        Viewshed,
    },
    map::Map,
    rect::Rect,
};

pub fn player(ecs: &mut World, player_x: u16, player_y: u16) -> Entity {
    ecs.create_entity()
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
        .build()
}

/// Fill rooms with stuff!
pub fn spawn_room(ecs: &mut World, map: &Map, room: &Rect, max_monsters: i32, max_items: i32) {
    let mut monster_spawn_points: Vec<usize> = Vec::new();
    let mut item_spawn_potions: Vec<usize> = Vec::new();

    {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        let num_monsters = rng.roll_dice(1, max_monsters) - 1;
        let num_items = rng.roll_dice(1, max_items) - 1;

        // Generate spawn points for monsters
        for _ in 0..num_monsters {
            let mut added = false;
            while !added {
                let room_width = (room.x2 - room.x1) as i32;
                let room_height = (room.y2 - room.y1) as i32;
                let x = (room.x1 as i32 + rng.roll_dice(1, room_width)) as u16;
                let y = (room.y1 as i32 + rng.roll_dice(1, room_height)) as u16;
                let idx = map.index_from_xy(x, y);
                if !monster_spawn_points.contains(&idx) {
                    monster_spawn_points.push(idx);
                    added = true;
                }
            }
        }

        // Generate spawn points for items
        for _ in 0..num_items {
            let mut added = false;
            while !added {
                let room_width = (room.x2 - room.x1) as i32;
                let room_height = (room.y2 - room.y1) as i32;
                let x = (room.x1 as i32 + rng.roll_dice(1, room_width)) as u16;
                let y = (room.y1 as i32 + rng.roll_dice(1, room_height)) as u16;
                let idx = map.index_from_xy(x, y);
                if !item_spawn_potions.contains(&idx) {
                    item_spawn_potions.push(idx);
                    added = true;
                }
            }
        }
    }

    // Spawn random monsters
    for idx in monster_spawn_points.iter() {
        let (x, y) = map.xy_from_index(idx);
        random_monster(ecs, x, y);
    }

    // Spawn potions
    for idx in item_spawn_potions.iter() {
        let (x, y) = map.xy_from_index(idx);
        health_potion(ecs, x, y);
    }
}

fn random_monster(ecs: &mut World, x: u16, y: u16) -> Entity {
    let roll = {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        rng.rand::<bool>()
    };
    if roll {
        new_orc(ecs, x, y)
    } else {
        new_goblin(ecs, x, y)
    }
}

fn health_potion(ecs: &mut World, x: u16, y: u16) {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: rltk::to_cp437('¡'),
            fg: RGB::named(rltk::MAGENTA),
            bg: RGB::named(rltk::BLACK),
        })
        .with(Name {
            name: "Health potion".to_string(),
        })
        .with(Item {})
        .with(RestoreHealth { amount: 8 })
        .build();
}

fn new_orc(ecs: &mut World, x: u16, y: u16) -> Entity {
    generic_mob(ecs, x, y, rltk::to_cp437('o'), "Orc")
}

fn new_goblin(ecs: &mut World, x: u16, y: u16) -> Entity {
    generic_mob(ecs, x, y, rltk::to_cp437('g'), "Goblin")
}

fn generic_mob(ecs: &mut World, x: u16, y: u16, glyph: rltk::FontCharType, name: &str) -> Entity {
    ecs.create_entity()
        .with(Position { x, y })
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
        .with(Name {
            name: name.to_string(),
        })
        .with(BlocksTile {})
        .with(CombatStats {
            max_hp: 16,
            hp: 16,
            defense: 1,
            power: 4,
        })
        .build()
}
