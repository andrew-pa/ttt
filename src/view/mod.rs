use ropey::Rope;
use winit::event::{KeyboardInput, ModifiersState, WindowEvent};

use crate::{model::NodeId, presenter::Presenter};

mod edit_mode;
mod insert_mode;
mod main_view;
mod motion;
mod tree_mode;

struct ViewState {
    presenter: Presenter,
    cur_node: NodeId,
    cur_edit: Option<(usize, Rope)>,
}

impl ViewState {
    pub fn new(presenter: Presenter) -> ViewState {
        ViewState {
            cur_node: presenter.model().root_id(),
            presenter,
            cur_edit: None,
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
        if let Some(prev_child) = self
            .presenter
            .model()
            .prev_child(self.cur_node)
            .or_else(|| self.presenter.model().node(self.cur_node).parent())
        {
            self.cur_node = prev_child;
        }
    }

    pub fn enter_node(&mut self) {
        if let Some(enter_node) = self.presenter.model().node(self.cur_node).children.first() {
            self.cur_node = *enter_node;
        }
    }

    pub fn exit_node(&mut self) {
        if let Some(exit_node) = self.presenter.model().node(self.cur_node).parent() {
            self.cur_node = exit_node;
        }
    }

    pub fn begin_editing(&mut self) {
        assert!(self.cur_edit.is_none());
        self.cur_edit = Some((
            0,
            Rope::from_str(&self.presenter.model().node(self.cur_node).text),
        ));
    }

    pub fn finish_editing(&mut self) {
        let (_, new_text) = self.cur_edit.take().expect("was editing");
        self.presenter
            .update_node_text(self.cur_node, new_text.to_string());
    }
}

trait Mode {
    fn process_key(
        &mut self,
        input: &KeyboardInput,
        mods: &ModifiersState,
        view_state: &mut ViewState,
    ) -> Option<Box<dyn Mode>>;

    fn process_char(&mut self, c: char, mods: &ModifiersState, view_state: &mut ViewState) {}

    fn name(&self) -> &'static str;
}

pub use main_view::View;
