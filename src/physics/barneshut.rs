use std::collections::HashMap;
use std::fmt;
use std::ops::Range;

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

    /// Inserts the given body into the tree.
    pub fn add(&mut self, body: Body) {
        self.insert(Pending(0, body));
    }

    /// Borrows the node for the given index, if it exists.
    fn node(&self, idx: Index) -> Option<&Node> {
        self.nodes.get(&idx)
    }

    /// Returns true if the given node is a leaf.
    fn is_leaf(&self, node: &Node) -> bool {
        node.iter().all(|n| self.node(n).is_none())
    }

    /// Inserts the given body into the tree at the given node.
    fn insert(&mut self, pending: Pending) {
        let action: Action;
        {
            // inspect the tree to find the necessary action
            action = self.action(self.node(pending.0).unwrap(), pending.1);
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

    // TODO: Test
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
                match self.node(idx) {
                    Some(child) => self.action(child, body),
                    None => Action::Insert(Node::new(idx, q.space().clone(), Some(body))),
                }
            })
        }
    }

    // TODO: test
    /// Returns the virtual body at the given node. The virtual body is
    /// a body computed by the real bodies contained descendant leaves.
    /// Its mass is the sum of all real body masses, its position is the
    /// center of mass of these bodies, and its velocity is zero.
    fn virtual_body(&self, node: Index) -> Body {
        unimplemented!()
    }

    /// Returns a vector of the leaves.
    fn leaves(&self) -> Vec<&Node> {
        self.preorder().filter(|n| self.is_leaf(*n)).collect()
    }

    /// Returns a preorder traversal iterator.
    fn preorder(&self) -> PreorderTraverser {
        PreorderTraverser::new(self)
    }

    /// Returns a descriptive string of the current tree state. Only existing
    /// nodes are printed with their id and body.
    fn report(&self) -> String {
        self.preorder().fold(String::new(), |acc, n| {
            if let Some(ref b) = n.body {
                acc + &format!("#{}\t({}, {})\n", n.id, b.position.x, b.position.y)
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
            self.stack.push(node.iter());
            return Some(node);
        }

        // next element
        while let Some(mut iter) = self.stack.pop() {
            while let Some(idx) = iter.next() {
                if let Some(node) = self.tree.node(idx) {
                    self.stack.push(iter);
                    self.stack.push(node.iter());
                    return Some(node)
                }
            }
        }
        None
    }
}

impl<'a> PreorderTraverser<'a> {
    /// Returns a new iterator at the root node of the given tree.
    fn new(tree: &'a BHTree) -> PreorderTraverser<'a> {
        let root = tree.node(0).unwrap();
        PreorderTraverser { tree, first: Some(root), stack: Vec::new() }
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
    pub body: Option<Body>,
}

impl fmt::Display for Node {
    /// Prints "#(id) (space) (body)". Eg: "#1 (0, 0, 4, 4) M(1) P(3, 4) V(3, 6)"
    fn fmt(&self, f: &mut fmt::Formatter<>) -> Result<(), fmt::Error> {
        write!(f, "#{}\t({}, {}, ", self.id, self.space.origin.x, self.space.origin.y)?;
        write!(f, "{}, {}) ", self.space.size.width, self.space.size.height)?;
        match &self.body {
            Some(body) => write!(f, "{}", body)?,
            None => write!(f, "None")?,
        }
        Ok(())
    }
}

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
        while curr != 0 {
            let next = parent(curr);
            if next == node.id { return true }
            curr = next;
        }
        false
    }

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

    /// Returns an iterator over the children indices.
    fn iter(&self) -> ChildIterator {
        ChildIterator::new(self.id, 4)
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
    use physics::types::Body;
    use physics::barneshut::Index;

    fn small_tree() -> BHTree {
        let body = |x, y| Body::new(1.0, Point::new(x, y), Vector::zero());
        let space = Rect::new(0.0, 0.0, 10, 10);

        let mut tree = BHTree::new(space);
        tree.add(body(1.0, 2.0));
        tree.add(body(6.0, 8.0));
        tree.add(body(4.0, 4.0));
        tree
    }

    #[test]
    fn tree_adds_bodies() {
        let body = |x, y| Body::new(1.0, Point::new(x, y), Vector::zero());
        let space = Rect::new(0.0, 0.0, 10, 10);

        let mut tree = BHTree::new(space);

        assert_eq!(tree.report(),
                   "#0\n".to_string());

        tree.add(body(1.0, 2.0));
        assert_eq!(tree.report(),
                   "#0\t(1, 2)\n".to_string());

        tree.add(body(6.0, 8.0));
        assert_eq!(tree.report(),
                   "#0\n\
                    #2\t(6, 8)\n\
                    #3\t(1, 2)\n".to_string());

        tree.add(body(4.0, 4.0));
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
        tree.add(body);
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
        assert!(!sut.is_leaf(sut.node(0u32).unwrap()));
        assert!(!sut.is_leaf(sut.node(3u32).unwrap()));

        assert!(sut.is_leaf(sut.node(2u32).unwrap()));
        assert!(sut.is_leaf(sut.node(13u32).unwrap()));
        assert!(sut.is_leaf(sut.node(14u32).unwrap()));
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
}
