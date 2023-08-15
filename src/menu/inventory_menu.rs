use rltk::{VirtualKeyCode, RGB};
use specs::{Entity, Join, World, WorldExt};

use crate::{
    components::{
        Equipped, InBackpack, Name, Position, Ranged, WantsToDropItem, WantsToUnequipItem,
        WantsToUseItem,
    },
    map::map::Map,
    state::RunState,
};

#[derive(PartialEq)]
pub enum ItemMenuResult {
    Cancel,
    NoResponse,
    Selected(Entity),
}

pub fn inventory(ecs: &mut World, ctx: &mut rltk::Rltk) -> RunState {
    let selected_menu = show_item_inventory(ecs, ctx, "Inventory");
    match selected_menu {
        ItemMenuResult::Cancel => RunState::AwaitingInput,
        ItemMenuResult::NoResponse => RunState::ShowInventory,
        ItemMenuResult::Selected(item) => {
            let ranged_storage = ecs.read_storage::<Ranged>();
            let is_item_ranged = ranged_storage.get(item);
            if let Some(ranged) = is_item_ranged {
                RunState::ShowTargeting {
                    range: ranged.range,
                    item,
                }
            } else {
                let mut intent = ecs.write_storage::<WantsToUseItem>();
                let player_entity = ecs.fetch::<Entity>();
                intent
                    .insert(*player_entity, WantsToUseItem { item, target: None })
                    .expect("Unable to insert intent");
                RunState::PlayerTurn
            }
        }
    }
}

pub fn drop_item_menu(ecs: &mut World, ctx: &mut rltk::Rltk) -> RunState {
    let selected_menu = show_item_inventory(ecs, ctx, "Drop which item?");
    match selected_menu {
        ItemMenuResult::Cancel => RunState::AwaitingInput,
        ItemMenuResult::NoResponse => RunState::ShowDropItem,
        ItemMenuResult::Selected(item) => {
            let mut intent = ecs.write_storage::<WantsToDropItem>();
            let player_entity = ecs.fetch::<Entity>();
            let positions = ecs.read_storage::<Position>();
            let player_position = positions.get(*player_entity).unwrap();
            intent
                .insert(
                    *player_entity,
                    WantsToDropItem {
                        item,
                        position: *player_position,
                    },
                )
                .expect("Unable to insert intent");
            RunState::PlayerTurn
        }
    }
}

pub fn unequip_menu(ecs: &mut World, ctx: &mut rltk::Rltk) -> RunState {
    let selected_menu = show_unequip_item(ecs, ctx);
    match selected_menu {
        ItemMenuResult::Cancel => RunState::AwaitingInput,
        ItemMenuResult::NoResponse => RunState::ShowUnequipItem,
        ItemMenuResult::Selected(item) => {
            let mut intent = ecs.write_storage::<WantsToUnequipItem>();
            let player_entity = ecs.fetch::<Entity>();
            intent
                .insert(*player_entity, WantsToUnequipItem { item })
                .expect("Unable to insert intent");
            RunState::PlayerTurn
        }
    }
}

fn show_item_inventory(
    ecs: &mut World,
    ctx: &mut rltk::Rltk,
    window_title: &str,
) -> ItemMenuResult {
    let player_entity = ecs.fetch::<Entity>();
    let names = ecs.read_storage::<Name>();
    let backpack = ecs.read_storage::<InBackpack>();
    let map = ecs.fetch::<Map>();
    let entities = ecs.entities();

    let inventory = (&backpack, &names)
        .join()
        .filter(|(item, _)| item.owner == *player_entity);
    let count = usize::min((map.window_height - 4) as usize, inventory.count()) as u16;

    let mut y = map.window_height / 2 - count / 2;
    ctx.draw_box(
        15,
        y - 2,
        31,
        (count + 3) as i32,
        RGB::named(rltk::WHITE),
        RGB::named(rltk::BLACK),
    );
    ctx.print_color(
        18,
        y - 2,
        RGB::named(rltk::YELLOW),
        RGB::named(rltk::BLACK),
        window_title,
    );
    ctx.print_color(
        18,
        y + count + 1,
        RGB::named(rltk::YELLOW),
        RGB::named(rltk::BLACK),
        "ESCAPE to cancel",
    );

    let mut equippable: Vec<Entity> = Vec::new();
    for (j, (entity, _pack, name)) in (&entities, &backpack, &names)
        .join()
        .filter(|(_, item, _)| item.owner == *player_entity)
        .enumerate()
    {
        ctx.set(
            17,
            y,
            RGB::named(rltk::WHITE),
            RGB::named(rltk::BLACK),
            rltk::to_cp437('('),
        );
        ctx.set(
            18,
            y,
            RGB::named(rltk::YELLOW),
            RGB::named(rltk::BLACK),
            97 + j as rltk::FontCharType,
        );
        ctx.set(
            19,
            y,
            RGB::named(rltk::WHITE),
            RGB::named(rltk::BLACK),
            rltk::to_cp437(')'),
        );

        ctx.print(21, y, &name.name.to_string());
        equippable.push(entity);
        y += 1;
    }

    select_menu(ctx, count, equippable)
}

fn show_unequip_item(ecs: &mut World, ctx: &mut rltk::Rltk) -> ItemMenuResult {
    let player_entity = ecs.fetch::<Entity>();
    let names = ecs.read_storage::<Name>();
    let backpack = ecs.read_storage::<Equipped>();
    let map = ecs.fetch::<Map>();
    let entities = ecs.entities();

    let inventory = (&backpack, &names)
        .join()
        .filter(|(item, _)| item.owner == *player_entity);
    let count = usize::min((map.window_height - 4) as usize, inventory.count()) as u16;

    let mut y = map.window_height / 2 - count / 2;
    ctx.draw_box(
        15,
        y - 2,
        31,
        (count + 3) as i32,
        RGB::named(rltk::WHITE),
        RGB::named(rltk::BLACK),
    );
    ctx.print_color(
        18,
        y - 2,
        RGB::named(rltk::YELLOW),
        RGB::named(rltk::BLACK),
        "Unequip which item?",
    );
    ctx.print_color(
        18,
        y + count + 1,
        RGB::named(rltk::YELLOW),
        RGB::named(rltk::BLACK),
        "ESCAPE to cancel",
    );

    let mut equippable: Vec<Entity> = Vec::new();
    for (j, (entity, _pack, name)) in (&entities, &backpack, &names)
        .join()
        .filter(|(_, item, _)| item.owner == *player_entity)
        .enumerate()
    {
        ctx.set(
            17,
            y,
            RGB::named(rltk::WHITE),
            RGB::named(rltk::BLACK),
            rltk::to_cp437('('),
        );
        ctx.set(
            18,
            y,
            RGB::named(rltk::YELLOW),
            RGB::named(rltk::BLACK),
            97 + j as rltk::FontCharType,
        );
        ctx.set(
            19,
            y,
            RGB::named(rltk::WHITE),
            RGB::named(rltk::BLACK),
            rltk::to_cp437(')'),
        );

        ctx.print(21, y, &name.name.to_string());
        equippable.push(entity);
        y += 1;
    }

    select_menu(ctx, count, equippable)
}

fn select_menu(ctx: &mut rltk::Rltk, count: u16, equippable: Vec<Entity>) -> ItemMenuResult {
    match ctx.key {
        None => ItemMenuResult::NoResponse,
        Some(key) => match key {
            VirtualKeyCode::Escape => ItemMenuResult::Cancel,
            _ => {
                let selection = rltk::letter_to_option(key);
                if selection > -1 && selection < count as i32 {
                    ItemMenuResult::Selected(equippable[selection as usize])
                } else {
                    ItemMenuResult::NoResponse
                }
            }
        },
    }
}
