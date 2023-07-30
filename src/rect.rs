use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Rect {
    pub x1: u16,
    pub x2: u16,
    pub y1: u16,
    pub y2: u16,
}

impl Rect {
    pub fn new(x: u16, y: u16, w: u8, h: u8) -> Self {
        Rect {
            x1: x,
            x2: x + w as u16,
            y1: y,
            y2: y + h as u16,
        }
    }

    pub fn intersect(&self, other: &Rect) -> bool {
        self.x1 <= other.x2 && self.x2 >= other.x1 && self.y1 <= other.y2 && self.y2 >= other.y1
    }

    pub fn center(&self) -> (u16, u16) {
        ((self.x1 + self.x2) / 2, (self.y1 + self.y2) / 2)
    }
}
