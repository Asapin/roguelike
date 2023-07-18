use std::cmp::{max, min};

use rltk::{GameState, Point, Rltk, VirtualKeyCode};
use specs::prelude::*;

use crate::{
    components::{
        CombatStats, Item, Player, Position, Renderable, Viewshed, WantsToMelee, WantsToPickupItem,
    },
    gamelog::GameLog,
    gui,
    map::Map,
    systems::{damage_system, Systems},
};

#[derive(PartialEq, Clone, Copy)]
pub enum RunState {
    AwaitingInput,
    PreRun,
    PlayerTurn,
    MonsterTurn,
}

pub struct State {
    pub ecs: World,
    pub systems: Systems,
}
impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();
        let mut new_runstate;
        {
            let runstate = self.ecs.fetch::<RunState>();
            new_runstate = *runstate;
        }
        match new_runstate {
            RunState::PreRun => {
                self.run_systems();
                new_runstate = RunState::AwaitingInput;
            }
            RunState::AwaitingInput => {
                new_runstate = self.player_input(ctx);
            }
            RunState::PlayerTurn => {
                self.run_systems();
                new_runstate = RunState::MonsterTurn;
            }
            RunState::MonsterTurn => {
                self.run_systems();
                new_runstate = RunState::AwaitingInput;
            }
        }

        {
            let mut run_writer = self.ecs.write_resource::<RunState>();
            *run_writer = new_runstate;
        }
        damage_system::delete_the_dead(&mut self.ecs);

        gui::draw(&self.ecs, ctx);

        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();
        let map = self.ecs.fetch::<Map>();

        for (pos, render) in (&positions, &renderables).join() {
            let idx = map.index_from_xy(pos.x, pos.y);
            if map.visible_tiles[idx] {
                ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
            }
        }
    }
}

impl State {
    fn run_systems(&mut self) {
        self.systems.run(&mut self.ecs);
    }

    fn player_input(&mut self, ctx: &mut Rltk) -> RunState {
        // Player movement
        match ctx.key {
            None => return RunState::AwaitingInput,
            Some(key) => match key {
                VirtualKeyCode::Numpad4 | VirtualKeyCode::A => self.try_move_player(-1, 0),
                VirtualKeyCode::Numpad6 | VirtualKeyCode::D => self.try_move_player(1, 0),
                VirtualKeyCode::Numpad8 | VirtualKeyCode::W => self.try_move_player(0, -1),
                VirtualKeyCode::Numpad2 | VirtualKeyCode::X => self.try_move_player(0, 1),

                // Diagonals
                VirtualKeyCode::Numpad9 | VirtualKeyCode::E => self.try_move_player(1, -1),
                VirtualKeyCode::Numpad7 | VirtualKeyCode::Q => self.try_move_player(-1, -1),
                VirtualKeyCode::Numpad3 | VirtualKeyCode::C => self.try_move_player(1, 1),
                VirtualKeyCode::Numpad1 | VirtualKeyCode::Z => self.try_move_player(-1, 1),

                // Pickup
                VirtualKeyCode::Numpad5 | VirtualKeyCode::S => self.pickup(),
                _ => return RunState::AwaitingInput,
            },
        }
        RunState::PlayerTurn
    }

    fn try_move_player(&mut self, delta_x: i32, delta_y: i32) {
        let mut positions = self.ecs.write_storage::<Position>();
        let mut players = self.ecs.write_storage::<Player>();
        let mut viewsheds = self.ecs.write_storage::<Viewshed>();
        let combat_stats = self.ecs.read_storage::<CombatStats>();
        let map = self.ecs.fetch::<Map>();
        let mut player_pos = self.ecs.write_resource::<Point>();
        let entities = self.ecs.entities();
        let mut wants_to_melee = self.ecs.write_storage::<WantsToMelee>();

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
                player_pos.x = pos.x as i32;
                player_pos.y = pos.y as i32;
            }
        }
    }

    fn pickup(&mut self) {
        let player_pos = self.ecs.fetch::<Point>();
        let player_entity = self.ecs.fetch::<Entity>();
        let entities = self.ecs.entities();
        let items = self.ecs.read_storage::<Item>();
        let positions = self.ecs.read_storage::<Position>();
        let mut gamelog = self.ecs.fetch_mut::<GameLog>();

        let mut target_item: Option<Entity> = None;
        for (item_entity, _, position) in (&entities, &items, &positions).join() {
            if position.x as i32 == player_pos.x && position.y as i32 == player_pos.y {
                target_item = Some(item_entity);
            }
        }

        match target_item {
            None => gamelog
                .entries
                .push("There is nothing here to pickup".to_string()),
            Some(item) => {
                let mut pickup = self.ecs.write_storage::<WantsToPickupItem>();
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
}
