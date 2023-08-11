use rltk::{GameState, Rltk};
use specs::prelude::*;

use crate::{
    game_loop,
    main_menu::{self, MainMenuSelection},
    menu::pause_menu::PauseMenuSelection,
    systems::Systems,
};

#[derive(PartialEq, Clone, Copy)]
pub enum RunState {
    AwaitingInput,
    PreRun,
    PlayerTurn,
    MonsterTurn,
    ShowInventory,
    ShowDropItem,
    ShowTargeting { range: u16, item: Entity },
    ShowUnequipItem,
    PauseMenu { selected_menu: PauseMenuSelection },
    Dead,
    NextLevel,
    NewGame,
}

#[derive(Clone, Copy)]
pub enum GlobalState {
    MainMenu { selected_menu: MainMenuSelection },
    Gameplay { phase: RunState, systems: Systems },
}

pub struct State {
    pub ecs: World,
}
impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        let global_state = *self.ecs.fetch::<GlobalState>();

        ctx.cls();
        let next_global_state = match global_state {
            GlobalState::MainMenu { selected_menu } => {
                main_menu::main_menu(&mut self.ecs, ctx, selected_menu)
            }
            GlobalState::Gameplay { phase, systems } => {
                game_loop::next_iteration(&mut self.ecs, ctx, phase, systems)
            }
        };

        {
            let mut run_writer = self.ecs.write_resource::<GlobalState>();
            *run_writer = next_global_state;
        }
    }
}
