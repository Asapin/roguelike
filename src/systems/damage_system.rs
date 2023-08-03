use specs::prelude::*;

use crate::components::{CombatStats, Player, SufferDamage};

#[derive(Clone, Copy)]
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

pub fn delete_the_dead(ecs: &mut World) {
    let mut dead: Vec<Entity> = Vec::new();
    {
        let combat_stats = ecs.read_storage::<CombatStats>();
        let players = ecs.read_storage::<Player>();
        let entities = ecs.entities();
        for (entity, stats) in (&entities, &combat_stats).join() {
            if stats.hp < 1 {
                let player = players.get(entity);
                if player.is_none() {
                    dead.push(entity);
                }
            }
        }
    }
    for victim in dead {
        ecs.delete_entity(victim).expect("Unable to delete");
    }
}
