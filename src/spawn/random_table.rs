use rltk::RandomNumberGenerator;

#[derive(Clone, Copy)]
pub enum SpawnEntity {
    Goblin,
    Orc,
    HealthPotion,
    FireballScroll,
    ConfusionScroll,
    MagicMissileScroll,
    Dagger,
    Longsword,
    Shield,
    TowerShield,
    Ration,
}

pub struct RandomEntry {
    entity: SpawnEntity,
    weight: i32,
}

impl RandomEntry {
    pub fn new(entity: SpawnEntity, weight: i32) -> Self {
        Self { entity, weight }
    }
}

pub struct RandomTable {
    entries: Vec<RandomEntry>,
    total_weight: i32,
}

impl RandomTable {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            total_weight: 0,
        }
    }

    pub fn add(mut self, entity: SpawnEntity, weight: i32) -> Self {
        if weight > 0 {
            self.total_weight += weight;
            self.entries.push(RandomEntry::new(entity, weight));
        }
        self
    }

    pub fn roll(&self, rng: &mut RandomNumberGenerator) -> Option<SpawnEntity> {
        if self.total_weight == 0 {
            return None;
        }
        let mut roll = rng.roll_dice(1, self.total_weight) - 1;
        let mut index: usize = 0;
        while roll > 0 {
            if roll < self.entries[index].weight {
                return Some(self.entries[index].entity);
            }

            roll -= self.entries[index].weight;
            index += 1;
        }

        None
    }
}
