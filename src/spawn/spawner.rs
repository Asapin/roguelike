use std::collections::{hash_map::Entry, HashMap};

use rltk::{RandomNumberGenerator, RGB};
use specs::{
    prelude::*,
    saveload::{MarkedBuilder, SimpleMarker},
};

use crate::{
    components::{
        AreaOfEffect, BlocksTile, CombatStats, Confusion, Consumable, DefenseBonus, EquipmentSlot,
        Equippable, InflictsDamage, Item, MeleePowerBonus, Monster, Name, Player, Position,
        ProvidesHealing, Ranged, Renderable, SerializeMe, Viewshed,
    },
    map::Map,
    rect::Rect,
};

use super::random_table::{RandomTable, SpawnEntity};

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
pub fn spawn_room(ecs: &mut World, map: &Map, room: &Rect, max_entities: i32) {
    let spawn_table = room_table(map.depth);
    let mut spawn_points = HashMap::new();
    {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        let num_spawns = rng.roll_dice(1, max_entities + 3) + (map.depth - 1) as i32 - 3;

        // Generate spawn points for entities
        for _ in 0..num_spawns {
            let mut added = false;
            let mut tries = 0;
            while !added && tries < 20 {
                let room_width = (room.x2 - room.x1) as i32;
                let room_height = (room.y2 - room.y1) as i32;
                let x = (room.x1 as i32 + rng.roll_dice(1, room_width)) as u16;
                let y = (room.y1 as i32 + rng.roll_dice(1, room_height)) as u16;
                let idx = map.index_from_xy(x, y);
                if let Entry::Vacant(e) = spawn_points.entry(idx) {
                    e.insert(spawn_table.roll(&mut rng));
                    added = true;
                } else {
                    tries += 1;
                }
            }
        }
    }

    // Spawn entities
    let mut item_counter = 0;
    for (idx, spawn) in spawn_points.into_iter() {
        item_counter += 1;
        let (x, y) = map.xy_from_index(&idx);
        if let Some(spawn) = spawn {
            match spawn {
                SpawnEntity::Goblin => new_goblin(ecs, x, y),
                SpawnEntity::Orc => new_orc(ecs, x, y),
                SpawnEntity::HealthPotion => health_potion(ecs, x, y),
                SpawnEntity::FireballScroll => fireball_scroll(ecs, x, y),
                SpawnEntity::ConfusionScroll => confusion_scroll(ecs, x, y),
                SpawnEntity::MagicMissileScroll => magic_missile_scroll(ecs, x, y),
                SpawnEntity::Dagger => dagger(ecs, x, y, item_counter),
                SpawnEntity::Longsword => longsword(ecs, x, y, item_counter),
                SpawnEntity::Shield => shield(ecs, x, y, item_counter),
                SpawnEntity::TowerShield => tower_shield(ecs, x, y, item_counter),
            };
        }
    }
}

fn room_table(map_depth: u32) -> RandomTable {
    RandomTable::new()
        .add(SpawnEntity::Goblin, 10)
        .add(SpawnEntity::Orc, 1 + map_depth as i32)
        .add(SpawnEntity::HealthPotion, 7)
        .add(SpawnEntity::FireballScroll, 2 + map_depth as i32)
        .add(SpawnEntity::ConfusionScroll, 2 + map_depth as i32)
        .add(SpawnEntity::MagicMissileScroll, 4)
        .add(SpawnEntity::Dagger, 3)
        .add(SpawnEntity::Longsword, map_depth as i32 - 1)
        .add(SpawnEntity::Shield, 3)
        .add(SpawnEntity::TowerShield, map_depth as i32 - 1)
}

fn new_orc(ecs: &mut World, x: u16, y: u16) {
    generic_mob(ecs, x, y, rltk::to_cp437('o'), "Orc");
}

fn new_goblin(ecs: &mut World, x: u16, y: u16) {
    generic_mob(ecs, x, y, rltk::to_cp437('g'), "Goblin");
}

fn generic_mob(ecs: &mut World, x: u16, y: u16, glyph: rltk::FontCharType, name: &str) {
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
        .build();
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

fn dagger(ecs: &mut World, x: u16, y: u16, counter: i32) {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: rltk::to_cp437('/'),
            fg: RGB::named(rltk::CYAN),
            bg: RGB::named(rltk::BLACK),
            render_order: 2,
        })
        .with(Name {
            name: format!("Dagger {}", counter),
        })
        .with(Item {})
        .with(Equippable {
            slot: EquipmentSlot::Melee,
        })
        .with(MeleePowerBonus { power: 2 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn longsword(ecs: &mut World, x: u16, y: u16, counter: i32) {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: rltk::to_cp437('/'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
            render_order: 2,
        })
        .with(Name {
            name: format!("Longsword {}", counter),
        })
        .with(Item {})
        .with(Equippable {
            slot: EquipmentSlot::Melee,
        })
        .with(MeleePowerBonus { power: 4 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn shield(ecs: &mut World, x: u16, y: u16, counter: i32) {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: rltk::to_cp437('('),
            fg: RGB::named(rltk::CYAN),
            bg: RGB::named(rltk::BLACK),
            render_order: 2,
        })
        .with(Name {
            name: format!("Shield {}", counter),
        })
        .with(Item {})
        .with(Equippable {
            slot: EquipmentSlot::Shield,
        })
        .with(DefenseBonus { defense: 1 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn tower_shield(ecs: &mut World, x: u16, y: u16, counter: i32) {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: rltk::to_cp437('('),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
            render_order: 2,
        })
        .with(Name {
            name: format!("Tower Shield {}", counter),
        })
        .with(Item {})
        .with(Equippable {
            slot: EquipmentSlot::Shield,
        })
        .with(DefenseBonus { defense: 3 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}
