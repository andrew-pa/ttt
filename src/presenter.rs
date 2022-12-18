use crate::model::{NodeId, Tree, ROOT_PARENT_ID};

pub struct Presenter {
    tree: Tree,
    current_root: NodeId,
    snip_stack_nodes: Vec<NodeId>,
    snip_stack_strs: Vec<String>,
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
            current_root: root,
            snip_stack_nodes: Vec::new(),
            snip_stack_strs: Vec::new(),
        }
    }

    pub fn model(&self) -> &Tree {
        &self.tree
    }

    pub fn current_root(&self) -> NodeId {
        self.current_root
    }

    pub fn set_current_root(&mut self, new_root: NodeId) {
        assert!(self.tree.nodes.contains_key(&new_root));
        self.current_root = new_root;
    }

    pub fn insert_node_in_parent(&mut self, cur_node: NodeId) -> Option<NodeId> {
        if let Some(parent) = self.tree.node(cur_node).parent() {
            Some(self.tree.insert_node(String::new(), parent, cur_node))
        } else {
            None
        }
    }

    pub fn insert_node_as_child(&mut self, cur_node: NodeId) -> NodeId {
        self.tree.add_node(String::new(), cur_node)
    }

    pub fn delete_node(&mut self, cur_node: NodeId) -> Option<NodeId> {
        let p = self.tree.node(cur_node).parent();
        if p.is_some() {
            self.tree.cut_node(cur_node);
            self.snip_stack_nodes.push(cur_node);
            p
        } else {
            None
        }
    }

    pub fn copy_node(&mut self, cur_node: NodeId) {
        self.snip_stack_nodes.push(cur_node);
    }

    fn move_or_clone_node_from_top_of_snips(
        &mut self,
        consume: bool,
        parent: NodeId,
        after: Option<NodeId>,
    ) -> Option<NodeId> {
        if consume {
            self.snip_stack_nodes.pop().map(|n| {
                self.tree
                    .reparent_node(n, parent, after.map(|n| (n, false)));
                n
            })
        } else {
            self.snip_stack_nodes
                .last()
                .map(|n| self.tree.clone_node(*n, parent, after))
        }
    }

    pub fn put_node(&mut self, cur_node: NodeId, consume: bool, as_child: bool) -> Option<NodeId> {
        if !as_child && self.tree.node(cur_node).parent != ROOT_PARENT_ID {
            let p = self.tree.node(cur_node).parent;
            self.move_or_clone_node_from_top_of_snips(consume, p, Some(cur_node))
        } else {
            self.move_or_clone_node_from_top_of_snips(consume, cur_node, None)
        }
    }

    pub fn swap_node(&mut self, cur_node: NodeId, direction: isize) {
        self.tree.swap_node(cur_node, direction);
    }

    pub fn update_node_text(&mut self, cur_node: usize, new_text: String) {
        self.tree.node_mut(cur_node).text = new_text;
    }

    pub fn copy_str(&mut self, s: String) {
        self.snip_stack_strs.push(s);
    }

    pub fn pop_snip_str(&mut self) -> Option<String> {
        self.snip_stack_strs.pop()
    }

    pub fn top_snip_str(&self) -> Option<&String> {
        self.snip_stack_strs.last()
    }

    /// move the node from being a child of its parent to a sibling of its parent
    pub fn make_child_sibling(&mut self, node: usize) {
        if let Some(parent) = self.tree.node(node).parent() {
            if let Some(grandparent) = self.tree.node(parent).parent() {
                self.tree
                    .reparent_node(node, grandparent, Some((parent, true)));
            }
        }
    }
}
