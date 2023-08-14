use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct GameLog {
    pub entries: Vec<String>,
}
