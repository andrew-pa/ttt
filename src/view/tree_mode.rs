use super::{cmd_mode::CmdMode, edit_mode::EditMode, insert_mode::InsertMode, Mode, ViewState};

use winit::{
    event::{ElementState, KeyEvent},
    keyboard::{KeyCode, ModifiersState, PhysicalKey},
};

#[derive(Default)]
pub struct TreeMode;

impl super::Mode for TreeMode {
    fn process_key(
        &mut self,
        input: &KeyEvent,
        mods: &ModifiersState,
        view_state: &mut ViewState,
    ) -> Option<Box<dyn Mode>> {
        let key = match input.physical_key {
            PhysicalKey::Code(c) => c,
            _ => return None,
        };

        if input.state == ElementState::Pressed {
            if mods.contains(ModifiersState::CONTROL) {
                match key {
                    KeyCode::KeyJ => {
                        view_state.presenter.swap_node(view_state.cur_node, 1);
                    }
                    KeyCode::KeyK => {
                        view_state.presenter.swap_node(view_state.cur_node, -1);
                    }
                    KeyCode::KeyL => {}
                    KeyCode::KeyH => {
                        view_state.presenter.make_child_sibling(view_state.cur_node);
                    }
                    _ => {}
                }
            } else {
                match key {
                    KeyCode::KeyJ => {
                        view_state.move_to_next_child();
                    }
                    KeyCode::KeyK => {
                        view_state.move_to_prev_child();
                    }
                    KeyCode::KeyL => {
                        view_state.enter_node();
                    }
                    KeyCode::KeyH => {
                        view_state.exit_node();
                    }
                    KeyCode::KeyI => {
                        view_state.begin_editing(!mods.shift_key());
                        return Some(Box::new(InsertMode));
                    }
                    KeyCode::KeyE => {
                        view_state.begin_editing(false);
                        return Some(Box::<EditMode>::default());
                    }
                    KeyCode::KeyC => {
                        view_state.cur_node = view_state
                            .presenter
                            .insert_node_as_child(view_state.cur_node, mods.shift_key());
                        view_state.begin_editing(false);
                        return Some(Box::new(InsertMode));
                    }
                    KeyCode::KeyO => {
                        if let Some(nn) = view_state
                            .presenter
                            .insert_node_in_parent(view_state.cur_node, !mods.shift_key())
                        {
                            view_state.cur_node = nn;
                            view_state.begin_editing(false);
                            return Some(Box::new(InsertMode));
                        }
                    }
                    KeyCode::KeyX => {
                        let nc = view_state.presenter.model().next_child(view_state.cur_node);
                        if let Some(nn) = view_state.presenter.delete_node(view_state.cur_node) {
                            view_state.cur_node = nc.unwrap_or(nn);
                        }
                    }
                    KeyCode::KeyY => {
                        view_state.presenter.copy_node(view_state.cur_node);
                    }
                    KeyCode::KeyP => {
                        if let Some(nn) = view_state.presenter.put_node(
                            view_state.cur_node,
                            mods.contains(ModifiersState::SHIFT),
                            mods.contains(ModifiersState::ALT),
                        ) {
                            view_state.cur_node = nn;
                        }
                    }
                    KeyCode::KeyF => {
                        view_state.toggle_folded();
                    }
                    KeyCode::Minus => {
                        view_state.presenter.toggle_strikeout(view_state.cur_node);
                    }
                    KeyCode::KeyR => {
                        view_state.presenter.set_current_root(view_state.cur_node);
                    }
                    KeyCode::Semicolon if mods.contains(ModifiersState::SHIFT) => {
                        view_state.begin_command_edit();
                        return Some(Box::<CmdMode>::default());
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
