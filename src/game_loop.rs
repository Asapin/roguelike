use specs::{Entity, Join, World, WorldExt};

use crate::{
    components::CombatStats,
    gui, level,
    menu::{game_over, inventory_menu, main_menu::MainMenuSelection, pause_menu, target_menu},
    player,
    state::{GlobalState, RunState},
    systems::Systems,
};

pub fn next_iteration(
    ecs: &mut World,
    ctx: &mut rltk::Rltk,
    phase: RunState,
    mut systems: Systems,
) -> GlobalState {
    match phase {
        RunState::NewGame => {}
        _ => {
            gui::draw(ecs, ctx);
        }
    }

    let next_phase = match phase {
        RunState::PreRun => {
            systems.run(ecs);
            RunState::AwaitingInput
        }
        RunState::AwaitingInput => player::player_input(ecs, ctx),
        RunState::PlayerTurn => {
            systems.run(ecs);
            RunState::MonsterTurn
        }
        RunState::MonsterTurn => {
            systems.run(ecs);
            if player_is_dead(ecs) {
                RunState::Dead
            } else {
                RunState::AwaitingInput
            }
        }
        RunState::ShowInventory => inventory_menu::inventory(ecs, ctx),
        RunState::ShowDropItem => inventory_menu::drop_item_menu(ecs, ctx),
        RunState::ShowUnequipItem => inventory_menu::unequip_menu(ecs, ctx),
        RunState::ShowTargeting { range, item } => target_menu::target_menu(ecs, ctx, range, item),
        RunState::Dead => {
            let result = game_over::game_over(ctx);
            match result {
                game_over::GameOverResult::NoSelection => RunState::Dead,
                game_over::GameOverResult::QuitToMenu => {
                    game_over_cleanup(ecs);
                    return GlobalState::MainMenu {
                        selected_menu: MainMenuSelection::NewGame,
                    };
                }
            }
        }
        RunState::NextLevel => {
            level::next_level(ecs);
            RunState::PreRun
        }
        RunState::NewGame => {
            level::new_game(ecs);
            RunState::PreRun
        }
        RunState::PauseMenu { selected_menu } => pause_menu::pause_menu(ecs, ctx, selected_menu),
    };

    ecs.insert(next_phase);

    GlobalState::Gameplay {
        phase: next_phase,
        systems,
    }
}

fn player_is_dead(ecs: &mut World) -> bool {
    let player_entity = ecs.fetch::<Entity>();
    let combat_stats_storage = ecs.read_component::<CombatStats>();
    let player_stats = combat_stats_storage.get(*player_entity);
    if let Some(stats) = player_stats {
        stats.hp == 0
    } else {
        false
    }
}

fn game_over_cleanup(ecs: &mut World) {
    let mut to_delete = Vec::new();
    for e in ecs.entities().join() {
        to_delete.push(e);
    }
    for del in to_delete {
        ecs.delete_entity(del).expect("Deletion failed");
    }
}
