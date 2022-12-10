use crate::model::{NodeId, Tree, ROOT_PARENT_ID};

pub struct Presenter {
    tree: Tree,
    snip_stack: Vec<NodeId>,
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
            snip_stack: Vec::new(),
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

    pub fn delete_node(&mut self, cur_node: NodeId) -> Option<NodeId> {
        let p = self.tree.node(cur_node).parent();
        if p.is_some() {
            self.tree.cut_node(cur_node);
            self.snip_stack.push(cur_node);
            p
        } else {
            None
        }
    }

    pub fn copy_node(&mut self, cur_node: NodeId) {
        self.snip_stack.push(cur_node);
    }

    pub fn put_node(&mut self, cur_node: NodeId, consume: bool, as_child: bool) -> Option<NodeId> {
        if !as_child && self.tree.node(cur_node).parent != ROOT_PARENT_ID {
            let p = self.tree.node(cur_node).parent;
            if consume {
                self.snip_stack.pop().map(|n| {
                    self.tree.reparent_node(n, p, Some(cur_node));
                    n
                })
            } else {
                self.snip_stack
                    .last()
                    .map(|n| self.tree.clone_node(*n, p, Some(cur_node)))
            }
        } else {
            if consume {
                self.snip_stack.pop().map(|n| {
                    self.tree.reparent_node(n, cur_node, None);
                    n
                })
            } else {
                self.snip_stack
                    .last()
                    .map(|n| self.tree.clone_node(*n, cur_node, None))
            }
        }
    }

    pub fn swap_node(&mut self, cur_node: NodeId, direction: isize) {
        self.tree.swap_node(cur_node, direction);
    }

    pub fn update_node_text(&mut self, cur_node: usize, new_text: String) {
        self.tree.node_mut(cur_node).text = new_text;
    }
}
