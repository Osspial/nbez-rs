extern crate bev;
use bev::{BezCube, BezNode};

fn main() {
    let arr =
        [BezNode::new_point(0.0, 0.0), 
            BezNode::new_control(0.0, 1.0), 
            BezNode::new_control(1.0, 0.0), 
        BezNode::new_point(1.0, 1.0),
            BezNode::new_control(1.0, 2.0),
            BezNode::new_control(2.0, 1.0),
        BezNode::new_point(2.0, 2.0)];
    let curve = BezCube::from_container(arr).unwrap();
}