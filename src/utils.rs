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

pub const fn get_prev_power_two(x: usize) -> usize {
    let mut num = x;
    num |= num >> 1;
    num |= num >> 2;
    num |= num >> 4;
    num |= num >> 8;
    num |= num >> 16;

    num ^ (num >> 1)
}
