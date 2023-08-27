use std::{collections::HashMap, process::id};

use rltk::RandomNumberGenerator;

use crate::{components::Position, spawn::spawner::spawn_region};

use super::{
    generate_voronoi_spawn_regions,
    map::{Map, TileType},
    remove_unreachable_areas, MapBuilder,
};

const TOP: usize = 0;
const RIGHT: usize = 1;
const BOTTOM: usize = 2;
const LEFT: usize = 3;

const MAX_ENTITIES: u16 = 4;

#[derive(Clone, Copy)]
enum Direction {
    TOP,
    RIGHT,
    BOTTOM,
    LEFT
}

struct Cell {
    x: u16,
    y: u16,
    walls: [bool; 4],
    visited: bool,
}

impl Cell {
    fn new(x: u16, y: u16) -> Self {
        Self {
            x,
            y,
            walls: [true, true, true, true],
            visited: false,
        }
    }

    fn remove_wall(&mut self, direction: &Direction) {
        match direction {
            Direction::TOP => self.walls[0] = false,
            Direction::RIGHT => self.walls[1] = false,
            Direction::BOTTOM => self.walls[2] = false,
            Direction::LEFT => self.walls[3] = false
        }
    }

    fn remove_opposite_wall(&mut self, direction: &Direction) {
        match direction {
            Direction::TOP => self.walls[2] = false,
            Direction::RIGHT => self.walls[3] = false,
            Direction::BOTTOM => self.walls[0] = false,
            Direction::LEFT => self.walls[1] = false
        }
    }
}

struct Grid<'a> {
    width: u16,
    height: u16,
    dimensions: usize,
    cells: Vec<Cell>,
    backtrace: Vec<usize>,
    rng: &'a mut RandomNumberGenerator,
}

impl<'a> Grid<'a> {
    fn new(width: u16, height: u16, rng: &'a mut RandomNumberGenerator) -> Self {
        let dimensions = width as usize * height as usize;
        let mut grid = Self {
            width,
            height,
            dimensions,
            cells: Vec::new(),
            backtrace: Vec::new(),
            rng,
        };

        for y in 0..height {
            for x in 0..width {
                grid.cells.push(Cell::new(x, y))
            }
        }

        grid
    }

    fn generate_maze(&mut self, generator: &mut MazeBuilder) -> usize {
        let first_cell = self.rng.roll_dice(1, self.dimensions as i32) - 1;
        self.backtrace.push(first_cell as usize);

        loop {
            let (current, backtrace_idx) = match self.select_next_cell() {
                None => break,
                Some(idx) => idx,
            };

            self.cells[current].visited = true;
            match self.select_available_neighbor(current) {
                None => {
                    self.backtrace.remove(backtrace_idx);
                }
                Some((direction, neighbor)) => {
                    self.cells[current].remove_wall(&direction);
                    self.cells[neighbor].remove_opposite_wall(&direction);
                    if self.backtrace.len() < 150 {
                        self.backtrace.push(neighbor);
                    } else {
                        self.cells[neighbor].visited = true;
                    }
                }
            };
        }
        self.copy_to_map(&mut generator.map);
        first_cell as usize
    }

    fn select_next_cell(&mut self) -> Option<(usize, usize)> {
        if self.backtrace.is_empty() {
            return None;
        }
        // By changing how we select the next cell, we can alter algorithm's behavior
        let backtrace_idx = self.backtrace.len() - 1;
        Some((self.backtrace[backtrace_idx], backtrace_idx))
    }

    fn calculate_idx(&self, x: u16, y: u16) -> usize {
        (x + self.width * y) as usize
    }

    fn select_available_neighbor(&mut self, idx: usize) -> Option<(Direction, usize)> {
        let mut neighbors = Vec::new();
        let current = &self.cells[idx];

        if current.x > 0 {
            let left_neighbor = self.calculate_idx(current.x - 1, current.y);
            if !self.cells[left_neighbor].visited {
                neighbors.push((Direction::LEFT, left_neighbor));
            }
        }
        if current.x < self.width - 1 {
            let right_neighbor = self.calculate_idx(current.x + 1, current.y);
            if !self.cells[right_neighbor].visited {
                neighbors.push((Direction::RIGHT, right_neighbor));
            }
        }
        if current.y > 0 {
            let top_neighbor = self.calculate_idx(current.x, current.y - 1);
            if !self.cells[top_neighbor].visited {
                neighbors.push((Direction::TOP, top_neighbor));
            }
        }
        if current.y < self.height - 1 {
            let bottom_neighbor = self.calculate_idx(current.x, current.y + 1);
            if !self.cells[bottom_neighbor].visited {
                neighbors.push((Direction::BOTTOM, bottom_neighbor));
            }
        }

        match neighbors.len() {
            0 => None,
            1 => Some(neighbors[0]),
            n => {
                let next_neighbor = self.rng.roll_dice(1, n as i32) - 1;
                Some(neighbors[next_neighbor as usize])
            }
        }
    }

    fn copy_to_map(&self, map: &mut Map) {
        for i in map.tiles.iter_mut() {
            *i = TileType::Wall;
        }

        for cell in self.cells.iter() {
            let x = cell.x * 2 + 1;
            let y = cell.y * 2 + 1;
            let idx = map.index_from_xy(x as u16, y as u16);

            map.tiles[idx] = TileType::Floor;
            if !cell.walls[TOP] {
                map.tiles[idx - map.width as usize] = TileType::Floor;
            }
            if !cell.walls[RIGHT] {
                map.tiles[idx + 1] = TileType::Floor;
            }
            if !cell.walls[BOTTOM] {
                map.tiles[idx + map.width as usize] = TileType::Floor;
            }
            if !cell.walls[LEFT] {
                map.tiles[idx - 1] = TileType::Floor;
            }
        }
    }
}

pub struct MazeBuilder {
    map: Map,
    starting_position: Position,
    noise_areas: HashMap<i32, Vec<usize>>,
}

impl MapBuilder for MazeBuilder {
    fn get_map(&mut self) -> Map {
        self.map.clone()
    }

    fn get_starting_position(&mut self) -> Position {
        self.starting_position.clone()
    }

    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator) {
        let grid_width = self.map.width / 2 - 1;
        let grid_height = self.map.height / 2;
        let mut grid = Grid::new(grid_width, grid_height, rng);
        let start_idx = grid.generate_maze(self);

        let y = start_idx / grid_width as usize;
        let x = start_idx - y * grid_width as usize;
        self.starting_position = Position { x: x as u16 * 2 + 1, y: y as u16 * 2 + 1 };
        let start_idx = self.map.index_from_xy(self.starting_position.x, self.starting_position.y);

        // Find all tiles we can reach from the starting point
        let exit_tile = remove_unreachable_areas(&mut self.map, start_idx);
        self.map.tiles[exit_tile] = TileType::DownStairs;

        self.noise_areas = generate_voronoi_spawn_regions(&self.map, rng);
    }

    fn spawn_entities(&mut self, ecs: &mut specs::World) {
        for (_, area) in self.noise_areas.iter() {
            spawn_region(ecs, &area, &self.map, MAX_ENTITIES)
        }
    }
}

impl MazeBuilder {
    pub fn new(new_depth: u32) -> Self {
        Self {
            map: Map::empty_map(new_depth),
            starting_position: Position { x: 0, y: 0 },
            noise_areas: HashMap::new(),
        }
    }
}