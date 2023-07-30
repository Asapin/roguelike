use specs::{prelude::*, saveload::SimpleMarkerAllocator};

use crate::{
    components::SerializeMe,
    gamelog::GameLog,
    state::{RunState, State},
    systems::Systems,
};

mod components;
mod gamelog;
mod gui;
mod map;
mod menu;
mod player;
mod rect;
mod spawner;
mod state;
mod systems;

fn main() -> rltk::BError {
    use rltk::RltkBuilder;
    let mut context = RltkBuilder::simple80x50()
        .with_title("Roguelike tutorial")
        .build()?;
    context.with_post_scanlines(true);

    let mut gs = State {
        ecs: World::new(),
        systems: Systems::new(),
    };
    components::register_components(&mut gs.ecs);
    gs.ecs.insert(SimpleMarkerAllocator::<SerializeMe>::new());

    let rng = rltk::RandomNumberGenerator::new();
    gs.ecs.insert(rng);

    gs.ecs.insert(GameLog { entries: vec![] });
    gs.ecs.insert(RunState::MainMenu {
        menu_selection: menu::MainMenuSelection::NewGame,
    });
    rltk::main_loop(context, gs)
}
