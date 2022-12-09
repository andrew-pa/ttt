use std::collections::HashMap;

pub type NodeId = usize;

pub const ROOT_PARENT_ID: NodeId = 0;

pub struct Node {
    pub id: NodeId,
    pub text: String,
    pub parent: NodeId,
    pub children: Vec<NodeId>,
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
            root_id: 0
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
}
