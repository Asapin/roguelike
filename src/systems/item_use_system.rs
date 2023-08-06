use rltk::Point;
use specs::{Entities, Entity, Join, ReadExpect, ReadStorage, System, WriteExpect, WriteStorage};

use crate::{
    components::{
        AreaOfEffect, CombatStats, Confusion, Consumable, InflictsDamage, Name, ProvidesHealing,
        SufferDamage, WantsToUseItem,
    },
    gamelog::GameLog,
    map::Map,
};

#[derive(Clone, Copy)]
pub struct ItemUseSystem {}

impl<'a> System<'a> for ItemUseSystem {
    type SystemData = (
        ReadExpect<'a, Entity>,
        WriteExpect<'a, GameLog>,
        ReadExpect<'a, Map>,
        Entities<'a>,
        WriteStorage<'a, WantsToUseItem>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, ProvidesHealing>,
        WriteStorage<'a, CombatStats>,
        ReadStorage<'a, Consumable>,
        ReadStorage<'a, InflictsDamage>,
        WriteStorage<'a, SufferDamage>,
        ReadStorage<'a, AreaOfEffect>,
        WriteStorage<'a, Confusion>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            player_entity,
            mut gamelog,
            map,
            entities,
            mut wants_use,
            names,
            healing,
            mut combat_stats,
            consumables,
            inflict_damage,
            mut suffer_damage,
            aoe,
            mut confusion,
        ) = data;

        for (entity, use_item) in (&entities, &wants_use).join() {
            // Targeting
            let mut targets = Vec::new();
            match use_item.target {
                None => targets.push(*player_entity),
                Some(target_position) => {
                    let area_effect = aoe.get(use_item.item);
                    match area_effect {
                        None => {
                            // Targeted single tile
                            let idx = map.index_from_xy(target_position.x, target_position.y);
                            for mob in map.tile_content[idx].iter() {
                                targets.push(*mob);
                            }
                        }
                        Some(area) => {
                            // AoE
                            let mut blast_tiles = rltk::field_of_view(
                                Point::new(target_position.x, target_position.y),
                                area.radius as i32,
                                &*map,
                            );
                            blast_tiles.retain(|p| {
                                p.x > 0
                                    && p.x < map.width as i32 - 1
                                    && p.y > 0
                                    && p.y < map.height as i32
                            });
                            for tile_idx in blast_tiles.iter() {
                                let idx = map.index_from_xy(tile_idx.x as u16, tile_idx.y as u16);
                                for mob in map.tile_content[idx].iter() {
                                    targets.push(*mob);
                                }
                            }
                        }
                    }
                }
            }

            let mut used_item = false;

            let healing_item = healing.get(use_item.item);
            if let Some(healing_item) = healing_item {
                for target in targets.iter() {
                    let stats = combat_stats.get_mut(*target);
                    if let Some(stats) = stats {
                        let hp_diff = stats.heal(healing_item.amount);
                        if entity == *player_entity {
                            gamelog.entries.push(format!(
                                "You drink the {}, healing {} hp.",
                                names.get(use_item.item).unwrap().name,
                                hp_diff
                            ));
                        }
                        used_item = true;
                    }
                }
            }

            let damage_item = inflict_damage.get(use_item.item);
            if let Some(damage_item) = damage_item {
                for target in targets.iter() {
                    let stats = combat_stats.get(*target);
                    if stats.is_some() {
                        SufferDamage::new_damage(&mut suffer_damage, *target, damage_item.damage);
                        if entity == *player_entity {
                            let item_name = names.get(use_item.item).unwrap();
                            if *target == *player_entity {
                                gamelog.entries.push(format!(
                                    "You use {} on yourself, inflicting {} hp.",
                                    item_name.name, damage_item.damage
                                ));
                            } else {
                                let mob_name = names.get(*target).unwrap();
                                gamelog.entries.push(format!(
                                    "You use {} on {}, inflicting {} hp.",
                                    item_name.name, mob_name.name, damage_item.damage
                                ));
                            }
                        }
                        used_item = true;
                    }
                }
            }

            let mut add_confusion = Vec::new();
            {
                let causes_confusion = confusion.get(use_item.item);
                if let Some(confusion) = causes_confusion {
                    for target in targets.iter() {
                        let stats = combat_stats.get(*target);
                        if stats.is_some() {
                            add_confusion.push((*target, confusion.turns));
                            if entity == *player_entity {
                                let item_name = names.get(use_item.item).unwrap();
                                if *target == *player_entity {
                                    gamelog.entries.push(format!(
                                        "You use {} on yourself, confusing yourself for {} turns",
                                        item_name.name, confusion.turns
                                    ));
                                } else {
                                    let mob_name = names.get(*target).unwrap();
                                    gamelog.entries.push(format!(
                                        "You use {} on {}, confusing them for {} turns.",
                                        item_name.name, mob_name.name, confusion.turns
                                    ));
                                }
                            }
                            used_item = true;
                        }
                    }
                }
            }

            for (mob, turns) in add_confusion.iter() {
                confusion
                    .insert(*mob, Confusion { turns: *turns })
                    .expect("Unable to insert status");
            }

            let consumable = consumables.get(use_item.item);
            if let (Some(_), true) = (consumable, used_item) {
                entities.delete(use_item.item).expect("Delete failed");
            }
        }

        wants_use.clear();
    }
}
