use winit::event::{ElementState, VirtualKeyCode};

use super::{edit_mode::EditMode, tree_mode::TreeMode};

pub struct InsertMode;

impl super::Mode for InsertMode {
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
        let (cursor_index, buf) = view_state.cur_edit.as_mut().unwrap();
        match input.virtual_keycode.unwrap() {
            VirtualKeyCode::Tab => Some(Box::<EditMode>::default()),
            VirtualKeyCode::Escape => {
                view_state.finish_editing();
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
                buf.insert_char(*cursor_index, '\n');
                *cursor_index += 1;
                None
            }
            VirtualKeyCode::Left => {
                *cursor_index = cursor_index.saturating_sub(1);
                None
            }
            VirtualKeyCode::Right => {
                *cursor_index += 1;
                None
            }
            _ => None,
        }
    }

    fn name(&self) -> &'static str {
        "INSERT"
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
        let (cursor_index, buf) = view_state.cur_edit.as_mut().unwrap();
        buf.insert_char(*cursor_index, c);
        *cursor_index += 1;
        None
    }
}
