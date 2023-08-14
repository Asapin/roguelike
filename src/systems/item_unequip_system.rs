use specs::{Entities, Join, System, WriteStorage};

use crate::components::{Equipped, InBackpack, WantsToUnequipItem};

#[derive(Clone, Copy)]
pub struct ItemUnequipSystem;

impl<'a> System<'a> for ItemUnequipSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, WantsToUnequipItem>,
        WriteStorage<'a, Equipped>,
        WriteStorage<'a, InBackpack>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut wants_unequip, mut equipped, mut backpack) = data;

        for (entity, to_remove) in (&entities, &wants_unequip).join() {
            equipped.remove(to_remove.item);
            backpack
                .insert(to_remove.item, InBackpack { owner: entity })
                .expect("Unable to insert backpack");
        }

        wants_unequip.clear();
    }
}
