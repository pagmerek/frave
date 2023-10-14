use std::ops;

#[derive(Clone, Copy, Debug)]
pub struct Coord {
    pub x: i32,
    pub y: i32,
}

impl ops::Add<Self> for Coord {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}
