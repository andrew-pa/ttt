use winit::event::{ElementState, VirtualKeyCode};

use super::tree_mode::TreeMode;

#[derive(Default)]
pub struct CmdMode {}

impl super::Mode for CmdMode {
    fn process_key(
        &mut self,
        input: &winit::event::KeyboardInput,
        mods: &winit::event::ModifiersState,
        view_state: &mut super::ViewState,
    ) -> Option<Box<dyn super::Mode>> {
        if !mods.is_empty()
            || input.virtual_keycode.is_none()
            || input.state == ElementState::Released
        {
            return None;
        }
        let (cursor_index, buf) = view_state.cur_cmd.as_mut().unwrap();
        match input.virtual_keycode.unwrap() {
            VirtualKeyCode::Escape => {
                view_state.abort_command_edit();
                Some(Box::new(TreeMode))
            }
            VirtualKeyCode::Back => {
                if buf.len_chars() > 0 && *cursor_index > 0 {
                    buf.remove((*cursor_index - 1)..*cursor_index);
                    *cursor_index -= 1;
                }
                None
            }
            VirtualKeyCode::Return => {
                view_state.process_command();
                Some(Box::new(TreeMode))
            }
            _ => None,
        }
    }

    fn cursor_shape(&self) -> Option<super::CursorShape> {
        Some(super::CursorShape::Line)
    }

    fn process_char(
        &mut self,
        c: char,
        _mods: &winit::event::ModifiersState,
        view_state: &mut super::ViewState,
    ) -> Option<Box<dyn super::Mode>> {
        if c.is_control() && c != '\n' {
            return None;
        }
        let (cursor_index, buf) = view_state.cur_cmd.as_mut().unwrap();
        buf.insert_char(*cursor_index, c);
        *cursor_index += 1;
        None
    }

    fn name(&self) -> &'static str {
        "CMD"
    }
}
