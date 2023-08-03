use specs::prelude::*;

use crate::{
    components::{CombatStats, Name, SufferDamage, WantsToMelee},
    gamelog::GameLog,
};

#[derive(Clone, Copy)]
pub struct MeleeCombatSystem {}

impl<'a> System<'a> for MeleeCombatSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, WantsToMelee>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, CombatStats>,
        WriteStorage<'a, SufferDamage>,
        WriteExpect<'a, GameLog>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut wants_melee, names, combat_stats, mut inflict_damage, mut gamelog) =
            data;

        for (_entity, wants_melee, name, stats) in
            (&entities, &wants_melee, &names, &combat_stats).join()
        {
            if stats.hp > 0 {
                let target_stats = combat_stats.get(wants_melee.target).unwrap();
                if target_stats.hp > 0 {
                    let target_name = names.get(wants_melee.target).unwrap();

                    let damage = i32::max(0, stats.power - target_stats.defense) as u32;
                    if damage == 0 {
                        let message =
                            format!("{} is unable to hurt {}", &name.name, &target_name.name);
                        gamelog.entries.push(message);
                    } else {
                        let message = format!(
                            "{} hits {} for {} hp",
                            &name.name, &target_name.name, damage
                        );
                        gamelog.entries.push(message);
                        SufferDamage::new_damage(&mut inflict_damage, wants_melee.target, damage);
                    }
                }
            }
        }

        wants_melee.clear();
    }
}
