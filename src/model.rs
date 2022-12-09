use std::collections::HashMap;

pub type NodeId = usize;

pub const ROOT_PARENT_ID: NodeId = 0;

pub struct Node {
    pub id: NodeId,
    pub text: String,
    pub parent: NodeId,
    pub children: Vec<NodeId>,
}

impl Node {
    pub fn parent(&self) -> Option<NodeId> {
        if self.parent == ROOT_PARENT_ID {
            None
        } else {
            Some(self.parent)
        }
    }
}

pub struct Tree {
    next_id: NodeId,
    root_id: NodeId,
    pub nodes: HashMap<NodeId, Node>,
}

impl Tree {
    pub fn new() -> Tree {
        Tree {
            nodes: HashMap::new(),
            next_id: 1,
            root_id: 0,
        }
    }

    pub fn root_id(&self) -> NodeId {
        self.root_id
    }

    pub fn add_node(&mut self, text: String, parent: NodeId) -> NodeId {
        let id = self.next_id;
        self.next_id += 1;
        self.nodes.insert(
            id,
            Node {
                id,
                text,
                parent,
                children: Vec::new(),
            },
        );
        if parent != ROOT_PARENT_ID {
            self.nodes.get_mut(&parent).unwrap().children.push(id);
        } else {
            self.root_id = id;
        }
        id
    }

    pub fn insert_node(&mut self, text: String, parent: NodeId, after: NodeId) -> NodeId {
        let id = self.nodes.len();
        self.nodes.insert(
            id,
            Node {
                id,
                text,
                parent,
                children: Vec::new(),
            },
        );
        if parent != ROOT_PARENT_ID {
            let after_ix = self.nodes[&parent]
                .children
                .iter()
                .enumerate()
                .find_map(|(i, n)| if *n == after { Some(i) } else { None })
                .expect("after is child of parent");
            self.nodes
                .get_mut(&parent)
                .unwrap()
                .children
                .insert(after_ix + 1, id);
        }
        id
    }

    pub fn node(&self, id: NodeId) -> &Node {
        &self.nodes[&id]
    }

    pub fn node_mut(&mut self, id: usize) -> &mut Node {
        self.nodes.get_mut(&id).unwrap()
    }

    /// Move the currently selected node to the next child node. If moving to the next child would
    /// move us past the end of the tree, then None is returned.
    /// This has the effect of moving "down" the tree.
    pub fn next_child(&self, node: NodeId) -> Option<NodeId> {
        // try to move to the child that occurs after the current node in its parent
        let parent = self.node(node).parent;
        if parent != ROOT_PARENT_ID {
            let parent = self.node(parent);
            if let Some((ix, _)) = parent
                .children
                .iter()
                .enumerate()
                .find(|(_, c)| **c == node)
            {
                if ix + 1 < parent.children.len() {
                    return Some(parent.children[ix + 1]);
                }
            }
        }

        None
    }

    /// Move the currently selected node to the previous child node. If moving to the previous child
    /// would move us past the start of the tree, then None is returned.
    /// This has the effect of moving "up" the tree.
    pub fn prev_child(&self, node: NodeId) -> Option<NodeId> {
        let parent = self.node(node).parent;
        if parent != ROOT_PARENT_ID {
            let parent = &self.node(parent);
            if let Some((ix, _)) = parent
                .children
                .iter()
                .enumerate()
                .find(|(_, c)| **c == node)
            {
                if ix != 0 {
                    return Some(parent.children[ix - 1]);
                }
            }
        }

        None
    }
}