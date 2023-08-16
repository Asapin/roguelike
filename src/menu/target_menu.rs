use rltk::{console, Rltk, RGB};
use specs::{Entity, World, WorldExt};

use crate::{
    components::{Position, Viewshed, WantsToUseItem},
    state::RunState,
};

#[derive(PartialEq)]
pub enum TargetSelectResult {
    Cancel,
    NoResponse,
    Selected(Position),
}

pub fn target_menu(ecs: &mut World, ctx: &mut Rltk, range: u16, item: Entity) -> RunState {
    let target = show_target_menu(ecs, ctx, range);
    match target {
        TargetSelectResult::Cancel => RunState::AwaitingInput,
        TargetSelectResult::NoResponse => RunState::ShowTargeting { range, item },
        TargetSelectResult::Selected(position) => {
            let mut intent = ecs.write_storage::<WantsToUseItem>();
            let player = ecs.fetch::<Entity>();
            intent
                .insert(
                    *player,
                    WantsToUseItem {
                        item,
                        target: Some(position),
                    },
                )
                .expect("Unable to insert intent");
            RunState::PlayerTurn
        }
    }
}

fn show_target_menu(ecs: &mut World, ctx: &mut Rltk, range: u16) -> TargetSelectResult {
    let player_entity = ecs.fetch::<Entity>();
    let player_pos = ecs.fetch::<Position>();
    let viewsheds = ecs.read_storage::<Viewshed>();

    let viewshed = viewsheds.get(*player_entity);
    let mouse_pos = ctx.mouse_pos();
    let valid_target = if let Some(viewshed) = viewshed {
        draw_menu(ctx, viewshed, *player_pos, range, mouse_pos)
    } else {
        console::log("Player entity doesn't have a viewshed");
        return TargetSelectResult::Cancel;
    };

    if valid_target {
        ctx.set_bg(mouse_pos.0, mouse_pos.1, RGB::named(rltk::CYAN));
        if ctx.left_click {
            return TargetSelectResult::Selected(Position {
                x: mouse_pos.0 as u16,
                y: mouse_pos.1 as u16,
            });
        }
    } else {
        ctx.set_bg(mouse_pos.0, mouse_pos.1, RGB::named(rltk::RED));
        if ctx.left_click {
            return TargetSelectResult::Cancel;
        }
    }

    TargetSelectResult::NoResponse
}

fn draw_menu<'a>(
    ctx: &mut Rltk,
    viewshed: &'a Viewshed,
    player_pos: Position,
    range: u16,
    mouse_pos: (i32, i32),
) -> bool {
    ctx.print_color(
        5,
        0,
        RGB::named(rltk::YELLOW),
        RGB::named(rltk::BLACK),
        "Select Target:",
    );

    // Highlight available target cells
    let mut available_cells = Vec::new();
    for idx in viewshed.visible_tiles.iter() {
        let distance = rltk::DistanceAlg::Pythagoras.distance2d(player_pos.into(), *idx);
        if distance < range as f32 {
            ctx.set_bg(idx.x, idx.y, RGB::named(rltk::BLUE));
            available_cells.push(idx);
        }
    }

    let mut valid_target = false;
    for idx in available_cells.iter() {
        if idx.x == mouse_pos.0 && idx.y == mouse_pos.1 {
            valid_target = true;
        }
    }

    valid_target
}
