use rltk::RGB;
use specs::World;

use crate::{
    state::RunState,
    systems::saveload_system::{self, save_game},
};

#[derive(PartialEq, Clone, Copy)]
pub enum PauseMenuSelection {
    Restart,
    SaveGame,
    LoadGame,
    Quit,
}

#[derive(PartialEq)]
pub enum PauseMenuResult {
    NoSelection { selected: PauseMenuSelection },
    Selected { selected: PauseMenuSelection },
    Cancel,
}

pub fn pause_menu(
    ecs: &mut World,
    ctx: &mut rltk::Rltk,
    mut current_selection: PauseMenuSelection,
) -> RunState {
    let save_exists = saveload_system::does_save_exist();
    print_pause_menu(ctx, current_selection, save_exists);
    let selected_menu = select_menu(ctx, current_selection, save_exists);
    match selected_menu {
        PauseMenuResult::NoSelection { selected } => RunState::PauseMenu {
            selected_menu: selected,
        },
        PauseMenuResult::Selected { selected } => match selected {
            PauseMenuSelection::Restart => RunState::NewGame,
            PauseMenuSelection::SaveGame => {
                save_game(ecs);
                RunState::AwaitingInput
            }
            PauseMenuSelection::LoadGame => {
                saveload_system::load_game(ecs);
                saveload_system::delete_save();
                RunState::AwaitingInput
            }
            PauseMenuSelection::Quit => {
                save_game(ecs);
                ::std::process::exit(0);
            }
        },
        PauseMenuResult::Cancel => RunState::AwaitingInput,
    }
}

fn print_pause_menu(
    ctx: &mut rltk::Rltk,
    mut current_selection: PauseMenuSelection,
    save_exists: bool,
) {
    ctx.print_color_centered(
        15,
        RGB::named(rltk::YELLOW),
        RGB::named(rltk::BLACK),
        "Pause",
    );

    let mut restart_game_fg = RGB::named(rltk::WHITE);
    let mut save_game_fg = RGB::named(rltk::WHITE);
    let mut load_game_fg = RGB::named(rltk::WHITE);
    let mut quit_game_fg = RGB::named(rltk::WHITE);

    match current_selection {
        PauseMenuSelection::Restart => restart_game_fg = RGB::named(rltk::MAGENTA),
        PauseMenuSelection::SaveGame => save_game_fg = RGB::named(rltk::MAGENTA),
        PauseMenuSelection::LoadGame => load_game_fg = RGB::named(rltk::MAGENTA),
        PauseMenuSelection::Quit => quit_game_fg = RGB::named(rltk::MAGENTA),
    };

    ctx.print_color_centered(24, restart_game_fg, RGB::named(rltk::BLACK), "Restart Game");
    ctx.print_color_centered(25, save_game_fg, RGB::named(rltk::BLACK), "Save Game");
    if save_exists {
        ctx.print_color_centered(26, load_game_fg, RGB::named(rltk::BLACK), "Load Game");
    } else if current_selection == PauseMenuSelection::LoadGame {
        current_selection = PauseMenuSelection::SaveGame;
    }
    ctx.print_color_centered(27, quit_game_fg, RGB::named(rltk::BLACK), "Quit");
}

fn select_menu(
    ctx: &mut rltk::Rltk,
    current_selection: PauseMenuSelection,
    save_exists: bool,
) -> PauseMenuResult {
    match ctx.key {
        None => PauseMenuResult::NoSelection {
            selected: current_selection,
        },
        Some(rltk::VirtualKeyCode::Escape) => PauseMenuResult::Cancel,
        Some(rltk::VirtualKeyCode::Numpad8 | rltk::VirtualKeyCode::W) => {
            let new_selection = match current_selection {
                PauseMenuSelection::Restart => PauseMenuSelection::Quit,
                PauseMenuSelection::Quit => {
                    if save_exists {
                        PauseMenuSelection::LoadGame
                    } else {
                        PauseMenuSelection::SaveGame
                    }
                }
                PauseMenuSelection::LoadGame => PauseMenuSelection::SaveGame,
                PauseMenuSelection::SaveGame => PauseMenuSelection::Restart,
            };
            PauseMenuResult::NoSelection {
                selected: new_selection,
            }
        }
        Some(rltk::VirtualKeyCode::Numpad2 | rltk::VirtualKeyCode::S) => {
            let new_selection = match current_selection {
                PauseMenuSelection::Restart => PauseMenuSelection::SaveGame,
                PauseMenuSelection::SaveGame => {
                    if save_exists {
                        PauseMenuSelection::LoadGame
                    } else {
                        PauseMenuSelection::Quit
                    }
                }
                PauseMenuSelection::LoadGame => PauseMenuSelection::Quit,
                PauseMenuSelection::Quit => PauseMenuSelection::Restart,
            };
            PauseMenuResult::NoSelection {
                selected: new_selection,
            }
        }
        Some(rltk::VirtualKeyCode::Return) => PauseMenuResult::Selected {
            selected: current_selection,
        },
        _ => PauseMenuResult::NoSelection {
            selected: current_selection,
        },
    }
}
