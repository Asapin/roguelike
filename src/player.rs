use rltk::{Rltk, VirtualKeyCode, RGB};
use specs::{prelude::*, Entity, World};
use std::cmp::{max, min};

use crate::{
    components::{
        CombatStats, Confusion, EntityMoved, HungerClock, HungerState, Item, Monster, Player,
        Position, Viewshed, WantsToMelee, WantsToPickupItem,
    },
    gamelog::GameLog,
    map::map::{Map, TileType},
    menu::pause_menu::PauseMenuSelection,
    state::RunState,
    systems::particle_system::ParticleBuilder,
};

pub fn player_input(ecs: &mut World, ctx: &mut Rltk) -> RunState {
    {
        let player = ecs.fetch::<Entity>();
        let mut confusion = ecs.write_storage::<Confusion>();
        if let Some(confused) = confusion.get_mut(*player) {
            let mut gamelog = ecs.fetch_mut::<GameLog>();
            gamelog.entries.push("You are still confused".to_string());
            confused.turns -= 1;
            if confused.turns == 0 {
                confusion.remove(*player);
            }
            let player_pos = ecs.write_resource::<Position>();
            ecs.fetch_mut::<ParticleBuilder>().request(
                player_pos.x,
                player_pos.y,
                RGB::named(rltk::MAGENTA),
                RGB::named(rltk::BLACK),
                rltk::to_cp437('?'),
                200.0,
            );
            return RunState::PlayerTurn;
        }
    };

    // Player movement
    match ctx.key {
        None => return RunState::AwaitingInput,
        Some(key) => match key {
            VirtualKeyCode::Numpad4 | VirtualKeyCode::A => try_move_player(ecs, -1, 0),
            VirtualKeyCode::Numpad6 | VirtualKeyCode::D => try_move_player(ecs, 1, 0),
            VirtualKeyCode::Numpad8 | VirtualKeyCode::W => try_move_player(ecs, 0, -1),
            VirtualKeyCode::Numpad2 | VirtualKeyCode::X => try_move_player(ecs, 0, 1),

            // Diagonals
            VirtualKeyCode::Numpad9 | VirtualKeyCode::E => try_move_player(ecs, 1, -1),
            VirtualKeyCode::Numpad7 | VirtualKeyCode::Q => try_move_player(ecs, -1, -1),
            VirtualKeyCode::Numpad3 | VirtualKeyCode::C => try_move_player(ecs, 1, 1),
            VirtualKeyCode::Numpad1 | VirtualKeyCode::Z => try_move_player(ecs, -1, 1),

            // Pickup
            VirtualKeyCode::Numpad5 | VirtualKeyCode::S => pickup(ecs),

            // Inventory
            VirtualKeyCode::I => return RunState::ShowInventory,
            VirtualKeyCode::R => return RunState::ShowDropItem,
            VirtualKeyCode::U => return RunState::ShowUnequipItem,

            // Save and Quit
            VirtualKeyCode::Escape => {
                return RunState::PauseMenu {
                    selected_menu: PauseMenuSelection::Restart,
                }
            }

            // Level changes
            VirtualKeyCode::Period => {
                if try_next_level(ecs) {
                    return RunState::NextLevel;
                }
            }

            // Skip turn
            VirtualKeyCode::Space => return skip_turn(ecs),
            _ => return RunState::AwaitingInput,
        },
    }
    RunState::PlayerTurn
}

fn try_move_player(ecs: &mut World, delta_x: i32, delta_y: i32) {
    let mut positions = ecs.write_storage::<Position>();
    let mut players = ecs.write_storage::<Player>();
    let mut viewsheds = ecs.write_storage::<Viewshed>();
    let combat_stats = ecs.read_storage::<CombatStats>();
    let map = ecs.fetch::<Map>();
    let mut player_pos = ecs.write_resource::<Position>();
    let entities = ecs.entities();
    let mut wants_to_melee = ecs.write_storage::<WantsToMelee>();
    let mut entity_moved = ecs.write_storage::<EntityMoved>();

    for (entity, _player, pos, viewshed) in
        (&entities, &mut players, &mut positions, &mut viewsheds).join()
    {
        let new_x = (pos.x as i32 + delta_x) as u16;
        let new_y = (pos.y as i32 + delta_y) as u16;
        if new_x < 1 || new_x > map.width - 1 || new_y < 1 || new_y > map.height - 1 {
            return;
        }

        let destination_idx = map.index_from_xy(new_x, new_y);

        for potential_target in map.tile_content[destination_idx].iter() {
            let target = combat_stats.get(*potential_target);
            if let Some(_target) = target {
                wants_to_melee
                    .insert(
                        entity,
                        WantsToMelee {
                            target: *potential_target,
                        },
                    )
                    .expect("Add target failed");
                return;
            }
        }
        if !map.blocked[destination_idx] {
            pos.x = min(map.width - 1, max(0, new_x));
            pos.y = min(map.height - 1, max(0, new_y));
            viewshed.dirty = true;
            player_pos.x = pos.x;
            player_pos.y = pos.y;
            entity_moved
                .insert(entity, EntityMoved {})
                .expect("Unable to insert marker");
        }
    }
}

fn pickup(ecs: &mut World) {
    let player_pos = ecs.fetch::<Position>();
    let player_entity = ecs.fetch::<Entity>();
    let entities = ecs.entities();
    let items = ecs.read_storage::<Item>();
    let positions = ecs.read_storage::<Position>();
    let mut gamelog = ecs.fetch_mut::<GameLog>();

    let mut target_item: Option<Entity> = None;
    for (item_entity, _, position) in (&entities, &items, &positions).join() {
        if position.x == player_pos.x && position.y == player_pos.y {
            target_item = Some(item_entity);
        }
    }

    match target_item {
        None => gamelog
            .entries
            .push("There is nothing here to pickup".to_string()),
        Some(item) => {
            let mut pickup = ecs.write_storage::<WantsToPickupItem>();
            pickup
                .insert(
                    *player_entity,
                    WantsToPickupItem {
                        collected_by: *player_entity,
                        item,
                    },
                )
                .expect("Unable to insert want to pickup");
        }
    }
}

fn try_next_level(ecs: &mut World) -> bool {
    let player_pos = ecs.fetch::<Position>();
    let map = ecs.fetch::<Map>();
    let player_idx = map.index_from_xy(player_pos.x, player_pos.y);
    if map.tiles[player_idx] == TileType::DownStairs {
        true
    } else {
        let mut gamelog = ecs.fetch_mut::<GameLog>();
        gamelog
            .entries
            .push("There is no way down from here.".to_string());
        false
    }
}

fn skip_turn(ecs: &mut World) -> RunState {
    let player_entity = ecs.fetch::<Entity>();
    let viewshed_storage = ecs.read_storage::<Viewshed>();
    let monsters = ecs.read_storage::<Monster>();
    let hunger_clocks = ecs.read_storage::<HungerClock>();

    let map = ecs.fetch::<Map>();

    let mut can_heal = true;
    if let Some(hunger) = hunger_clocks.get(*player_entity) {
        if hunger.state == HungerState::Hungry || hunger.state == HungerState::Starving {
            can_heal = false;
        }
    }

    // Player can't heal when hungry anyway, so no need to check mob visibility
    if !can_heal {
        return RunState::PlayerTurn;
    }

    let viewshed = viewshed_storage.get(*player_entity).unwrap();
    for tile in viewshed.visible_tiles.iter() {
        let idx = map.index_from_xy(tile.x as u16, tile.y as u16);
        for entity_id in map.tile_content[idx].iter() {
            if monsters.get(*entity_id).is_some() {
                can_heal = false;
                break;
            }
        }
    }

    if can_heal {
        let mut health_storage = ecs.write_storage::<CombatStats>();
        let player_hp = health_storage.get_mut(*player_entity).unwrap();
        player_hp.heal(1);
    }

    RunState::PlayerTurn
}
