use winit::event::{ElementState, KeyEvent};
use winit::keyboard::{Key, ModifiersState, NamedKey};

use super::motion::*;

#[derive(Default)]
pub struct EditMode {
    cmd_buffer: String,
}

impl super::Mode for EditMode {
    fn process_key(
        &mut self,
        input: &KeyEvent,
        mods: &ModifiersState,
        view_state: &mut super::ViewState,
    ) -> Option<Box<dyn super::Mode>> {
        if !mods.is_empty() || input.state == ElementState::Released {
            return None;
        }
        match &input.logical_key {
            Key::Named(NamedKey::Escape) => {
                view_state.finish_editing();
                Some(Box::new(super::tree_mode::TreeMode))
            }
            Key::Character(ch) => {
                self.cmd_buffer.push_str(ch.as_str());
                match Command::parse(&self.cmd_buffer) {
                    Ok(cmd) => {
                        // println!("cmd {cmd:?}");
                        self.cmd_buffer.clear();
                        view_state.process_normal_cmd(cmd)
                    }
                    Err(ParseError::Unknown) | Err(ParseError::Invalid) => {
                        view_state.prev_error = Some(anyhow::anyhow!(
                            "unknown/invalid command: {}",
                            self.cmd_buffer
                        ));
                        self.cmd_buffer.clear();
                        None
                    }
                    Err(ParseError::Incomplete) => None,
                }
            }
            _ => None,
        }
    }

    fn name(&self) -> &'static str {
        "EDIT"
    }

    fn cursor_shape(&self) -> Option<super::CursorShape> {
        Some(super::CursorShape::Block)
    }
}
