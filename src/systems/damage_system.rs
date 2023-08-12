use specs::prelude::*;

use crate::{components::{CombatStats, Player, SufferDamage, Position}, map::Map};

#[derive(Clone, Copy)]
pub struct DamageSystem {}

impl<'a> System<'a> for DamageSystem {
    type SystemData = (
        WriteStorage<'a, CombatStats>,
        WriteStorage<'a, SufferDamage>,
        ReadStorage<'a, Position>,
        WriteExpect<'a, Map>,
        Entities<'a>
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut stats, mut damage, positions, mut map, entities) = data;

        for (entity, mut stats, damage) in (&entities, &mut stats, &damage).join() {
            let total_damage = damage.amount.iter().sum::<u32>();
            stats.damage(total_damage);
            if let Some(pos) = positions.get(entity) {
                let idx = map.index_from_xy(pos.x, pos.y);
                map.bloodstains.insert(idx);
            }
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
            if stats.hp == 0 {
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
