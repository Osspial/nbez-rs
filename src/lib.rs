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
    pub fn new_point(x: f32, y: f32) -> BezNode {
        BezNode::Point {
            x: x,
            y: y
        }
    }

    pub fn new_control(x: f32, y: f32) -> BezNode {
        BezNode::Control {
            x: x,
            y: y
        }
    }

    pub fn x(self) -> f32 {
        <BezNode as Into<(f32, f32)>>::into(self).0
    }

    pub fn y(self) -> f32 {
        <BezNode as Into<(f32, f32)>>::into(self).1
    }

    pub fn is_point(self) -> bool {
        use self::BezNode::*;
        match self {
            Point{..} => true,
            Control{..} => false
        }
    }

    pub fn is_control(self) -> bool {
        use self::BezNode::*;
        match self {
            Point{..} => false,
            Control{..} => true
        }
    }
}

impl Into<[f32; 2]> for BezNode {
    fn into(self) -> [f32; 2] {
        use self::BezNode::*;

        match self {
            Point{x, y}    |
            Control{x, y} => [x, y]
        }
    }
}

impl Into<(f32, f32)> for BezNode {
    fn into(self) -> (f32, f32) {
        use self::BezNode::*;

        match self {
            Point{x, y}    |
            Control{x, y} => (x, y)
        }
    }
}

pub struct BezQuad<I: Index<usize, Output = BezNode>> {
    array: I
}

