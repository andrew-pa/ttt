use std::collections::HashMap;

use ropey::Rope;
use serde::{Serialize, Serializer};

pub type NodeId = usize;

pub const ROOT_PARENT_ID: NodeId = 0;

fn rope_ser<S: Serializer>(r: &Rope, s: S) -> Result<S::Ok, S::Error> {
    s.serialize_str(&r.to_string())
}

#[derive(Serialize)]
pub struct Node {
    pub id: NodeId,
    #[serde(serialize_with = "rope_ser")]
    pub text: Rope,
    pub parent: NodeId,
    pub children: Vec<NodeId>,
}

#[derive(Serialize)]
pub struct Tree {
    next_id: NodeId,
    pub nodes: HashMap<NodeId, Node>,
}

impl Tree {
    pub fn new() -> Tree {
        Tree {
            nodes: HashMap::new(),
            next_id: 1,
        }
    }

    pub fn add_node(&mut self, text: Rope, parent: NodeId) -> NodeId {
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
        }
        id
    }

    pub fn insert_node(&mut self, text: Rope, parent: NodeId, after: NodeId) -> NodeId {
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
