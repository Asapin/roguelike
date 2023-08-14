use rltk::RGB;
use specs::{Entities, System, WriteExpect, WriteStorage};

use crate::components::{Lifetime, Position, Renderable};

struct ParticleRequest {
    x: u16,
    y: u16,
    fg: RGB,
    bg: RGB,
    glyph: rltk::FontCharType,
    lifetime: f32,
}

pub struct ParticleBuilder {
    requests: Vec<ParticleRequest>,
}

impl ParticleBuilder {
    pub fn new() -> Self {
        Self {
            requests: Vec::new(),
        }
    }

    pub fn request(
        &mut self,
        x: u16,
        y: u16,
        fg: RGB,
        bg: RGB,
        glyph: rltk::FontCharType,
        lifetime: f32,
    ) {
        self.requests.push(ParticleRequest {
            x,
            y,
            fg,
            bg,
            glyph,
            lifetime,
        });
    }
}

#[derive(Clone, Copy)]
pub struct ParticleSpawnSystem;

impl<'a> System<'a> for ParticleSpawnSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, Renderable>,
        WriteStorage<'a, Lifetime>,
        WriteExpect<'a, ParticleBuilder>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut positions, mut renderables, mut lifetimes, mut particle_builder) = data;

        for new_particle in particle_builder.requests.iter() {
            let p = entities.create();
            positions
                .insert(
                    p,
                    Position {
                        x: new_particle.x,
                        y: new_particle.y,
                    },
                )
                .expect("Couldn't create particle position");
            renderables
                .insert(
                    p,
                    Renderable {
                        glyph: new_particle.glyph,
                        fg: new_particle.fg,
                        bg: new_particle.bg,
                        render_order: 0,
                    },
                )
                .expect("Couldn't create particle render");
            lifetimes
                .insert(
                    p,
                    Lifetime {
                        lifetime_ms: new_particle.lifetime,
                    },
                )
                .expect("Couldn't create particle lifetime");
        }

        particle_builder.requests.clear();
    }
}
