use winit::{
    event::{ElementState, KeyEvent},
    keyboard::{Key, NamedKey},
};

use super::tree_mode::TreeMode;

#[derive(Default)]
pub struct CmdMode {}

impl super::Mode for CmdMode {
    fn process_key(
        &mut self,
        input: &KeyEvent,
        _mods: &winit::keyboard::ModifiersState,
        view_state: &mut super::ViewState,
    ) -> Option<Box<dyn super::Mode>> {
        if input.state == ElementState::Released {
            return None;
        }
        let (cursor_index, buf) = view_state.cur_cmd.as_mut().unwrap();
        match &input.logical_key {
            Key::Named(NamedKey::Escape) => {
                view_state.abort_command_edit();
                Some(Box::new(TreeMode))
            }
            Key::Named(NamedKey::Backspace) => {
                if buf.len_chars() > 0 && *cursor_index > 0 {
                    buf.remove((*cursor_index - 1)..*cursor_index);
                    *cursor_index -= 1;
                }
                None
            }
            Key::Named(NamedKey::Enter) => {
                view_state.process_command();
                Some(Box::new(TreeMode))
            }
            Key::Character(c) => {
                let (cursor_index, buf) = view_state.cur_cmd.as_mut().unwrap();
                buf.insert(*cursor_index, c.as_str());
                *cursor_index += 1;
                None
            }
            _ => None,
        }
    }

    fn cursor_shape(&self) -> Option<super::CursorShape> {
        Some(super::CursorShape::Line)
    }

    fn name(&self) -> &'static str {
        "CMD"
    }
}
