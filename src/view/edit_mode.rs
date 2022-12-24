use winit::event::{ElementState, KeyboardInput, ModifiersState, VirtualKeyCode};

use super::motion::*;

pub struct EditMode {
    cmd_buffer: String,
}

impl Default for EditMode {
    fn default() -> Self {
        Self {
            cmd_buffer: Default::default(),
        }
    }
}

impl super::Mode for EditMode {
    fn process_key(
        &mut self,
        input: &KeyboardInput,
        mods: &ModifiersState,
        view_state: &mut super::ViewState,
    ) -> Option<Box<dyn super::Mode>> {
        if !mods.is_empty()
            || input.virtual_keycode.is_none()
            || input.state == ElementState::Released
        {
            return None;
        }
        match input.virtual_keycode.unwrap() {
            VirtualKeyCode::Escape => {
                view_state.finish_editing();
                Some(Box::new(super::tree_mode::TreeMode))
            }
            _ => None,
        }
    }

    fn process_char(
        &mut self,
        c: char,
        _mods: &ModifiersState,
        view_state: &mut super::ViewState,
    ) -> Option<Box<dyn super::Mode>> {
        if c.is_control() {
            return None;
        }
        self.cmd_buffer.push(c);
        match Command::parse(&self.cmd_buffer) {
            Ok(cmd) => {
                // println!("cmd {cmd:?}");
                self.cmd_buffer.clear();
                view_state.process_normal_cmd(cmd)
            }
            Err(ParseError::UnknownCommand) | Err(ParseError::InvalidCommand) => {
                view_state.prev_error = Some(anyhow::anyhow!(
                    "unknown/invalid command: {}",
                    self.cmd_buffer
                ));
                self.cmd_buffer.clear();
                None
            }
            Err(ParseError::IncompleteCommand) => None,
        }
    }

    fn name(&self) -> &'static str {
        "EDIT"
    }

    fn cursor_shape(&self) -> Option<super::CursorShape> {
        Some(super::CursorShape::Block)
    }
}
