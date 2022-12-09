pub struct EditMode;

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
}
