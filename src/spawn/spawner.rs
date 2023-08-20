use std::collections::HashMap;

use rltk::{RandomNumberGenerator, RGB};
use specs::{
    prelude::*,
    saveload::{MarkedBuilder, SimpleMarker},
};

use crate::{
    components::{
        AreaOfEffect, BlocksTile, CombatStats, Confusion, Consumable, DefenseBonus, EntryTrigger,
        EquipmentSlot, Equippable, Hidden, HungerClock, HungerState, InflictsDamage, Item,
        MeleePowerBonus, Monster, Name, Player, Position, ProvidesFood, ProvidesHealing, Ranged,
        Renderable, SerializeMe, SingleActivation, Viewshed,
    },
    map::map::{Map, TileType},
    rect::Rect,
};

use super::random_table::{RandomTable, SpawnEntity};

pub fn player(ecs: &mut World, player_pos: Position) -> Entity {
    ecs.create_entity()
        .with(player_pos)
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
        .with(HungerClock {
            state: HungerState::WellFed,
            duration: 20,
        })
        .marked::<SimpleMarker<SerializeMe>>()
        .build()
}

/// Fill rooms with stuff!
pub fn spawn_room(ecs: &mut World, map: &Map, room: &Rect, max_entities: u16) {
    let mut possible_targets = Vec::new();
    for y in room.y1 + 1..room.y2 {
        for x in room.x1 + 1..room.x2 {
            let idx = map.index_from_xy(x, y);
            if map.tiles[idx] == TileType::Floor {
                possible_targets.push(idx);
            }
        }
    }

    //TODO
    spawn_region(ecs, &possible_targets, map, max_entities);
}

pub fn spawn_region(ecs: &mut World, possible_targets: &[usize], map: &Map, max_entities: u16) {
    let spawn_table = RandomTable::generate_loot_table(map.depth);
    let mut spawn_points = HashMap::new();
    let mut areas = Vec::from(possible_targets);

    {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        let num_spawns = u32::min(
            areas.len() as u32,
            rng.roll_dice(1, max_entities as i32) as u32,
        );
        if num_spawns == 0 {
            return;
        }

        for _ in 0..num_spawns {
            let array_index = if areas.len() == 1 {
                0
            } else {
                rng.roll_dice(1, areas.len() as i32) as usize - 1
            };
            let map_idx = areas[array_index];
            spawn_points.insert(map_idx, spawn_table.roll(&mut rng));
            areas.remove(array_index);
        }
    }

    for (idx, entity) in spawn_points {
        spawn_entity(ecs, entity, idx, map)
    }
}

fn spawn_entity(ecs: &mut World, entity: Option<SpawnEntity>, idx: usize, map: &Map) {
    let entity = match entity {
        None => return,
        Some(e) => e,
    };
    let (x, y) = map.xy_from_index(&idx);
    match entity {
        SpawnEntity::Goblin => new_goblin(ecs, x, y),
        SpawnEntity::Orc => new_orc(ecs, x, y),
        SpawnEntity::HealthPotion => health_potion(ecs, x, y),
        SpawnEntity::FireballScroll => fireball_scroll(ecs, x, y),
        SpawnEntity::ConfusionScroll => confusion_scroll(ecs, x, y),
        SpawnEntity::MagicMissileScroll => magic_missile_scroll(ecs, x, y),
        SpawnEntity::Dagger => dagger(ecs, x, y),
        SpawnEntity::Longsword => longsword(ecs, x, y),
        SpawnEntity::Shield => shield(ecs, x, y),
        SpawnEntity::TowerShield => tower_shield(ecs, x, y),
        SpawnEntity::Ration => rations(ecs, x, y),
        SpawnEntity::BearTrap => bear_trap(ecs, x, y),
    };
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

fn dagger(ecs: &mut World, x: u16, y: u16) {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: rltk::to_cp437('/'),
            fg: RGB::named(rltk::CYAN),
            bg: RGB::named(rltk::BLACK),
            render_order: 2,
        })
        .with(Name {
            name: "Dagger".to_string(),
        })
        .with(Item {})
        .with(Equippable {
            slot: EquipmentSlot::Melee,
        })
        .with(MeleePowerBonus { power: 2 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn longsword(ecs: &mut World, x: u16, y: u16) {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: rltk::to_cp437('/'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
            render_order: 2,
        })
        .with(Name {
            name: "Longsword".to_string(),
        })
        .with(Item {})
        .with(Equippable {
            slot: EquipmentSlot::Melee,
        })
        .with(MeleePowerBonus { power: 4 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn shield(ecs: &mut World, x: u16, y: u16) {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: rltk::to_cp437('('),
            fg: RGB::named(rltk::CYAN),
            bg: RGB::named(rltk::BLACK),
            render_order: 2,
        })
        .with(Name {
            name: "Shield".to_string(),
        })
        .with(Item {})
        .with(Equippable {
            slot: EquipmentSlot::Shield,
        })
        .with(DefenseBonus { defense: 1 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn tower_shield(ecs: &mut World, x: u16, y: u16) {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: rltk::to_cp437('('),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
            render_order: 2,
        })
        .with(Name {
            name: "Tower Shield {}".to_string(),
        })
        .with(Item {})
        .with(Equippable {
            slot: EquipmentSlot::Shield,
        })
        .with(DefenseBonus { defense: 3 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn rations(ecs: &mut World, x: u16, y: u16) {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: rltk::to_cp437('%'),
            fg: RGB::named(rltk::GREEN),
            bg: RGB::named(rltk::BLACK),
            render_order: 2,
        })
        .with(Name {
            name: "Rations".to_string(),
        })
        .with(Item {})
        .with(ProvidesFood {})
        .with(Consumable {})
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn bear_trap(ecs: &mut World, x: u16, y: u16) {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: rltk::to_cp437('^'),
            fg: RGB::named(rltk::RED),
            bg: RGB::named(rltk::BLACK),
            render_order: 2,
        })
        .with(Name {
            name: "Bear Trap".to_string(),
        })
        .with(Hidden {})
        .with(EntryTrigger {})
        .with(InflictsDamage { damage: 6 })
        .with(SingleActivation {})
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}
