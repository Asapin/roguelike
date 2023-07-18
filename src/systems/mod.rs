use specs::{RunNow, World, WorldExt};

use self::{
    damage_system::DamageSystem, inventory_system::ItemCollectionSystem,
    map_indexing_system::MapIndexingSystem, melee_combat_system::MeleeCombatSystem,
    monster_ai_system::MonsterAI, visibility_system::VisibilitySystem,
};

pub mod damage_system;
pub mod inventory_system;
pub mod map_indexing_system;
pub mod melee_combat_system;
pub mod monster_ai_system;
pub mod visibility_system;

pub struct Systems {
    map_indexing: MapIndexingSystem,
    visibility: VisibilitySystem,
    monster_ai: MonsterAI,
    melee_combat: MeleeCombatSystem,
    damage_system: DamageSystem,
    item_collection: ItemCollectionSystem,
}

impl Systems {
    pub fn new() -> Self {
        Systems {
            map_indexing: MapIndexingSystem {},
            visibility: VisibilitySystem {},
            monster_ai: MonsterAI {},
            melee_combat: MeleeCombatSystem {},
            damage_system: DamageSystem {},
            item_collection: ItemCollectionSystem {},
        }
    }

    pub fn run(&mut self, ecs: &mut World) {
        self.visibility.run_now(&ecs);
        self.monster_ai.run_now(&ecs);
        self.map_indexing.run_now(&ecs);
        self.melee_combat.run_now(&ecs);
        self.damage_system.run_now(&ecs);
        self.item_collection.run_now(&ecs);
        ecs.maintain();
    }
}
