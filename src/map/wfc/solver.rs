use std::collections::{HashMap, HashSet};

use super::constraints::CompatibilityMatrix;

enum ConstraintDirection {
    North,
    South,
    East,
    West,
}

pub struct Solver {
    pub collapsed: HashMap<usize, usize>,
    possible_states: Vec<Vec<usize>>,
    matrix: CompatibilityMatrix,
    horizontal_chunks: u16,
    vertical_chunks: u16,
}

impl Solver {
    pub fn new(
        matrix: CompatibilityMatrix,
        number_of_modules: usize,
        horizontal_chunks: u16,
        vertical_chunks: u16,
    ) -> Self {
        let mut possible_states = Vec::with_capacity(number_of_modules);
        let mut possible_block_states = Vec::new();
        for i in 0..number_of_modules {
            possible_block_states.push(i);
        }

        for _ in 0..horizontal_chunks as usize * vertical_chunks as usize {
            possible_states.push(possible_block_states.clone());
        }

        Self {
            collapsed: HashMap::new(),
            possible_states,
            matrix,
            horizontal_chunks,
            vertical_chunks,
        }
    }

    pub fn is_collapsed(&self) -> bool {
        self.possible_states.iter().all(|states| states.is_empty())
    }

    pub fn is_solved(&self) -> bool {
        let total_chunks = self.horizontal_chunks as usize * self.vertical_chunks as usize;
        self.collapsed.len() == total_chunks
    }

    pub fn iterate(&mut self, rng: &mut rltk::RandomNumberGenerator) {
        let chunk_idx = self.get_min_entropy_chunk(rng);
        self.collapse_at(chunk_idx, rng);
        self.propagate(chunk_idx);
    }

    fn get_min_entropy_chunk(&self, rng: &mut rltk::RandomNumberGenerator) -> usize {
        let mut min_entropy = usize::MAX;
        for state in self.possible_states.iter() {
            if !state.is_empty() && state.len() < min_entropy {
                min_entropy = state.len();
            }
        }

        let mut possible_candidates = Vec::new();
        for (idx, state) in self.possible_states.iter().enumerate() {
            if state.len() == min_entropy {
                possible_candidates.push(idx)
            }
        }

        let candidate_idx = rng.roll_dice(1, possible_candidates.len() as i32) as usize - 1;
        possible_candidates[candidate_idx]
    }

    fn collapse_at(&mut self, chunk_idx: usize, rng: &mut rltk::RandomNumberGenerator) {
        let possible_states = &self.possible_states[chunk_idx];
        let state_idx = rng.roll_dice(1, possible_states.len() as i32) as usize - 1;
        self.collapsed.insert(chunk_idx, possible_states[state_idx]);
        self.possible_states[chunk_idx].clear();
    }

    fn propagate(&mut self, chunk_idx: usize) {
        let mut stack = vec![chunk_idx];
        while let Some(idx) = stack.pop() {
            let (chunk_x, chunk_y) = self.chunk_idx_to_xy(idx);

            if chunk_x > 0 {
                let west_neighbor = self.chunk_xy_to_idx(chunk_x - 1, chunk_y);
                if self.constrain(idx, west_neighbor, ConstraintDirection::West) {
                    stack.push(west_neighbor);
                }
            }

            if chunk_x < self.horizontal_chunks - 1 {
                let east_neighbor = self.chunk_xy_to_idx(chunk_x + 1, chunk_y);
                if self.constrain(idx, east_neighbor, ConstraintDirection::East) {
                    stack.push(east_neighbor);
                }
            }

            if chunk_y > 0 {
                let north_neighbor = self.chunk_xy_to_idx(chunk_x, chunk_y - 1);
                if self.constrain(idx, north_neighbor, ConstraintDirection::North) {
                    stack.push(north_neighbor);
                }
            }

            if chunk_y < self.vertical_chunks - 1 {
                let south_neighbor = self.chunk_xy_to_idx(chunk_x, chunk_y + 1);
                if self.constrain(idx, south_neighbor, ConstraintDirection::South) {
                    stack.push(south_neighbor);
                }
            }
        }
    }

    fn constrain(
        &mut self,
        current_idx: usize,
        neighbor_idx: usize,
        direction: ConstraintDirection,
    ) -> bool {
        // If the neighbor is already collapsed, no need to check anything
        if self.collapsed.contains_key(&neighbor_idx) {
            return false;
        }

        let neighbors_possible_states = &self.possible_states[neighbor_idx];

        // If current block is collapsed, then we need to leave only candidates
        // that are fully compatible with the current block
        if let Some(current_module_idx) = self.collapsed.get(&current_idx) {
            let new_possible_states =
                self.filter_compatible(current_module_idx, neighbors_possible_states, &direction);
            let updated = new_possible_states.len() < neighbors_possible_states.len();
            if updated {
                self.possible_states[neighbor_idx] = new_possible_states;
            }
            return updated;
        }

        // Current block is not yet collapsed, so we need to validate all neighbor's potential states
        // against all current block's potential states.
        let mut new_neighbors_possible_states: HashSet<usize> = HashSet::new();
        for possible_current_module in self.possible_states[current_idx].iter() {
            let new_possible_states = self.filter_compatible(
                possible_current_module,
                neighbors_possible_states,
                &direction,
            );
            new_neighbors_possible_states.extend(new_possible_states.iter());
        }

        let updated = new_neighbors_possible_states.len() < neighbors_possible_states.len();
        if updated {
            let mut result = Vec::new();
            result.extend(new_neighbors_possible_states.into_iter());
            self.possible_states[neighbor_idx] = result;
        }
        updated
    }

    fn filter_compatible(
        &self,
        module_idx: &usize,
        possible_states: &[usize],
        direction: &ConstraintDirection,
    ) -> Vec<usize> {
        let current_module_constraints = self.matrix.get_constraints(module_idx);
        possible_states
            .iter()
            .filter(|candidate_idx| match direction {
                ConstraintDirection::North => {
                    current_module_constraints.north.contains(candidate_idx)
                }
                ConstraintDirection::South => {
                    current_module_constraints.south.contains(candidate_idx)
                }
                ConstraintDirection::East => {
                    current_module_constraints.east.contains(candidate_idx)
                }
                ConstraintDirection::West => {
                    current_module_constraints.west.contains(candidate_idx)
                }
            })
            .map(|candidate_idx| *candidate_idx)
            .collect()
    }

    fn chunk_idx_to_xy(&self, chunk_idx: usize) -> (u16, u16) {
        let x = chunk_idx % self.horizontal_chunks as usize;
        let y = chunk_idx / self.horizontal_chunks as usize;
        (x as u16, y as u16)
    }

    fn chunk_xy_to_idx(&self, chunk_x: u16, chunk_y: u16) -> usize {
        chunk_y as usize * self.horizontal_chunks as usize + chunk_x as usize
    }
}
