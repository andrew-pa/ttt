use winit::event::{KeyboardInput, ModifiersState};

mod cmd_mode;
mod edit_mode;
mod insert_mode;
mod main_view;
mod motion;
mod tree_mode;

mod view_state;
pub use view_state::ViewState;

enum CursorShape {
    Block,
    Line,
}

trait Mode {
    fn process_key(
        &mut self,
        input: &KeyboardInput,
        mods: &ModifiersState,
        view_state: &mut ViewState,
    ) -> Option<Box<dyn Mode>>;

    fn process_char(
        &mut self,
        c: char,
        mods: &ModifiersState,
        view_state: &mut ViewState,
    ) -> Option<Box<dyn Mode>> {
        None
    }

    fn name(&self) -> &'static str;

    fn cursor_shape(&self) -> Option<CursorShape> {
        None
    }
}

pub use main_view::View;
