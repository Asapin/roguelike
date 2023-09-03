use rltk::{FastNoise, RandomNumberGenerator};
use specs::World;
use std::{
    cmp::{max, min},
    collections::HashMap,
};

use crate::{components::Position, rect::Rect};

use self::{
    bsp_dungeon::BspDungeonBuilder,
    bsp_interior::BspInteriorBuilder,
    cellular_automata::CellularAutomataBuilder,
    dla::DLABuilder,
    drunkard::DrunkardsWalkBuilder,
    map::{Map, TileType},
    maze::MazeBuilder,
};

pub mod bsp_dungeon;
pub mod bsp_interior;
pub mod cellular_automata;
pub mod dla;
pub mod drunkard;
pub mod map;
pub mod maze;

pub enum Symmetry {
    None,
    Horizontal,
    Vertical,
    Both,
}

pub trait MapBuilder {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator);
    fn spawn_entities(&mut self, ecs: &mut World);
    fn get_map(&mut self) -> Map;
    fn get_starting_position(&mut self) -> Position;
}

pub fn random_builder(new_depth: u32, rng: &mut RandomNumberGenerator) -> Box<dyn MapBuilder> {
    let builder_idx = rng.roll_dice(1, 13);
    match builder_idx {
        1 => Box::new(BspDungeonBuilder::new(new_depth)),
        2 => Box::new(BspInteriorBuilder::new(new_depth)),
        3 => Box::new(CellularAutomataBuilder::new(new_depth)),
        4 => Box::new(DrunkardsWalkBuilder::open_area(new_depth)),
        5 => Box::new(DrunkardsWalkBuilder::open_halls(new_depth)),
        6 => Box::new(DrunkardsWalkBuilder::winding_passages(new_depth)),
        7 => Box::new(DrunkardsWalkBuilder::fat_passages(new_depth)),
        8 => Box::new(DrunkardsWalkBuilder::fearful_symmetry(new_depth)),
        9 => Box::new(MazeBuilder::new(new_depth)),
        10 => Box::new(DLABuilder::new_walk_inwards(new_depth)),
        11 => Box::new(DLABuilder::new_walk_outwards(new_depth)),
        12 => Box::new(DLABuilder::new_central_attractor(new_depth)),
        _ => Box::new(DLABuilder::insectoid(new_depth)),
    }
}

pub fn apply_room_to_map(map: &mut Map, room: &Rect) {
    for y in room.y1 + 1..=room.y2 {
        for x in room.x1 + 1..=room.x2 {
            let idx = map.index_from_xy(x, y);
            map.tiles[idx] = TileType::Floor;
        }
    }
}

pub fn apply_horizontal_tunnel(map: &mut Map, x1: u16, x2: u16, y: u16) {
    let min_x = min(x1, x2);
    let max_x = max(x1, x2);
    for x in min_x..=max_x {
        let idx = map.index_from_xy(x, y);
        if idx > 0 && idx < map.tiles.len() {
            map.tiles[idx] = TileType::Floor;
        }
    }
}

pub fn apply_vertical_tunnel(map: &mut Map, y1: u16, y2: u16, x: u16) {
    let min_y = min(y1, y2);
    let max_y = max(y1, y2);
    for y in min_y..=max_y {
        let idx = map.index_from_xy(x, y);
        if idx > 0 && idx < map.tiles.len() {
            map.tiles[idx] = TileType::Floor;
        }
    }
}

pub fn remove_unreachable_areas(map: &mut Map, start_idx: usize) -> usize {
    map.populate_blocked();
    let poi: Vec<usize> = vec![start_idx];
    let dijkstra_map = rltk::DijkstraMap::new(map.width, map.height, &poi, map, 300.0);
    let (mut exit_idx, mut exit_distance) = (start_idx, 0.0f32);
    for (i, tile) in map.tiles.iter_mut().enumerate() {
        if *tile != TileType::Floor {
            continue;
        }
        let distance_to_start = dijkstra_map.map[i];
        if distance_to_start == f32::MAX {
            // We can't pathfind to this tile - so make it a wall
            *tile = TileType::Wall;
        } else {
            if distance_to_start > exit_distance {
                exit_distance = distance_to_start;
                exit_idx = i;
            }
        }
    }
    exit_idx
}

pub fn generate_voronoi_spawn_regions(
    map: &Map,
    rng: &mut RandomNumberGenerator,
) -> HashMap<i32, Vec<usize>> {
    let mut noise_areas: HashMap<i32, Vec<usize>> = HashMap::new();
    let mut noise = FastNoise::seeded(rng.roll_dice(1, u16::MAX as i32) as u64);
    noise.set_noise_type(rltk::NoiseType::Cellular);
    noise.set_frequency(0.08);
    noise.set_cellular_distance_function(rltk::CellularDistanceFunction::Manhattan);

    for y in 1..map.height - 1 {
        for x in 1..map.width - 1 {
            let idx = map.index_from_xy(x, y);
            if map.tiles[idx] != TileType::Floor {
                continue;
            }
            let cell_value_f = noise.get_noise(x as f32, y as f32) * 10240.0;
            let cell_value = cell_value_f as i32;

            match noise_areas.get_mut(&cell_value) {
                Some(entry) => {
                    entry.push(idx);
                }
                None => {
                    noise_areas.insert(cell_value, vec![idx]);
                }
            };
        }
    }

    noise_areas
}

pub fn paint(map: &mut Map, mode: &Symmetry, brush_size: u16, x: u16, y: u16) {
    match mode {
        Symmetry::None => apply_paint(map, brush_size, x, y),
        Symmetry::Horizontal => {
            let center_x = map.width / 2;
            if x == center_x {
                apply_paint(map, brush_size, x, y);
            } else {
                let dist_x = i32::abs(center_x as i32 - x as i32) as u16;
                apply_paint(map, brush_size, center_x + dist_x, y);
                apply_paint(map, brush_size, center_x - dist_x, y);
            }
        }
        Symmetry::Vertical => {
            let center_y = map.height / 2;
            if y == center_y {
                apply_paint(map, brush_size, x, y);
            } else {
                let dist_y = i32::abs(center_y as i32 - y as i32) as u16;
                apply_paint(map, brush_size, x, center_y + dist_y);
                apply_paint(map, brush_size, x, center_y - dist_y);
            }
        }
        Symmetry::Both => {
            let center_x = map.width / 2;
            let center_y = map.height / 2;
            if x == center_x && y == center_y {
                apply_paint(map, brush_size, x, y);
            } else {
                let dist_x = i32::abs(center_x as i32 - x as i32) as u16;
                let dist_y = i32::abs(center_y as i32 - y as i32) as u16;
                apply_paint(map, brush_size, center_x + dist_x, y);
                apply_paint(map, brush_size, center_x - dist_x, y);
                apply_paint(map, brush_size, x, center_y + dist_y);
                apply_paint(map, brush_size, x, center_y - dist_y);
            }
        }
    }
}

fn apply_paint(map: &mut Map, brush_size: u16, x: u16, y: u16) {
    match brush_size {
        1 => {
            let digger_idx = map.index_from_xy(x, y);
            map.tiles[digger_idx] = TileType::Floor;
        }
        _ => {
            let half_brush_size = brush_size / 2;
            for brush_y in y - half_brush_size..y + half_brush_size {
                for brush_x in x - half_brush_size..x + half_brush_size {
                    if brush_x > 1
                        && brush_x < map.width - 1
                        && brush_y > 1
                        && brush_y < map.height - 1
                    {
                        let idx = map.index_from_xy(brush_x, brush_y);
                        map.tiles[idx] = TileType::Floor;
                    }
                }
            }
        }
    }
}
