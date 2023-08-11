use rltk::Point;
use specs::{Entities, Entity, Join, ReadExpect, ReadStorage, System, WriteExpect, WriteStorage};

use crate::{
    components::{
        AreaOfEffect, CombatStats, Confusion, Consumable, Equippable, Equipped, InBackpack,
        InflictsDamage, Name, ProvidesHealing, SufferDamage, WantsToUseItem,
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
        ReadStorage<'a, Equippable>,
        WriteStorage<'a, Equipped>,
        WriteStorage<'a, InBackpack>,
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
            equippable,
            mut equipped,
            mut backpack,
        ) = data;

        for (entity, use_item) in (&entities, &wants_use).join() {
            if try_to_equip(
                use_item,
                *player_entity,
                entity,
                &entities,
                &equippable,
                &mut equipped,
                &mut backpack,
                &mut gamelog,
                &names,
            ) {
                // Equipping items shouldn't trigger any other immediate effects
                continue;
            }

            // Targeting
            let targets = targets(use_item, entity, &map, &aoe);
            let mut used_item = false;

            if try_to_heal(
                use_item,
                *player_entity,
                entity,
                &targets,
                &healing,
                &mut combat_stats,
                &mut gamelog,
                &names,
            ) {
                used_item = true;
            }

            if try_to_damage(
                use_item,
                *player_entity,
                entity,
                &targets,
                &mut combat_stats,
                &inflict_damage,
                &mut suffer_damage,
                &mut gamelog,
                &names,
            ) {
                used_item = true;
            }

            let confused_entities = try_to_confuse(
                use_item,
                *player_entity,
                entity,
                &targets,
                &mut confusion,
                &mut combat_stats,
                &mut gamelog,
                &names,
            );

            if !confused_entities.is_empty() {
                used_item = true;
            }

            for (mob, turns) in confused_entities.iter() {
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

fn targets(
    use_item: &WantsToUseItem,
    entity: Entity,
    map: &Map,
    aoe: &ReadStorage<AreaOfEffect>,
) -> Vec<Entity> {
    let target_position = match use_item.target {
        None => return vec![entity],
        Some(target_position) => target_position,
    };

    let mut targets = Vec::new();
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
                p.x > 0 && p.x < map.width as i32 - 1 && p.y > 0 && p.y < map.height as i32
            });
            for tile_idx in blast_tiles.iter() {
                let idx = map.index_from_xy(tile_idx.x as u16, tile_idx.y as u16);
                for mob in map.tile_content[idx].iter() {
                    targets.push(*mob);
                }
            }
        }
    }

    targets
}

fn try_to_heal(
    use_item: &WantsToUseItem,
    player_entity: Entity,
    entity: Entity,
    targets: &[Entity],
    healing: &ReadStorage<ProvidesHealing>,
    combat_stats: &mut WriteStorage<CombatStats>,
    gamelog: &mut GameLog,
    names: &ReadStorage<Name>,
) -> bool {
    let healing_item = match healing.get(use_item.item) {
        None => return false,
        Some(item) => item,
    };

    let mut used_item = false;
    for target in targets.iter() {
        let stats = match combat_stats.get_mut(*target) {
            None => continue,
            Some(stats) => stats,
        };
        let hp_diff = stats.heal(healing_item.amount);
        used_item = true;
        if entity == player_entity {
            gamelog.entries.push(format!(
                "You drink the {}, healing {} hp.",
                names.get(use_item.item).unwrap().name,
                hp_diff
            ));
        }
    }
    used_item
}

fn try_to_damage(
    use_item: &WantsToUseItem,
    player_entity: Entity,
    entity: Entity,
    targets: &[Entity],
    combat_stats: &mut WriteStorage<CombatStats>,
    inflict_damage: &ReadStorage<InflictsDamage>,
    suffer_damage: &mut WriteStorage<SufferDamage>,
    gamelog: &mut GameLog,
    names: &ReadStorage<Name>,
) -> bool {
    let damage_item = match inflict_damage.get(use_item.item) {
        None => return false,
        Some(item) => item,
    };

    let mut used_item = false;
    for target in targets.iter() {
        if combat_stats.get(*target).is_none() {
            continue;
        }

        SufferDamage::new_damage(suffer_damage, *target, damage_item.damage);
        used_item = true;
        if entity == player_entity {
            let item_name = names.get(use_item.item).unwrap();
            if *target == player_entity {
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
    }
    used_item
}

fn try_to_confuse(
    use_item: &WantsToUseItem,
    player_entity: Entity,
    entity: Entity,
    targets: &[Entity],
    confusion: &mut WriteStorage<Confusion>,
    combat_stats: &mut WriteStorage<CombatStats>,
    gamelog: &mut GameLog,
    names: &ReadStorage<Name>,
) -> Vec<(Entity, u8)> {
    let confusion_item = match confusion.get(use_item.item) {
        None => return vec![],
        Some(confusion_item) => confusion_item,
    };

    let mut add_confusion = Vec::new();
    for target in targets.iter() {
        if combat_stats.get(*target).is_none() {
            continue;
        }

        add_confusion.push((*target, confusion_item.turns));
        if entity == player_entity {
            let item_name = names.get(use_item.item).unwrap();
            if *target == player_entity {
                gamelog.entries.push(format!(
                    "You use {} on yourself, confusing yourself for {} turns",
                    item_name.name, confusion_item.turns
                ));
            } else {
                let mob_name = names.get(*target).unwrap();
                gamelog.entries.push(format!(
                    "You use {} on {}, confusing them for {} turns.",
                    item_name.name, mob_name.name, confusion_item.turns
                ));
            }
        }
    }
    add_confusion
}

fn try_to_equip(
    use_item: &WantsToUseItem,
    player_entity: Entity,
    entity: Entity,
    entities: &Entities,
    equips: &ReadStorage<Equippable>,
    equipped: &mut WriteStorage<Equipped>,
    backpack: &mut WriteStorage<InBackpack>,
    gamelog: &mut GameLog,
    names: &ReadStorage<Name>,
) -> bool {
    let equippable_item = match equips.get(use_item.item) {
        None => return false,
        Some(item) => item,
    };

    let target_slot = equippable_item.slot;
    let mut to_unequip: Vec<Entity> = Vec::new();
    for (item_entity, already_equipped, name) in (entities, &*equipped, names).join() {
        if already_equipped.owner == entity && already_equipped.slot == target_slot {
            to_unequip.push(item_entity);
            if entity == player_entity {
                gamelog.entries.push(format!("You unequip {}.", name.name));
            }
        }
    }

    for item in to_unequip {
        equipped.remove(item);
        backpack
            .insert(item, InBackpack { owner: entity })
            .expect("Unable to insert backpack entry");
    }
    equipped
        .insert(
            use_item.item,
            Equipped {
                owner: entity,
                slot: target_slot,
            },
        )
        .expect("Unable to insert equipped component");
    backpack.remove(use_item.item);
    if entity == player_entity {
        gamelog.entries.push(format!(
            "You equip {}.",
            names.get(use_item.item).unwrap().name
        ));
    }

    true
}
