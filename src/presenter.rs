use crate::{
    model::{NodeId, Tree, ROOT_PARENT_ID},
    storage::{self, Storage},
};

use anyhow::Result;

pub struct Presenter {
    tree: Tree,
    storage: Option<Box<dyn Storage>>,
    current_root: NodeId,
    snip_stack_nodes: Vec<NodeId>,
    snip_stack_strs: Vec<String>,
    tree_modified: bool,
    should_exit: bool,
}

impl Presenter {
    pub fn new() -> Result<Presenter> {
        let mut args = std::env::args().skip(1);

        let (tree, storage) = if let Some(path) = args.next() {
            let (t, s) = storage::open_storage(&path)?;
            (t.unwrap_or_default(), Some(s))
        } else {
            (Tree::default(), None)
        };

        Ok(Presenter {
            current_root: tree.root_id(),
            tree,
            storage,
            snip_stack_nodes: Vec::new(),
            snip_stack_strs: Vec::new(),
            should_exit: false,
            tree_modified: false,
        })
    }

    pub fn storage_name(&self) -> Option<String> {
        self.storage.as_ref().map(|s| s.src_name())
    }

    pub fn model(&self) -> &Tree {
        &self.tree
    }

    pub fn current_root(&self) -> NodeId {
        self.current_root
    }

    pub fn should_exit(&self) -> bool {
        self.should_exit
    }

    pub fn tree_modified(&self) -> bool {
        self.tree_modified
    }

    pub fn set_current_root(&mut self, new_root: NodeId) {
        assert!(self.tree.nodes.contains_key(&new_root));
        self.current_root = new_root;
    }

    pub fn insert_node_in_parent(&mut self, cur_node: NodeId) -> Option<NodeId> {
        self.tree_modified = true;
        if let Some(parent) = self.tree.node(cur_node).parent() {
            Some(self.tree.insert_node(String::new(), parent, cur_node))
        } else {
            None
        }
    }

    pub fn insert_node_as_child(&mut self, cur_node: NodeId) -> NodeId {
        self.tree_modified = true;
        self.tree.add_node(String::new(), cur_node)
    }

    pub fn delete_node(&mut self, cur_node: NodeId) -> Option<NodeId> {
        self.tree_modified = true;
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
        self.snip_stack_nodes
            .push(self.tree.clone_node(cur_node, ROOT_PARENT_ID, None));
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
        self.tree_modified = true;
        if !as_child && self.tree.node(cur_node).parent != ROOT_PARENT_ID {
            let p = self.tree.node(cur_node).parent;
            self.move_or_clone_node_from_top_of_snips(consume, p, Some(cur_node))
        } else {
            self.move_or_clone_node_from_top_of_snips(consume, cur_node, None)
        }
    }

    pub fn swap_node(&mut self, cur_node: NodeId, direction: isize) {
        self.tree.swap_node(cur_node, direction);
        self.tree_modified = true;
    }

    pub fn update_node_text(&mut self, cur_node: usize, new_text: String) {
        self.tree.node_mut(cur_node).text = new_text;
        self.tree_modified = true;
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
        self.tree_modified = true;
    }

    pub fn manual_sync(&mut self) -> Result<()> {
        if let Some(s) = self.storage.as_mut() {
            s.sync(&mut self.tree)?;
            self.tree_modified = false;
        }
        Ok(())
    }

    pub fn process_command(&mut self, cmd: String) -> Result<()> {
        let mut parts = cmd.split(' ');
        match parts.next() {
            Some("e") => {
                // TODO: should we sync the previously open tree?
                let (tree, storage) = storage::open_storage(
                    parts.next().ok_or_else(|| anyhow::anyhow!("missing URL"))?,
                )?;
                self.tree = tree.unwrap_or_default();
                self.storage = Some(storage);
                self.tree_modified = false;
                Ok(())
            }
            Some("s") => {
                if let Some(new_path) = parts.next() {
                    let (_, storage) = storage::open_storage(new_path)?;
                    self.storage = Some(storage);
                }

                if let Some(s) = self.storage.as_mut() {
                    s.sync(&mut self.tree)?;
                    self.tree_modified = false;
                }
                Ok(())
            }
            Some("q") => {
                self.should_exit = true;
                Ok(())
            }
            Some(cmd) => Err(anyhow::anyhow!("unknown command: {cmd}")),
            None => Err(anyhow::anyhow!("empty command")),
        }
    }
}
