use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{self, Write as IoWrite};

pub type NodeId = usize;

pub const ROOT_PARENT_ID: NodeId = 0;

#[derive(Serialize, Deserialize)]
pub struct Node {
    pub id: NodeId,
    pub text: String,
    pub parent: NodeId,
    pub children: Vec<NodeId>,
    #[serde(default)]
    pub struckout: bool,
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

#[derive(Serialize, Deserialize)]
pub struct Tree {
    next_id: NodeId,
    root_id: NodeId,
    pub nodes: HashMap<NodeId, Node>,
}

impl Default for Tree {
    fn default() -> Self {
        let mut t = Tree::new();
        let rid = t.add_node(String::new(), ROOT_PARENT_ID);
        t.set_root_id(rid);
        t
    }
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

    fn set_root_id(&mut self, id: NodeId) {
        assert_eq!(self.root_id, 0);
        self.root_id = id;
    }

    fn create_node(&mut self, text: String, parent: NodeId) -> NodeId {
        let id = self.next_id;
        self.next_id += 1;
        self.nodes.insert(
            id,
            Node {
                id,
                text,
                parent,
                children: Vec::new(),
                struckout: false,
            },
        );
        id
    }

    pub fn add_node(&mut self, text: String, parent: NodeId) -> NodeId {
        let id = self.create_node(text, parent);
        if parent != ROOT_PARENT_ID {
            self.nodes.get_mut(&parent).unwrap().children.push(id);
        }
        id
    }

    pub fn add_node_at_beginning(&mut self, text: String, parent: NodeId) -> NodeId {
        let id = self.create_node(text, parent);
        if parent != ROOT_PARENT_ID {
            self.nodes.get_mut(&parent).unwrap().children.insert(0, id);
        }
        id
    }

    pub fn insert_node(
        &mut self,
        text: String,
        parent: NodeId,
        at: NodeId,
        after_or_before: bool,
    ) -> NodeId {
        let id = self.create_node(text, parent);
        if parent != ROOT_PARENT_ID {
            let at_ix = self.nodes[&parent]
                .children
                .iter()
                .enumerate()
                .find_map(|(i, n)| if *n == at { Some(i) } else { None })
                .expect("at is child of parent");
            self.nodes
                .get_mut(&parent)
                .unwrap()
                .children
                .insert(at_ix + if after_or_before { 1 } else { 0 }, id);
        }
        id
    }

    pub fn node(&self, id: NodeId) -> &Node {
        &self.nodes[&id]
    }

    pub fn node_mut(&mut self, id: usize) -> &mut Node {
        self.nodes.get_mut(&id).unwrap()
    }

    pub fn cut_node(&mut self, node: NodeId) {
        if let Some(parent) = self.node(node).parent() {
            let parent = self.node_mut(parent);
            parent.children.retain(|n| *n != node);
            self.node_mut(node).parent = ROOT_PARENT_ID;
        }
    }

    pub fn reparent_node(
        &mut self,
        node: NodeId,
        new_parent: NodeId,
        next_to: Option<(NodeId, bool)>,
    ) {
        if let Some(parent) = self.node(node).parent() {
            let parent = self.node_mut(parent);
            parent.children.retain(|n| *n != node);
        }

        self.node_mut(node).parent = new_parent;

        if new_parent != ROOT_PARENT_ID {
            if let Some((next_to, before_or_after)) = next_to {
                let nx_to_ix = self.nodes[&new_parent]
                    .children
                    .iter()
                    .enumerate()
                    .find_map(|(i, n)| if *n == next_to { Some(i) } else { None })
                    .expect("next_to is child of parent");
                self.nodes.get_mut(&new_parent).unwrap().children.insert(
                    if before_or_after {
                        nx_to_ix
                    } else {
                        nx_to_ix + 1
                    },
                    node,
                );
            } else {
                self.node_mut(new_parent).children.push(node);
            }
        }
    }

    pub fn clone_node(
        &mut self,
        node: NodeId,
        new_parent: NodeId,
        after: Option<NodeId>,
    ) -> NodeId {
        let new_node = if let Some(after) = after {
            self.insert_node(self.node(node).text.clone(), new_parent, after, true)
        } else {
            self.add_node(self.node(node).text.clone(), new_parent)
        };

        for child in self.node(node).children.clone() {
            self.clone_node(child, new_node, None);
        }

        new_node
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

    pub fn swap_node(&mut self, node: usize, direction: isize) {
        if let Some(parent) = self.node(node).parent() {
            let parent_node = self.node_mut(parent);
            let ix = parent_node
                .children
                .iter()
                .position(|n| *n == node)
                .unwrap();
            let other_ix = ix as isize + direction;
            if (0..parent_node.children.len() as isize).contains(&other_ix) {
                parent_node.children.swap(ix, other_ix as usize);
            }
        }
    }

    pub fn write_markdown<W: IoWrite>(&self, writer: &mut W) -> io::Result<()> {
        self.append_markdown(writer, self.root_id, 0)
    }

    fn append_markdown<W: IoWrite>(
        &self,
        writer: &mut W,
        node_id: NodeId,
        depth: usize,
    ) -> io::Result<()> {
        if let Some(node) = self.nodes.get(&node_id) {
            write!(writer, "{}- ", "  ".repeat(depth),)?;

            if node.struckout {
                writeln!(writer, "~~{}~~", node.text)?;
            } else {
                writeln!(writer, "{}", node.text)?;
            }

            for &child_id in &node.children {
                self.append_markdown(writer, child_id, depth + 1)?;
            }
        }
        Ok(())
    }
}
