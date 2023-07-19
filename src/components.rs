use rltk::RGB;
use specs::prelude::*;
use specs_derive::Component;

#[derive(Component, Clone, Copy)]
pub struct Position {
    pub x: u16,
    pub y: u16,
}

#[derive(Component)]
pub struct Renderable {
    pub glyph: rltk::FontCharType,
    pub fg: RGB,
    pub bg: RGB,
    pub render_order: i32,
}

#[derive(Component)]
pub struct Player {}

#[derive(Component)]
pub struct Viewshed {
    pub visible_tiles: Vec<rltk::Point>,
    pub range: u16,
    pub dirty: bool,
}

#[derive(Component)]
pub struct Monster {}

#[derive(Component)]
pub struct Name {
    pub name: String,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct BlocksTile {}

#[derive(Component)]
pub struct CombatStats {
    pub max_hp: u32,
    pub hp: u32,
    pub defense: i32,
    pub power: i32,
}

#[derive(Component)]
pub struct WantsToMelee {
    pub target: Entity,
}

#[derive(Component)]
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

#[derive(Component)]
pub struct Item {}

#[derive(Component)]
pub struct Potion {
    pub amount: u32,
}

#[derive(Component)]
pub struct InBackpack {
    pub owner: Entity,
}

#[derive(Component)]
pub struct WantsToPickupItem {
    pub collected_by: Entity,
    pub item: Entity,
}

#[derive(Component)]
pub struct WantsToDrinkPotion {
    pub potion: Entity,
}

#[derive(Component)]
pub struct WantsToDropItem {
    pub item: Entity,
    pub position: Position,
}

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
    ecs.register::<Potion>();
    ecs.register::<InBackpack>();
    ecs.register::<WantsToPickupItem>();
    ecs.register::<WantsToDrinkPotion>();
    ecs.register::<WantsToDropItem>();
}
