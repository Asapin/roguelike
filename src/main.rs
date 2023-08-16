use specs::{prelude::*, saveload::SimpleMarkerAllocator};

use crate::{
    components::SerializeMe,
    menu::main_menu,
    state::{GlobalState, State},
};

mod components;
mod game_loop;
mod gamelog;
mod gui;
mod level;
mod map;
mod menu;
mod player;
mod rect;
mod spawn;
mod state;
mod systems;

fn main() -> rltk::BError {
    use rltk::RltkBuilder;
    let mut context = RltkBuilder::simple80x50()
        .with_title("Roguelike tutorial")
        .build()?;
    context.with_post_scanlines(true);

    let mut gs = State { ecs: World::new() };
    components::register_components(&mut gs.ecs);
    gs.ecs.insert(SimpleMarkerAllocator::<SerializeMe>::new());

    let rng = rltk::RandomNumberGenerator::new();
    gs.ecs.insert(rng);

    gs.ecs.insert(GlobalState::MainMenu {
        selected_menu: main_menu::MainMenuSelection::NewGame,
    });
    rltk::main_loop(context, gs)
}
