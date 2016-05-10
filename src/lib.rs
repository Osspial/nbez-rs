use std::ops::Index;
use std::convert::Into;

#[derive(Debug, Clone, Copy)]
pub enum BezNode {
    Point {
        x: f32,
        y: f32
    },

    Control {
        x: f32,
        y: f32
    }
}

impl BezNode {
    pub fn x(self) -> f32 {
        <BezNode as Into<(f32, f32)>>::into(self).0
    }

    pub fn y(self) -> f32 {
        <BezNode as Into<(f32, f32)>>::into(self).1
    }
}

impl Into<[f32; 2]> for BezNode {
    fn into(self) -> [f32; 2] {
        use self::BezNode::*;

        match self {
            Point{x, y} |
            Control{x, y} => [x, y]
        }
    }
}

impl Into<(f32, f32)> for BezNode {
    fn into(self) -> (f32, f32) {
        use self::BezNode::*;

        match self {
            Point{x, y} |
            Control{x, y} => (x, y)
        }
    }
}

pub struct BezQuad<I: Index<usize, Output = BezNode>> {
    array: I
}

