use std::collections::HashMap;

use geometry::types::Point;
use geometry::types::Quadrant;
use geometry::types::Quadrant::*;
use geometry::types::Rect;

use super::types::Body;

// Action ////////////////////////////////////////////////////////////////////
//
// Represents an action to take on a `BHTree`. Either insert the given node
// or internalize the node at the given index, before considering the
// pending state.

enum Action {
    Insert(Node),
    Internalize(Index, Pending),
}

// Pending ///////////////////////////////////////////////////////////////////
//
// Represents an intermediate state of the insertion algorithm. When adding
// a body to the tree it is inspected top down. It could happen that a node
// in the path needs to be internalized before the body can be inserted, in
// which case, the current index and the body need to be stored temporarily
// so that the insertion algorithm can continue at a later time.

struct Pending(Index, Body);

// BHTree ////////////////////////////////////////////////////////////////////
//
// The Barnes-Hut Tree is a quadtree used to efficiently calculate forces
// between n bodies. It requires O(nlogn) time to calculate force vectors for
// each of the n bodies, a substantial improvement over the quadratic brute
// force algorithm.
//
// It achieves this by spatially storing the bodies in its leaves. The force
// vector for each body is then calculated by traversing the tree across
// internal nodes of varying depth. These internal nodes are viewed as virtual
// bodies and are used to calculate the force vectors. The body's final force
// vector is simply the sum of all intermediate force vectors at each virtual
// body.

type Index = u32;

#[derive(Debug)]
pub struct BHTree {
    nodes: HashMap<Index, Node>,
}

impl BHTree {
    /// Initialized tree with a root nodes spanning the given space.
    pub fn new(space: Rect) -> BHTree {
        let mut nodes: HashMap<Index, Node> = HashMap::new();
        let root = Node::new(0, space, None);
        nodes.insert(root.id, root);
        BHTree { nodes }
    }

    /// Borrows the node for the given index, if it exists.
    fn node(&self, idx: &Index) -> Option<&Node> {
        self.nodes.get(idx)
    }

    /// Returns true if the given node is a leaf.
    fn is_leaf(&self, node: &Node) -> bool {
        self.node(&node.nw()).is_none() &&
        self.node(&node.ne()).is_none() &&
        self.node(&node.sw()).is_none() &&
        self.node(&node.se()).is_none()
    }

    /// Inserts the given body into the tree.
    pub fn add(&mut self, body: Body) {
        self.insert(Pending(0, body));
    }

    /// Inserts the given body into the tree at the given node.
    fn insert(&mut self, pending: Pending) {
        let action: Action;
        {
            // inspect the tree to find the necessary action
            action = self.action(self.node(&pending.0).unwrap(), pending.1);
        }
        {
            match self.process(action) {
                Some(pending) => self.insert(pending),
                None => (),
            }
        }
    }

    /// Processes the action by either inserting a new node, internalizing an
    /// existing node, or doing nothing. Optionally returns a pending state.
    fn process(&mut self, action: Action) -> Option<Pending> {
        match action {
            Action::Insert(node) => {
                self.nodes.insert(node.id, node);
                None
            },
            Action::Internalize(id, pending) => {
                self.internalize(id);
                Some(pending)
            },
        }
    }

    /// Internalizes the node at the given index by taking the node's body
    /// and inserting it in the appropriate child.
    fn internalize(&mut self, id: Index) {
        let mut node = self.nodes.remove(&id).expect("Node doesn't exist.");
        debug_assert!(self.is_leaf(&node), "Can't internalize an internal node");

        let child = node.child_from_self().expect("Can't internalize empty leaf.");

        self.nodes.insert(node.id, node);
        self.nodes.insert(child.id, child);
    }

    /// Dives into the tree rooted at the given node searching for the
    /// position to insert the given body. If an empty leaf is found,
    /// a new node is created, wrapped in an Insert variant and returned
    /// for subsequent insertion. If an occupied leaf is found, then the
    /// Internalize variant is return, which signifies the occupied leaf
    /// needs to be internalize. Additionally, the current index and body
    /// is included in the return so that the search can continue after
    /// the leaf has been internalized.
    fn action(&self, node: &Node, body: Body) -> Action {
        debug_assert!(node.space.contains(&body.position));

        if self.is_leaf(node) {
            if node.is_empty() { Action::Insert(node.with(body)) }
            else { Action::Internalize(node.id, Pending(node.id, body)) }
        } else {
            node.map_quadrant(body.position.clone(), move |idx: Index, q: Quadrant| {
                match self.node(&idx) {
                    Some(child) => self.action(child, body),
                    None => Action::Insert(Node::new(idx, q.space().clone(), Some(body))),
                }
            })
        }
    }
}

// Node //////////////////////////////////////////////////////////////////////
//
// Represents a node in the BHTree. Since the BHTree uses indices to relate
// nodes together, each node must minimally be aware of its own index. The
// indices of related nodes can be calculated in constant time and thus
// do not need to be stored.

// TODO: Consider if its possible to use an enum

#[derive(Clone, Debug)]
struct Node {
    id: Index,
    space: Rect,
    body: Option<Body>,
}

// TODO: needs testing
impl Node {
    /// Creates a new node.
    fn new(id: Index, space: Rect, body: Option<Body>) -> Node {
        Node { id, space, body }
    }

    /// Creates a copy with the given body.
    fn with(&self, body: Body) -> Node {
        Node { id: self.id, space: self.space.clone(), body: Some(body) }
    }

    /// Returns true if the node has no body.
    fn is_empty(&self) -> bool {
        self.body.is_none()
    }

    /// Index of the parent node.
    fn parent(&self) -> Option<Index> {
        if self.id == 0 { None }
        else { Some((self.id - 1) / 4) }
    }

    /// Index of the north west child.
    fn nw(&self) -> Index { 4 * self.id + 1 }

    /// Index of the north east child.
    fn ne(&self) -> Index { 4 * self.id + 2 }

    /// Index of the south west child.
    fn sw(&self) -> Index { 4 * self.id + 3 }

    /// Index of the south east child.
    fn se(&self) -> Index { 4 * self.id + 4 }

    /// Moves the body into a new child node and returns it, if it exists.
    fn child_from_self(&mut self) -> Option<Node> {
        let body = match self.body.take() {
            None => return None,
            Some(body) => body,
        };

        let child = self.map_quadrant(body.position.clone(), move |idx, q| {
            Node::new(idx, q.space().clone(), Some(body))
        });

        Some(child)
    }

    /// Finds quadrant containing the given point and passes it together with
    /// its index to the function f. Panics if the point is out of bounds.
    fn map_quadrant<U, F>(&self, point: Point, f: F) -> U
    where F: FnOnce(Index, Quadrant) -> U {
        let quadrant = self.space.quadrant(&point).unwrap_or_else(|err| {
            panic!("Couldn't find quadrant: {}", err.kind());
        });

        match quadrant {
            NW(space) => f(self.nw(), NW(space)),
            NE(space) => f(self.ne(), NE(space)),
            SW(space) => f(self.sw(), SW(space)),
            SE(space) => f(self.se(), SE(space)),
        }
    }
}

// Tests /////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use geometry::types::Point;
    use geometry::types::Rect;
    use geometry::types::Vector;
    use physics::barneshut::BHTree;
    use physics::types::Body;

    #[test]
    fn create_tree() {
        let space = Rect::new(0.0, 0.0, 10, 10);
        let mut tree = BHTree::new(space);
        tree.add(Body::new(1.0, Point::new(1.0, 2.0), Vector::zero()));
        tree.add(Body::new(1.0, Point::new(6.0, 8.0), Vector::zero()));
        tree.add(Body::new(1.0, Point::new(4.0, 4.0), Vector::zero()));
        println!("\nRESULTS ---------------------------------\n");
        for (idx, node) in tree.nodes {
            println!("NODE: {:?}: {:?}", idx, node.body);
        }
        println!("\n");
    }
}
