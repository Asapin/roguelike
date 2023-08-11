use specs::prelude::*;

use crate::{
    components::{
        CombatStats, DefenseBonus, Equipped, MeleePowerBonus, Name, SufferDamage, WantsToMelee,
    },
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
        ReadStorage<'a, MeleePowerBonus>,
        ReadStorage<'a, DefenseBonus>,
        ReadStorage<'a, Equipped>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities,
            mut wants_melee,
            names,
            combat_stats,
            mut inflict_damage,
            mut gamelog,
            melee_bonus,
            defense_bonus,
            equipped,
        ) = data;

        for (entity, wants_melee, name, stats) in
            (&entities, &wants_melee, &names, &combat_stats).join()
        {
            if stats.hp == 0 {
                continue;
            }

            let target_stats = combat_stats.get(wants_melee.target).unwrap();
            if target_stats.hp == 0 {
                continue;
            }

            let mut offensive_bonus = 0;
            for (_item_entity, power_bonus, equipped_by) in
                (&entities, &melee_bonus, &equipped).join()
            {
                if equipped_by.owner == entity {
                    offensive_bonus += power_bonus.power;
                }
            }

            let target_name = names.get(wants_melee.target).unwrap();
            let mut target_defense_bonus = 0;
            for (_item_entity, defense_bonus, equipped_by) in
                (&entities, &defense_bonus, &equipped).join()
            {
                if equipped_by.owner == wants_melee.target {
                    target_defense_bonus += defense_bonus.defense;
                }
            }

            let entity_power = stats.power + offensive_bonus;
            let target_defense = target_stats.defense + target_defense_bonus;
            let damage = i32::max(0, entity_power - target_defense) as u32;
            if damage == 0 {
                let message = format!("{} is unable to hurt {}", &name.name, &target_name.name);
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

        wants_melee.clear();
    }
}
