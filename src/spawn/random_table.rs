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
    BearTrap,
}

struct RandomEntry {
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
    pub fn generate_loot_table(map_depth: u32) -> Self {
        RandomTable::new()
            .add(SpawnEntity::Goblin, 10)
            .add(SpawnEntity::Orc, 1 + map_depth as i32)
            .add(SpawnEntity::HealthPotion, 7)
            .add(SpawnEntity::FireballScroll, 2 + map_depth as i32)
            .add(SpawnEntity::ConfusionScroll, 2 + map_depth as i32)
            .add(SpawnEntity::MagicMissileScroll, 4)
            .add(SpawnEntity::Dagger, 3)
            .add(SpawnEntity::Longsword, map_depth as i32 - 1)
            .add(SpawnEntity::Shield, 3)
            .add(SpawnEntity::TowerShield, map_depth as i32 - 1)
            .add(SpawnEntity::Ration, 8)
            .add(SpawnEntity::BearTrap, 2)
    }

    fn new() -> Self {
        Self {
            entries: Vec::new(),
            total_weight: 0,
        }
    }

    fn add(mut self, entity: SpawnEntity, weight: i32) -> Self {
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
