use specs::{RunNow, World, WorldExt};

use self::{
    damage_system::{delete_the_dead, DamageSystem},
    hunger_system::HungerSystem,
    item_collection::ItemCollectionSystem,
    item_drop_system::ItemDropSystem,
    item_unequip_system::ItemUnequipSystem,
    item_use_system::ItemUseSystem,
    lifetime_system::remove_expired_entities,
    map_indexing_system::MapIndexingSystem,
    melee_combat_system::MeleeCombatSystem,
    monster_ai_system::MonsterAI,
    particle_system::ParticleSpawnSystem,
    visibility_system::VisibilitySystem,
};

pub mod damage_system;
pub mod hunger_system;
pub mod item_collection;
pub mod item_drop_system;
pub mod item_unequip_system;
pub mod item_use_system;
pub mod lifetime_system;
pub mod map_indexing_system;
pub mod melee_combat_system;
pub mod monster_ai_system;
pub mod particle_system;
pub mod saveload_system;
pub mod visibility_system;

#[derive(Clone, Copy)]
pub struct Systems {
    map_indexing: MapIndexingSystem,
    visibility: VisibilitySystem,
    monster_ai: MonsterAI,
    melee_combat: MeleeCombatSystem,
    damage_system: DamageSystem,
    item_collection: ItemCollectionSystem,
    item_use: ItemUseSystem,
    item_drop: ItemDropSystem,
    item_unequip: ItemUnequipSystem,
    particle_spawn: ParticleSpawnSystem,
    hunger: HungerSystem,
}

impl Systems {
    pub fn new() -> Self {
        Self {
            map_indexing: MapIndexingSystem {},
            visibility: VisibilitySystem {},
            monster_ai: MonsterAI {},
            melee_combat: MeleeCombatSystem {},
            damage_system: DamageSystem {},
            item_collection: ItemCollectionSystem {},
            item_use: ItemUseSystem {},
            item_drop: ItemDropSystem {},
            item_unequip: ItemUnequipSystem {},
            particle_spawn: ParticleSpawnSystem {},
            hunger: HungerSystem,
        }
    }

    pub fn run(&mut self, ecs: &mut World, ctx: &rltk::Rltk) {
        self.item_use.run_now(ecs);
        self.item_unequip.run_now(ecs);
        self.item_drop.run_now(ecs);
        self.visibility.run_now(ecs);
        self.monster_ai.run_now(ecs);
        self.map_indexing.run_now(ecs);
        self.melee_combat.run_now(ecs);
        self.damage_system.run_now(ecs);
        self.item_collection.run_now(ecs);
        self.hunger.run_now(ecs);
        self.particle_spawn.run_now(ecs);
        delete_the_dead(ecs);
        remove_expired_entities(ecs, ctx);
        ecs.maintain();
    }

    pub fn run_during_pause(&mut self, ecs: &mut World, ctx: &rltk::Rltk) {
        self.item_use.run_now(ecs);
        self.item_unequip.run_now(ecs);
        self.item_drop.run_now(ecs);
        self.item_collection.run_now(ecs);
        self.particle_spawn.run_now(ecs);
        remove_expired_entities(ecs, ctx);
        ecs.maintain();
    }
}
