use rltk::RGB;
use specs::World;

use crate::{
    state::{GlobalState, RunState},
    systems::{saveload_system, Systems},
};

#[derive(PartialEq, Clone, Copy)]
pub enum MainMenuSelection {
    NewGame,
    LoadGame,
    Quit,
}

#[derive(PartialEq)]
pub enum MainMenuResult {
    NoSelection { selected: MainMenuSelection },
    Selected { selected: MainMenuSelection },
}

pub fn main_menu(
    ecs: &mut World,
    ctx: &mut rltk::Rltk,
    mut current_selection: MainMenuSelection,
) -> GlobalState {
    let save_exists = saveload_system::does_save_exist();
    print_main_menu(ctx, current_selection, save_exists);
    let selected_menu = select_menu(ctx, current_selection, save_exists);
    match selected_menu {
        MainMenuResult::NoSelection { selected } => GlobalState::MainMenu {
            selected_menu: selected,
        },
        MainMenuResult::Selected { selected } => match selected {
            MainMenuSelection::NewGame => GlobalState::Gameplay {
                phase: RunState::NewGame,
                systems: Systems::new(),
            },
            MainMenuSelection::LoadGame => {
                saveload_system::load_game(ecs);
                saveload_system::delete_save();
                GlobalState::Gameplay {
                    phase: RunState::AwaitingInput,
                    systems: Systems::new(),
                }
            }
            MainMenuSelection::Quit => ::std::process::exit(0),
        },
    }
}

fn print_main_menu(
    ctx: &mut rltk::Rltk,
    mut current_selection: MainMenuSelection,
    save_exists: bool,
) {
    ctx.print_color_centered(
        15,
        RGB::named(rltk::YELLOW),
        RGB::named(rltk::BLACK),
        "Rust Roguelike Tutorial",
    );

    let mut new_game_fg = RGB::named(rltk::WHITE);
    let mut load_game_fg = RGB::named(rltk::WHITE);
    let mut quit_game_fg = RGB::named(rltk::WHITE);

    match current_selection {
        MainMenuSelection::NewGame => new_game_fg = RGB::named(rltk::MAGENTA),
        MainMenuSelection::LoadGame => load_game_fg = RGB::named(rltk::MAGENTA),
        MainMenuSelection::Quit => quit_game_fg = RGB::named(rltk::MAGENTA),
    };

    ctx.print_color_centered(24, new_game_fg, RGB::named(rltk::BLACK), "Begin New Game");
    if save_exists {
        ctx.print_color_centered(25, load_game_fg, RGB::named(rltk::BLACK), "Load Game");
    } else if current_selection == MainMenuSelection::LoadGame {
        current_selection = MainMenuSelection::NewGame;
    }
    ctx.print_color_centered(26, quit_game_fg, RGB::named(rltk::BLACK), "Quit");
}

fn select_menu(
    ctx: &mut rltk::Rltk,
    current_selection: MainMenuSelection,
    save_exists: bool,
) -> MainMenuResult {
    match ctx.key {
        None => MainMenuResult::NoSelection {
            selected: current_selection,
        },
        Some(rltk::VirtualKeyCode::Escape) => MainMenuResult::NoSelection {
            selected: MainMenuSelection::Quit,
        },
        Some(rltk::VirtualKeyCode::Numpad8 | rltk::VirtualKeyCode::W) => {
            let new_selection = match current_selection {
                MainMenuSelection::NewGame => MainMenuSelection::Quit,
                MainMenuSelection::LoadGame => MainMenuSelection::NewGame,
                MainMenuSelection::Quit => {
                    if save_exists {
                        MainMenuSelection::LoadGame
                    } else {
                        MainMenuSelection::NewGame
                    }
                }
            };
            MainMenuResult::NoSelection {
                selected: new_selection,
            }
        }
        Some(rltk::VirtualKeyCode::Numpad2 | rltk::VirtualKeyCode::S) => {
            let new_selection = match current_selection {
                MainMenuSelection::NewGame => {
                    if save_exists {
                        MainMenuSelection::LoadGame
                    } else {
                        MainMenuSelection::Quit
                    }
                }
                MainMenuSelection::LoadGame => MainMenuSelection::Quit,
                MainMenuSelection::Quit => MainMenuSelection::NewGame,
            };
            MainMenuResult::NoSelection {
                selected: new_selection,
            }
        }
        Some(rltk::VirtualKeyCode::Return) => MainMenuResult::Selected {
            selected: current_selection,
        },
        _ => MainMenuResult::NoSelection {
            selected: current_selection,
        },
    }
}
