use std::collections::HashSet;

use ropey::Rope;

use super::{motion::Command, Mode};
use crate::{model::NodeId, presenter::Presenter, view::insert_mode::InsertMode};

// TODO: should this just be part of the presenter?
pub struct ViewState {
    pub presenter: Presenter,
    pub cur_node: NodeId,
    pub cur_edit: Option<(usize, Rope)>,
    pub cur_cmd: Option<(usize, Rope)>,
    pub prev_error: Option<anyhow::Error>,
    pub folded_nodes: HashSet<NodeId>,
}

impl ViewState {
    pub fn new(presenter: Presenter) -> ViewState {
        ViewState {
            cur_node: presenter.model().root_id(),
            presenter,
            cur_edit: None,
            cur_cmd: None,
            prev_error: None,
            folded_nodes: HashSet::new(),
        }
    }

    pub fn move_to_next_child(&mut self) {
        if let Some(next_child) = self
            .presenter
            .model()
            .next_child(self.cur_node)
            .or_else(|| {
                self.presenter
                    .model()
                    .node(self.cur_node)
                    .children
                    .first()
                    .copied()
            })
        {
            self.cur_node = next_child;
        }
    }

    pub fn move_to_prev_child(&mut self) {
        if let Some(prev_child) = self.presenter.model().prev_child(self.cur_node) {
            self.cur_node = prev_child;
        } else {
            // TODO: sometimes when the current root is not the tree global root we
            // should be exiting up a level and changing the current root, but we don't
            // because prev_child() doesn't stop until it gets to the global root.
            // Some kind of tree slice could probably fix this?
            self.exit_node();
        }
    }

    pub fn enter_node(&mut self) {
        if let Some(enter_node) = self.presenter.model().node(self.cur_node).children.first() {
            self.cur_node = *enter_node;
        }
    }

    pub fn exit_node(&mut self) {
        if let Some(exit_node) = self.presenter.model().node(self.cur_node).parent() {
            if self.cur_node == self.presenter.current_root() {
                self.presenter.set_current_root(exit_node);
            }
            self.cur_node = exit_node;
        }
    }

    pub fn begin_editing(&mut self, start_at_end: bool) {
        assert!(self.cur_edit.is_none());
        let text = &self.presenter.model().node(self.cur_node).text;
        self.cur_edit = Some((
            if start_at_end { text.len() } else { 0 },
            Rope::from_str(text),
        ));
    }

    pub fn finish_editing(&mut self) {
        let (_, new_text) = self.cur_edit.take().expect("was editing");
        self.presenter
            .update_node_text(self.cur_node, new_text.to_string());
    }

    pub fn begin_command_edit(&mut self) {
        assert!(self.cur_cmd.is_none());
        self.cur_cmd = Some((0, Rope::new()));
    }

    pub fn abort_command_edit(&mut self) {
        self.cur_cmd = None;
    }

    pub fn process_command(&mut self) {
        let (_, cmd_rope) = self.cur_cmd.take().expect("was editing a command");
        match self.presenter.process_command(cmd_rope.into()) {
            Ok(()) => {}
            Err(e) => {
                self.prev_error = Some(e);
            }
        }
    }

    pub fn process_normal_cmd(&mut self, cmd: Command) -> Option<Box<dyn Mode>> {
        use super::motion::*;
        let (cursor_index, buf) = self.cur_edit.as_mut().unwrap();
        match cmd {
            Command::Move(m) => {
                *cursor_index = m.range(buf, *cursor_index, 1, &mut None).end;
            }
            Command::ReplaceChar(c) => {
                buf.remove(*cursor_index..*cursor_index + 1);
                buf.insert_char(*cursor_index, c);
            }
            Command::Change(m) => {
                let r = m.range(buf, *cursor_index, 1, &mut None);
                self.presenter.copy_str(buf.slice(r.clone()).to_string());
                buf.remove(r);
                return Some(Box::new(InsertMode));
            }
            Command::Delete(m) => {
                let r = m.range(buf, *cursor_index, 1, &mut None);
                self.presenter.copy_str(buf.slice(r.clone()).to_string());
                buf.remove(r);
            }
            Command::Copy(m) => {
                let r = m.range(buf, *cursor_index, 1, &mut None);
                self.presenter.copy_str(buf.slice(r).to_string());
            }
            Command::Put { consume: _ } => {
                if let Some(s) = self.presenter.pop_snip_str() {
                    buf.insert(*cursor_index, &s);
                }
            }
            Command::Insert { at, new_line } => {
                if let Some(at) = at {
                    *cursor_index = at.range(buf, *cursor_index, 1, &mut None).end;
                }
                if new_line {
                    *cursor_index += 1;
                    buf.insert_char(*cursor_index, '\n');
                    *cursor_index += 1;
                }
                return Some(Box::new(InsertMode));
            }
        }
        None
    }

    pub fn toggle_folded(&mut self) {
        if !self
            .presenter
            .model()
            .node(self.cur_node)
            .children
            .is_empty()
            && !self.folded_nodes.remove(&self.cur_node)
        {
            self.folded_nodes.insert(self.cur_node);
        }
    }
}
