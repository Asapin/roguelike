use rltk::{GameState, Point, RandomNumberGenerator, Rltk};
use specs::prelude::*;

use crate::{
    components::{
        CombatStats, InBackpack, Player, Position, Ranged, Viewshed, WantsToDropItem,
        WantsToUseItem,
    },
    gamelog::GameLog,
    gui::{self, ItemMenuResult, TargetSelectResult},
    map::Map,
    menu::{main_menu, MainMenuResult, MainMenuSelection},
    player, spawner,
    systems::{
        damage_system::delete_the_dead,
        saveload_system::{self, save_game},
        Systems,
    },
};

pub const MAP_WIDTH: u16 = 80;
pub const MAP_HEIGHT: u16 = 43;
const ROOM_COUNT: u8 = 30;
const MIN_ROOM_SIZE: u8 = 6;
const MAX_ROOM_SIZE: u8 = 10;
const MAX_MONSTERS: i32 = 4;
const MAX_ITEMS: i32 = 2;

#[derive(PartialEq, Clone, Copy)]
pub enum RunState {
    AwaitingInput,
    PreRun,
    PlayerTurn,
    MonsterTurn,
    ShowInventory,
    ShowDropItem,
    ShowTargeting { range: u16, item: Entity },
    MainMenu { menu_selection: MainMenuSelection },
    SaveGame,
    Dead,
    NextLevel,
}

pub struct State {
    pub ecs: World,
    pub systems: Systems,
}
impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        let mut new_runstate = {
            let runstate = self.ecs.fetch::<RunState>();
            *runstate
        };

        ctx.cls();

        match new_runstate {
            RunState::MainMenu { .. } | RunState::NextLevel => {}
            RunState::Dead => gui::draw(&self.ecs, ctx),
            _ => {
                gui::draw(&self.ecs, ctx);
                if let Some(state) = delete_the_dead(&mut self.ecs) {
                    new_runstate = state;
                }
            }
        }

        match new_runstate {
            RunState::PreRun => {
                self.run_systems();
                new_runstate = RunState::AwaitingInput;
            }
            RunState::AwaitingInput => {
                new_runstate = player::player_input(&mut self.ecs, ctx);
            }
            RunState::PlayerTurn => {
                self.run_systems();
                new_runstate = RunState::MonsterTurn;
            }
            RunState::MonsterTurn => {
                self.run_systems();
                new_runstate = RunState::AwaitingInput;
            }
            RunState::ShowInventory => {
                let result = gui::inventory(self, ctx);
                match result {
                    ItemMenuResult::Cancel => new_runstate = RunState::AwaitingInput,
                    ItemMenuResult::NoResponse => {}
                    ItemMenuResult::Selected(item) => {
                        let ranged_storage = self.ecs.read_storage::<Ranged>();
                        let is_item_ranged = ranged_storage.get(item);
                        if let Some(ranged) = is_item_ranged {
                            new_runstate = RunState::ShowTargeting {
                                range: ranged.range,
                                item,
                            }
                        } else {
                            let mut intent = self.ecs.write_storage::<WantsToUseItem>();
                            let player_entity = self.ecs.fetch::<Entity>();
                            intent
                                .insert(*player_entity, WantsToUseItem { item, target: None })
                                .expect("Unable to insert intent");
                            new_runstate = RunState::PlayerTurn;
                        }
                    }
                }
            }
            RunState::ShowDropItem => {
                let result = gui::drop_item_menu(self, ctx);
                match result {
                    ItemMenuResult::Cancel => new_runstate = RunState::AwaitingInput,
                    ItemMenuResult::NoResponse => {}
                    ItemMenuResult::Selected(item) => {
                        let mut intent = self.ecs.write_storage::<WantsToDropItem>();
                        let player_entity = self.ecs.fetch::<Entity>();
                        let positions = self.ecs.read_storage::<Position>();
                        let player_position = positions.get(*player_entity).unwrap();
                        intent
                            .insert(
                                *player_entity,
                                WantsToDropItem {
                                    item,
                                    position: *player_position,
                                },
                            )
                            .expect("Unable to insert intent");
                        new_runstate = RunState::PlayerTurn;
                    }
                }
            }
            RunState::ShowTargeting { range, item } => {
                let target = gui::ranged_target(self, ctx, range);
                match target {
                    TargetSelectResult::Cancel => new_runstate = RunState::AwaitingInput,
                    TargetSelectResult::NoResponse => {}
                    TargetSelectResult::Selected(position) => {
                        let mut intent = self.ecs.write_storage::<WantsToUseItem>();
                        let player = self.ecs.fetch::<Entity>();
                        intent
                            .insert(
                                *player,
                                WantsToUseItem {
                                    item,
                                    target: Some(position),
                                },
                            )
                            .expect("Unable to insert intent");
                        new_runstate = RunState::PlayerTurn;
                    }
                }
            }
            RunState::Dead => {}
            RunState::MainMenu { menu_selection } => {
                let result = main_menu(ctx, menu_selection);
                match result {
                    MainMenuResult::NoSelection { selected } => {
                        new_runstate = RunState::MainMenu {
                            menu_selection: selected,
                        }
                    }
                    MainMenuResult::Selected { selected } => match selected {
                        MainMenuSelection::NewGame => new_runstate = RunState::NextLevel,
                        MainMenuSelection::LoadGame => {
                            saveload_system::load_game(&mut self.ecs);
                            saveload_system::delete_save();
                            new_runstate = RunState::AwaitingInput;
                        }
                        MainMenuSelection::Quit => ::std::process::exit(0),
                    },
                }
            }
            RunState::SaveGame => {
                save_game(&mut self.ecs);
                new_runstate = RunState::MainMenu {
                    menu_selection: MainMenuSelection::LoadGame,
                };
            }
            RunState::NextLevel => {
                self.goto_next_level();
                new_runstate = RunState::PreRun;
            }
        }

        {
            let mut run_writer = self.ecs.write_resource::<RunState>();
            *run_writer = new_runstate;
        }
    }
}

impl State {
    fn run_systems(&mut self) {
        self.systems.run(&mut self.ecs);
    }

    fn goto_next_level(&mut self) {
        let mut first_level = false;
        // Generate new world map
        let worldmap = {
            let optional_map = self.ecs.try_fetch_mut::<Map>();
            let mut rng = self.ecs.write_resource::<RandomNumberGenerator>();
            if let Some(map) = optional_map {
                map.new_map(ROOM_COUNT, MIN_ROOM_SIZE, MAX_ROOM_SIZE, &mut rng)
            } else {
                let new_map = Map::new_map_rooms_and_corridors(
                    MAP_WIDTH,
                    MAP_HEIGHT,
                    80,
                    50,
                    ROOM_COUNT,
                    MIN_ROOM_SIZE,
                    MAX_ROOM_SIZE,
                    &mut rng,
                    1,
                );
                first_level = true;
                new_map
            }
        };

        let (player_x, player_y) = worldmap.rooms[0].center();
        self.ecs.insert(Point::new(player_x, player_y));

        if first_level {
            let player_entity = { spawner::player(&mut self.ecs, player_x, player_y) };
            self.ecs.insert(player_entity);
            let mut gamelog = self.ecs.fetch_mut::<GameLog>();
            gamelog.entries.push("While roaming in the wilds, you stumbled upon a mysterious cave.".to_string());
            gamelog.entries.push("Having no shelter from the harsh weather outside, you decide to explore it.".to_string());
            gamelog.entries.push("But as soon as you enter the cave, the entrance colapses trapping you inside.".to_string());
        } else {
            let mut position_component = self.ecs.write_storage::<Position>();
            let player_entity = self.ecs.fetch::<Entity>();
            let player_position_component = position_component.get_mut(*player_entity);
            if let Some(player_position) = player_position_component {
                player_position.x = player_x;
                player_position.y = player_y;
            }

            // Mark the player's visibility as dirty
            let mut viewshed_components = self.ecs.write_storage::<Viewshed>();
            let viewshed = viewshed_components.get_mut(*player_entity);
            if let Some(viewshed) = viewshed {
                viewshed.dirty = true;
            }

            // Notify the player and give them some health
            let mut gamelog = self.ecs.fetch_mut::<GameLog>();
            gamelog
                .entries
                .push("You descent to the next level, and take a moment to heal.".to_string());
            let mut player_health_store = self.ecs.write_storage::<CombatStats>();
            let player_health = player_health_store.get_mut(*player_entity);
            if let Some(player_health) = player_health {
                player_health.hp = u32::max(player_health.hp, player_health.max_hp / 2);
            }
        }

        // Delete entities that aren't the player or player's equipment
        let to_delete = self.entities_to_remove_on_level_change();
        for target in to_delete {
            self.ecs
                .delete_entity(target)
                .expect("Unable to delete entity");
        }

        // Generate new mobs and items
        for room in worldmap.rooms.iter().skip(1) {
            spawner::spawn_room(&mut self.ecs, &worldmap, &room, MAX_MONSTERS, MAX_ITEMS);
        }

        self.ecs.insert(worldmap);
    }

    fn entities_to_remove_on_level_change(&mut self) -> Vec<Entity> {
        let entities = self.ecs.entities();
        let player = self.ecs.read_storage::<Player>();
        let backpack = self.ecs.read_storage::<InBackpack>();
        let player_entity = self.ecs.fetch::<Entity>();

        let mut to_delete: Vec<Entity> = Vec::new();
        for entity in entities.join() {
            let mut should_delete = true;

            // Don't delete the player
            if player.contains(entity) {
                should_delete = false;
            }

            // Don't delete the player's equipment
            let bp = backpack.get(entity);
            if let Some(bp) = bp {
                if bp.owner == *player_entity {
                    should_delete = false;
                }
            }

            if should_delete {
                to_delete.push(entity);
            }
        }

        to_delete
    }
}
