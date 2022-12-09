use crate::model::{Tree, ROOT_PARENT_ID, NodeId};

pub struct Presenter {
    tree: Tree,
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

        Presenter { tree }
    }

    pub fn model(&self) -> &Tree {
        &self.tree
    }

    pub fn insert_node_in_parent(&self, cur_node: NodeId) -> NodeId {
        todo!()
    }

    pub fn insert_node(&self, cur_node: NodeId) -> NodeId {
        todo!()
    }

    pub fn delete_node(&self, cur_node: NodeId) {
        todo!()
    }

    pub fn copy_node(&self, cur_node: NodeId) {
        todo!()
    }

    pub fn put_node(&self, cur_node: NodeId, consume: bool) {
        todo!()
    }

    pub fn swap_node(&self, cur_node: NodeId, direction: isize) {
        todo!()
    }
}
