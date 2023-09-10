use std::collections::{HashMap, HashSet};

use super::modules::Module;

#[derive(Clone)]
pub struct Constraints {
    pub north: HashSet<usize>,
    pub south: HashSet<usize>,
    pub east: HashSet<usize>,
    pub west: HashSet<usize>,
}

impl Constraints {
    fn new(current_module: &Module, available_modules: &[Module]) -> Self {
        let mut north = HashSet::new();
        let mut south = HashSet::new();
        let mut east = HashSet::new();
        let mut west = HashSet::new();

        for (idx, candidate) in available_modules.iter().enumerate() {
            if current_module.compatible_on_north_side(candidate) {
                north.insert(idx);
            }

            if current_module.compatible_on_south_side(candidate) {
                south.insert(idx);
            }

            if current_module.compatible_on_east_side(candidate) {
                east.insert(idx);
            }

            if current_module.compatible_on_west_side(candidate) {
                west.insert(idx);
            }
        }

        Self {
            north,
            south,
            east,
            west,
        }
    }
}

#[derive(Clone)]
pub struct CompatibilityMatrix {
    matrix: HashMap<usize, Constraints>,
}

impl CompatibilityMatrix {
    pub fn build(modules: &[Module]) -> Self {
        let mut matrix = HashMap::new();
        for i in 0..modules.len() {
            let constraints = Constraints::new(&modules[i], modules);
            matrix.insert(i, constraints);
        }

        CompatibilityMatrix { matrix }
    }

    pub fn get_constraints(&self, module_idx: &usize) -> &Constraints {
        &self.matrix[module_idx]
    }
}
