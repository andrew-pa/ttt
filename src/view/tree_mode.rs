use super::{Mode, ViewState, insert_mode::InsertMode, edit_mode::EditMode};
use crate::{model::ROOT_PARENT_ID, presenter::Presenter};
use winit::event::{ElementState, KeyboardInput, ModifiersState, VirtualKeyCode, WindowEvent};

pub struct TreeMode;

impl super::Mode for TreeMode {
    fn process_key(
        &mut self,
        input: &KeyboardInput,
        mods: &ModifiersState,
        view_state: &mut ViewState,
    ) -> Option<Box<dyn Mode>> {
        if input.state == ElementState::Pressed {
            if mods.contains(ModifiersState::CTRL) {
                match input.virtual_keycode {
                    Some(VirtualKeyCode::J) => {
                        view_state.presenter.swap_node(view_state.cur_node, 1);
                    }
                    Some(VirtualKeyCode::K) => {
                        view_state.presenter.swap_node(view_state.cur_node, -1);
                    }
                    Some(VirtualKeyCode::L) => {
                    }
                    Some(VirtualKeyCode::H) => {
                    }
                    _ => {}
                }
            } else {
                match input.virtual_keycode {
                    Some(VirtualKeyCode::J) => {
                        view_state.move_to_next_child();
                    }
                    Some(VirtualKeyCode::K) => {
                        view_state.move_to_prev_child();
                    }
                    Some(VirtualKeyCode::L) => {
                        view_state.enter_node();
                    }
                    Some(VirtualKeyCode::H) => {
                        view_state.exit_node();
                    }
                    Some(VirtualKeyCode::I) => {
                        return Some(Box::new(InsertMode));
                    }
                    Some(VirtualKeyCode::E) => {
                        return Some(Box::new(EditMode));
                    }
                    Some(VirtualKeyCode::O) if mods.contains(ModifiersState::SHIFT) => {
                        view_state.cur_node = view_state.presenter.insert_node_in_parent(view_state.cur_node);
                        return Some(Box::new(InsertMode));
                    }
                    Some(VirtualKeyCode::O) => {
                        view_state.cur_node = view_state.presenter.insert_node(view_state.cur_node);
                        return Some(Box::new(InsertMode));
                    }
                    Some(VirtualKeyCode::X) => {
                        view_state.presenter.delete_node(view_state.cur_node);
                    }
                    Some(VirtualKeyCode::Y) => {
                        view_state.presenter.copy_node(view_state.cur_node);
                    }
                    Some(VirtualKeyCode::P) => {
                        view_state.presenter.put_node(view_state.cur_node, mods.contains(ModifiersState::SHIFT));
                    }
                    _ => {}
                }
            }
        }
        None
    }

    fn name(&self) -> &'static str {
        "TREE"
    }
}

