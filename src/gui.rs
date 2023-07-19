use rltk::{Point, Rltk, VirtualKeyCode, RGB};
use specs::{prelude::*, shred::Fetch};

use crate::{
    components::{CombatStats, InBackpack, Name, Player, Position, Renderable},
    gamelog::GameLog,
    map::{Map, TileType},
    state::State,
};

#[derive(PartialEq)]
pub enum ItemMenuResult {
    Cancel,
    NoResponse,
    Selected(Entity),
}

pub fn draw(ecs: &World, ctx: &mut Rltk) {
    let map = ecs.fetch::<Map>();
    draw_map(ctx, &map);
    draw_entities(ecs, ctx, &map);
    draw_ui(ecs, ctx, &map);
    draw_tooltip(ecs, ctx, &map);
}

fn draw_map(ctx: &mut Rltk, map: &Fetch<Map>) {
    let floor_fg = RGB::from_f32(0.5, 0.5, 0.5);
    let wall_fg = RGB::from_f32(0.0, 1.0, 0.0);
    let bg = RGB::from_f32(0., 0., 0.);
    let floor = rltk::to_cp437('.');
    let wall = rltk::to_cp437('#');

    let mut y = 0;
    let mut x = 0;
    for (idx, tile) in map.tiles.iter().enumerate() {
        // Render a title depending upon the tile type
        if map.revealed_tiles[idx] {
            let glyph;
            let mut fg;
            match tile {
                TileType::Floor => {
                    glyph = floor;
                    fg = floor_fg;
                }
                TileType::Wall => {
                    glyph = wall;
                    fg = wall_fg;
                }
            }
            if !map.visible_tiles[idx] {
                fg = fg.to_greyscale();
            }
            ctx.set(x, y, fg, bg, glyph);
        }

        // Move coordinates
        x += 1;
        if x >= map.width {
            x = 0;
            y += 1;
        }
    }
}

fn draw_entities(ecs: &World, ctx: &mut Rltk, map: &Fetch<Map>) {
    let positions = ecs.read_storage::<Position>();
    let renderables = ecs.read_storage::<Renderable>();

    for (pos, render) in (&positions, &renderables).join() {
        let idx = map.index_from_xy(pos.x, pos.y);
        if map.visible_tiles[idx] {
            ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
        }
    }
}

fn draw_ui(ecs: &World, ctx: &mut Rltk, map: &Fetch<Map>) {
    ctx.draw_box(
        0,
        map.height,
        map.width - 1,
        map.window_height - map.height - 1,
        RGB::named(rltk::WHITE),
        RGB::named(rltk::BLACK),
    );

    let combat_stats = ecs.read_storage::<CombatStats>();
    let players = ecs.read_storage::<Player>();
    for (_player, stats) in (&players, &combat_stats).join() {
        let health = format!(" HP: {} / {} ", stats.hp, stats.max_hp);
        ctx.print_color(
            12,
            map.height,
            RGB::named(rltk::YELLOW),
            RGB::named(rltk::BLACK),
            &health,
        );

        ctx.draw_bar_horizontal(
            28,
            map.height,
            51,
            stats.hp,
            stats.max_hp,
            RGB::named(rltk::RED),
            RGB::named(rltk::BLACK),
        );
    }

    let log = ecs.fetch::<GameLog>();
    let mut y = map.window_height - 2;
    for s in log.entries.iter().rev() {
        if y <= map.height {
            break;
        } else {
            ctx.print(2, y, s);
        }
        y -= 1;
    }
}

fn draw_tooltip(ecs: &World, ctx: &mut Rltk, map: &Fetch<Map>) {
    let names = ecs.read_storage::<Name>();
    let positions = ecs.read_storage::<Position>();

    let mouse_pos = ctx.mouse_pos();
    ctx.set_bg(mouse_pos.0, mouse_pos.1, RGB::named(rltk::MAGENTA));
    if mouse_pos.0 >= map.width as i32 || mouse_pos.1 >= map.height as i32 {
        return;
    }

    let mut tooltip: Vec<String> = Vec::new();
    for (name, position) in (&names, &positions).join() {
        let idx = map.index_from_xy(position.x, position.y);
        if position.x as i32 == mouse_pos.0
            && position.y as i32 == mouse_pos.1
            && map.visible_tiles[idx]
        {
            tooltip.push(name.name.to_string());
        }
    }

    if tooltip.is_empty() {
        return;
    }

    let mut width: i32 = 0;
    for s in tooltip.iter() {
        if width < s.len() as i32 {
            width = s.len() as i32;
        }
    }
    width += 3;

    if mouse_pos.0 > (map.width / 2) as i32 {
        let arrow_pos = Point::new(mouse_pos.0 - 2, mouse_pos.1);
        let left_x = mouse_pos.0 - width;
        let mut y = mouse_pos.1;
        for s in tooltip.iter() {
            ctx.print_color(
                left_x,
                y,
                RGB::named(rltk::WHITE),
                RGB::named(rltk::GREY),
                s,
            );
            let padding = (width - s.len() as i32) - 1;
            for i in 0..padding {
                ctx.print_color(
                    arrow_pos.x - i,
                    y,
                    RGB::named(rltk::WHITE),
                    RGB::named(rltk::GREY),
                    &" ".to_string(),
                );
            }
            y += 1;
        }
        ctx.print_color(
            arrow_pos.x,
            arrow_pos.y,
            RGB::named(rltk::WHITE),
            RGB::named(rltk::GREY),
            &"->".to_string(),
        );
    } else {
        let arrow_pos = Point::new(mouse_pos.0 + 1, mouse_pos.1);
        let left_x = mouse_pos.0 + 3;
        let mut y = mouse_pos.1;
        for s in tooltip.iter() {
            ctx.print_color(
                left_x + 1,
                y,
                RGB::named(rltk::WHITE),
                RGB::named(rltk::GREY),
                s,
            );
            let padding = (width - s.len() as i32) - 1;
            for i in 0..padding {
                ctx.print_color(
                    arrow_pos.x + i + 1,
                    y,
                    RGB::named(rltk::WHITE),
                    RGB::named(rltk::GREY),
                    &" ".to_string(),
                );
            }
            y += 1;
        }
        ctx.print_color(
            arrow_pos.x,
            arrow_pos.y,
            RGB::named(rltk::WHITE),
            RGB::named(rltk::GREY),
            &"<-".to_string(),
        );
    }
}

pub fn show_inventory(gs: &mut State, ctx: &mut Rltk) -> ItemMenuResult {
    let player_entity = gs.ecs.fetch::<Entity>();
    let names = gs.ecs.read_storage::<Name>();
    let backpack = gs.ecs.read_storage::<InBackpack>();
    let map = gs.ecs.fetch::<Map>();
    let entities = gs.ecs.entities();

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
        "Inventory",
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
