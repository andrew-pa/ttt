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
        input: &winit::event::KeyboardInput,
        mods: &winit::event::ModifiersState,
        view_state: &mut super::ViewState,
    ) -> Option<Box<dyn super::Mode>> {
        todo!()
    }

    fn name(&self) -> &'static str {
        "EDIT"
    }

    fn process_char(
        &mut self,
        c: char,
        mods: &winit::event::ModifiersState,
        view_state: &mut super::ViewState,
    ) {
    }
}
