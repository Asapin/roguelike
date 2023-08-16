use rltk::{Point, RGB};
use serde::{Deserialize, Serialize};
use specs::error::NoError;
use specs::prelude::*;
use specs::saveload::{ConvertSaveload, Marker, SimpleMarker};
use specs_derive::*;

use crate::gamelog::GameLog;
use crate::map::map::Map;

#[derive(Component, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Position {
    pub x: u16,
    pub y: u16,
}

impl From<&Position> for Point {
    fn from(value: &Position) -> Self {
        Point {
            x: value.x as i32,
            y: value.y as i32,
        }
    }
}

impl From<&mut Position> for Point {
    fn from(value: &mut Position) -> Self {
        Point {
            x: value.x as i32,
            y: value.y as i32,
        }
    }
}

impl From<Position> for Point {
    fn from(value: Position) -> Self {
        Point {
            x: value.x as i32,
            y: value.y as i32,
        }
    }
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct Renderable {
    pub glyph: rltk::FontCharType,
    pub fg: RGB,
    pub bg: RGB,
    pub render_order: i32,
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct Player {}

#[derive(Component, ConvertSaveload, Clone)]
pub struct Viewshed {
    pub visible_tiles: Vec<rltk::Point>,
    pub range: u16,
    pub dirty: bool,
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct Monster {}

#[derive(Component, ConvertSaveload, Clone)]
pub struct Name {
    pub name: String,
}

#[derive(Component, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BlocksTile {}

#[derive(Component, ConvertSaveload, Clone)]
pub struct CombatStats {
    pub max_hp: u32,
    pub hp: u32,
    pub defense: i32,
    pub power: i32,
}

impl CombatStats {
    pub fn heal(&mut self, amount: u32) -> u32 {
        let prev = self.hp;
        self.hp = u32::min(self.max_hp, self.hp + amount);
        self.hp - prev
    }

    pub fn damage(&mut self, amount: u32) {
        if amount >= self.hp {
            self.hp = 0;
        } else {
            self.hp -= amount;
        }
    }
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct WantsToMelee {
    pub target: Entity,
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct SufferDamage {
    pub amount: Vec<u32>,
}

impl SufferDamage {
    pub fn new_damage(store: &mut WriteStorage<SufferDamage>, victim: Entity, amount: u32) {
        if let Some(suffering) = store.get_mut(victim) {
            suffering.amount.push(amount);
        } else {
            let dmg = SufferDamage {
                amount: vec![amount],
            };
            store.insert(victim, dmg).expect("Unable to insert damage");
        }
    }
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct Item {}

#[derive(Component, ConvertSaveload, Clone)]
pub struct ProvidesHealing {
    pub amount: u32,
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct InBackpack {
    pub owner: Entity,
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct WantsToPickupItem {
    pub collected_by: Entity,
    pub item: Entity,
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct WantsToUseItem {
    pub item: Entity,
    pub target: Option<Position>,
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct WantsToDropItem {
    pub item: Entity,
    pub position: Position,
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct Consumable {}

#[derive(Component, ConvertSaveload, Clone)]
pub struct Ranged {
    pub range: u16,
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct InflictsDamage {
    pub damage: u32,
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct AreaOfEffect {
    pub radius: u16,
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct Confusion {
    pub turns: u8,
}

pub struct SerializeMe {}

// Special component that exists to help serialize the game data
#[derive(Component, ConvertSaveload, Clone)]
pub struct MapSerializationHelper {
    pub map: Map,
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct GameLogSerializationHelper {
    pub gamelog: GameLog,
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum EquipmentSlot {
    Melee,
    Shield,
}

#[derive(Component, ConvertSaveload, Clone, Copy)]
pub struct Equippable {
    pub slot: EquipmentSlot,
}

#[derive(Component, ConvertSaveload, Clone, Copy)]
pub struct Equipped {
    pub owner: Entity,
    pub slot: EquipmentSlot,
}

#[derive(Component, ConvertSaveload, Clone, Copy)]
pub struct MeleePowerBonus {
    pub power: i32,
}

#[derive(Component, ConvertSaveload, Clone, Copy)]
pub struct DefenseBonus {
    pub defense: i32,
}

#[derive(Component, ConvertSaveload, Clone, Copy)]
pub struct WantsToUnequipItem {
    pub item: Entity,
}

#[derive(Component, ConvertSaveload, Clone, Copy)]
pub struct Lifetime {
    pub lifetime_ms: f32,
}

#[derive(Component, Serialize, Deserialize, Clone, Copy)]
pub struct Particle {}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq)]
pub enum HungerState {
    WellFed,
    Normal,
    Hungry,
    Starving,
}

#[derive(Component, ConvertSaveload, Clone, Copy)]
pub struct HungerClock {
    pub state: HungerState,
    pub duration: u32,
}

#[derive(Component, Serialize, Deserialize, Clone, Copy)]
pub struct ProvidesFood {}

#[derive(Component, Serialize, Deserialize, Clone, Copy)]
pub struct Hidden {}

#[derive(Component, Serialize, Deserialize, Clone, Copy)]
pub struct EntryTrigger {}

#[derive(Component, Serialize, Deserialize, Clone, Copy)]
pub struct EntityMoved {}

#[derive(Component, Serialize, Deserialize, Clone, Copy)]
pub struct SingleActivation {}

pub fn register_components(ecs: &mut World) {
    ecs.register::<Position>();
    ecs.register::<Renderable>();
    ecs.register::<Player>();
    ecs.register::<Viewshed>();
    ecs.register::<Monster>();
    ecs.register::<Name>();
    ecs.register::<BlocksTile>();
    ecs.register::<CombatStats>();
    ecs.register::<WantsToMelee>();
    ecs.register::<SufferDamage>();
    ecs.register::<Item>();
    ecs.register::<ProvidesHealing>();
    ecs.register::<InBackpack>();
    ecs.register::<WantsToPickupItem>();
    ecs.register::<WantsToUseItem>();
    ecs.register::<WantsToDropItem>();
    ecs.register::<Consumable>();
    ecs.register::<Ranged>();
    ecs.register::<InflictsDamage>();
    ecs.register::<AreaOfEffect>();
    ecs.register::<Confusion>();
    ecs.register::<SimpleMarker<SerializeMe>>();
    ecs.register::<MapSerializationHelper>();
    ecs.register::<GameLogSerializationHelper>();
    ecs.register::<Equippable>();
    ecs.register::<Equipped>();
    ecs.register::<MeleePowerBonus>();
    ecs.register::<DefenseBonus>();
    ecs.register::<WantsToUnequipItem>();
    ecs.register::<Lifetime>();
    ecs.register::<Particle>();
    ecs.register::<HungerClock>();
    ecs.register::<ProvidesFood>();
    ecs.register::<Hidden>();
    ecs.register::<EntryTrigger>();
    ecs.register::<EntityMoved>();
    ecs.register::<SingleActivation>();
}
