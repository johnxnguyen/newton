use geometry::types::Point;
use geometry::types::Rect;
use geometry::types::Quadrant::{NW, NE, SW, SE};
use super::types::Body;
use std::collections::HashMap;
use geometry::types::Vector;

// TODO: NewType this
type Index = i32;

enum Changes {
    None,
    Insert(Node),
    Internalize(Index, Pending),
}

/// Represents a pending state. Body needs to be added at Index
struct Pending(Index, Body);

#[derive(Debug)]
pub struct BHTree {
    nodes: HashMap<Index, Node>,
}

impl BHTree {
    /// Initialized the tree with a root node.
    fn new(space: Rect) -> BHTree {
        let mut nodes: HashMap<i32, Node> = HashMap::new();
        let root = Node::new(0, space, None);
        nodes.insert(root.id, root);
        BHTree { nodes }
    }

    /// Returns true if the tree is empty.
    fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Borrows the root node.
    fn root(&self) -> &Node {
        match self.node(&0) {
            Some(root) => root,
            None => unreachable!("There should always be a root node."),
        }
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
    fn add(&mut self, body: Body) {
        self.insert(Pending(0, body));
    }

    /// Inserts the given body into the tree at the given node.
    fn insert(&mut self, pending: Pending) {
        let mut changes = Changes::None;
        {
            // inspect the tree to find the necessary changes
            changes = self.changes(self.node(&pending.0).unwrap(), pending.1);
        }
        {
            match self.process(changes) {
                Some(pending) => self.insert(pending),
                _ => (),
            }
        }
    }

    /// Processes the change by either inserting, moving, or doing nothing.
    fn process(&mut self, change: Changes) -> Option<Pending> {
        match change {
            Changes::Insert(node) => {
                self.nodes.insert(node.id, node);
                None
            },
            Changes::Internalize(id, pending) => {
                self.internalize(id);
                Some(pending)
            },
            Changes::None => {
                None
            },
        }
    }


    /// Internalizes the node at the given index by taking the node's body
    /// and inserting it in the appropriate child.
    fn internalize(&mut self, id: Index) {
        let mut node = self.nodes.remove(&id).expect("Where's the node?!");
        debug_assert!(self.is_leaf(&node), "Can't internalize an internal node");

        let child = node.child_from_self().expect("Where's the child?!");
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
    fn changes(&self, node: &Node, body: Body) -> Changes {
        debug_assert!(node.space.contains(&body.position));

        if self.is_leaf(node) {
            if node.is_empty() {
                Changes::Insert(node.with(body))
            } else {
                let pending = Pending(node.id, body);
                Changes::Internalize(node.id, pending)
            }
        } else {
            // TODO: we could use Result for this.
            match node.space.which_quadrant(&body.position) {
                None => unreachable!("There must be a quadrant!"),
                Some(quadrant) => match quadrant {
                    NW(subspace) => match self.node(&node.nw()) {
                        Some(nw) => self.changes(nw, body),
                        None => {
                            Changes::Insert(Node::new(node.nw(), subspace, Some(body)))
                        },
                    },
                    NE(subspace) => match self.node(&node.ne()) {
                        Some(ne) => self.changes(ne, body),
                        None => {
                            Changes::Insert(Node::new(node.ne(), subspace, Some(body)))
                        },
                    },
                    SW(subspace) => match self.node(&node.sw()) {
                        Some(sw) => self.changes(sw, body),
                        None => {
                            Changes::Insert(Node::new(node.sw(), subspace, Some(body)))
                        },
                    },
                    SE(subspace) => match self.node(&node.se()) {
                        Some(se) => self.changes(se, body),
                        None => {
                            Changes::Insert(Node::new(node.se(), subspace, Some(body)))
                        },
                    },
                },
            }
        }
    }
}

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

    /// Moves the self body into a new child node, if it exists.
    fn child_from_self(&mut self) -> Option<Node> {
        let body = match self.body.take() {
            None => return None,
            Some(body) => body,
        };

        let warning = "There must be a quadrant for body's node.";
        let quadrant = self.space.which_quadrant(&body.position).expect(warning);

        let child = match quadrant {
            NW(space) => Node::new(self.nw(), space, Some(body)),
            NE(space) => Node::new(self.ne(), space, Some(body)),
            SW(space) => Node::new(self.sw(), space, Some(body)),
            SE(space) => Node::new(self.se(), space, Some(body)),
        };

        Some(child)
    }
}

#[test]
fn create_tree() {
    let space = Rect::new(0.0, 0.0, 10.0, 10.0);
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