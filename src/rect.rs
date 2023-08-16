use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Rect {
    pub x1: u16,
    pub x2: u16,
    pub y1: u16,
    pub y2: u16,
}

impl Rect {
    pub fn new(x: u16, y: u16, w: u16, h: u16) -> Self {
        Self {
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

    pub fn width(&self) -> u16 {
        self.x2 - self.x1
    }

    pub fn height(&self) -> u16 {
        self.y2 - self.y1
    }

    pub fn area(&self) -> u32 {
        self.width() as u32 * self.height() as u32
    }
}
