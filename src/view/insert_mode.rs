use winit::{
    event::{ElementState, KeyEvent},
    keyboard::{Key, NamedKey},
};

use super::{edit_mode::EditMode, tree_mode::TreeMode};

pub struct InsertMode;

impl super::Mode for InsertMode {
    fn process_key(
        &mut self,
        input: &KeyEvent,
        _mods: &winit::keyboard::ModifiersState,
        view_state: &mut super::ViewState,
    ) -> Option<Box<dyn super::Mode>> {
        if input.state == ElementState::Released {
            return None;
        }
        let (cursor_index, buf) = view_state.cur_edit.as_mut().unwrap();
        match &input.logical_key {
            Key::Named(NamedKey::Tab) => Some(Box::<EditMode>::default()),
            Key::Named(NamedKey::Escape) => {
                view_state.finish_editing();
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
                buf.insert_char(*cursor_index, '\n');
                *cursor_index += 1;
                None
            }
            Key::Named(NamedKey::Space) => {
                buf.insert_char(*cursor_index, ' ');
                *cursor_index += 1;
                None
            }
            Key::Named(NamedKey::ArrowLeft) => {
                *cursor_index = cursor_index.saturating_sub(1);
                None
            }
            Key::Named(NamedKey::ArrowRight) => {
                *cursor_index += 1;
                None
            }
            Key::Character(c) => {
                let (cursor_index, buf) = view_state.cur_edit.as_mut().unwrap();
                buf.insert(*cursor_index, c.as_str());
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
}
