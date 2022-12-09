use super::{Mode, ViewState};
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
        if input.state == ElementState::Pressed {
            match input.virtual_keycode {
                Some(VirtualKeyCode::J) => {
                    view_state.move_to_next_child();
                }
                Some(VirtualKeyCode::K) => {
                    view_state.move_to_prev_child();
                }
                Some(VirtualKeyCode::L) => {
                    view_state.enter_node();
                }
                Some(VirtualKeyCode::H) => {
                    view_state.exit_node();
                }
                _ => {}
            }
        }
        None
    }

    fn name(&self) -> &'static str {
        "TREE"
    }
}
