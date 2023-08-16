use rltk::RandomNumberGenerator;
use specs::{Entity, Join, World, WorldExt};

use crate::{
    components::{CombatStats, Equipped, InBackpack, Player, Position, Viewshed},
    gamelog::GameLog,
    map::{map::Map, random_builder},
    spawn::spawner,
    systems::particle_system::ParticleBuilder,
};

pub fn new_game(ecs: &mut World) {
    // Remove all existing entities
    ecs.delete_all();

    let player_pos = generate_map(1, ecs);
    let player_entity = spawner::player(ecs, player_pos);
    ecs.insert(player_entity);
    ecs.insert(ParticleBuilder::new());

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
}

pub fn next_level(ecs: &mut World) {
    // Delete entities that aren't the player or player's equipment
    remove_entities_on_level_change(ecs);
    let player_entity = *ecs.fetch::<Entity>();
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

    // Generate new world map
    let new_depth = ecs.fetch_mut::<Map>().depth + 1;
    let player_pos = generate_map(new_depth, ecs);
    {
        let mut position_component = ecs.write_storage::<Position>();
        let player_position_component = position_component.get_mut(player_entity);
        if let Some(player_position) = player_position_component {
            player_position.x = player_pos.x;
            player_position.y = player_pos.y;
        }
    }
}

fn generate_map(new_depth: u32, ecs: &mut World) -> Position {
    let mut map_builder = {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        random_builder(new_depth, &mut rng)
    };
    {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        map_builder.build_map(&mut rng);
    }
    map_builder.spawn_entities(ecs);
    let player_pos = map_builder.get_starting_position();
    ecs.insert(player_pos);
    ecs.insert(map_builder.get_map());
    player_pos
}

fn remove_entities_on_level_change(ecs: &mut World) {
    let mut to_delete: Vec<Entity> = Vec::new();
    {
        let entities = ecs.entities();
        let player = ecs.read_storage::<Player>();
        let backpack = ecs.read_storage::<InBackpack>();
        let player_entity = ecs.fetch::<Entity>();
        let equipped = ecs.read_storage::<Equipped>();

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
    }
    for target in to_delete {
        ecs.delete_entity(target).expect("Unable to delete entity");
    }
}
