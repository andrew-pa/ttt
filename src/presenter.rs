use crate::model::{NodeId, Tree, ROOT_PARENT_ID};

pub struct Presenter {
    tree: Tree,
    copy_stack: Vec<String>,
}

impl Presenter {
    pub fn new() -> Presenter {
        let mut tree = Tree::new();

        let root = tree.add_node("Root node".into(), ROOT_PARENT_ID);
        tree.add_node("Leaf node 1".into(), root);
        tree.add_node("Leaf node 2".into(), root);
        let a = tree.add_node("Interior A".into(), root);
        tree.add_node("Leaf node 2\nwith two lines!".into(), a);
        tree.add_node("Leaf node 3".into(), a);
        tree.add_node("Leaf node 4".into(), root);
        let b = tree.add_node("Interior B".into(), root);
        tree.add_node("Leaf node 5\nwith\nthree lines!".into(), b);
        tree.add_node("Leaf node 6".into(), b);
        tree.add_node("Leaf node 7".into(), root);
        let c = tree.add_node("Interior C".into(), b);
        tree.add_node("Leaf node 8\nwith\nfour\nlines!".into(), c);
        tree.add_node("Leaf node 9".into(), c);
        tree.add_node("Leaf node 10".into(), root);

        Presenter {
            tree,
            copy_stack: Vec::new(),
        }
    }

    pub fn model(&self) -> &Tree {
        &self.tree
    }

    pub fn insert_node_in_parent(&mut self, cur_node: NodeId) -> Option<NodeId> {
        if let Some(parent) = self.tree.node(cur_node).parent() {
            Some(
                self.tree
                    .insert_node(String::from("new node"), parent, cur_node),
            )
        } else {
            None
        }
    }

    pub fn insert_node_as_child(&mut self, cur_node: NodeId) -> NodeId {
        self.tree.add_node(String::from("new node"), cur_node)
    }

    pub fn delete_node(&mut self, cur_node: NodeId) {
        if let Some(text) = self.tree.delete_node(cur_node) {
            self.copy_stack.push(text);
        }
    }

    pub fn copy_node(&mut self, cur_node: NodeId) {
        self.copy_stack.push(self.tree.node(cur_node).text.clone());
    }

    pub fn put_node(&mut self, cur_node: NodeId, consume: bool) {
        if let Some(text) = if consume {
            self.copy_stack.pop()
        } else {
            self.copy_stack.last().cloned()
        } {
            self.tree.add_node(text, cur_node);
        }
    }

    pub fn swap_node(&mut self, cur_node: NodeId, direction: isize) {
        self.tree.swap_node(cur_node, direction);
    }
}
