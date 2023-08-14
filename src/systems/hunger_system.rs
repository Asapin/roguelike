use specs::{Entities, Entity, Join, ReadExpect, System, WriteExpect, WriteStorage};

use crate::{
    components::{HungerClock, HungerState, SufferDamage},
    gamelog::GameLog,
    state::RunState,
};

#[derive(Clone, Copy)]
pub struct HungerSystem;

impl<'a> System<'a> for HungerSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, HungerClock>,
        ReadExpect<'a, Entity>,
        ReadExpect<'a, RunState>,
        WriteStorage<'a, SufferDamage>,
        WriteExpect<'a, GameLog>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut hunger_clock, player, run_state, mut inflict_damage, mut gamelog) = data;

        for (entity, mut clock) in (&entities, &mut hunger_clock).join() {
            let proceed = match *run_state {
                RunState::PlayerTurn => entity == *player,
                RunState::MonsterTurn => entity != *player,
                _ => false,
            };
            if !proceed {
                return;
            }

            if clock.duration > 0 {
                clock.duration -= 1;
            }
            if clock.duration > 0 {
                return;
            }

            match clock.state {
                HungerState::WellFed => {
                    clock.state = HungerState::Normal;
                    clock.duration = 200;
                    if entity == *player {
                        gamelog
                            .entries
                            .push("You are no longer well fed.".to_string());
                    }
                }
                HungerState::Normal => {
                    clock.state = HungerState::Hungry;
                    clock.duration = 200;
                    if entity == *player {
                        gamelog.entries.push("You are hungry.".to_string());
                    }
                }
                HungerState::Hungry => {
                    clock.state = HungerState::Starving;
                    clock.duration = 200;
                    if entity == *player {
                        gamelog.entries.push("You are starving!".to_string());
                    }
                }
                HungerState::Starving => {
                    if entity == *player {
                        gamelog.entries.push(
                            "Your hunger pangs are getting painful! You suffer 1 hp damage."
                                .to_string(),
                        );
                    }
                    SufferDamage::new_damage(&mut inflict_damage, entity, 1);
                }
            }
        }
    }
}
