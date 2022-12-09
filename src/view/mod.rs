use winit::event::{KeyboardInput, ModifiersState, WindowEvent};

use crate::{model::NodeId, presenter::Presenter};

mod main_view;
mod tree_mode;

struct ViewState {
    presenter: Presenter,
    cur_node: NodeId,
}

impl ViewState {
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
}

trait Mode {
    fn process_key(
        &mut self,
        input: &KeyboardInput,
        mods: &ModifiersState,
        view_state: &mut ViewState,
    ) -> Option<Box<dyn Mode>>;

    fn name(&self) -> &'static str;
}

pub use main_view::View;
