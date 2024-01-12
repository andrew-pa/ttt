use winit::event::KeyEvent;
use winit::keyboard::ModifiersState;

mod cmd_mode;
mod edit_mode;
mod insert_mode;
mod main_view;
mod motion;
mod tree_mode;

mod view_state;
pub use view_state::ViewState;

pub enum CursorShape {
    Block,
    Line,
}

pub trait Mode {
    fn process_key(
        &mut self,
        input: &KeyEvent,
        mods: &ModifiersState,
        view_state: &mut ViewState,
    ) -> Option<Box<dyn Mode>>;

    fn name(&self) -> &'static str;

    fn cursor_shape(&self) -> Option<CursorShape> {
        None
    }
}

pub use main_view::View;
