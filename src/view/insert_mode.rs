pub struct InsertMode;

impl super::Mode for InsertMode {
    fn process_key(
        &mut self,
        input: &winit::event::KeyboardInput,
        mods: &winit::event::ModifiersState,
        view_state: &mut super::ViewState,
    ) -> Option<Box<dyn super::Mode>> {
        todo!()
    }

    fn name(&self) -> &'static str {
        "INSERT"
    }
}
