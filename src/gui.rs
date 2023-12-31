use rltk::{Point, Rltk, RGB};
use specs::{prelude::*, shred::Fetch};

use crate::{
    components::{
        CombatStats, Hidden, HungerClock, HungerState, Name, Player, Position, Renderable,
    },
    gamelog::GameLog,
    map::map::{Map, TileType},
};

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
    let downstairs_fg = RGB::from_f32(0.0, 1.0, 1.0);
    let floor = rltk::to_cp437('.');
    let downstairs = rltk::to_cp437('>');

    let default_bg = RGB::from_f32(0., 0., 0.);
    let bloodstain_bg = RGB::from_f32(0.75, 0., 0.);

    for (idx, tile) in map.tiles.iter().enumerate() {
        let mut bg = default_bg;
        // Render a title depending upon the tile type
        if map.revealed_tiles[idx] {
            let (x, y) = map.xy_from_index(&idx);
            let glyph;
            let mut fg;
            match tile {
                TileType::Floor => {
                    glyph = floor;
                    fg = floor_fg;
                }
                TileType::Wall => {
                    glyph = map.wall_glyph(x, y);
                    fg = wall_fg;
                }
                TileType::DownStairs => {
                    glyph = downstairs;
                    fg = downstairs_fg;
                }
            }
            if map.bloodstains.contains(&idx) {
                bg = bloodstain_bg;
            }
            if !map.visible_tiles[idx] {
                fg = fg.to_greyscale();
            }
            ctx.set(x, y, fg, bg, glyph);
        }
    }
}

fn draw_entities(ecs: &World, ctx: &mut Rltk, map: &Fetch<Map>) {
    let positions = ecs.read_storage::<Position>();
    let renderables = ecs.read_storage::<Renderable>();
    let hidden = ecs.read_storage::<Hidden>();

    let mut data = (&positions, &renderables, !&hidden)
        .join()
        .collect::<Vec<_>>();
    data.sort_by(|(_, render1, _), (_, render2, _)| {
        render2.render_order.cmp(&render1.render_order)
    });
    for (pos, render, _hidden) in data.iter() {
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
    let depth = format!("Depth: {}", map.depth);
    ctx.print_color(
        2,
        map.height,
        RGB::named(rltk::YELLOW),
        RGB::named(rltk::BLACK),
        &depth,
    );

    let combat_stats = ecs.read_storage::<CombatStats>();
    let players = ecs.read_storage::<Player>();
    let hunger_clock = ecs.read_storage::<HungerClock>();
    for (_player, stats, hunger) in (&players, &combat_stats, &hunger_clock).join() {
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

        match hunger.state {
            HungerState::WellFed => ctx.print_color(
                71,
                map.height - 1,
                RGB::named(rltk::GREEN),
                RGB::named(rltk::BLACK),
                "Well fed",
            ),
            HungerState::Normal => {}
            HungerState::Hungry => ctx.print_color(
                71,
                map.height - 1,
                RGB::named(rltk::ORANGE),
                RGB::named(rltk::BLACK),
                "Hungry",
            ),
            HungerState::Starving => ctx.print_color(
                71,
                map.height - 1,
                RGB::named(rltk::RED),
                RGB::named(rltk::BLACK),
                "Starving",
            ),
        };
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
    let hidden = ecs.read_storage::<Hidden>();

    let mouse_pos = ctx.mouse_pos();
    ctx.set_bg(mouse_pos.0, mouse_pos.1, RGB::named(rltk::MAGENTA));
    if mouse_pos.0 >= map.width as i32 || mouse_pos.1 >= map.height as i32 {
        return;
    }

    let mut tooltip: Vec<String> = Vec::new();
    for (name, position, _hidden) in (&names, &positions, !&hidden).join() {
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
