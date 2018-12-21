use std::collections::HashMap;
use std::fmt;
use std::ops::Range;

use geometry::types::Point;
use geometry::types::Quadrant;
use geometry::types::Quadrant::*;
use geometry::types::Rect;
use geometry::types::Vector;

use super::types::Body;

// VirtualBody ///////////////////////////////////////////////////////////////
//
// A virtual body represents an amalgamation of real bodies. Its mass is the
// total sum of the collected masses and its position is the center of mass
// of the group.

#[derive(Clone, PartialEq, Debug)]
struct VirtualBody {
    mass: f32,
    position: Point,
}

impl fmt::Display for VirtualBody {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{:?}, ({:?}, {:?})", self.mass, self.position.x, self.position.y)?;
        Ok(())
    }
}

impl From<&Body> for VirtualBody {
    fn from(body: &Body) -> Self {
        VirtualBody {
            mass: body.mass.value(),
            position: body.position.clone(),
        }
    }
}

impl VirtualBody {
    fn new(mass: f32, x: f32, y: f32) -> VirtualBody {
        VirtualBody {
            mass,
            position: Point::new(x, y),
        }
    }

    fn zero() -> VirtualBody {
        VirtualBody::new(0.0, 0.0, 0.0)
    }

    fn weighted_position(&self) -> Point {
        Point::new(self.mass * self.position.x, self.mass * self.position.y)
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

struct Pending(Index, VirtualBody);

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
    /// Initialized tree with a root node spanning the given space.
    pub fn new(space: Rect) -> BHTree {
        let mut nodes: HashMap<Index, Node> = HashMap::new();
        let root = Node::new(0, space, VirtualBody::zero());
        nodes.insert(root.id, root);
        BHTree { nodes }
    }

    /// Inserts the given body into the tree.
    pub fn add(&mut self, body: &Body) {
        self.insert(Pending(0, VirtualBody::from(body)));
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
                    parent.body.position += node.body.weighted_position();
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

    // TODO: Test
    /// Internalizes the node at the given index by taking the node's body
    /// and inserting it in the appropriate child.
    fn internalize(&mut self, id: Index) {
        let mut node = self.nodes.remove(&id).expect("Node doesn't exist.");
        debug_assert!(self.is_leaf(&node), "Can't internalize an internal node");

        let child = node.child_from_self();

        // internal nodes have weighted positions
        node.body.position = node.body.weighted_position();

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
    fn action(&self, node: &Node, body: VirtualBody) -> Action {
        debug_assert!(node.space.contains(&body.position));

        if self.is_leaf(node) {
            if node.is_empty() { Action::Insert(node.with(body)) }
            else { Action::Internalize(node.id, Pending(node.id, body)) }
        } else {
            node.map_quadrant(body.position.clone(), move |idx: Index, q: Quadrant| {
                match self.node(idx) {
                    Some(child) => self.action(child, body),
                    None => Action::Insert(Node::new(idx, q.space().clone(), body)),
                }
            })
        }
    }

    /// Returns the virtual body at the given node. The virtual body is
    /// computed from the real bodies contained descendant leaves.
    /// Its mass is the sum of all real body masses, its position is the
    /// center of mass of these bodies, and its velocity is zero.
    /// If there are no descendant leaves (node is itself an empty leaf),
    /// returns None.
    // TODO: would be nice to separate this for internal nodes and leaves.
    fn virtual_body(&self, node: Index) -> VirtualBody {
        let n = self.node(node).expect("Expected a node");
        debug_assert!(n.body.mass > 0.0, "Mass must be positive");

        if self.is_leaf(n) {
            n.body.clone()
        } else {
            VirtualBody {
                mass: n.body.mass,
                position: &n.body.position / n.body.mass,
            }
        }
    }

    /// Returns a vector of the leaves in order.
    fn leaves(&self) -> Vec<&Node> {
        self.leaves_at(0)
    }

    // TODO: test
    /// Returns a vector of the leaves in order below the given index.
    fn leaves_at(&self, idx: Index) -> Vec<&Node> {
        self.preorder_at(idx).filter(|n| self.is_leaf(*n)).collect()
    }

    /// Returns a preorder traversal iterator starting at the root node.
    fn preorder(&self) -> PreorderTraverser {
        self.preorder_at(0)
    }

    // TODO: test
    /// Returns a preorder traversal iterator starting at given index.
    fn preorder_at(&self, idx: Index) -> PreorderTraverser {
        PreorderTraverser::new(self, idx)
    }

    /// Returns a descriptive string of the current tree state. Only existing
    /// nodes are printed with their id and body.
    fn report(&self) -> String {
        self.preorder().fold(String::new(), |acc, n| {
            if self.is_leaf(n) {
                acc + &format!("#{}\t({}, {})\n", n.id, n.body.position.x, n.body.position.y)
            } else {
                acc + &format!("#{}\n", n.id)
            }
        })
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
    pub id: Index,
    pub space: Rect,
    pub body: VirtualBody,
}

impl fmt::Display for Node {
    /// Prints "#(id) (space) (body)". Eg: "#1 (0, 0, 4, 4) M(1) P(3, 4) V(3, 6)"
    fn fmt(&self, f: &mut fmt::Formatter<>) -> Result<(), fmt::Error> {
        write!(f, "#{}\t({}, {}, ", self.id, self.space.origin.x, self.space.origin.y)?;
        write!(f, "{}, {}) ", self.space.size.width, self.space.size.height)?;
        write!(f, "{}", self.body)?;
        Ok(())
    }
}

impl Node {
    /// Creates a new node.
    fn new(id: Index, space: Rect, body: VirtualBody) -> Node {
        Node { id, space, body }
    }

    /// Creates a copy with the given body.
    fn with(&self, body: VirtualBody) -> Node {
        Node { id: self.id, space: self.space.clone(), body }
    }

    /// Returns true if the node has no body.
    fn is_empty(&self) -> bool {
        self.body == VirtualBody::zero()
    }

    /// Index of the parent node.
    fn parent(&self) -> Option<Index> {
        if self.id == 0 { None }
        else { Some((self.id - 1) / 4) }
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

    /// Returns true if the given node is an ancestor.
    fn is_ancestor(&self, node: &Node) -> bool {
        let parent = |idx: Index| (idx - 1) / 4;
        let mut curr = self.id;
        // ancestor nodes always have a lower index
        while curr > node.id {
            let next = parent(curr);
            if next == node.id { return true }
            curr = next;
        }
        false
    }

    /// Moves the body into a new child node and returns it, if it exists.
    fn child_from_self(&self) -> Node {
        // TODO: check that this is a leaf
        let body = self.body.clone();
        let child = self.map_quadrant(body.position.clone(), move |idx, q| {
            Node::new(idx, q.space().clone(), body)
        });

        child
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

    /// Returns an iterator over the ancestor indices.
    fn ancestors(&self) -> AncestorIterator {
        AncestorIterator::new(self.id)
    }

    /// Returns an iterator over the children indices.
    fn children(&self) -> ChildIterator {
        ChildIterator::new(self.id, 4)
    }
}

// AncestorIterator /////////////////////////////////////////////////////////////

// TODO: Test
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
    /// Returns a new iterator for children of the given index. Degree specifies
    /// number of children at this node.
    fn new(parent: Index, degree: u32) -> ChildIterator {
        let start = degree * parent + 1;
        ChildIterator(start..(start + degree))
    }
}

// Tests /////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use geometry::types::Point;
    use geometry::types::Rect;
    use geometry::types::Vector;
    use physics::barneshut::BHTree;
    use physics::barneshut::Index;
    use physics::types::Body;
    use physics::barneshut::VirtualBody;

    // helpers
    fn body(mass: f32, x: f32, y: f32) -> Body {
        Body::new(mass, Point::new(x, y), Vector::zero())
    }

    fn virtual_body(mass: f32, x: f32, y: f32) -> VirtualBody {
        VirtualBody {
            mass,
            position: Point::new(x, y),
        }
    }

    fn small_tree() -> BHTree {
        let space = Rect::new(0.0, 0.0, 10, 10);

        let mut tree = BHTree::new(space);
        tree.add(&body(1.0, 1.0, 2.0));
        tree.add(&body(1.0, 6.0, 8.0));
        tree.add(&body(1.0, 4.0, 4.0));
        tree
    }

    #[test]
    fn tree_adds_bodies() {
        let space = Rect::new(0.0, 0.0, 10, 10);
        let mut tree = BHTree::new(space);

        assert_eq!(tree.report(),
                   "#0\t(0, 0)\n".to_string());

        tree.add(&body(1.0, 1.0, 2.0));
        assert_eq!(tree.report(),
                   "#0\t(1, 2)\n".to_string());

        tree.add(&body(1.0, 6.0, 8.0));
        assert_eq!(tree.report(),
                   "#0\n\
                    #2\t(6, 8)\n\
                    #3\t(1, 2)\n".to_string());

        tree.add(&body(1.0, 4.0, 4.0));
        assert_eq!(tree.report(),
                   "#0\n\
                    #2\t(6, 8)\n\
                    #3\n\
                    #13\t(1, 2)\n\
                    #14\t(4, 4)\n".to_string());

        println!("\nRESULTS ---------------------------------\n");
        println!("{}", tree.report());
    }

    #[test]
    #[should_panic]
    fn tree_panics_if_body_out_of_bounds() {
        // given
        let mut tree = BHTree::new(Rect::new(0.0, 0.0, 5, 5));
        let body = Body::new(1.0, Point::new(0.0, 5.5), Vector::zero());

        // when, then
        tree.add(&body);
    }

    #[test]
    fn tree_iterates() {
        // given
        let tree = small_tree();
        let mut sut = tree.preorder();

        // then
        assert_eq!(0, sut.next().unwrap().id);
        assert_eq!(2, sut.next().unwrap().id);
        assert_eq!(3, sut.next().unwrap().id);
        assert_eq!(13, sut.next().unwrap().id);
        assert_eq!(14, sut.next().unwrap().id);
    }

    #[test]
    fn tree_is_leaf() {
        // given
        let sut = small_tree();

        // then
        assert!(!sut.is_leaf(sut.node(0).unwrap()));
        assert!(!sut.is_leaf(sut.node(3).unwrap()));

        assert!(sut.is_leaf(sut.node(2).unwrap()));
        assert!(sut.is_leaf(sut.node(13).unwrap()));
        assert!(sut.is_leaf(sut.node(14).unwrap()));
    }

    #[test]
    fn tree_leaves() {
        // given
        let sut = small_tree();

        // when
        let result: Vec<Index> = sut.leaves().iter().map(|n| n.id).collect();
        
        // then
        assert_eq!(vec![2, 13, 14], result);
    }

    #[test]
    fn tree_virtual_body() {
        // given
        let space = Rect::new(0.0, 0.0, 10, 10);
        let mut tree = BHTree::new(space);
        tree.add(&body(2.0, 1.0, 2.0)); // A
        tree.add(&body(4.1, 6.0, 8.0)); // B
        tree.add(&body(3.6, 4.0, 4.0)); // C

        // bodies equate on reference, hence this helper
//        fn assert_eq(lhs: Body, rhs: Body) {
//            assert_eq!(lhs.mass.value(), rhs.mass.value());
//            assert_eq!(lhs.position, rhs.position);
//        }

        // then

        // A B and C
        // (41.0, 51.2) / 9.7 = (4.226804124, 5.278350515)
        let expected = virtual_body(9.7, 4.226804124, 5.278350515);
        assert_eq!(expected, tree.virtual_body(0));

        // just B
        let expected = virtual_body(4.1, 6.0, 8.0);
        assert_eq!(expected, tree.virtual_body(2));

        // A and C
        // (16.4, 18.4) / 5.6 = (2.928571429, 3.285714286)
        let expected = virtual_body(5.6, 2.928571429, 3.285714286);
        assert_eq!(expected, tree.virtual_body(3));

        // just A
        let expected = virtual_body(2.0, 1.0, 2.0);
        assert_eq!(expected, tree.virtual_body(13));

        // just C
        let expected = virtual_body(3.6, 4.0, 4.0);
        assert_eq!(expected, tree.virtual_body(14));
    }

    #[test]
    #[should_panic]
    fn tree_virtual_body_empty_leaf() {
        // given
        let tree = BHTree::new(Rect::new(0.0, 0.0, 2, 2));

        // when
        tree.virtual_body(0);
    }

    #[test]
    fn virtual_body_weighted_position() {
        // given, then
        let sut = VirtualBody::new(3.7, 4.6, 7.5);
        assert_eq!(Point::new(17.02, 27.75), sut.weighted_position());

        // given, then
        let sut = VirtualBody::new(2.1, -24.6, -9.0);
        assert_eq!(Point::new(-51.66, -18.9), sut.weighted_position());

        // given, then
        let sut = VirtualBody::new(14.5, 0.0, 0.0);
        assert_eq!(Point::zero(), sut.weighted_position());
    }
}
