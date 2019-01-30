use std::collections::HashMap;
use std::fmt;
use std::ops::Range;

use crate::geometry::types::Point;
use crate::geometry::types::Quadrant;
use crate::geometry::types::Quadrant::*;
use crate::geometry::types::Square;
use crate::geometry::types::Vector;

use super::types::Body;

// VirtualBody ///////////////////////////////////////////////////////////////
//
// A virtual body represents an amalgamation of real bodies. Its mass is the
// total sum of the collected masses and its position is the total sum of mass
// weighted positions. To obtain a copy with the position centered on its
// mass, call the `centered()` method.

#[derive(Clone, PartialEq, Debug)]
pub struct VirtualBody {
    mass: f32,
    position: Point,
}

impl fmt::Display for VirtualBody {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let v = self.centered();
        write!(f, "{:?}, ({:?}, {:?})", v.mass, v.position.x, v.position.y)?;
        Ok(())
    }
}

impl From<Body> for VirtualBody {
    fn from(body: Body) -> Self {
        VirtualBody {
            mass: body.mass.value(),
            position: &body.position * body.mass.value(),
        }
    }
}

impl VirtualBody {
    pub fn to_body(&self) -> Body {
        Body::new(self.mass, self.position.clone(), Vector::zero())
    }

    fn new(mass: f32, x: f32, y: f32) -> VirtualBody {
        VirtualBody {
            mass,
            position: Point::new(x, y),
        }
    }

    fn zero() -> VirtualBody {
        VirtualBody::new(0.0, 0.0, 0.0)
    }

    fn centered(&self) -> VirtualBody {
        debug_assert!(self.mass > 0.0, "Mass must be positive. Got {}", self.mass);
        VirtualBody {
            mass: self.mass,
            position: &self.position / self.mass,
        }
    }
}

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
    space: Square,
    nodes: HashMap<Index, Node>,
}

impl BHTree {
    /// Initialized tree with a root node spanning the given space.
    pub fn new(space: Square) -> BHTree {
        let mut nodes: HashMap<Index, Node> = HashMap::new();
        let root = Node::new(0, space.clone(), VirtualBody::zero());
        nodes.insert(root.id, root);
        BHTree { space, nodes }
    }

    /// Inserts the given body into the tree.
    pub fn add(&mut self, body: Body) {
        // TODO: this is just temporary workaround. We should properly
        // handle the case when bodies leave the root space
        if self.space.contains(&body.position) {
            self.insert(Pending(0, body));
        }
    }

    /// Returns a list of virtual bodies for the given body. Each virtual
    /// body is potentially an abstraction of a group of real bodes. The
    /// degree of abstraction is determined by a ratio between the distance
    /// of the virtual body to the given body, and the diameter of the space
    /// containing the virtual body. The diameter may not be more than twice
    /// the distance.
    ///
    /// The virtual bodies are collected as follows. 1) Starting at the root,
    /// we dive depth first until a node that fulfills the condition
    /// mentioned above is reached, or the a leaf is reached. In both cases,
    /// the virtual body at that node is collected. Then we move the cursor
    /// to the next sibling if any, or to the next node. Then the next virtual
    /// body is found by repeating this process.
    ///
    pub fn virtual_bodies(&self, body: &Body) -> Vec<VirtualBody> {
        let mut result = vec![];
        let mut traverser = self.preorder();

        let condition = |node: &Node| {
            let pos = node.body.centered().position;
            let dist = body.position.distance_to(&pos);
            node.space.diameter() / dist < 2.0
        };

        loop {
            match traverser.next() {
                None => break,
                Some(node) => {
                    if condition(&node) {
                        traverser.skip_children();
                        result.push(node.body.centered());

                    } else if self.is_leaf(node) {
                        // subtract the body form the virtual body
                        let mut virtual_body = node.body.clone();
                        virtual_body.mass -= body.mass.value();
                        virtual_body.position -= &body.position * body.mass.value();

                        if virtual_body.mass <= 0.0 {
                            // this is reachable when there are two
                            // adjacent points in separate quadrants.
                        } else {
                            result.push(virtual_body.centered());
                        }
                    }
                },
            }
        }

        result
    }

    /// Borrows the node for the given index, if it exists.
    fn node(&self, idx: Index) -> Option<&Node> {
        self.nodes.get(&idx)
    }

    /// Returns true if the given node is a leaf.
    fn is_leaf(&self, node: &Node) -> bool {
        node.children().all(|n| self.node(n).is_none())
    }

    /// Inserts the given body into the tree at the given node.
    fn insert(&mut self, pending: Pending) {
        let action: Action;
        {
            // inspect the tree to find the necessary action
            let node = self.node(pending.0).expect("Expected a node");
            let body = pending.1;
            action = self.action(node, body);
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
                // go up from node to root, update virtual bodies along the way
                for idx in node.ancestors() {
                    // TODO: check if we can edit in place
                    let mut parent = self.nodes.remove(&idx).expect("Expected a parent");
                    parent.body.mass += node.body.mass;
                    parent.body.position += node.body.position.clone();
                    self.nodes.insert(idx, parent);
                }

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
        let node = self.nodes.remove(&id).expect("Node doesn't exist.");
        debug_assert!(self.is_leaf(&node), "Can't internalize an internal node");

        let child = node.child_from_self();

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
            if node.is_empty() || node.space.is_unit_rect() {
                Action::Insert(node.with(body))
            } else {
                Action::Internalize(node.id, Pending(node.id, body))
            }
        } else {
            node.map_quadrant(body.position.clone(), move |idx: Index, q: Quadrant| {
                match self.node(idx) {
                    Some(child) => self.action(child, body),
                    None => Action::Insert(Node::new(idx, q.space().clone(), VirtualBody::from(body))),
                }
            })
        }
    }

    /// Returns a preorder traversal iterator starting at the root node.
    fn preorder(&self) -> PreorderTraverser {
        self.preorder_at(0)
    }

    /// Returns a preorder traversal iterator starting at given index.
    fn preorder_at(&self, idx: Index) -> PreorderTraverser {
        PreorderTraverser::new(self, idx)
    }

}

// PreorderTraverser /////////////////////////////////////////////////////////
//
// An iterator over nodes of a tree in preorder (Root, child 1, ..., child n).

struct PreorderTraverser<'a> {
    tree: &'a BHTree,
    first: Option<&'a Node>,
    stack: Vec<ChildIterator>,
}

impl<'a> Iterator for PreorderTraverser<'a> {
    type Item = &'a Node;

    fn next(&mut self) -> Option<Self::Item> {
        // first element
        if let Some(node) = self.first.take() {
            self.stack.push(node.children());
            return Some(node);
        }

        // next element
        while let Some(mut iter) = self.stack.pop() {
            while let Some(idx) = iter.next() {
                if let Some(node) = self.tree.node(idx) {
                    self.stack.push(iter);
                    self.stack.push(node.children());
                    return Some(node)
                }
            }
        }
        None
    }
}

impl<'a> PreorderTraverser<'a> {
    /// Returns a new iterator at the node for the given index.
    fn new(tree: &'a BHTree, idx: Index) -> PreorderTraverser<'a> {
        let node = tree.node(idx).expect("Node doesn't exist");
        PreorderTraverser { tree, first: Some(node), stack: Vec::new() }
    }

    fn skip_children(&mut self) {
        self.stack.pop();
    }
}

// Node //////////////////////////////////////////////////////////////////////
//
// Represents a node in the BHTree. Since the BHTree uses indices to relate
// nodes together, each node must minimally be aware of its own index. The
// indices of related nodes can be calculated in constant time and thus
// do not need to be stored.

#[derive(Clone, Debug)]
struct Node {
    pub id: Index,
    pub space: Square,
    pub body: VirtualBody,
}

impl fmt::Display for Node {
    /// Prints "#(id) (x, y, size) (body)". Eg: "#1 (0, 0, 4) M(1) P(3, 4) V(3, 6)"
    fn fmt(&self, f: &mut fmt::Formatter<>) -> Result<(), fmt::Error> {
        write!(f, "#{}\t({}, {}, ", self.id, self.space.origin.x, self.space.origin.y)?;
        write!(f, "{}) ", self.space.size)?;
        write!(f, "{}", self.body)?;
        Ok(())
    }
}

impl Node {
    /// Creates a new node.
    fn new(id: Index, space: Square, body: VirtualBody) -> Node {
        Node { id, space, body }
    }

    /// Creates a copy of the node after adding the given body.
    fn with(&self, body: Body) -> Node {
        let mut body = VirtualBody::from(body);
        body.mass += self.body.mass;
        body.position += self.body.position.clone();
        Node::new(self.id, self.space.clone(), body)
    }

    /// Returns true if the node has no body.
    fn is_empty(&self) -> bool {
        self.body == VirtualBody::zero()
    }

    /// Index of the north west child.
    fn nw(&self) -> Index {
        4 * self.id + 1
    }

    /// Index of the north east child.
    fn ne(&self) -> Index {
        4 * self.id + 2
    }

    /// Index of the south west child.
    fn sw(&self) -> Index {
        4 * self.id + 3
    }

    /// Index of the south east child.
    fn se(&self) -> Index {
        4 * self.id + 4
    }

    /// Moves the body into a new child node and returns it, if it exists.
    fn child_from_self(&self) -> Node {
        // TODO: check that this is a leaf
        let body = self.body.clone();
        let child = self.map_quadrant(body.centered().position, move |idx, q| {
            Node::new(idx, q.space().clone(), body)
        });

        child
    }

    /// Finds quadrant containing the given point and passes it together with
    /// its index to the function f. Panics if the point is out of bounds.
    fn map_quadrant<U, F>(&self, point: Point, f: F) -> U
    where F: FnOnce(Index, Quadrant) -> U {
        let quadrant = self.space.quadrant(&point).unwrap_or_else(|err| {
            panic!("Couldn't find quadrant. Reason: {} Got {:?}", err.kind(), point);
        });

        match quadrant {
            NW(space) => f(self.nw(), NW(space)),
            NE(space) => f(self.ne(), NE(space)),
            SW(space) => f(self.sw(), SW(space)),
            SE(space) => f(self.se(), SE(space)),
        }
    }

    /// Returns an iterator over the ancestor indices.
    fn ancestors(&self) -> AncestorIterator {
        AncestorIterator::new(self.id)
    }

    /// Returns an iterator over the children indices.
    fn children(&self) -> ChildIterator {
        ChildIterator::new(self.id)
    }
}

// AncestorIterator /////////////////////////////////////////////////////////////
//
// An iterator over the ancestor indices up towards the root index.

struct AncestorIterator {
    current: Index,
}

impl Iterator for AncestorIterator {
    type Item = Index;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current <= 0 {
            None
        } else {
            self.current -= 1;
            self.current /= 4;
            Some(self.current)
        }
    }
}

impl AncestorIterator {
    fn new(start: Index) -> AncestorIterator {
        AncestorIterator { current: start }
    }
}

// ChildIterator /////////////////////////////////////////////////////////////
//
// An iterator over child indices in order (child 1, ..., child n).

struct ChildIterator(Range<Index>);

impl Iterator for ChildIterator {
    type Item = Index;
    fn next(&mut self) -> Option<Self::Item> { self.0.next() }
}

impl ChildIterator {
    /// Returns a new iterator for children of the given index.
    fn new(parent: Index) -> ChildIterator {
        let start = 4 * parent + 1;
        ChildIterator(start..(start + 4))
    }
}

// Tests /////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use crate::geometry::types::Point;
    use crate::geometry::types::Square;
    use crate::geometry::types::Vector;
    use crate::physics::barneshut::BHTree;
    use crate::physics::barneshut::Index;
    use crate::physics::types::Body;
    use crate::physics::barneshut::VirtualBody;
    use crate::physics::barneshut::Pending;
    use crate::physics::barneshut::ChildIterator;
    use crate::physics::barneshut::AncestorIterator;

    // helpers
    /// Returns a descriptive string of the current tree state. Only existing
    /// nodes are printed with their id and body.
    fn report(tree: &BHTree) -> String {
        tree.preorder().fold(String::new(), |acc, n| {
            if tree.is_leaf(n) {
                let v = n.body.centered();
                acc + &format!("#{}\t({}, {})\n", n.id, v.position.x, v.position.y)
            } else {
                acc + &format!("#{}\n", n.id)
            }
        })
    }

    fn body(mass: f32, x: f32, y: f32) -> Body {
        Body::new(mass, Point::new(x, y), Vector::zero())
    }

    fn virtual_body(mass: f32, x: f32, y: f32) -> VirtualBody {
        VirtualBody {
            mass,
            position: Point::new(x, y),
        }
    }

    ///                           []
    ///              _____________|_______________
    ///            /       /             \        \
    ///           X       []            [C]       X
    ///              _____|_____
    ///            /    /  \    \
    ///           X   [A]  [B]  X
    ///
    fn small_tree() -> BHTree {
        let space = Square::new(-8.0, -8.0, 4);

        let mut tree = BHTree::new(space);
        tree.add(body(2.0, 6.0, 7.0));    // A
        tree.add(body(3.6, 1.0, 2.0));    // B
        tree.add(body(1.5, -4.0, -4.0));  // C
        tree
    }

    ///                           []
    ///              _____________|_______________
    ///            /       /             \        \
    ///          []       C              []       [H]
    ///     _____|____              _____|____
    ///    /   /   \  \           /   /   \   \
    ///  X   [A]  [B]  X         []  X   [G]   X
    ///                      ____|____
    ///                    /   /  \   \
    ///                   X   X   []  [F]
    ///                       ____|____
    ///                      /  /   \  \
    ///                   [D] [E]   X   X
    ///
    fn medium_tree() -> BHTree {
        let space = Square::new(0.0, 0.0, 5);

        let mut tree = BHTree::new(space);
        tree.add(body(2.0, 10.0, 25.0));    // A
        tree.add(body(1.0, 6.0, 20.0));     // B
        tree.add(body(4.0, 31.0, 31.0));    // C
        tree.add(body(3.0, 1.0, 10.0));     // D
        tree.add(body(2.5, 3.0, 11.0));     // E
        tree.add(body(1.5, 5.0, 10.0));     // F
        tree.add(body(2.0, 1.0, 1.0));      // G
        tree.add(body(3.5, 20.0, 10.0));    // H
        tree
    }

    #[test]
    fn tree_adds_bodies() {
        let space = Square::new(-8.0, -8.0, 4);
        let mut tree = BHTree::new(space);

        tree.add(body(2.0, 1.0, 2.0));
        assert_eq!(report(&tree),
                   "#0\t(1, 2)\n".to_string());

        tree.add(body(1.0, 6.0, 8.0));
        assert_eq!(report(&tree),
                   "#0\n\
                    #2\n\
                    #10\t(6, 8)\n\
                    #11\t(1, 2)\n".to_string());

        tree.add(body(4.0, -4.0, -4.0));
        assert_eq!(report(&tree),
                   "#0\n\
                    #2\n\
                    #10\t(6, 8)\n\
                    #11\t(1, 2)\n\
                    #3\t(-4, -4)\n".to_string());

        println!("\nRESULTS ---------------------------------\n");
        println!("{}", report(&tree));
    }

    #[test]
    #[should_panic]
    fn tree_panics_if_body_out_of_bounds() {
        // given
        let mut tree = BHTree::new(Square::new(-8.0, -8.0, 4));
        let body = Body::new(1.0, Point::new(0.0, 8.1), Vector::zero());

        // when, then
        tree.insert(Pending(0, body));
    }

    #[test]
    fn tree_internalizes() {
        // given
        let mut sut = small_tree();
        assert!(sut.is_leaf(sut.node(3).unwrap()));

        // when
        sut.internalize(3);

        // then
        assert!(!sut.is_leaf(sut.node(3).unwrap()));

        let body = sut.node(13).unwrap().body.centered();
        assert_eq!(VirtualBody::new(1.5, -4.0, -4.0), body);
    }

    #[test]
    fn tree_iterates() {
        // given
        let tree = small_tree();
        let mut sut = tree.preorder();

        // then
        assert_eq!(0,  sut.next().unwrap().id);
        assert_eq!(2,  sut.next().unwrap().id);
        assert_eq!(10, sut.next().unwrap().id);
        assert_eq!(11, sut.next().unwrap().id);
        assert_eq!(3,  sut.next().unwrap().id);
    }

    #[test]
    fn tree_iterator_skips() {
        // given
        let tree = medium_tree();
        let mut sut = tree.preorder();

        // then
        assert_eq!(0, sut.next().unwrap().id);
        assert_eq!(1, sut.next().unwrap().id);
        assert_eq!(6, sut.next().unwrap().id);
        assert_eq!(7, sut.next().unwrap().id);
        assert_eq!(2, sut.next().unwrap().id);
        assert_eq!(3, sut.next().unwrap().id);
        assert_eq!(13, sut.next().unwrap().id);

        // when
        sut.skip_children();

        // then
        assert_eq!(15, sut.next().unwrap().id);

        // when
        sut.skip_children();

        // then
        assert_eq!(4, sut.next().unwrap().id);
    }

    #[test]
    fn tree_is_leaf() {
        // given
        let sut = small_tree();

        // then
        assert!(!sut.is_leaf(sut.node(0).unwrap()));
        assert!(!sut.is_leaf(sut.node(2).unwrap()));

        assert!(sut.is_leaf(sut.node(3).unwrap()));
        assert!(sut.is_leaf(sut.node(10).unwrap()));
        assert!(sut.is_leaf(sut.node(11).unwrap()));
    }

    #[test]
    fn tree_virtual_body() {
        // given
        let tree = small_tree();
        let result = |idx: Index| tree.node(idx).unwrap().body.centered();

        // then

        // the virtual body for A, B, and C is the sum of their weighted positions
        // divided by the sum of their masses. Weighted position is position scaled
        // by mass.

        // A B and C
        let expected = virtual_body(7.1, 1.3521128, 2.1408453);
        assert_eq!(expected, result(0));

        // just C
        let expected = virtual_body(1.5, -4.0, -4.0);
        assert_eq!(expected, result(3));

        // A and B
        let expected = virtual_body(5.6, 2.785714286, 3.785714286);
        assert_eq!(expected, result(2));

        // just A
        let expected = virtual_body(2.0, 6.0, 7.0);
        assert_eq!(expected, result(10));

        // just B
        let expected = virtual_body(3.6, 1.0, 2.0);
        assert_eq!(expected, result(11));
    }
    
    #[test]
    fn tree_virtual_bodies_small() {
        // given
        let sut = small_tree();

        // B
        let body = body(3.6, 1.0, 2.0);

        // when
        let result = sut.virtual_bodies(&body);

        // then
        assert_eq!(2, result.len());

        // A
        assert_eq!(VirtualBody::new(2.0, 6.0, 7.0), result[0]);

        // C
        assert_eq!(VirtualBody::new(1.5, -4.0, -4.0), result[1]);
    }
    
    #[test]
    fn tree_virtual_bodies_medium_1() {
        // given
        let sut = medium_tree();

        // A
        let body = body(2.0, 10.0, 25.0);

        // when
        let result = sut.virtual_bodies(&body);

        // then
        assert_eq!(4, result.len());

        // B
        assert_eq!(VirtualBody::new(1.0, 6.0, 20.0), result[0]);

        // C
        assert_eq!(VirtualBody::new(4.0, 31.0, 31.0), result[1]);

        // D, E, F & G
        assert_eq!(VirtualBody::new(9.0, 2.222222222, 8.277777778), result[2]);

        // H
        assert_eq!(VirtualBody::new(3.5, 20.0, 10.0), result[3]);
    }

    #[test]
    fn tree_virtual_bodies_medium_2() {
        // given
        let sut = medium_tree();

        // G
        let body = body(2.0, 1.0, 1.0);

        // when
        let result = sut.virtual_bodies(&body);

        // then
        assert_eq!(4, result.len());

        // A & B
        assert_eq!(VirtualBody::new(3.0, 8.666666667, 23.333333333), result[0]);

        // C
        assert_eq!(VirtualBody::new(4.0, 31.0, 31.0), result[1]);
        
        // D, E & F
        assert_eq!(VirtualBody::new(7.0, 2.571428571, 10.357142857), result[2]);
        
        // H
        assert_eq!(VirtualBody::new(3.5, 20.0, 10.0), result[3]);
    }

    #[test]
    fn tree_virtual_bodies_does_not_include_given_body() {
        // given
        let mut sut = BHTree::new(Square::new(0.0, 0.0, 2));
        let body = body(1.0, 2.0, 2.0);
        sut.add(body.clone());

        // when, then
        assert_eq!(0, sut.virtual_bodies(&body).len());
    }

    #[test]
    fn tree_virtual_body_subtracts_given_body_from_leaf() {
        // given
        let mut sut = BHTree::new(Square::new(0.0, 0.0, 1));

        // two bodies within same unit
        sut.add(body(0.5, 0.5, 2.0));
        sut.add(body(2.0, 0.5, 1.5));

        // when
        let result = sut.virtual_bodies(&body(0.5, 0.5, 2.0));

        // then
        assert_eq!(1, result.len());
        assert_eq!(VirtualBody::new(2.0, 0.5, 1.5), result[0]);
    }

    #[test]
    fn tree_has_maximum_depth() {
        // given
        let mut sut = BHTree::new(Square::new(0.0, 0.0, 1));

        // when two bodies within same unit
        sut.add(body(0.5, 0.5, 2.0));
        sut.add(body(2.0, 0.5, 1.5));

        // then there is only root & nw child
        assert_eq!(2, sut.nodes.len());

        // and the virtual body contains both bodies
        let body = sut.node(1).unwrap().body.clone();
        assert_eq!(VirtualBody::new(2.5, 1.25, 4.0), body);
        assert_eq!(VirtualBody::new(2.5, 0.5, 1.6), body.centered());
    }

    #[test]
    fn virtual_body_centered() {
        // given, then
        let sut = VirtualBody::new(2.5, 5.0, 7.5);
        assert_eq!(Point::new(2.0, 3.0), sut.centered().position);

        // given, then
        let sut = VirtualBody::new(2.4, -24.6, -4.8);
        assert_eq!(Point::new(-10.25, -2.0), sut.centered().position);

        // given, then
        let sut = VirtualBody::new(14.5, 0.0, 0.0);
        assert_eq!(Point::zero(), sut.centered().position);
    }

    #[test]
    #[should_panic]
    fn virtual_body_centered_zero_mass() {
        // given, when
        VirtualBody::new(0.0, 5.0, 7.5).centered();
    }

    #[test]
    fn ancestor_iterator_iterates() {
        // given
        let mut sut = AncestorIterator { current: 632 };

        // then
        assert_eq!(Some(157), sut.next());
        assert_eq!(Some(39), sut.next());
        assert_eq!(Some(9), sut.next());
        assert_eq!(Some(2), sut.next());
        assert_eq!(Some(0), sut.next());
        assert_eq!(None, sut.next());
    }

    #[test]
    fn child_iterator_iterates() {
        // given
        let mut sut = ChildIterator::new(6);

        // then
        assert_eq!(Some(25), sut.next());
        assert_eq!(Some(26), sut.next());
        assert_eq!(Some(27), sut.next());
        assert_eq!(Some(28), sut.next());
        assert_eq!(None, sut.next());
    }
}
