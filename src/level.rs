use rltk::{Point, RandomNumberGenerator};
use specs::{Entity, Join, World, WorldExt};

use crate::{
    components::{CombatStats, Equipped, InBackpack, Player, Position, Viewshed},
    gamelog::GameLog,
    map::{map::Map, generate_map},
    spawn::spawner,
    systems::particle_system::ParticleBuilder,
};

const MAX_ENTITIES: i32 = 4;

pub fn new_game(ecs: &mut World) {
    // Remove all existing entities
    let to_delete: Vec<Entity> = {
        let entities = ecs.entities();
        entities.join().collect()
    };
    for target in to_delete {
        ecs.delete_entity(target).expect("Unable to delete entity");
    }

    let worldmap = {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        generate_map(1, &mut rng)
    };

    let (player_x, player_y) = worldmap.rooms[0].center();
    ecs.insert(Point::new(player_x, player_y));

    let player_entity = spawner::player(ecs, player_x, player_y);
    ecs.insert(player_entity);
    {
        let mut gamelog = ecs.fetch_mut::<GameLog>();
        gamelog
            .entries
            .push("While roaming in the wilds, you stumbled upon a mysterious cave.".to_string());
        gamelog.entries.push(
            "Having no shelter from the harsh weather outside, you decide to explore it."
                .to_string(),
        );
        gamelog.entries.push(
            "But as soon as you enter the cave, the entrance colapses trapping you inside."
                .to_string(),
        );
    }

    // Generate new mobs and items
    for room in worldmap.rooms.iter().skip(1) {
        spawner::spawn_room(ecs, &worldmap, room, MAX_ENTITIES);
    }

    ecs.insert(worldmap);
    ecs.insert(ParticleBuilder::new());
}

pub fn next_level(ecs: &mut World) {
    // Generate new world map
    let worldmap = {
        let map = ecs.fetch_mut::<Map>();
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        generate_map(map.depth + 1, &mut rng)
    };

    let (player_x, player_y) = worldmap.rooms[0].center();
    ecs.insert(Point::new(player_x, player_y));

    let player_entity = *ecs.fetch::<Entity>();
    {
        let mut position_component = ecs.write_storage::<Position>();
        let player_position_component = position_component.get_mut(player_entity);
        if let Some(player_position) = player_position_component {
            player_position.x = player_x;
            player_position.y = player_y;
        }
    }

    {
        // Mark the player's visibility as dirty
        let mut viewshed_storage = ecs.write_storage::<Viewshed>();
        let viewshed = viewshed_storage.get_mut(player_entity);
        if let Some(viewshed) = viewshed {
            viewshed.dirty = true;
        }
    }

    {
        // Notify the player and give them some health
        let mut gamelog = ecs.fetch_mut::<GameLog>();
        gamelog
            .entries
            .push("You descend to the next level, and take a moment to heal.".to_string());
        let mut health_storage = ecs.write_storage::<CombatStats>();
        let player_health = health_storage.get_mut(player_entity);
        if let Some(player_health) = player_health {
            player_health.hp = u32::max(player_health.hp, player_health.max_hp / 2);
        }
    }

    // Delete entities that aren't the player or player's equipment
    let to_delete = entities_to_remove_on_level_change(ecs);
    for target in to_delete {
        ecs.delete_entity(target).expect("Unable to delete entity");
    }

    // Generate new mobs and items
    for room in worldmap.rooms.iter().skip(1) {
        spawner::spawn_room(ecs, &worldmap, room, MAX_ENTITIES);
    }

    ecs.insert(worldmap);
}

fn entities_to_remove_on_level_change(ecs: &mut World) -> Vec<Entity> {
    let entities = ecs.entities();
    let player = ecs.read_storage::<Player>();
    let backpack = ecs.read_storage::<InBackpack>();
    let player_entity = ecs.fetch::<Entity>();
    let equipped = ecs.read_storage::<Equipped>();

    let mut to_delete: Vec<Entity> = Vec::new();
    for entity in entities.join() {
        let mut should_delete = true;

        // Don't delete the player
        if player.contains(entity) {
            should_delete = false;
        }

        // Don't delete items from player's backpack
        if let Some(bp) = backpack.get(entity) {
            if bp.owner == *player_entity {
                should_delete = false;
            }
        }

        // Don't delete equipped items
        if let Some(eq) = equipped.get(entity) {
            if eq.owner == *player_entity {
                should_delete = false;
            }
        }

        if should_delete {
            to_delete.push(entity);
        }
    }

    to_delete
}
