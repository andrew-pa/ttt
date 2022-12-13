use winit::event::{ElementState, VirtualKeyCode};

use super::edit_mode::EditMode;

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
            VirtualKeyCode::Escape => Some(Box::new(EditMode::default())),
            VirtualKeyCode::Back => {
                if buf.len_chars() > 0 && *cursor_index > 0 {
                    buf.remove((*cursor_index - 1)..*cursor_index);
                    *cursor_index -= 1;
                }
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
        mods: &winit::event::ModifiersState,
        view_state: &mut super::ViewState,
    ) -> Option<Box<dyn super::Mode>> {
        if c.is_control() {
            return None;
        }
        let (cursor_index, buf) = view_state.cur_edit.as_mut().unwrap();
        buf.insert_char(*cursor_index, c);
        *cursor_index += 1;
        None
    }
}
