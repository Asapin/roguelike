use rltk::Point;
use specs::prelude::*;

use crate::{
    components::{Confusion, Monster, Position, Viewshed, WantsToMelee},
    map::Map,
    state::RunState,
};

#[derive(Clone, Copy)]
pub struct MonsterAI {}

impl<'a> System<'a> for MonsterAI {
    type SystemData = (
        WriteExpect<'a, Map>,
        ReadExpect<'a, Point>,
        ReadExpect<'a, Entity>,
        ReadExpect<'a, RunState>,
        Entities<'a>,
        WriteStorage<'a, Viewshed>,
        ReadStorage<'a, Monster>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, WantsToMelee>,
        WriteStorage<'a, Confusion>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            mut map,
            player_pos,
            player_entity,
            runstate,
            entities,
            mut viewshed,
            monster,
            mut position,
            mut wants_to_melee,
            mut confused,
        ) = data;

        if *runstate != RunState::MonsterTurn {
            return;
        }

        for (entity, mut viewshed, _monster, mut pos) in
            (&entities, &mut viewshed, &monster, &mut position).join()
        {
            if let Some(confusion) = confused.get_mut(entity) {
                confusion.turns -= 1;
                if confusion.turns == 0 {
                    confused.remove(entity);
                }
                continue;
            }

            let distance =
                rltk::DistanceAlg::Pythagoras.distance2d(Point::new(pos.x, pos.y), *player_pos);
            if distance < 1.5 {
                wants_to_melee
                    .insert(
                        entity,
                        WantsToMelee {
                            target: *player_entity,
                        },
                    )
                    .expect("Unable to insert attack");
            } else if viewshed.visible_tiles.contains(&*player_pos) {
                let path = rltk::a_star_search(
                    map.index_from_xy(pos.x, pos.y),
                    map.index_from_xy(player_pos.x as u16, player_pos.y as u16),
                    &mut *map,
                );
                if path.success && path.steps.len() > 1 {
                    pos.x = (path.steps[1] % map.width as usize) as u16;
                    pos.y = (path.steps[1] / map.width as usize) as u16;
                    viewshed.dirty = true;
                    map.blocked[path.steps[0]] = false;
                    map.blocked[path.steps[1]] = true;
                }
            }
        }
    }
}
