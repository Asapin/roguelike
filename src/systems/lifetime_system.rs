use specs::{Entity, Join, World, WorldExt};

use crate::components::Lifetime;

pub fn remove_expired_entities(ecs: &mut World, ctx: &rltk::Rltk) {
    let mut expired_entities: Vec<Entity> = Vec::new();
    {
        // Age out entities
        let mut lifetimes = ecs.write_storage::<Lifetime>();
        let entities = ecs.entities();
        for (entity, mut lifetime) in (&entities, &mut lifetimes).join() {
            lifetime.lifetime_ms -= ctx.frame_time_ms;
            if lifetime.lifetime_ms < 0.0 {
                expired_entities.push(entity);
            }
        }
    }
    for expired in expired_entities {
        ecs.delete_entity(expired).expect("Entity do not expire");
    }
}
