use rltk::RGB;
use specs::{Entities, Join, ReadExpect, ReadStorage, System, WriteExpect, WriteStorage};

use crate::{
    components::{
        EntityMoved, EntryTrigger, Hidden, InflictsDamage, Name, Position, SingleActivation,
        SufferDamage,
    },
    gamelog::GameLog,
    map::Map,
};

use super::particle_system::ParticleBuilder;

#[derive(Clone, Copy)]
pub struct TriggerSystem {}

impl<'a> System<'a> for TriggerSystem {
    type SystemData = (
        ReadExpect<'a, Map>,
        WriteStorage<'a, EntityMoved>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, EntryTrigger>,
        WriteStorage<'a, Hidden>,
        ReadStorage<'a, Name>,
        Entities<'a>,
        WriteExpect<'a, GameLog>,
        ReadStorage<'a, InflictsDamage>,
        WriteStorage<'a, SufferDamage>,
        WriteExpect<'a, ParticleBuilder>,
        ReadStorage<'a, SingleActivation>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            map,
            mut entity_moved,
            positions,
            entry_triggers,
            mut hidden,
            names,
            entities,
            mut gamelog,
            inflicts_damage,
            mut suffer_damage,
            mut particle_builder,
            single_activation,
        ) = data;

        let mut remove_entities = Vec::new();
        for (entity, mut _entity_moved, pos) in (&entities, &mut entity_moved, &positions).join() {
            let idx = map.index_from_xy(pos.x, pos.y);
            for entity_id in map.tile_content[idx].iter() {
                // No need to check itself
                if entity == *entity_id {
                    continue;
                }

                if let Some(_trigger) = entry_triggers.get(*entity_id) {
                    // Entity is no longer hidden
                    hidden.remove(*entity_id);

                    if let Some(name) = names.get(*entity_id) {
                        gamelog.entries.push(format!("{} triggers!", &name.name));
                    }

                    if let Some(damage) = inflicts_damage.get(*entity_id) {
                        particle_builder.request(
                            pos.x,
                            pos.y,
                            RGB::named(rltk::ORANGE),
                            RGB::named(rltk::BLACK),
                            rltk::to_cp437('‼'),
                            200.0,
                        );
                        SufferDamage::new_damage(&mut suffer_damage, entity, damage.damage);
                    }

                    if let Some(_) = single_activation.get(*entity_id) {
                        remove_entities.push(*entity_id);
                    }
                }
            }
        }

        // Remove all single activation entities
        for trap in remove_entities {
            entities.delete(trap).expect("Unable to delete entity");
        }
        // Remove all entity movement markers
        entity_moved.clear();
    }
}
