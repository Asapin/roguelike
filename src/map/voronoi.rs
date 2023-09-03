use std::collections::HashMap;

use rltk::{RandomNumberGenerator, Point};

use crate::{components::Position, spawn::spawner::spawn_region};

use super::{map::{Map, TileType}, MapBuilder, remove_unreachable_areas, generate_voronoi_spawn_regions};

const MAX_ENTITIES: u16 = 4;

pub enum DistanceAlgorithm {
    Pythagoras,
    Manhattan,
    Chebyshev
}

pub struct VoronoiBuilder {
    map: Map,
    starting_position: Position,
    noise_areas: HashMap<i32, Vec<usize>>,
    n_seeds: usize,
    distance_algorith: DistanceAlgorithm
}

impl MapBuilder for VoronoiBuilder {
    fn get_map(&mut self) -> Map {
        self.map.clone()
    }

    fn get_starting_position(&mut self) -> Position {
        self.starting_position.clone()
    }

    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator) {
        let voronoi_seeds = self.build_seeds(rng);
        let mut voronoi_membership: Vec<usize> = vec![0; self.map.width as usize * self.map.height as usize];
        for (i, vid) in voronoi_membership.iter_mut().enumerate() {
            let x = i % self.map.width as usize;
            let y = i / self.map.width as usize;

            let start = Point::new(x, y);
            let mut min_seed = 0;
            let mut min_distance = self.distance(start, voronoi_seeds[min_seed].1);
            for (seed, pos) in voronoi_seeds.iter().enumerate().skip(1) {
                let distance = self.distance(start, pos.1);
                if distance < min_distance {
                    min_distance = distance;
                    min_seed = seed;
                }
            }

            *vid = min_seed;
        }

        for y in 1..self.map.height - 1 {
            for x in 1..self.map.width - 1 {
                let mut neighbors = 0;
                let my_idx = self.map.index_from_xy(x, y);
                let my_seed = voronoi_membership[my_idx];
                if voronoi_membership[self.map.index_from_xy(x - 1, y)] != my_seed { neighbors += 1; }
                if voronoi_membership[self.map.index_from_xy(x + 1, y)] != my_seed { neighbors += 1; }
                if voronoi_membership[self.map.index_from_xy(x, y - 1)] != my_seed { neighbors += 1; }
                if voronoi_membership[self.map.index_from_xy(x, y + 1)] != my_seed { neighbors += 1; }

                if neighbors < 2 {
                    self.map.tiles[my_idx] = TileType::Floor;
                }
            }
        }

        self.starting_position = Position {
            x: self.map.width / 2,
            y: self.map.height / 2,
        };

        let start_idx = self
            .map
            .index_from_xy(self.starting_position.x, self.starting_position.y);
        let exit_idx = remove_unreachable_areas(&mut self.map, start_idx);
        self.map.tiles[exit_idx] = TileType::DownStairs;
        self.noise_areas = generate_voronoi_spawn_regions(&self.map, rng);
    }

    fn spawn_entities(&mut self, ecs: &mut specs::World) {
        for (_, area) in self.noise_areas.iter() {
            spawn_region(ecs, &area, &self.map, MAX_ENTITIES);
        }
    }
}

impl VoronoiBuilder {
    pub fn pythagoras(new_depth: u32) -> Self {
        Self {
            map: Map::empty_map(new_depth),
            starting_position: Position { x: 0, y: 0 },
            noise_areas: HashMap::new(),
            n_seeds: 64,
            distance_algorith: DistanceAlgorithm::Pythagoras
        }
    }

    pub fn manhattan(new_depth: u32) -> Self {
        Self {
            map: Map::empty_map(new_depth),
            starting_position: Position { x: 0, y: 0 },
            noise_areas: HashMap::new(),
            n_seeds: 64,
            distance_algorith: DistanceAlgorithm::Manhattan
        }
    }

    pub fn chebyshev(new_depth: u32) -> Self {
        Self {
            map: Map::empty_map(new_depth),
            starting_position: Position { x: 0, y: 0 },
            noise_areas: HashMap::new(),
            n_seeds: 64,
            distance_algorith: DistanceAlgorithm::Chebyshev
        }
    }

    fn build_seeds(&self, rng: &mut RandomNumberGenerator) -> Vec<(usize, rltk::Point)> {
        let mut voronoi_seeds: Vec<(usize, rltk::Point)> = Vec::new();

        while voronoi_seeds.len() < self.n_seeds {
            let vx = rng.roll_dice(1, self.map.width as i32 - 1) as u16;
            let vy = rng.roll_dice(1, self.map.height as i32 - 1) as u16;
            let vidx = self.map.index_from_xy(vx, vy);
            let candidate = (vidx, rltk::Point::new(vx, vy));
            if !voronoi_seeds.contains(&candidate) {
                voronoi_seeds.push(candidate);
            }
        }
        voronoi_seeds
    }

    fn distance(&self, start: Point, end: Point) -> f32 {
        match self.distance_algorith {
            DistanceAlgorithm::Pythagoras => rltk::DistanceAlg::PythagorasSquared.distance2d(start, end),
            DistanceAlgorithm::Manhattan => rltk::DistanceAlg::Manhattan.distance2d(start, end),
            DistanceAlgorithm::Chebyshev => rltk::DistanceAlg::Chebyshev.distance2d(start, end)
        }
    }
}