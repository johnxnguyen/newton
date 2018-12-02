use geometry::types::Point;
use geometry::types::Rect;
use geometry::types::Quadrant::{NW, NE, SW, SE};
use super::types::Body;

// RealBody //////////////////////////////////////////////////////////////////

#[derive(Clone, Debug)]
pub struct RealBody {
    id: usize,
    mass: f32,
    position: Point,
}

impl RealBody {
    // TODO: validate
    fn new(id: usize, mass: f32, x: f32, y: f32) -> RealBody {
        RealBody {
            id,
            mass,
            position: Point { x, y },
        }
    }
}

// BHNode ////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct BHNode {
    space: Rect,
    body: Option<RealBody>,
    nw: Option<Box<BHNode>>,
    ne: Option<Box<BHNode>>,
    sw: Option<Box<BHNode>>,
    se: Option<Box<BHNode>>,
}

impl BHNode {
    fn new(space: Rect) -> Self {
        BHNode {
            space,
            body: None,
            nw: None,
            ne: None,
            sw: None,
            se: None,
        }
    }

//    fn body(&self) -> RealBody {
//        // we need to find the center of mass for this node.
//        RealBody {
//            id: 0,
//            mass: 0.0,
//            position: Point::origin(),
//        }
//    }

    // TODO: needs testing
    pub fn insert(&mut self, body: RealBody) {
        // body must be contained in the space
        if !self.space.contains(&body.position) {
            return
        }

        if self.is_leaf() {
            // leaf already contains body
            if let Some(old_body) = self.body.take() {
                // make this node internal
                self.pass_down(old_body);
                self.pass_down(body);
            }
                else {
                    self.body = Some(body);
                }
        } else {
            self.pass_down(body);
        }
    }
    
    fn pass_down(&mut self, body: RealBody) {
        match self.space.which_quadrant(&body.position) {
            Some((quadrant, subspace)) => {
                // in case the child doesn't exist, we'll create one
                let node =  BHNode::new(subspace);
                let child = Box::new(node);

                match quadrant {
                    NW => self.nw.get_or_insert(child).insert(body),
                    NE => self.ne.get_or_insert(child).insert(body),
                    SW => self.sw.get_or_insert(child).insert(body),
                    SE => self.se.get_or_insert(child).insert(body),
                }
            },
            None => unreachable!("Not in any quadrant!"),
        }
    
    }
    
    // TODO: needs testing
    fn is_leaf(&self) -> bool {
        self.nw.is_none() && self.ne.is_none() && self.sw.is_none() && self.se.is_none()
    }
    
    fn print(&self, indent: usize, name: String) {
        print!("{:width$}{}: ", "", name, width = indent);
        print!("space: ({}, {}, {}, {}) ", self.space.origin.x, self.space.origin.y, self.space.size.width, self.space.size.height);

        if let Some(body) = &self.body {
            println!("body: #{}", body.id);
        } else {
            println!("body: X");
        }


        if let Some(nw) = &self.nw {
            nw.print(indent + 3, String::from("|NW"));
        } else {
            println!("{:width$}|NW: X", "", width = indent + 3);
        }

        if let Some(ne) = &self.ne {
            ne.print(indent + 3, String::from("|NE"));
        } else {
            println!("{:width$}|NE: X", "", width = indent + 3);
        }

        if let Some(sw) = &self.sw {
            sw.print(indent + 3, String::from("|SW"));
        } else {
            println!("{:width$}|SW: X", "", width = indent + 3);
        }

        if let Some(se) = &self.se {
            se.print(indent + 3, String::from("|SE"));
        } else {
            println!("{:width$}|SE: X", "", width = indent + 3);
        }
    }
}

#[test]
fn it_creates_tree() {
    let mut n = BHNode::new(Rect::new(0.0, 0.0, 40.0, 40.0));
    n.insert(RealBody::new(0, 1.0, 9.0, 12.5));
    n.insert(RealBody::new(1, 10.0, 30.0, 20.5));
    n.insert(RealBody::new(2, 0.2, 30.0, 20.501));
    n.print(0, String::from("Root"));
}
