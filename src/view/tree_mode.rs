use super::{cmd_mode::CmdMode, edit_mode::EditMode, insert_mode::InsertMode, Mode, ViewState};

use winit::event::{ElementState, KeyboardInput, ModifiersState, VirtualKeyCode};

pub struct TreeMode;

impl super::Mode for TreeMode {
    fn process_key(
        &mut self,
        input: &KeyboardInput,
        mods: &ModifiersState,
        view_state: &mut ViewState,
    ) -> Option<Box<dyn Mode>> {
        input.virtual_keycode?;

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
                    VirtualKeyCode::H => {
                        view_state.presenter.make_child_sibling(view_state.cur_node);
                    }
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
                        view_state.begin_editing();
                        return Some(Box::new(InsertMode));
                    }
                    VirtualKeyCode::E => {
                        view_state.begin_editing();
                        return Some(Box::new(EditMode::default()));
                    }
                    VirtualKeyCode::C => {
                        view_state.cur_node = view_state
                            .presenter
                            .insert_node_as_child(view_state.cur_node);
                        view_state.begin_editing();
                        return Some(Box::new(InsertMode));
                    }
                    VirtualKeyCode::O => {
                        if let Some(nn) = view_state
                            .presenter
                            .insert_node_in_parent(view_state.cur_node, !mods.shift())
                        {
                            view_state.cur_node = nn;
                            view_state.begin_editing();
                            return Some(Box::new(InsertMode));
                        }
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
                        if let Some(nn) = view_state.presenter.put_node(
                            view_state.cur_node,
                            mods.contains(ModifiersState::SHIFT),
                            mods.contains(ModifiersState::ALT),
                        ) {
                            view_state.cur_node = nn;
                        }
                    }
                    VirtualKeyCode::F => {
                        view_state.toggle_folded();
                    }
                    VirtualKeyCode::R => {
                        view_state.presenter.set_current_root(view_state.cur_node);
                    }
                    VirtualKeyCode::Colon | VirtualKeyCode::Semicolon
                        if mods.contains(ModifiersState::SHIFT) =>
                    {
                        view_state.begin_command_edit();
                        return Some(Box::new(CmdMode::default()));
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
