use rltk::{Point, RGB};
use specs::prelude::*;

use crate::{
    components::{Confusion, EntityMoved, Monster, Position, Viewshed, WantsToMelee},
    map::map::Map,
    state::RunState,
};

use super::particle_system::ParticleBuilder;

#[derive(Clone, Copy)]
pub struct MonsterAI;

impl<'a> System<'a> for MonsterAI {
    type SystemData = (
        WriteExpect<'a, Map>,
        ReadExpect<'a, Position>,
        ReadExpect<'a, Entity>,
        ReadExpect<'a, RunState>,
        Entities<'a>,
        WriteStorage<'a, Viewshed>,
        ReadStorage<'a, Monster>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, WantsToMelee>,
        WriteStorage<'a, Confusion>,
        WriteExpect<'a, ParticleBuilder>,
        WriteStorage<'a, EntityMoved>,
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
            mut particle_builder,
            mut entity_moved,
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
                particle_builder.request(
                    pos.x,
                    pos.y,
                    RGB::named(rltk::MAGENTA),
                    RGB::named(rltk::BLACK),
                    rltk::to_cp437('?'),
                    200.0,
                );
                continue;
            }

            let player_point: Point = (*player_pos).into();
            let distance = rltk::DistanceAlg::Pythagoras.distance2d(pos.into(), player_point);
            if distance < 1.5 {
                wants_to_melee
                    .insert(
                        entity,
                        WantsToMelee {
                            target: *player_entity,
                        },
                    )
                    .expect("Unable to insert attack");
            } else if viewshed.visible_tiles.contains(&player_point) {
                let path = rltk::a_star_search(
                    map.index_from_xy(pos.x, pos.y),
                    map.index_from_xy(player_pos.x, player_pos.y),
                    &mut *map,
                );
                if path.success && path.steps.len() > 1 {
                    pos.x = (path.steps[1] % map.width as usize) as u16;
                    pos.y = (path.steps[1] / map.width as usize) as u16;
                    viewshed.dirty = true;
                    map.blocked[path.steps[0]] = false;
                    map.blocked[path.steps[1]] = true;
                    entity_moved
                        .insert(entity, EntityMoved {})
                        .expect("Unable to insert marker");
                }
            }
        }
    }
}
