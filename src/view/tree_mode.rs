use super::{edit_mode::EditMode, insert_mode::InsertMode, Mode, ViewState};
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
        if input.virtual_keycode.is_none() {
            return None;
        }

        if input.state == ElementState::Pressed {
            if mods.contains(ModifiersState::CTRL) {
                match input.virtual_keycode.unwrap() {
                    VirtualKeyCode::J => {
                        view_state.presenter.swap_node(view_state.cur_node, 1);
                    }
                    VirtualKeyCode::K => {
                        view_state.presenter.swap_node(view_state.cur_node, -1);
                    }
                    VirtualKeyCode::L => {}
                    VirtualKeyCode::H => {}
                    _ => {}
                }
            } else {
                match input.virtual_keycode.unwrap() {
                    VirtualKeyCode::J => {
                        view_state.move_to_next_child();
                    }
                    VirtualKeyCode::K => {
                        view_state.move_to_prev_child();
                    }
                    VirtualKeyCode::L => {
                        view_state.enter_node();
                    }
                    VirtualKeyCode::H => {
                        view_state.exit_node();
                    }
                    VirtualKeyCode::I => {
                        return Some(Box::new(InsertMode));
                    }
                    VirtualKeyCode::E => {
                        return Some(Box::new(EditMode));
                    }
                    VirtualKeyCode::O if mods.contains(ModifiersState::SHIFT) => {
                        if let Some(nn) = view_state
                            .presenter
                            .insert_node_in_parent(view_state.cur_node)
                        {
                            view_state.cur_node = nn;
                            return Some(Box::new(InsertMode));
                        }
                    }
                    VirtualKeyCode::O => {
                        view_state.cur_node = view_state
                            .presenter
                            .insert_node_as_child(view_state.cur_node);
                        return Some(Box::new(InsertMode));
                    }
                    VirtualKeyCode::X => {
                        let nc = view_state.presenter.model().next_child(view_state.cur_node);
                        if let Some(nn) = view_state.presenter.delete_node(view_state.cur_node) {
                            view_state.cur_node = nc.unwrap_or(nn);
                        }
                    }
                    VirtualKeyCode::Y => {
                        view_state.presenter.copy_node(view_state.cur_node);
                    }
                    VirtualKeyCode::P => {
                        if let Some(nn) = view_state
                            .presenter
                            .put_node(view_state.cur_node, mods.contains(ModifiersState::SHIFT), mods.contains(ModifiersState::CTRL)) {
                                view_state.cur_node = nn;
                        }
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
