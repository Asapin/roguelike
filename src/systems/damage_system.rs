use rltk::console;
use specs::prelude::*;

use crate::{
    components::{CombatStats, Player, SufferDamage},
    state::RunState,
};

pub struct DamageSystem {}

impl<'a> System<'a> for DamageSystem {
    type SystemData = (
        WriteStorage<'a, CombatStats>,
        WriteStorage<'a, SufferDamage>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut stats, mut damage) = data;

        for (mut stats, damage) in (&mut stats, &damage).join() {
            let total_damage = damage.amount.iter().sum::<u32>();
            stats.damage(total_damage);
        }

        damage.clear();
    }
}

pub fn delete_the_dead(ecs: &mut World) -> Option<RunState> {
    let mut player_died = false;
    let mut dead: Vec<Entity> = Vec::new();
    {
        let combat_stats = ecs.read_storage::<CombatStats>();
        let players = ecs.read_storage::<Player>();
        let entities = ecs.entities();
        for (entity, stats) in (&entities, &combat_stats).join() {
            if stats.hp < 1 {
                let player = players.get(entity);
                match player {
                    None => dead.push(entity),
                    Some(_) => {
                        console::log("You are dead");
                        player_died = true;
                    }
                }
            }
        }
    }
    for victim in dead {
        ecs.delete_entity(victim).expect("Unable to delete");
    }

    if player_died {
        Some(RunState::Dead)
    } else {
        None
    }
}
