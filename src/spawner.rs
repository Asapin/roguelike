use rltk::{RandomNumberGenerator, RGB};
use specs::{
    prelude::*,
    saveload::{MarkedBuilder, SimpleMarker},
};

use crate::{
    components::{
        AreaOfEffect, BlocksTile, CombatStats, Confusion, Consumable, InflictsDamage, Item,
        Monster, Name, Player, Position, ProvidesHealing, Ranged, Renderable, SerializeMe,
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
            render_order: 0,
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
        .marked::<SimpleMarker<SerializeMe>>()
        .build()
}

/// Fill rooms with stuff!
pub fn spawn_room(ecs: &mut World, map: &Map, room: &Rect, max_monsters: i32, max_items: i32) {
    let mut monster_spawn_points: Vec<usize> = Vec::new();
    let mut item_spawn_points: Vec<usize> = Vec::new();

    {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        let num_monsters = rng.roll_dice(1, max_monsters + 1) - 1;
        let num_items = rng.roll_dice(1, max_items + 1) - 1;

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
                if !item_spawn_points.contains(&idx) {
                    item_spawn_points.push(idx);
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
    for idx in item_spawn_points.iter() {
        let (x, y) = map.xy_from_index(idx);
        random_item(ecs, x, y);
    }
}

fn random_monster(ecs: &mut World, x: u16, y: u16) {
    let roll = {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        rng.roll_dice(1, 2)
    };
    match roll {
        1 => new_orc(ecs, x, y),
        _ => new_goblin(ecs, x, y),
    };
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
            render_order: 1,
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
        .marked::<SimpleMarker<SerializeMe>>()
        .build()
}

fn random_item(ecs: &mut World, x: u16, y: u16) {
    let roll = {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        rng.roll_dice(1, 4)
    };
    match roll {
        1 => health_potion(ecs, x, y),
        2 => magic_missile_scroll(ecs, x, y),
        3 => confusion_scroll(ecs, x, y),
        _ => fireball_scroll(ecs, x, y),
    };
}

fn health_potion(ecs: &mut World, x: u16, y: u16) {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: rltk::to_cp437('¡'),
            fg: RGB::named(rltk::MAGENTA),
            bg: RGB::named(rltk::BLACK),
            render_order: 2,
        })
        .with(Name {
            name: "Health potion".to_string(),
        })
        .with(Item {})
        .with(Consumable {})
        .with(ProvidesHealing { amount: 8 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn magic_missile_scroll(ecs: &mut World, x: u16, y: u16) {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: rltk::to_cp437(')'),
            fg: RGB::named(rltk::CYAN),
            bg: RGB::named(rltk::BLACK),
            render_order: 2,
        })
        .with(Name {
            name: "Magic Missile Scroll".to_string(),
        })
        .with(Item {})
        .with(Consumable {})
        .with(Ranged { range: 6 })
        .with(InflictsDamage { damage: 8 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn fireball_scroll(ecs: &mut World, x: u16, y: u16) {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: rltk::to_cp437(')'),
            fg: RGB::named(rltk::ORANGE),
            bg: RGB::named(rltk::BLACK),
            render_order: 2,
        })
        .with(Name {
            name: "Fireball Scroll".to_string(),
        })
        .with(Item {})
        .with(Consumable {})
        .with(Ranged { range: 6 })
        .with(InflictsDamage { damage: 20 })
        .with(AreaOfEffect { radius: 3 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn confusion_scroll(ecs: &mut World, x: u16, y: u16) {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: rltk::to_cp437(')'),
            fg: RGB::named(rltk::PINK),
            bg: RGB::named(rltk::BLACK),
            render_order: 2,
        })
        .with(Name {
            name: "Confusion Scroll".to_string(),
        })
        .with(Item {})
        .with(Consumable {})
        .with(Ranged { range: 6 })
        .with(Confusion { turns: 4 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}
